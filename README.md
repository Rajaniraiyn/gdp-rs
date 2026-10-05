<img src=".github/assets/logo.svg" width="168" height="132" alt="A pixel ghost with orange crab claws carrying a checked Rust document" />

# GDP for Rust

[![Rust checks](https://github.com/Rajaniraiyn/gdp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Rajaniraiyn/gdp-rs/actions/workflows/ci.yml)

`ghostproof` connects runtime checks to the exact values used by an operation. Fresh lifetime brands distinguish subjects; private proof types carry evidence; function signatures and capabilities require it.

The core is `no_std` and allocation-free. Rust 1.85 or later is required. The optional `macros` feature generates proofs, capabilities, and borrowed views. Crates are available from GitHub and are unpublished on crates.io.

## Install

```toml
[dependencies]
ghostproof = { git = "https://github.com/Rajaniraiyn/gdp-rs", features = ["macros"] }
```

Use `rev` to pin a reviewed commit, or `path` for a local checkout.

## Check, then operate

Declare evidence in the module that checks the fact. Each lifetime identifies one named subject. Subject accessor names follow lifetime order.

```rust
use ghostproof::{Named, with_names};

mod policy {
    use ghostproof::Named;

    #[ghostproof::proof(subjects(actor, project))]
    pub struct MayEdit<'actor, 'project>;

    pub fn check<'a, 'p>(
        actor: &Named<'a, u64>,
        project: &Named<'p, u64>,
    ) -> Option<MayEdit<'a, 'p>> {
        (*actor.value() == 1 && *project.value() == 7)
            .then(|| MayEdit::issue(actor, project))
    }
}

fn edit<'a, 'p>(
    actor: &Named<'a, u64>,
    project: &Named<'p, u64>,
    _proof: &policy::MayEdit<'a, 'p>,
) {
    println!("{} edits {}", actor.value(), project.value());
}

fn main() {
    with_names!(actor = 1_u64, project = 7_u64; {
        let proof = policy::check(&actor, &project).expect("denied");
        let view = proof.view(&actor, &project);
        edit(view.actor(), view.project(), view.proof());

        let capability = proof.bind(actor, project);
        assert_eq!(*capability.project().value(), 7);
    });
}
```

`MayEdit::issue` and constructor fields are private to `policy` and its descendants. Protected APIs require the matching evidence or operate on the generated capability. Use domain-specific ID types in application code.

`bind` moves subjects and evidence into an owned capability. `view` borrows them, and `as_view` borrows an existing capability. Consuming operations take the capability by value. Reusable operations borrow it.

## Naming and composition

| API | Use |
| --- | --- |
| `name!(a = value, b = other)` | Fresh names in the current scope |
| `with_names!(a = value; { body })` | Scoped names and an ordinary return value |
| `make_guard!`, `Named::new` | Explicit guard-based naming |
| `Named::value`, `into_inner` | Shared payload access or consuming extraction |
| `And::new`, `as_ref`, `into_parts` | Combine, borrow, and recover evidence |
| `all!(a, b, c)` | Right-nested `And<A, And<B, C>>` |
| `Either::Left`, `Right`, `as_ref`, `fold` | Select and handle an alternative |
| `const_assert!(condition, "message")` | Validate a constant during compilation |

`#[proof]` supports subject lifetimes, type and const parameters, bounds, and private payload fields. `#[proof(subjects(actor, resource))]` adds domain-named accessors alongside `subject_0` and `subject_1`. Construction and duplication derives are rejected. Inference functions belong in the module that owns the resulting proposition.

`And` construction and borrowed projections support const evaluation. Expressions in naming and composition macros run once, in order. `with_names!` supports `.await` inside its block. The [configuration example](examples/configuration.rs) shares a const predicate between static assertions and runtime proof checks.

## Cargo and editors

```sh
git clone https://github.com/Rajaniraiyn/gdp-rs.git
cd gdp-rs
cargo test-all
cargo lint-all
cargo gdp check --workspace --all-features --all-targets --locked
cargo gdp doctor --workspace
```

The checkout includes Cargo aliases. For another project, install the command with `cargo install --path crates/cargo-gdp --locked` from this checkout.

`check` runs Cargo followed by GDP syntax checks. `lint` runs the syntax checks alone. Rules cover exported issuers, mutable evidence access, unchecked construction, and duplication. Selection supports exact local package names and IDs; scanning follows Cargo source roots and module paths.

Use `--message-format=json` for schema version 1 GDP records. Use `--message-format=cargo-json` for Cargo-compatible editor diagnostics and the combined build result. The [editor guide](docs/EDITOR.md) includes rust-analyzer settings. Clippy runs separately.

## Examples

| Example | Run with `cargo run --example NAME --features macros` |
| --- | --- |
| `authorization` | Async admin and plan checks, inference, and protected operations |
| `validation` | Sorted data, payload evidence, and consuming transitions |
| `relationships` | Ternary facts and alternatives |
| `versioned` | Atomic revision checks, revocation, and concurrent writers |
| `configuration` | Const predicates, static assertions, and checked runtime configuration |

`cargo run --example manual` demonstrates the guard API and handwritten evidence.

## Contracts

The compiler checks subject identity, proposition type, privacy, borrowing, and ownership. Equal raw values named separately have distinct brands. Evidence stays within its naming scope.

Checkers implement application policy. Interior mutability and remote state need atomic writes, version checks, or transaction semantics; see [freshness](docs/FRESHNESS.md). Create naming scopes inside tasks requiring `'static`. Scoped evidence is local to the process.

The Cargo analyzer checks source syntax. Alias resolution, macro expansion, cfg evaluation, and policy semantics are outside its analysis. See [design](DESIGN.md), [implementation coverage](docs/IMPLEMENTATION.md), and [contributing](CONTRIBUTING.md).

## Agent skill

```sh
npx skills add Rajaniraiyn/gdp-rs --skill gdp-rs
```

The [skill](skills/gdp-rs/SKILL.md) covers adoption, subject matching, lifetime errors, capabilities, const validation, and tooling. It can also be copied into a supported agent skill directory.

Inspired by [gdp-ts](https://github.com/rauchg/gdp-ts) and Ghosts of Departed Proofs. The [logo](.github/assets/README.md) is an original drawing. Licensed under MIT or Apache 2.0.
