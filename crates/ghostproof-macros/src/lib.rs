//! Generate nominal proofs with private issuing methods and branded capabilities.

use proc_macro::TokenStream;
use syn::{ItemStruct, parse_macro_input};

mod codegen;
mod declaration;
mod options;

/// Declare a trusted nominal proof.
///
/// Each lifetime parameter names one subject. The generated private `issue`
/// method takes references to those named subjects, followed by payload fields.
/// The generated `bind` method consumes matching named subjects and the proof,
/// returning `<Name>Capability` with immutable `subject_N` and `proof` accessors.
/// `view` borrows subjects and evidence into `<Name>View`. Owned capabilities
/// provide `as_view` for shared operations.
///
/// Invoke inside a dedicated checking module. That module and its descendants
/// are trusted to issue evidence. Type and const generics describe payloads;
/// lifetime parameters describe subjects. Existing bounds are preserved.
///
/// `#[proof(subjects(actor, resource))]` adds named subject accessors to the
/// capability and view. Names follow lifetime order; positional accessors remain.
#[proc_macro_attribute]
pub fn proof(args: TokenStream, input: TokenStream) -> TokenStream {
    let options = parse_macro_input!(args as options::Options);
    let item = parse_macro_input!(input as ItemStruct);
    codegen::expand(item, options)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
