# GDP implementation and coverage

The repository now contains a general GDP library and optional tooling. The library package is named `ghostproof` because `gdp_rs` already exists. The implementation uses nominal application-owned propositions rather than a public generic proof factory. This keeps issuance authority inside the checking module and avoids introducing a public trait that arbitrary downstream evidence could implement to impersonate an existing proposition.

## Implemented packages

| Package | Implementation |
| --- | --- |
| `ghostproof` | `no_std` named values and evidence composition; no runtime allocation or executor dependency |
| `ghostproof-macros` | Optional nominal proof declarations, private issuance, owned capabilities, and borrowed views |
| `cargo-gdp` | Cargo check forwarding, workspace selection, syntax checks, and diagnostics about analysis limits |

The root package builds without procedural macros by default. `examples/manual.rs` shows how to define an invariant private proof by hand. Enabling `macros` adds build-time dependencies, without changing the core runtime model.

## Naming and invariance

`Named<'id, T>` owns a payload and an invariant `generativity::Id<'id>`. Its constructor consumes a fresh guard. It exposes shared access and consuming extraction, but no clone, mutable access, brand reconstruction, or replacement API.

The naming macro evaluates each expression before shadowing its binding and then establishes a separate fresh guard. Tests cover borrowed values, shadowing, raw identifiers, repeated loop invocations, and nested scopes. Equal raw values named separately remain distinct.

Generated proofs use a function-shaped phantom marker with each lifetime in both input and output positions. This preserves invariance in every subject without imposing unrelated auto-trait restrictions. Payloads still determine their normal `Send`, `Sync`, and other auto-trait behavior.

## Nominal proofs and payloads

`#[proof]` turns a unit or named-field struct into a proof with private fields. Every declared lifetime is a subject name. Type parameters, const parameters, defaults, and bounds describe payloads and are preserved. The private `issue` method takes a reference to each matching named subject followed by payload arguments in declaration order.

There is no global unchecked issuer. The defining module and its descendants own logical trust. Keep that module small and put untrusted handlers in sibling modules. A checker may return `Option`, `Result`, or an async future using ordinary application APIs.

Payloads have no automatically generated mutable or extraction accessors. Applications expose deliberate shared accessors or consuming transformations in trusted modules. The declaration macro rejects public payload fields and direct derives of `Default`, `Deserialize`, `Copy`, or `Clone`.

Whole-declaration `cfg` is supported. Put `cfg_attr` before the proof attribute so Rust resolves it first; conditional individual payload fields are rejected with a targeted diagnostic. Subject helper names prefixed with `__GdpSubject`, the `__gdp_view` lifetime, and the `__gdp_brand` field name are reserved. Tuple payload structs and declarations without any subject lifetime are rejected. Declaration validation and code generation live in separate modules.

These restrictions reduce accidental bypasses. They cannot prevent the trusted module from intentionally writing an invalid checker, unchecked constructor, or duplication implementation.

## Composition and inference

`And<A, B>` stores two actual components. Its shared projections borrow them, and `into_parts` moves them out. `Either<A, B>` stores only its selected alternative and supports borrowing and consuming case analysis. Neither wrapper implements `Copy` or `Clone`.

Applications derive new nominal propositions through explicit trusted functions. The authorization example combines admin and plan evidence into an operation-specific permission. It performs no additional check during that inference. Failed checks do not automatically produce negative facts.

The generic library does not define an unsealed evidence trait that sensitive APIs should accept from arbitrary implementers. Application-sensitive functions require concrete nominal proof types or capabilities. The public API therefore supports general application-defined facts without creating an unchecked generic route to manufacture another module's evidence.

## Capabilities and transitions

For `Fact<'a, 'b>`, the macro generates `FactCapability<'a, 'b, SubjectA, SubjectB>`. `bind` consumes evidence and matching named subjects. Constructor fields are private. Capabilities expose shared subject and proof accessors plus consuming decomposition, so applications can implement operations without accepting a second unrelated resource.

`FactView<'borrow, 'a, 'b, SubjectA, SubjectB>` borrows matching subjects and evidence. Proofs provide `view`; owned capabilities provide `as_view`. Views retain the same invariant subject names, have private constructor fields, and cannot outlive any borrowed owner. A view cannot satisfy an operation requiring the owned capability, so it does not duplicate a consumable permission. Applications choose explicitly which operations accept shared evidence.

The validation example bundles an owned sorted vector with data carrying evidence. Appending consumes the old capability, recovers a raw vector, and requires fresh naming and validation before another sorted operation. This is the initial owned typestate pattern; no assertion survives unchecked changes automatically.

Reusable facts can be borrowed. A consumed token cannot be reused, but applications can choose to issue another token after another check. This is affine ownership, not a guarantee of durable exactly-once execution.

## Async and mutation boundaries

Runtime tests verify evidence across `.await`, borrowing concurrency, cancellation, and panic unwinding. Downstream fixtures reject moving scoped evidence into a task requiring `'static`. Tests also reject sending named or evidence payloads that contain `Rc` through a `Send` requirement.

The generated phantom fields do not claim to freeze payloads. Shared `Cell` or lock access can still mutate contents, and external database rows can change after a check. The examples use owned immutable snapshots or stable IDs. Atomic effects, permission freshness, and backend isolation remain application responsibilities.

## Cargo checks

The command forwards `check` arguments to Cargo and preserves failed compiler exit status. Its metadata selection handles workspace defaults, explicit local package names or full package IDs, and exclusions. Manifest, configuration, features, and offline or locked options are passed to metadata where applicable. Unsupported package selector forms are rejected for syntax analysis.

The syntax scanner checks local unqualified implementations next to `#[proof]` declarations in source and example directories. Its implemented rules are:

| Rule | Trigger |
| --- | --- |
| `gdp::exported_issuer` | Nonprivate `issue`, `issue_unchecked`, or `new_unchecked` method |
| `gdp::mutable_evidence` | Nonprivate method with an `&mut self` receiver on a proof, generated capability, or view |
| `gdp::unchecked_construction` | Handwritten `Default` or `Deserialize` implementation on the recognized declaration |

The scanner visits source and example directories plus custom library, binary, and example roots reported by Cargo metadata. It follows outlined modules and literal `#[path]` declarations, preserving the different path rules for inline and outlined modules. Files are deduplicated by canonical path. These filesystem rules do not resolve imports, aliases, expanded macros, out-of-line cross-module type references, or `cfg`. Symlink files and test fixtures are excluded. The scanner never claims that a clean scan proves all authorization paths are covered.

All commands support JSON output through `--message-format=json`. `check` preserves Cargo's JSON records and appends GDP findings and a summary. GDP records have `schema_version: 1` and reasons `gdp-diagnostic`, `gdp-summary`, or `gdp-doctor`. Diagnostics include a rule code, path, one-based line and column, level, and message. A compiler failure produces a summary identifying the compiler stage and retains Cargo's failure exit status. GDP syntax findings remain separate from rustc diagnostics.

Stock Clippy remains independently usable. Dylint or a custom rustc driver would add compiler maintenance without improving the implemented syntax rules, so no compiler plugin is required. Add such a package only when a precise semantic rule has examples, expected diagnostics, and a measured false-positive rate.

## Acceptance coverage

The compile-contract harness creates a real downstream package with the library renamed to `gp`. It checks 53 cases: ten successful compilations, including the actual README example, and 43 rejected cases with expected diagnostic fragments or Rust error codes. It does not rely on snapshots tied to one compiler version. This tests dependency renaming and cross-crate privacy on the supported compilers.

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
| Async and auto-trait bounds are honest | Static task and non-Send payload fixtures; runtime borrowing and drop tests |
| Macro diagnostics are intentional | Unsupported forms, reserved names, public fields, derives, and conditional payload fixtures |
| Cargo behavior is usable | Selection tests, custom module paths, JSON records, compiler failure propagation, and syntax rule tests |

CI runs the feature matrix on Rust 1.85 and stable, plus examples, Clippy, formatting, and rustdoc. Embedded compilation verifies the core on `thumbv7em-none-eabi`. The implementation contains no unsafe code; Miri is not a substitute for checking logical policy truth.

## Performance and release status

Runtime tests assert zero size for phantom proofs and no extra storage for a named `u32`. `bench/run.py` records a narrow runtime comparison, fixture compile timings, and executable sizes in `bench/results.json`. Those observations include toolchain and machine details. Payload-bearing proofs naturally have payload storage, and these observations do not establish general performance bounds.

Package names were unregistered when checked through the crates.io API during implementation. They have not been reserved or published. License files, MSRV declarations, a lockfile, documentation, CI, and benchmark tooling are present. Repository metadata points to `Rajaniraiyn/gdp-rs`; GitHub publication and crates.io publication are separate actions. Publish the macro dependency before publishing the root library with registry dependencies. A stable API commitment remains a separate release decision.

The checkout also includes local Cargo aliases, an installable `skills/gdp-rs` agent skill, dependency update configuration, contribution guidance, and an original SVG logo. The skill describes actual privacy, ownership, lifetime, and syntax-analysis boundaries. No global toolchain or editor configuration is changed.

All implementation stages in the design have a concrete initial deliverable. Conditional compiler-plugin work, universal quantification, a theorem solver, serialization of scoped proofs, and backend-specific transaction adapters are outside this initial API. Additions should preserve the existing compile contracts rather than broaden claims without evidence.
