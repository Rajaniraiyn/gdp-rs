# Design

GDP for Rust connects checks to operations through scoped value identity and nominal evidence. Authorization, validation, relationships, and state transitions use the same mechanism.

## Value identity

`Named<'id, T>` owns a payload and an invariant lifetime brand from `generativity`. Construction consumes a fresh guard. Each invocation of `name!` establishes a separate guard for every value. `with_names!` encloses naming and operations in a block that returns an ordinary result.

The wrapper exposes shared payload access and consuming extraction. Brand invariance prevents substituting another named value. The payload's borrow lifetime remains separate from its brand. See [Rust variance](https://doc.rust-lang.org/reference/subtyping.html).

## Evidence

Applications declare nominal facts in checking modules. A generated proof stores an invariant phantom marker for every subject lifetime and type parameter, plus its declared payload. Phantom evidence is zero-sized; payload evidence stores its payload.

The declaring module and its descendants own issuance authority through private fields and a private `issue` method. Checkers implement the predicate and return evidence with `Option`, `Result`, or ordinary async APIs.

Every lifetime in a proof declaration identifies a subject. Type and const parameters describe the proposition or payload. The attribute macro preserves bounds and source spans. Its optional `subjects(...)` argument supplies domain names for accessors.

## Composition and operations

`And` stores both supplied components. `Either` stores the selected alternative. Borrowed projections share existing evidence; consuming projections recover ownership. `all!` builds right-nested conjunctions. Inference functions issue a new nominal fact through its owning module.

A capability owns matching named subjects and evidence. A view borrows them. Protected operations use the bundled subjects. Taking a capability by value consumes the permission; taking a reference allows reuse.

`const_assert!` evaluates static conditions during compilation. Const predicates can serve both static configuration validation and runtime checkers. These assertions validate constants; scoped facts are issued by their checking modules.

## Mutable state and async

Shared references permit interior mutation in payloads. Facts about remote state need storage-level freshness. The versioned example compares a checked revision and writes under one lock. Production adapters need equivalent atomic conditions covering all mutable policy inputs. See [freshness](docs/FRESHNESS.md).

Scoped evidence supports `.await` and borrowing concurrency. Tasks requiring `'static` create their naming scope inside the task. Payloads determine normal `Send` and `Sync` bounds.

## Tooling

The library targets stable Rust 1.85 and `no_std`. Procedural macros are optional build-time dependencies. Source parsing uses Syn 3. Shared workspace metadata and dependency declarations keep package settings consistent.

`cargo gdp` forwards compiler checks and scans local declarations for issuance, mutation, construction, and duplication conventions. Cargo metadata provides package selection and source roots. Findings support text, GDP JSON, and Cargo-compatible editor output.

The analyzer operates on syntax. Aliases, macro expansion, cfg evaluation, and policy semantics are outside its scope. Clippy and rust-analyzer retain their normal compiler checks.

## Verification

Runtime tests cover checker results, payloads, borrowing, async execution, cancellation, unwinding, and atomic writes. Downstream compile fixtures check subject mismatches, privacy, ownership, scope escape, macro inputs, and renamed dependencies. The README example compiles through the same harness.

CI runs Rust 1.85 and stable, default and macro features, embedded compilation, examples, Clippy, formatting, and rustdoc. [The coverage ledger](docs/IMPLEMENTATION.md) records the contracts; [benchmarks](bench/README.md) record local timing and size observations.
