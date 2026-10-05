# Cargo GDP checks

`cargo gdp check` runs Cargo's compiler check followed by GDP syntax analysis. `cargo gdp lint` runs analysis alone. `cargo gdp doctor` reports the selected packages and source roots.

Selection supports workspace defaults, `--workspace`, exclusions, exact local package names, and full Cargo package IDs. Scanning follows source and example directories, custom crate roots, outlined modules, and literal path attributes.

| Rule | Finding |
| --- | --- |
| `gdp::exported_issuer` | Exported issuing methods |
| `gdp::mutable_evidence` | Shared public APIs with mutable proof, capability, or view receivers |
| `gdp::unchecked_construction` | Handwritten construction traits |
| `gdp::duplicated_evidence` | Handwritten `Clone` or `Copy` implementations |

Mutable receiver detection covers `&mut self` and `self: &mut Self`. The analyzer matches local unqualified proof declarations and their generated type names. Alias resolution, macro expansion, cfg evaluation, and checker semantics are outside its analysis.

`--message-format=json` emits schema version 1 GDP records with paths, locations, rule codes, and messages. `check` preserves Cargo records and identifies compiler or syntax failure. `doctor` also supports JSON.

`check` and `lint` accept `--message-format=cargo-json` for Cargo/rustc-compatible editor records and primary spans. One final build result covers both stages. See the repository's `docs/EDITOR.md` for rust-analyzer settings. Clippy runs separately.

Requires Rust 1.85 or later. Licensed under MIT or Apache 2.0.
