//! Generate nominal owned capabilities and shared views.
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{GenericParam, ItemStruct};

pub fn expand(
    item: ItemStruct,
    options: super::options::Options,
) -> syn::Result<proc_macro2::TokenStream> {
    super::declaration::validate(&item)?;
    options.validate(&item)?;
    let lifetimes: Vec<_> = item
        .generics
        .lifetimes()
        .map(|p| p.lifetime.clone())
        .collect();
    let fields: Vec<_> = item.fields.iter().collect();
    let root = match crate_name("ghostproof")
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?
    {
        FoundCrate::Itself => quote!(::ghostproof),
        FoundCrate::Name(name) => {
            let id = format_ident!("{}", name);
            quote!(::#id)
        }
    };
    let name = &item.ident;
    let capability = format_ident!("{}Capability", name.to_string().trim_start_matches("r#"));
    let view = format_ident!("{}View", name.to_string().trim_start_matches("r#"));
    let vis = &item.vis;
    let attrs = &item.attrs;
    let cfg_attrs: Vec<_> = attrs.iter().filter(|a| a.path().is_ident("cfg")).collect();
    let generics = &item.generics;
    let (impl_g, type_g, where_g) = generics.split_for_impl();
    let types: Vec<_> = (0..lifetimes.len())
        .map(|i| format_ident!("__GdpSubject{i}"))
        .collect();
    let mut cap_generics = generics.clone();
    for parameter in &mut cap_generics.params {
        match parameter {
            GenericParam::Type(p) => p.default = None,
            GenericParam::Const(p) => p.default = None,
            GenericParam::Lifetime(_) => {}
        }
    }
    for subject_type in &types {
        cap_generics.params.push(syn::parse_quote!(#subject_type));
    }
    let (cap_impl_g, cap_type_g, cap_where_g) = cap_generics.split_for_impl();
    let mut view_generics = cap_generics.clone();
    let mut view_params = syn::punctuated::Punctuated::new();
    view_params.push(syn::parse_quote!('__gdp_view));
    view_params.extend(view_generics.params.clone());
    view_generics.params = view_params;
    let (view_impl_g, view_type_g, view_where_g) = view_generics.split_for_impl();
    let subjects: Vec<_> = (0..lifetimes.len())
        .map(|i| format_ident!("subject_{i}"))
        .collect();
    let mut capability_accessors = Vec::new();
    let mut view_accessors = Vec::new();
    if let Some(aliases) = &options.subjects {
        for (((alias, subject), lifetime), ty) in
            aliases.iter().zip(&subjects).zip(&lifetimes).zip(&types)
        {
            let doc = format!(
                "Borrow the {} subject.",
                alias.to_string().trim_start_matches("r#")
            );
            capability_accessors.push(quote! {
                #[doc = #doc]
                pub fn #alias(&self) -> &#root::Named<#lifetime, #ty> { &self.#subject }
            });
            view_accessors.push(quote! {
                #[doc = #doc]
                pub fn #alias(&self) -> &'__gdp_view #root::Named<#lifetime, #ty> { self.#subject }
            });
        }
    }
    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let issue_subjects: Vec<_> = (0..lifetimes.len())
        .map(|index| {
            let mut candidate = format!("__gdp_subject_{index}");
            while field_names
                .iter()
                .any(|name| name.to_string().trim_start_matches("r#") == candidate)
            {
                candidate.push('_');
            }
            format_ident!("{candidate}")
        })
        .collect();
    let mut markers: Vec<_> = lifetimes.iter().map(|lt| quote!(&#lt ())).collect();
    markers.extend(generics.type_params().map(|p| {
        let id = &p.ident;
        quote!(*const #id)
    }));
    let brand = quote!(::core::marker::PhantomData<fn((#(#markers,)*)) -> (#(#markers,)*)>);
    Ok(quote! {
        #(#attrs)*
        #[must_use = "supply evidence to a dependent operation"]
        #vis struct #name #generics #where_g {
            #(#fields,)*
            __gdp_brand: #brand,
        }
        #(#cfg_attrs)*
        impl #impl_g #name #type_g #where_g {
            // Only the checking module and its descendants may issue this fact.
            #[allow(dead_code)]
            fn issue<#(#types),*>(
                #(#issue_subjects: &#root::Named<#lifetimes, #types>,)*
                #(#field_names: #field_types,)*
            ) -> Self {
                #(let _ = #issue_subjects;)*
                Self { #(#field_names,)* __gdp_brand: ::core::marker::PhantomData }
            }

            /// Attach this evidence to exactly the named subjects it concerns.
            pub fn bind<#(#types),*>(self, #(#subjects: #root::Named<#lifetimes, #types>),*)
                -> #capability #cap_type_g
            {
                #capability { #(#subjects,)* proof: self }
            }

            /// Borrow matching subjects and evidence.
            pub fn view<'__gdp_view, #(#types),*>(
                &'__gdp_view self,
                #(#subjects: &'__gdp_view #root::Named<#lifetimes, #types>),*
            ) -> #view #view_type_g {
                #view { #(#subjects,)* proof: self }
            }
        }

        /// Subjects bundled with matching evidence. Construction is private.
        #(#cfg_attrs)*
        #[must_use = "use the capability in a dependent operation"]
        #vis struct #capability #cap_generics #cap_where_g {
            #(#subjects: #root::Named<#lifetimes, #types>,)*
            proof: #name #type_g,
        }
        #(#cfg_attrs)*
        impl #cap_impl_g #capability #cap_type_g #cap_where_g {
            #(#capability_accessors)*
            #(
                /// Borrow a named subject.
                pub fn #subjects(&self) -> &#root::Named<#lifetimes, #types> { &self.#subjects }
            )*
            /// Borrow the evidence carried by this capability.
            pub fn proof(&self) -> &#name #type_g { &self.proof }

            /// Borrow this capability's subjects and evidence together.
            pub fn as_view<'__gdp_view>(&'__gdp_view self) -> #view #view_type_g {
                #view { #(#subjects: &self.#subjects,)* proof: &self.proof }
            }

            /// Consume the capability and recover its subjects and evidence.
            pub fn into_parts(self) -> ((#(#root::Named<#lifetimes, #types>,)*), #name #type_g) {
                ((#(self.#subjects,)*), self.proof)
            }
        }

        /// A shared view of exactly matching subjects and evidence.
        /// It cannot outlive any of its owners or consume their owned permission.
        #(#cfg_attrs)*
        #[must_use = "use the borrowed capability in a dependent operation"]
        #vis struct #view #view_generics #view_where_g {
            #(#subjects: &'__gdp_view #root::Named<#lifetimes, #types>,)*
            proof: &'__gdp_view #name #type_g,
        }
        #(#cfg_attrs)*
        impl #view_impl_g #view #view_type_g #view_where_g {
            #(#view_accessors)*
            #(
                /// Read a named subject within the view's borrow lifetime.
                pub fn #subjects(&self) -> &'__gdp_view #root::Named<#lifetimes, #types> { self.#subjects }
            )*
            /// Read the borrowed evidence.
            pub fn proof(&self) -> &'__gdp_view #name #type_g { self.proof }
        }
    })
}
