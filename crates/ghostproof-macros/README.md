# Proof declarations

Enable the library's `macros` feature and use its `proof` attribute:

```rust,ignore
mod policy {
    #[ghostproof::proof(subjects(actor, resource))]
    pub struct Allowed<'actor, 'resource>;
}
```

The attribute generates `Allowed`, `AllowedCapability`, and `AllowedView`. Each lifetime identifies one named subject. `subjects(...)` adds domain-named accessors to capabilities and views; positional `subject_N` accessors remain available. Names must be distinct, match the lifetime count, and avoid `proof`, `as_view`, `into_parts`, and generated positional names.

The private `issue` method takes named subject references followed by payload fields. Type parameters, const parameters, bounds, and private named fields are supported. `bind` owns matching subjects and evidence; `view` and `as_view` borrow them.

The declaring module and its descendants implement the checks and own issuance authority. Constructor fields are private. Construction and duplication derives are rejected. Rust 1.85 or later is required. Licensed under MIT or Apache 2.0.
