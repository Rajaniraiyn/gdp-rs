# Cargo GDP checks

Run `cargo gdp check` to forward options to `cargo check` and then perform narrow syntax checks for evidence bypass conventions. `cargo gdp lint` performs only the syntax checks, and `cargo gdp doctor` reports selected packages and analysis limits.

The command recognizes workspace defaults, `--workspace`, exclusions, exact local package names, and full package IDs. It checks source and example directories and custom Cargo source roots, following outlined modules and literal path attributes. Its rules detect exported issuing methods, public mutable proof methods, and unchecked construction traits near proof declarations.

Use `--message-format=json` for schema version 1 GDP records. Findings include source paths, one-based line and column, rule identifiers, and messages. `check` preserves Cargo JSON output and reports whether failure occurred during the compiler or syntax stage. `doctor` can report the selected packages and source roots as JSON too.

These rules do not run inside stock Clippy. Syntax analysis does not expand macros, resolve aliases, evaluate conditional compilation, or verify the truth of authorization policies. A clean scan does not certify all protected operations.

Requires Rust 1.85 or later. No command changes manifests or toolchains. Licensed under MIT or Apache 2.0.

`check` and `lint` accept `--message-format=cargo-json` for Cargo/rustc-compatible editor records and primary spans. The command reports one final build result after both stages. See `docs/EDITOR.md` in the repository for a rust-analyzer override and analysis limits.

`gdp::duplicated_evidence` flags handwritten `Clone` and `Copy` implementations on recognized proofs, capabilities, and views. Borrow reusable evidence rather than duplicating an owned permission.
