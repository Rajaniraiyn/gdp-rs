# Contributing

The workspace has a `no_std` library, an optional procedural macro, and a Cargo subcommand. Rust 1.85 is the minimum supported version. Changes should preserve the library's lack of allocation and runtime dependencies unless the feature explicitly requires them.

Run the checkout's native shortcuts before opening a pull request:

```sh
cargo test-all
cargo test --no-default-features --locked
cargo fmt --all -- --check
cargo lint-all
cargo gdp check --workspace --all-features --all-targets --locked
RUSTDOCFLAGS="-D warnings" cargo docs-all
```

For core changes, also check the embedded target after installing it with `rustup target add thumbv7em-none-eabi`:

```sh
cargo check -p ghostproof --no-default-features --target thumbv7em-none-eabi --locked
```

Tests under `tests/ui` compile in a real downstream package with a renamed dependency. Use an `error-pattern` comment for compile-fail cases and choose a stable diagnostic fragment. Test a meaningful boundary, not an incidental expansion detail. Check new macro behavior on both Rust 1.85 and stable.

Keep public issuers and unchecked construction out of generated evidence. Document the limits of syntax-only lint rules. Changes involving external state must explain how an application should keep the checked fact valid until the effect.

For benchmarks, follow [bench/README.md](bench/README.md) and include the measurement environment. For agent-skill changes, keep `skills/gdp-rs/SKILL.md` consistent with the real API and validate its frontmatter with a skill validator when available.

Crates are unpublished and the API is pre-release.
