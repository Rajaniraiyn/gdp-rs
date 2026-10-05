//! Optional domain names for generated subject accessors.
use std::collections::BTreeSet;
use syn::{
    Ident, ItemStruct, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(Default)]
pub struct Options {
    pub subjects: Option<Vec<Ident>>,
}

impl Parse for Options {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }
        let option: Ident = input.parse()?;
        if option != "subjects" {
            return Err(syn::Error::new_spanned(
                option,
                "expected subjects(name, ...)",
            ));
        }
        let content;
        parenthesized!(content in input);
        let subjects = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
        if !input.is_empty() {
            return Err(input.error("expected only subjects(name, ...)"));
        }
        Ok(Self {
            subjects: Some(subjects.into_iter().collect()),
        })
    }
}

impl Options {
    pub fn validate(&self, item: &ItemStruct) -> syn::Result<()> {
        let Some(subjects) = &self.subjects else {
            return Ok(());
        };
        let count = item.generics.lifetimes().count();
        if subjects.len() != count {
            return Err(syn::Error::new_spanned(
                &item.ident,
                format!("expected {count} subject names, one per lifetime"),
            ));
        }
        let mut seen = BTreeSet::new();
        for subject in subjects {
            let spelling = subject.to_string();
            let name = spelling.trim_start_matches("r#");
            if !seen.insert(name.to_owned()) {
                return Err(syn::Error::new_spanned(
                    subject,
                    "subject names must be distinct",
                ));
            }
            let positional = (0..count).any(|i| name == format!("subject_{i}"));
            if positional || matches!(name, "proof" | "as_view" | "into_parts") {
                return Err(syn::Error::new_spanned(
                    subject,
                    "subject name conflicts with a generated accessor",
                ));
            }
        }
        Ok(())
    }
}
