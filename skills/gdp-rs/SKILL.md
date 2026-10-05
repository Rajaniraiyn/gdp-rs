---
name: gdp-rs
description: Adopt, change, or review Ghosts of Departed Proofs in Rust using ghostproof. Use for proof-gated authorization or validation, Named lifetime errors, capabilities, and cargo-gdp diagnostics. Does not apply to unrelated Rust code or general theorem proving.
---

# GDP for Rust

Connect sensitive operations to evidence about their exact arguments. Use `ghostproof` for fresh value names, private nominal evidence, and capabilities. Keep trusted checking modules small. Prove the preconditions that matter for the requested operation rather than converting every boolean into evidence.

## Adopt or change a checked operation

- Start with domain newtypes for IDs. A domain type identifies the kind of value; `name!` gives one particular value a fresh invariant brand.
- Declare a proposition inside its checking module with `#[ghostproof::proof] pub struct MayEdit<'user, 'project>;`. Each lifetime names one subject. Named payload fields must be private.
- Export a checker accepting matching `&Named<'id, T>` subjects. Call the generated private `MayEdit::issue(...)` only after the real check succeeds. The declaring module and its descendants can issue evidence, so place unrelated code outside that boundary.
- Distinguish denial from backend failure. Return `Option<Proof>` for a pure check or `Result<Option<Proof>, Error>` when failures need separate handling.
- Require matching subjects and evidence in the sensitive API, or implement its operation on the generated capability or view. Check that another public API cannot perform the same effect unchecked.
- Use `name!(user = user_id, project = project_id)` in the request, job, or operation scope. Obtain evidence, then perform the operation within that scope.
- Use `proof.view(&user, &project)` to borrow matching subjects. Use `proof.bind(user, project)` for ownership and consuming permissions. `capability.as_view()` borrows an existing owner; a view cannot replace a consumable capability.
- Combine evidence with `And` or branch with `Either`. Derive a new proposition through an explicit trusted function when that inference is justified. Do not add public blanket proof issuers or construction traits to make call sites compile.

## Rust constraints

Brand lifetime errors often identify a wrong subject or an escaping naming scope. Name a resource once and reuse references to that name. Equal raw values named separately remain different subjects. Do not weaken invariance, manufacture guards, or use unsafe conversion to silence these errors.

The core is `no_std` and allocation-free; macro support is optional. Avoid adding an executor, allocator, or application framework to the core. Normal Rust ownership and auto traits apply to payloads and subjects.

Evidence can survive `.await` while its owners stay alive. A task requiring `'static` usually needs its naming and checking scope inside the task. Borrowing concurrency can share a view when the underlying types support it. Move errors on a capability may mean the API intentionally consumes a permission; recheck only when issuing another permission is valid.

Shared access does not freeze interior mutability or a remote database row. For facts that can change between check and effect, use transaction isolation, version checks, atomic conditional writes, or revalidation at the effect. Evidence is neither a serialized authorization token nor a distributed single-use guarantee.

## Verify the boundary

Test the checker's success, denial, and meaningful failure paths. Add downstream compile-fail coverage for a newly introduced boundary, such as missing evidence, the wrong resource, forged construction, or reuse after consumption. In the gdp-rs workspace, reuse the existing `tests/ui` harness and stable diagnostic fragments rather than compiler-version snapshots.

From a gdp-rs checkout, run:

```sh
cargo test-all
cargo test --no-default-features --locked
cargo fmt --all -- --check
cargo lint-all
cargo gdp check --workspace --all-features --all-targets --locked
```

Downstream, install `cargo-gdp` from its checkout with `cargo install --path crates/cargo-gdp --locked`, then use `cargo gdp check` with the consuming project's normal Cargo options. Do not add the workspace's local alias to a downstream project unless it also contains the tool package.

`cargo gdp` checks syntax conventions. It does not resolve aliases, expand macros, evaluate `cfg`, verify checker truth, or certify every sensitive operation. Its rules are separate from stock Clippy. Use `cargo gdp doctor --message-format=json` for scope and `cargo gdp lint --message-format=json` for schema version 1 findings.

Read the project's [README](https://github.com/Rajaniraiyn/gdp-rs/blob/main/README.md) for the current API and [implementation ledger](https://github.com/Rajaniraiyn/gdp-rs/blob/main/docs/IMPLEMENTATION.md) for guarantees and analysis limits. In a local checkout, prefer those local files. The examples cover authorization, validation, and relationships; preserve the user's runtime and policy choices when adapting them.

For on-save editor findings, use `cargo gdp check --message-format=cargo-json` with rust-analyzer's check override. Read `docs/EDITOR.md` before changing editor settings. The duplication lint flags handwritten `Clone` or `Copy` on recognized evidence types.

For mutable external facts, `examples/versioned.rs` and `docs/FRESHNESS.md` show a revision compared atomically at the write. Bind backend identity as a subject when evidence must not transfer between stores. Revision invalidation must cover every policy-relevant change.
