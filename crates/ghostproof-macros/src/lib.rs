//! Generate nominal proofs with private issuing methods and branded capabilities.

use proc_macro::TokenStream;
use syn::{ItemStruct, parse_macro_input};

mod codegen;
mod declaration;

/// Declare a trusted nominal proof.
///
/// Each lifetime parameter names one subject. The generated private `issue`
/// method takes references to those named subjects, followed by payload fields.
/// The generated `bind` method consumes matching named subjects and the proof,
/// returning `<Name>Capability` with immutable `subject_N` and `proof` accessors.
/// `view` borrows subjects and evidence into `<Name>View`. Owned capabilities
/// provide `as_view`, allowing shared operations without consuming permission.
///
/// Invoke inside a dedicated checking module. That module and its descendants
/// are trusted to issue evidence. Type and const generics describe payloads;
/// lifetime parameters describe subjects. Existing bounds are preserved.
#[proc_macro_attribute]
pub fn proof(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(proc_macro2::Span::call_site(), "proof takes no arguments")
            .to_compile_error()
            .into();
    }
    let item = parse_macro_input!(input as ItemStruct);
    codegen::expand(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
