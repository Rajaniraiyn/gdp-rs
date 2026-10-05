# GDP implementation and coverage

The workspace contains `ghostproof`, its optional declaration macros, and Cargo tooling. Applications own nominal propositions and their checking modules.

## Implemented packages

| Package | Implementation |
| --- | --- |
| `ghostproof` | `no_std` named values and evidence composition; no runtime allocation or executor dependency |
| `ghostproof-macros` | Optional nominal proof declarations, private issuance, owned capabilities, and borrowed views |
| `cargo-gdp` | Cargo check forwarding, workspace selection, syntax checks, and diagnostics about analysis limits |

The default feature set contains the core library. `examples/manual.rs` defines handwritten evidence. The `macros` feature adds build-time declaration generation.

## Naming and invariance

`Named<'id, T>` owns a payload and an invariant `generativity::Id<'id>`. Its constructor consumes a fresh guard. It exposes shared access and consuming extraction, but no clone, mutable access, brand reconstruction, or replacement API.

The naming macro evaluates each expression before shadowing its binding and establishes a separate fresh guard. `with_names!` adds a scoped block returning an ordinary result. Tests cover borrowed values, shadowing, raw identifiers, repeated loop invocations, and nested scopes. Equal raw values named separately remain distinct.

Generated proofs use a function-shaped phantom marker with each lifetime in both input and output positions. Subject brands are invariant. Payloads determine `Send`, `Sync`, and other auto traits.

## Nominal proofs and payloads

`#[proof]` turns a unit or named-field struct into a proof with private fields. `subjects(actor, resource)` adds domain-named accessors alongside positional accessors. Names are distinct, match lifetime order, and exclude generated method names. Every declared lifetime is a subject name. Type parameters, const parameters, defaults, and bounds describe payloads and are preserved. The private `issue` method takes a reference to each matching named subject followed by payload arguments in declaration order.

There is no global unchecked issuer. The defining module and its descendants own logical trust. Keep that module small and put untrusted handlers in sibling modules. A checker may return `Option`, `Result`, or an async future using ordinary application APIs.

Payloads have no automatically generated mutable or extraction accessors. Applications expose deliberate shared accessors or consuming transformations in trusted modules. The declaration macro rejects public payload fields and direct derives of `Default`, `Deserialize`, `Copy`, or `Clone`.

Whole-declaration `cfg` is supported. Put `cfg_attr` before the proof attribute so Rust resolves it first; conditional individual payload fields are rejected with a targeted diagnostic. Subject helper names prefixed with `__GdpSubject`, the `__gdp_view` lifetime, and the `__gdp_brand` field name are reserved. Tuple payload structs and declarations without any subject lifetime are rejected. Declaration validation, option parsing, and code generation live in separate modules.

## Composition and inference

`And<A, B>` stores two actual components. Its shared projections borrow them, and `into_parts` moves them out. `Either<A, B>` stores only its selected alternative and supports borrowing and consuming case analysis. `all!` composes two or more components into a right-nested conjunction. Construction and shared projections support const evaluation. Neither wrapper implements `Copy` or `Clone`.

Applications derive new nominal propositions through explicit trusted functions. The authorization example combines admin and plan evidence into an operation-specific permission. Negative facts use separately declared predicates and checkers.

Sensitive functions require concrete nominal proof types or capabilities.

## Capabilities and transitions

For `Fact<'a, 'b>`, the macro generates `FactCapability<'a, 'b, SubjectA, SubjectB>`. `bind` consumes evidence and matching named subjects. Constructor fields are private. Capabilities expose shared subject and proof accessors plus consuming decomposition, for operations on the bundled resource.

`FactView<'borrow, 'a, 'b, SubjectA, SubjectB>` borrows matching subjects and evidence. Proofs provide `view`; owned capabilities provide `as_view`. Views retain the same invariant subject names, have private constructor fields, and cannot outlive any borrowed owner. Shared operations accept views; consuming operations require owned capabilities.

The validation example bundles an owned sorted vector with data carrying evidence. Appending consumes the old capability, recovers a raw vector, and requires fresh naming and validation before another sorted operation. The changed snapshot receives a fresh name and check.

Reusable facts are borrowed. Consuming operations move their token. Checkers may issue another token after another check.

## Async and mutation boundaries

Runtime tests verify evidence across `.await`, borrowing concurrency, cancellation, and panic unwinding. Downstream fixtures reject moving scoped evidence into a task requiring `'static`. Tests also reject sending named or evidence payloads that contain `Rc` through a `Send` requirement.

Shared `Cell` or lock access can mutate contents, and external database rows can change after a check. The examples use owned immutable snapshots or stable IDs. Atomic effects, permission freshness, and backend isolation remain application responsibilities.

## Cargo checks

The command forwards `check` arguments to Cargo and preserves failed compiler exit status. Its metadata selection handles workspace defaults, explicit local package names or full package IDs, and exclusions. Manifest, configuration, features, and offline or locked options are passed to metadata where applicable. Unsupported package selector forms are rejected for syntax analysis.

The syntax scanner checks local unqualified implementations next to `#[proof]` declarations in source and example directories. Its implemented rules are:

| Rule | Trigger |
| --- | --- |
| `gdp::exported_issuer` | Nonprivate `issue`, `issue_unchecked`, or `new_unchecked` method |
| `gdp::mutable_evidence` | Nonprivate method with an `&mut self` or `self: &mut Self` receiver on a proof, generated capability, or view |
| `gdp::unchecked_construction` | Handwritten `Default` or `Deserialize` implementation on the recognized declaration |
| `gdp::duplicated_evidence` | Handwritten `Clone` or `Copy` implementation on a recognized proof, capability, or view |

The scanner visits source and example directories plus custom library, binary, and example roots reported by Cargo metadata. It follows outlined modules and literal `#[path]` declarations, preserving the different path rules for inline and outlined modules. Files are deduplicated by canonical path. Imports, aliases, expanded macros, cross-module type references, and `cfg` remain unresolved. Symlink files and test fixtures are excluded.

All commands support JSON output through `--message-format=json`. `check` preserves Cargo's JSON records and appends GDP findings and a summary. GDP records have `schema_version: 1` and reasons `gdp-diagnostic`, `gdp-summary`, or `gdp-doctor`. Diagnostics include a rule code, path, one-based line and column, level, and message. A compiler failure produces a summary identifying the compiler stage and retains Cargo's failure exit status. GDP syntax findings remain separate from rustc diagnostics.

Clippy runs independently of the GDP syntax analyzer.

Editor mode `--message-format=cargo-json` emits Cargo/rustc-compatible findings and one final combined build status. Declared module traversal retains target provenance; leftover scanned files use the first product target only for editor association. Unicode and CRLF spans, custom paths, and failure status are tested. See [editor integration](EDITOR.md).

The [versioned write example](../examples/versioned.rs) connects evidence to a store, user, and project, carrying a checked revision to an atomic compare-and-write. Five tests cover valid writes, revocation, owner changes away and back, missing versus denied resources, and concurrent permissions. See [freshness assumptions](FRESHNESS.md).

`const_assert!` checks static predicates during compilation. The configuration example shares a const predicate with a runtime checker and carries its bound through a const generic. Source parsing uses Syn 3 with package-specific feature sets.

## Acceptance coverage

The compile-contract harness creates a real downstream package with the library renamed to `gp`. Fixtures cover successful compilation, including the README example, and rejected code with expected diagnostic fragments or Rust error codes. They exercise dependency renaming and cross-crate privacy on the supported compilers.

| Contract | Evidence |
| --- | --- |
| Correct protected calls compile | Native, payload, generic payload, and arbitrary-arity success fixtures |
| Every relational subject remains distinct | Wrong user, wrong project, wrong binding, and three separate ternary-subject fixtures |
| Facts cannot be forged outside their owner | Private issuer, forged proof, and forged capability fixtures |
| Names are fresh and scoped | Guard reuse, loop unification, lifetime widening, escaping value and proof fixtures |
| Mutation access is restricted | Named assignment, payload field access, and mutable capability accessor fixtures |
| Ownership is preserved | Repeated capability consumption, proof cloning, and duplicate conjunction fixtures |
| Borrowed views retain their bounds | Shared views compile; wrong subjects, forged fields, escaping owners, and attempts to consume views fail |
| Alternatives and inference require evidence | Wrong-kind, invalid alternative, and invalid implication fixtures |
| Async and auto-trait bounds | Static task and non-Send payload fixtures; runtime borrowing and drop tests |
| Declaration diagnostics | Unsupported forms, reserved names, public fields, derives, and conditional payload fixtures |
| Cargo command behavior | Selection tests, custom module paths, JSON records, compiler failure propagation, and syntax rule tests |

CI runs the feature matrix on Rust 1.85 and stable, plus examples, Clippy, formatting, and rustdoc. Embedded compilation verifies the core on `thumbv7em-none-eabi`. The implementation forbids unsafe code.

## Performance and release status

Runtime tests assert zero size for phantom proofs and no extra storage for a named `u32`. `bench/run.py` records a narrow runtime comparison, fixture compile timings, and executable sizes in `bench/results.json`. Those observations include toolchain and machine details. Payload-bearing proofs store their declared payloads.

Crates are unpublished. Publish the macro package before the library that depends on it. The API is pre-release.

The checkout also includes local Cargo aliases, an installable `skills/gdp-rs` agent skill, dependency update configuration, contribution guidance, and an original SVG logo.
