---
name: gdp-rs
description: Adopt, change, or review Rust Ghosts of Departed Proofs with ghostproof. Use for proof-gated operations, Named lifetime errors, scoped naming, const configuration checks, and cargo-gdp findings.
---

# GDP for Rust

Connect each protected operation to evidence about its exact subjects. Keep the checking module small and preserve the project's runtime, policy, and error model.

## Build the boundary

- Use domain-specific ID types. Give each value a fresh brand with `name!`, or group checks and effects in `with_names!(actor = actor_id, resource = resource_id; { ... })`.
- Declare the fact in its checking module with `#[ghostproof::proof(subjects(actor, resource))] pub struct Allowed<'actor, 'resource>;`. Accessor names follow lifetime order. Payload fields are private; type and const generics describe payloads or proposition parameters.
- Export a checker taking matching `&Named<'id, T>` arguments. Issue evidence after the actual predicate succeeds. The declaring module and its descendants hold issuance authority.
- Keep denial, missing resources, and backend failures distinct where the application requires them. Use its existing `Option` or `Result` conventions.
- Require matching subjects and evidence in a protected function, or implement the operation on its generated capability or view. Keep raw writes behind this boundary.
- Use `proof.bind(actor, resource)` for owned permissions and `proof.view(&actor, &resource)` for borrowing. `capability.as_view()` shares the existing owner. Consuming effects take the owned capability by value.
- Combine existing evidence with `And` or `all!`. Use `Either` for alternatives. Implement inference in the module that owns the resulting proposition.

## Static configuration

Use a const predicate and `const_assert!(condition, "message")` for values known at compile time. Reuse that predicate in the runtime checker for dynamic input. Const assertions validate static conditions; application checks issue branded evidence. See `examples/configuration.rs`.

`And::new`, `And::as_ref`, shared projections, `Either::as_ref`, and `Named::value` support const functions. Scoped naming macros support Rust 2024 expressions. The core is allocation-free and `no_std`; procedural macros are optional.

## Resolve Rust errors

Name a resource once and reuse that name. Equal raw values named separately have distinct brands. A lifetime error can indicate a wrong subject or an escaping scope. Keep the evidence and operation in the same scope, preserving invariance and private construction.

Evidence can survive `.await` while its owners remain alive. Create a naming scope inside a task requiring `'static`. Borrowing concurrency uses views and the underlying types' `Send` and `Sync` bounds. A moved capability represents a consumed permission; issue another only after a valid new check.

For changing external facts, use an atomic conditional write, version check, or transaction semantics. Bind the backend as a subject when store identity matters. Revision invalidation must cover every mutable policy input. See `examples/versioned.rs` and `docs/FRESHNESS.md`.

## Verify

Test the checker and the new boundary's failure paths. Compile-fail cases should exercise missing evidence, wrong subjects, scope escape, or repeated consumption. Reuse `tests/ui` in this workspace and stable diagnostic fragments.

```sh
cargo test-all
cargo test --no-default-features --locked
cargo fmt --all -- --check
cargo lint-all
cargo gdp check --workspace --all-features --all-targets --locked
```

Downstream projects install the command with `cargo install --path crates/cargo-gdp --locked` from a gdp-rs checkout. Its local Cargo aliases apply to this workspace.

Use `cargo gdp doctor --message-format=json` for package scope and `cargo gdp lint --message-format=json` for schema version 1 findings. `--message-format=cargo-json` supports editor check commands; read `docs/EDITOR.md` before changing settings. Clippy runs separately. GDP analysis covers source syntax; aliases, expanded macros, cfg evaluation, and checker semantics require application review.

Read the local README and `docs/IMPLEMENTATION.md` for the API and coverage. The published [README](https://github.com/Rajaniraiyn/gdp-rs/blob/main/README.md) and [coverage ledger](https://github.com/Rajaniraiyn/gdp-rs/blob/main/docs/IMPLEMENTATION.md) are fallbacks when the checkout is unavailable.
