//! Syntax rules for local evidence declarations.
use std::collections::BTreeSet;
use syn::{ImplItem, Item, Type, Visibility};

pub fn inspect(items: &[Item]) -> Vec<(usize, usize, &'static str, &'static str)> {
    let mut proofs = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for item in items {
        if let Item::Struct(s) = item {
            if s.attrs
                .iter()
                .any(|a| a.path().segments.last().is_some_and(|s| s.ident == "proof"))
            {
                proofs.insert(s.ident.to_string());
                proofs.insert(format!(
                    "{}Capability",
                    s.ident.to_string().trim_start_matches("r#")
                ));
                proofs.insert(format!(
                    "{}View",
                    s.ident.to_string().trim_start_matches("r#")
                ));
            }
        }
    }
    for item in items {
        if let Item::Mod(m) = item {
            if let Some((_, children)) = &m.content {
                diagnostics.extend(inspect(children));
            }
        }
        let Item::Impl(implementation) = item else {
            continue;
        };
        let Type::Path(path) = implementation.self_ty.as_ref() else {
            continue;
        };
        // Restrict matching to local unqualified declarations. Resolution across
        // modules and aliases requires semantic analysis, not this syntax check.
        if path.path.segments.len() != 1 {
            continue;
        }
        let name = &path.path.segments[0].ident;
        if !proofs.contains(&name.to_string()) {
            continue;
        }
        if let Some((trait_path, _)) = &implementation.trait_ {
            if implementation.modifiers.polarity.is_some() {
                continue;
            }
            if trait_path
                .segments
                .last()
                .is_some_and(|s| matches!(s.ident.to_string().as_str(), "Clone" | "Copy"))
            {
                diagnostics.push((
                    name.span().start().line,
                    name.span().start().column + 1,
                    "gdp::duplicated_evidence",
                    "borrow reusable evidence instead of duplicating an owned permission",
                ));
            }
            if trait_path
                .segments
                .last()
                .is_some_and(|s| matches!(s.ident.to_string().as_str(), "Default" | "Deserialize"))
            {
                diagnostics.push((
                    name.span().start().line,
                    name.span().start().column + 1,
                    "gdp::unchecked_construction",
                    "construction traits on proof declarations require removing this bypass",
                ));
            }
            continue;
        }
        for member in &implementation.items {
            let ImplItem::Fn(method) = member else {
                continue;
            };
            if matches!(method.vis, Visibility::Inherited) {
                continue;
            }
            if matches!(
                method.sig.ident.to_string().as_str(),
                "issue" | "issue_unchecked" | "new_unchecked"
            ) {
                diagnostics.push((
                    method.sig.ident.span().start().line,
                    method.sig.ident.span().start().column + 1,
                    "gdp::exported_issuer",
                    "keep unchecked evidence issuance private to the checking module",
                ));
            }
            if method.sig.receiver().is_some_and(mutable_receiver) {
                diagnostics.push((method.sig.ident.span().start().line, method.sig.ident.span().start().column + 1, "gdp::mutable_evidence", "public mutable access to proof or capability state requires a new checked transition"));
            }
        }
    }
    diagnostics
}

fn mutable_receiver(receiver: &syn::Receiver) -> bool {
    match &receiver.kind {
        syn::ReceiverKind::Reference(_, _, mutability) => mutability.is_some(),
        syn::ReceiverKind::Typed(_, ty) => {
            matches!(ty.as_ref(), Type::Reference(reference) if reference.mutability.is_some())
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rules(source: &str) -> Vec<&'static str> {
        inspect(&syn::parse_file(source).unwrap().items)
            .into_iter()
            .map(|(_, _, rule, _)| rule)
            .collect()
    }
    #[test]
    fn flags_construction_mutation_and_exports() {
        assert_eq!(
            rules(
                "#[gp::proof] pub struct P<'a>; impl<'a> P<'a> { pub fn issue_unchecked() {} pub fn edit(&mut self) {} } impl<'a> Default for P<'a> {}"
            ),
            [
                "gdp::exported_issuer",
                "gdp::mutable_evidence",
                "gdp::unchecked_construction"
            ]
        );
    }
    #[test]
    fn permits_checks_private_issuance_and_owned_transitions() {
        assert!(rules("#[gp::proof] pub struct P<'a>; impl<'a> P<'a> { fn issue() {} pub fn check() {} pub fn transition(self) {} pub fn get(&self) {} }").is_empty());
        assert!(rules("struct Plain; impl Plain { pub fn edit(&mut self) {} }").is_empty());
    }
    #[test]
    fn follows_inline_modules_without_confusing_qualified_names() {
        assert_eq!(
            rules(
                "mod p { #[gp::proof] struct P<'a>; impl<'a> P<'a> { pub fn new_unchecked() {} } }"
            ),
            ["gdp::exported_issuer"]
        );
        assert!(
            rules("#[gp::proof] struct P<'a>; impl other::P { pub fn new_unchecked() {} }")
                .is_empty()
        );
    }

    #[test]
    fn flags_duplication_of_local_proofs_capabilities_and_views() {
        assert_eq!(
            rules(
                "#[gp::proof] struct P<'a>; impl Clone for P<'_> {} impl Copy for PCapability<'_> {} impl core::clone::Clone for PView<'_> {}"
            ),
            [
                "gdp::duplicated_evidence",
                "gdp::duplicated_evidence",
                "gdp::duplicated_evidence"
            ]
        );
        assert!(rules("struct Plain; impl Clone for Plain {} impl Copy for Plain {}").is_empty());
        assert!(rules("#[gp::proof] struct P<'a>; impl Clone for other::P<'_> {}").is_empty());
    }

    #[test]
    fn recognizes_shorthand_and_explicit_mutable_receivers() {
        assert_eq!(
            rules(
                "#[gp::proof] struct P<'a>; impl P<'_> { pub fn a(&mut self) {} pub fn b(self: &mut Self) {} pub fn c(self: &Self) {} pub fn d(mut self) {} }"
            ),
            ["gdp::mutable_evidence", "gdp::mutable_evidence"]
        );
    }

    #[test]
    fn negative_impls_do_not_construct_or_duplicate_evidence() {
        assert!(
            rules("#[gp::proof] struct P<'a>; impl !Clone for P<'_> {} impl !Default for P<'_> {}")
                .is_empty()
        );
    }
}
