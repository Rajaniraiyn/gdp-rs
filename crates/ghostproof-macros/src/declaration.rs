//! Validate proof declarations before generating any public API.
use syn::{Fields, GenericParam, ItemStruct};

pub fn validate(item: &ItemStruct) -> syn::Result<()> {
    let lifetimes: Vec<_> = item
        .generics
        .lifetimes()
        .map(|p| p.lifetime.clone())
        .collect();
    if lifetimes.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "declare at least one subject lifetime",
        ));
    }
    for param in &item.generics.params {
        let reserved = match param {
            GenericParam::Type(p) => p.ident.to_string().starts_with("__GdpSubject"),
            GenericParam::Const(p) => p.ident.to_string().starts_with("__GdpSubject"),
            GenericParam::Lifetime(p) => p.lifetime.ident == "__gdp_view",
        };
        if reserved {
            return Err(syn::Error::new_spanned(
                param,
                "__GdpSubject names and the __gdp_view lifetime are reserved",
            ));
        }
    }
    let fields: Vec<_> = match &item.fields {
        Fields::Unit => Vec::new(),
        Fields::Named(f) => f.named.iter().collect(),
        Fields::Unnamed(f) => return Err(syn::Error::new_spanned(f, "use named payload fields")),
    };
    for field in &fields {
        if field
            .attrs
            .iter()
            .any(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        {
            return Err(syn::Error::new_spanned(
                field,
                "condition the whole proof declaration instead of individual payload fields",
            ));
        }
        if !matches!(field.vis, syn::Visibility::Inherited) {
            return Err(syn::Error::new_spanned(
                &field.vis,
                "proof payload fields must be private; expose shared accessors",
            ));
        }
        if field.ident.as_ref().is_some_and(|id| id == "__gdp_brand") {
            return Err(syn::Error::new_spanned(field, "__gdp_brand is reserved"));
        }
    }
    // Duplication and deserialization can bypass an affine or trusted contract.
    for attr in &item.attrs {
        if attr.path().is_ident("cfg_attr") {
            return Err(syn::Error::new_spanned(
                attr,
                "place cfg_attr before the proof attribute so Rust resolves it first",
            ));
        }
        if attr.path().is_ident("derive") {
            let paths = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )?;
            for path in paths {
                if path.segments.last().is_some_and(|s| {
                    matches!(
                        s.ident.to_string().as_str(),
                        "Default" | "Deserialize" | "Copy" | "Clone"
                    )
                }) {
                    return Err(syn::Error::new_spanned(
                        path,
                        "proofs cannot derive construction or duplication traits",
                    ));
                }
            }
        }
    }
    Ok(())
}
