# Editor diagnostics

`cargo gdp check --message-format=cargo-json` emits GDP findings using Cargo's `compiler-message` structure and rustc primary spans. Package IDs and target metadata come from Cargo. Existing compiler output is forwarded. One final `build-finished` record reports the combined compiler and GDP result.

The existing `--message-format=json` mode retains its schema version 1 GDP records. The tool-specific `cargo-json` format is translated to `json` before invoking Cargo. Use ordinary `json` with `doctor`.

## rust-analyzer

Configure your editor's check override with this argument array:

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "gdp", "check",
    "--workspace", "--all-features", "--all-targets",
    "--message-format=cargo-json"
  ]
}
```

Use this object in VS Code workspace settings. Other LSP clients use their own configuration format. See the [rust-analyzer reference](https://rust-analyzer.github.io/book/configuration.html#rust-analyzercheckoverridecommand). Adjust feature and target options to the project, especially when features are mutually incompatible.

The checkout's Cargo alias builds the local tool. Downstream projects must first install `cargo-gdp`. The override replaces the editor's usual on-save command; keep running Clippy separately.

## Source spans

Findings cover source syntax. Spans highlight the first character of the relevant identifier and account for Unicode and CRLF byte offsets.

Outlined modules retain their originating Cargo target. Leftover files found by directory scanning use the package's first product target for editor association. These files may belong to inactive targets.

Tests cover custom nested source paths, spans, compiler failures, format normalization, and final build status. The protocol follows the [Cargo JSON format](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages) and [rustc diagnostic format](https://doc.rust-lang.org/rustc/json.html).
