# Ghostproof declaration macros

Enable `ghostproof`'s optional `macros` feature to use this package through the library's `proof` reexport.

```rust,ignore
mod policy {
    #[ghostproof::proof]
    pub struct Allowed<'user, 'resource>;
}
```

The declaration creates a nominal proof with private fields and a private issuing method. Each lifetime identifies one named subject. Type parameters, const parameters, bounds, and private payload fields are supported. The generated capability bundles only matching subjects with its evidence. `proof.view(...)` and `capability.as_view()` borrow those subjects and their evidence into a matching view without consuming their owners.

The owning module and its descendants are trusted to issue facts correctly. The macro does not inspect a policy query or prove its truth. Scoped lifetime brands and private constructors enforce subject identity and construction boundaries.

Requires Rust 1.85 or later. Licensed under MIT or Apache 2.0.
