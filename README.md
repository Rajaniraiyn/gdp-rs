<img src=".github/assets/logo.svg" width="168" height="132" alt="A pixel ghost with orange crab claws carrying a checked Rust document" />

# GDP for Rust

[![Rust checks](https://github.com/Rajaniraiyn/gdp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Rajaniraiyn/gdp-rs/actions/workflows/ci.yml)

`ghostproof` implements Ghosts of Departed Proofs with fresh value names, private nominal evidence, and operations that require that evidence. It supports authorization, immutable validation, relationships between values, and consuming state transitions.

The core uses stable Rust 1.85 or later, is `no_std`, and does not allocate or select an async runtime. The optional `macros` feature generates proof declarations and capabilities. Crates are not yet published on crates.io.

## Install from GitHub

```sh
git clone https://github.com/Rajaniraiyn/gdp-rs.git
cd gdp-rs
cargo test-all
cargo gdp doctor --workspace
```

The checkout includes Cargo aliases for `gdp`, `test-all`, `lint-all`, and `docs-all`. The GDP alias builds the local tool, so installation is optional in this workspace. To use the library directly from GitHub, add:

```toml
[dependencies]
ghostproof = { git = "https://github.com/Rajaniraiyn/gdp-rs", features = ["macros"] }
```

Pin a reviewed commit with `rev` for reproducible Git dependencies. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full checks.

## Agent skill

The portable [gdp-rs skill](skills/gdp-rs/SKILL.md) guides agents through proof-gated Rust APIs, lifetime errors, consuming permissions, borrowed views, and Cargo diagnostics. Install it with the Skills CLI:

```sh
npx skills add Rajaniraiyn/gdp-rs --skill gdp-rs
```

You can also copy `skills/gdp-rs` into your agent's supported skill directory. The skill does not require a specific framework or async runtime.

## Start here

For a downstream checkout, add the library by path:

```toml
[dependencies]
ghostproof = { path = "../gdp-rs", features = ["macros"] }
```

A trusted checking module declares a fact and creates evidence after a runtime check. Each lifetime parameter represents one specific named subject.

```rust
use ghostproof::{Named, name};

mod policy {
    use ghostproof::Named;

    #[ghostproof::proof]
    pub struct MayEdit<'user, 'project>;

    pub fn check<'u, 'p>(
        user: &Named<'u, u64>,
        project: &Named<'p, u64>,
    ) -> Option<MayEdit<'u, 'p>> {
        // Replace this example rule with your database or policy engine.
        (*user.value() == 1 && *project.value() == 7)
            .then(|| MayEdit::issue(user, project))
    }
}

fn edit<'u, 'p>(
    user: &Named<'u, u64>,
    project: &Named<'p, u64>,
    _evidence: &policy::MayEdit<'u, 'p>,
) {
    println!("{} edits {}", user.value(), project.value());
}

fn main() {
    name!(user = 1_u64, project = 7_u64);
    let evidence = policy::check(&user, &project).expect("denied");
    edit(&user, &project, &evidence);

    // Borrow the same subjects for a checked operation without moving them.
    let view = evidence.view(&user, &project);
    edit(view.subject_0(), view.subject_1(), view.proof());

    // The generated capability owns matching subjects and evidence.
    let capability = evidence.bind(user, project);
    assert_eq!(*capability.subject_1().value(), 7);
    assert_eq!(*capability.as_view().subject_1().value(), 7);
}
```

`MayEdit::issue` is private to `policy` and its descendants. External callers cannot construct the proof or its capability. The generated `MayEditCapability<'u, 'p, UserType, ProjectType>` exposes shared `subject_0`, `subject_1`, and `proof` accessors, plus a consuming `into_parts`. Applications can implement protected methods on the capability in the checking module.

`MayEditView<'borrow, 'u, 'p, UserType, ProjectType>` borrows matching subjects and evidence. Use `proof.view(...)` when several facts concern the same resource, or `capability.as_view()` when you already own a capability. A view cannot outlive its proof or subjects, and cannot substitute for an owned consumable permission.

Use domain newtypes rather than plain integers in real applications. A domain type identifies the kind of value; a fresh brand identifies which value the evidence concerns.

## Core API

| API | Purpose |
| --- | --- |
| `name!(a = value, b = other)` | Name values with separate fresh brands in the current scope |
| `make_guard!` and `Named::new` | Explicit naming without the declaration macro |
| `Named::value` | Shared access to the payload |
| `Named::into_inner` | Consume the name and recover a raw value |
| `And::new`, `left`, `right`, `into_parts` | Compose and project actual evidence |
| `Either::Left`, `Right`, `as_ref`, `fold` | Select and handle a policy alternative |
| `#[proof]` | Generate a nominal proof, private issuing method, owned capability, and borrowed view |

Proof declarations support multiple subject lifetimes, generic payload types, const generics, and bounds. Payload fields remain private; expose explicit shared accessors. Proofs and capabilities have no default construction or automatic duplication. Reusable facts may be borrowed. A trusted module can deliberately implement sharing, but must not accidentally duplicate a permission intended for one consumption.

Inference rules belong in trusted application modules. To derive member evidence from owner evidence, implement that transformation explicitly in the module that owns the member proof. The library does not infer propositions from names or turn a failed positive check into a negative proof.

## Run the examples

```sh
cargo run --example authorization --features macros
cargo run --example validation --features macros
cargo run --example relationships --features macros
cargo run --example versioned --features macros
cargo run --example manual
```

The authorization example checks admin and plan concurrently, derives an operation-specific permission, and binds it to the actor and resource. Validation uses an owned immutable sequence, payload evidence, and a transition requiring new validation. Relationships demonstrates a ternary fact and alternatives. The manual example works without procedural macros.

The versioned example binds evidence to the store, actor, and project, then compares its checked revision atomically at the write. Tests cover revocation, ownership changing away and back, and concurrent permissions. See [evidence freshness](docs/FRESHNESS.md) for backend assumptions.

## Cargo tooling

Install the local command when you want additional syntax checks:

```sh
cargo install --path crates/cargo-gdp --locked
cargo gdp check --workspace --all-features --all-targets
cargo gdp doctor --workspace
```

You can also run it without installing:

```sh
cargo run -p cargo-gdp -- check --workspace --all-features --all-targets
```

`check` forwards options to `cargo check`, then checks selected packages for narrowly defined evidence bypass conventions. `lint` runs just the syntax checks. `doctor` reports the selected packages and analysis limits. No command changes manifests or toolchains.

Syntax rules flag exported issuing methods, public mutable proof, capability, or view methods, and handwritten construction or duplication traits on annotated declarations. They inspect local unqualified implementations. They do not resolve aliases, expand macros, or evaluate `cfg`; test fixtures are excluded. Select packages by exact local name or full Cargo package ID; unsupported selectors are rejected rather than silently skipped.

The scanner also follows custom library, binary, and example source paths declared in Cargo metadata, including their outlined modules and literal `#[path]` declarations. Imports and type aliases across files remain unresolved. Files are deduplicated by canonical path.

Use `--message-format=json` for machine-readable output. `check` preserves Cargo's JSON records and appends GDP records. `lint` emits `gdp-diagnostic` records and a `gdp-summary`; `doctor` emits `gdp-doctor`. GDP records use schema version 1. Findings include a stable rule code, source path, and one-based line and column. When the compiler check fails, the summary identifies the compiler stage and the command retains its failure status.

These are project convention checks, not compiler proofs. They run through `cargo gdp`, not a new namespace inside stock Clippy. Compiler-coupled Dylint integration remains unnecessary for the current rules and is deferred until a demonstrated semantic gap warrants it.

For editor diagnostics, use `cargo gdp check --message-format=cargo-json`. This emits Cargo/rustc-compatible findings with primary spans and one combined final build result. See the [editor guide](docs/EDITOR.md) for a rust-analyzer check override. Existing GDP JSON mode retains its schema version 1 records.

## Guarantees and limits

Different names cannot substitute for one another, even when their raw values compare equal. Missing evidence, wrong proposition types, wrong subjects, private construction, and repeated use of moved capabilities are compiler errors. Naming and evidence cannot escape their brand scope. Some brand mismatches appear as lifetime or temporary-borrow diagnostics because generativity enforces distinctness through scoped lifetimes.

Keep trusted modules small. Their code owns the truth of each check. The library cannot verify a business policy, identify every sensitive operation, or protect a database reachable through an independent unchecked API.

Shared access does not freeze `Cell`, locks, or other interior mutability. Naming an ID does not freeze its database row. For changing external facts, choose appropriate atomic writes, isolation, versioning, or rechecks. Consuming a permission prevents reuse of that token; globally single-use effects require external coordination.

Scoped evidence can survive `.await` and borrowing concurrency. It generally cannot move into tasks requiring `'static`; create the naming scope inside such a task or recheck at its destination. The core provides no evidence deserialization or cross-process proof transport.

## Verification

```sh
cargo test --workspace --all-features --locked
cargo test --no-default-features --locked
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
cargo doc --workspace --all-features --no-deps
```

The suite contains runtime tests and real downstream compile-pass and compile-fail fixtures. It exercises privacy, every subject of a ternary proof, invariance, consumption, generic payloads, renamed dependencies, borrowing concurrency, cancellation, and unwinding. CI checks Rust 1.85 and stable.

See [DESIGN.md](DESIGN.md) for the design and [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) for implementation decisions and the coverage ledger. The benchmark command is documented in [bench/README.md](bench/README.md). Measurements are local observations, not universal performance guarantees.

Inspired by [gdp-ts](https://github.com/rauchg/gdp-ts) and the Ghosts of Departed Proofs pattern. This is an independent Rust implementation. The [logo](.github/assets/README.md) is an original drawing with a nod to the upstream ghost.

Licensed under MIT or Apache 2.0, at your option.
