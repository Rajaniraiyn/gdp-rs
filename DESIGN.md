# General GDP library for Rust

This is the proposed implementation plan for this repository. The goal is a general Ghosts of Departed Proofs library that connects runtime checks to the exact values used by dependent operations. Authorization is one application. Validation, relationships between values, and state transitions must also fit the design.

The initial implementation now lives in this repository as the `ghostproof` library, optional `ghostproof-macros` package, and `cargo-gdp` command. This document preserves the design scope and stage gates; [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) records the implemented APIs, coverage, and deferred conditional work. Packages are unpublished and APIs may change before a stable release.

## Scope and guarantees

The core guarantee is that an operation requiring evidence cannot successfully execute through its protected API unless the caller supplies evidence of the required kind about the correct named values. This assumes correctly implemented trusted checkers and safe callers using the intended API boundary.

The compiler does not establish that a check implements the correct business rule. It does not discover every sensitive operation, validate database isolation, or make external state stop changing. A panic or nonterminating expression can typecheck in a proof position without producing a proof or successfully executing the operation.

| Concern | Planned enforcement | Limit |
| --- | --- | --- |
| Missing evidence | Required function arguments or capability types | Every sensitive entry point must adopt the contract |
| Wrong named value | Fresh invariant brands on values and evidence | Unchecked brand reconstruction must remain inaccessible |
| Wrong proposition | Nominal proof types or sealed contracts | Broad application traits must not accept fabricated evidence |
| Proof construction | Private fields and authority owned by the checking module | The defining module and its descendants are trusted |
| Mutation of checked data | Restricted access and borrowing where sufficient | Interior mutability and external state need explicit treatment |
| Reusing a consumed permission | Ownership with no duplication API | Prevents reuse of that token, not global duplication of an action |
| Exported bypasses | API review and optional targeted lints | Syntax checks cannot certify all security boundaries |

## Core value naming

Use a fresh lifetime brand for each named value. `Named<'id, T>` owns its value and keeps its brand private and invariant. A fresh guard is consumed when constructing the wrapper, so the same guard cannot brand two unrelated values.

Use the existing [generativity mechanism](https://docs.rs/generativity/latest/generativity/) for the first prototype. Its guards provide fresh invariant lifetime brands. Do not substitute arbitrary caller lifetimes, public reusable brand IDs, a counter hidden behind casts, or static marker types reused across invocations.

The initial access API returns shared references and allows consuming the wrapper to recover the raw value. It exposes no mutable access, arbitrary value replacement, generic brand-preserving mapping, or constructor from a borrowed reusable brand. Safe cloning, if added, must document exactly which identity it preserves and how it interacts with mutable payloads.

Naming an ID means naming that ID value. It does not identify a permanently unchanged database row. Naming an owned object with interior mutability does not freeze its contents. Shape validation should initially use immutable snapshots or a wrapper that owns the validated value and prevents invalidating mutation.

Keep the brand lifetime separate from the lifetime of borrowed payloads and proof borrows. Verify invariance for every subject position rather than assuming an outer wrapper protects all parameters. [Rust variance rules](https://doc.rust-lang.org/reference/subtyping.html)

Provide both an explicit guard API and a small declaration macro. Prototype macro ergonomics in ordinary blocks before promising a callback API that returns futures. Scoped brands may not escape the scope that establishes freshness; returning ordinary owned results is allowed.

## Propositions and trusted evidence

Applications declare nominal propositions and implement their own checks. The first examples cover a unary fact, a binary relationship, and a fact with three subjects. Later, use subject tuples or a macro to support arbitrary practical arity without a family of handwritten APIs.

For example, `ProjectAdmin<'user, 'project>` relates a user and a project, while `Sorted<'sequence>` concerns one immutable sequence. `BelongsTo<'child, 'parent>` demonstrates a relationship outside authorization.

Start with private application proof structs. Once their guarantees work, prototype a common evidence representation whose constructor requires an authority token held only by the defining module. A public generic `prove`, `From<()>`, `Default`, deserializer, or unchecked constructor must never mint protected evidence.

A declaration macro may generate a proof type and private issuing function inside a dedicated module. That module and its descendants form the trusted boundary. Do not expose an issuer to the entire crate merely to make generated code convenient. A published library cannot determine whether application code asserting its own fact is honest.

Separate safe library construction from logical trust. Trusted checkers should not need Rust `unsafe`: they assert propositions through an ordinary private API. Use `unsafe` only when a genuine Rust memory safety obligation requires it. If an unsafe operation relies on GDP evidence, document why the trusted checker establishes the memory safety precondition.

Support evidence carrying data, such as a verified row or selected policy branch. Preserve the connection between that data and its subjects. A phantom proof can have zero size; data carrying evidence has the size of its payload. Measure overhead rather than promising every representation is free.

## Proof composition and inference

Composition operates on existing evidence. It must not fabricate application facts.

| Operation | Proposed behavior |
| --- | --- |
| Conjunction | Combine owned evidence for A and B, or borrow both without duplicating tokens |
| Projection | Borrow one component, or consume a conjunction and recover its components |
| Disjunction | Select a branch containing actual evidence, represented by an enum |
| Case analysis | Match the selected branch and handle each policy alternative |
| Weakening | Apply an explicitly trusted rule, such as owner implies member |
| Evidence mapping | Transform payloads only while preserving the declared proposition |

Represent inference rules explicitly. The application owns the trust in a rule such as owner implies member; the library does not infer that implication from names. Keep predicates nominal so an unrelated crate cannot silently redefine a proposition owned by another crate.

No automatic proof of a negative fact follows from a failed positive check. A timeout, missing data, or rejected authorization may have different meanings. Negation needs a separately defined proposition and checker with explicit semantics.

Do not start with a solver, arbitrary universal quantification, or a promise of full dependent types. Prototype scoped packaging of a value with its evidence when an example needs it. Any future existential API must retain freshness and prevent subjects from escaping through erased wrappers.

## Capabilities and typestate

Offer a low-level proof API for composition and an optional capability wrapper for common operations. A capability bundles the relevant resource, evidence, and actor binding so callers cannot accidentally pair it with another subject.

The intended application experience is a checker returning an authorized handle, followed by operations on that handle. Sensitive operations must use the bundled resource; they must not take an unrelated raw resource ID alongside the capability.

Capability constructors remain restricted to trusted checks or transformations of existing valid evidence. Low-level database access belongs behind the protected application boundary. A library cannot prevent callers with independent database credentials from bypassing it.

Use typestate for transitions that the wrapper owns. For example, validating an owned immutable value can return `Validated<T>`, and a consuming transition can return a new state with fresh evidence. Do not claim that phantom state remains accurate after unrestricted external mutation.

Evidence is not automatically `Copy` or `Clone`. Reusable facts may expose deliberate sharing or borrowing. Consumable permissions own their token and offer no duplication method. Rust ownership enforces at most one consumption per token; durable single-use effects need database coordination.

## Async and external state

The library must fit existing `async fn`, `Result`, and database APIs without selecting an executor or policy engine. Preserve distinct errors for denied access, missing resources, and backend failures where applications need them.

Test evidence across `.await`, multiple concurrent futures inside the scope, cancellation, and unwinding. Derive `Send` and `Sync` behavior from actual payloads and lifetimes; never add blanket unsafe implementations to satisfy an executor.

Scoped evidence cannot generally move into a task requiring a `'static` future. Create the naming scope inside that task, use borrowing concurrency when supported, or recheck in the destination. Passing branded evidence across processes or deserializing it is outside the core contract.

For mutable external facts, document several application choices: recheck at the operation, combine check and write atomically, use versioned state, or constrain operations to an appropriate transaction. Each choice requires backend-specific semantics. A transaction lifetime alone does not prove that its isolation level prevents permission races, and consuming a token does not make it fresh.

## Native Rust integration

The core is proposed to use `core`, with no default allocation or executor requirement. Verify that the selected branding dependency supports the intended targets before committing to `no_std`. Optional convenience layers may use `alloc` or `std` without changing the core guarantees.

Choose and test an explicit minimum supported Rust version. Use modern stable features when they simplify an actual API. Associated types, generic associated types, const generics, and async traits are options, not milestones to include for their own sake. Prototype inference, trait coherence, and cross-crate extension before committing to a generalized trait hierarchy.

Ordinary consumers should use `cargo check`, `cargo test`, `cargo clippy`, and `cargo doc`. Keep compiler failures meaningful with concrete nominal types, short aliases, and documented examples. Use `#[must_use]` for actionable discarded results and `#[diagnostic::on_unimplemented]` where a trait contract benefits from a custom explanation. The latter is a diagnostic hint, not a way to rewrite all compiler errors. [Rust diagnostics](https://doc.rust-lang.org/reference/attributes/diagnostics.html)

Macros are an optional layer. They generate auditable ordinary Rust and maintain accurate source spans. They cannot access whole-program type information. Renamed dependencies, module privacy, nested scopes, cross-crate expansion, and raw identifiers belong in macro tests. [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)

## Optional Cargo commands and lints

Stock Clippy does not load arbitrary project lint libraries. Its new-lint workflow adds checks to Clippy itself. Do not promise that installing this library creates a `clippy::gdp` namespace. [Clippy development](https://doc.rust-lang.org/clippy/development/adding_lints.html)

If needed later, a proposed `cargo-gdp` binary can offer `cargo gdp check` and `cargo gdp doctor`. It can read Cargo metadata, forward checks, and produce documented diagnostics. Respect workspace selection, targets, features, and existing configurations. Do not quietly change manifests or toolchains. [Cargo external tools](https://doc.rust-lang.org/cargo/reference/external-tools.html)

A stable syntax analyzer can catch narrowly defined declaration mistakes. It must report that it does not fully resolve aliases, macros, conditional compilation, or inferred types. Never present its success as a certification of all authorization paths.

Use an optional Dylint package if examples demonstrate a need for semantic checks. [Dylint](https://github.com/trailofbits/dylint) supports custom dynamically loaded lints and workspace configuration. Keep compiler-coupled maintenance isolated from the stable core, pin and test the lint toolchain, and distinguish its diagnostics from stock Clippy.

Candidate checks include exporting an issuing authority, unchecked public proof constructors, and capabilities that expose invalidating mutation. Start with precise rules and measure false positives. Missing proofs and mismatched subjects remain compiler responsibilities. Broad claims such as detecting every protected database bypass need a separately defined analysis model.

## Packages and naming

Start with one library package and examples. Split packages only when implemented functionality needs different dependencies or release requirements. The eventual structure may include the core library, a procedural macro crate, a Cargo command, and a separately maintained lint package.

The published [gdp_rs crate](https://docs.rs/gdp_rs/latest/gdp_rs/) already provides GDP-related types. Treat the repository name as a working name, choose an available package identity before publishing, and review existing implementations before duplicating mechanisms. Also review [mononym](https://docs.rs/mononym/latest/mononym/) and [departed](https://docs.rs/departed/latest/departed/). No compatibility or maintenance assessment has been completed yet.

## Coverage and acceptance criteria

Coverage means covering the specified guarantees and failure paths. A line coverage percentage alone cannot demonstrate them.

| Area | Required successful cases | Required rejected or failing cases |
| --- | --- | --- |
| Naming | Multiple values, nested scopes, borrowed payloads | Brand reuse, escaping subjects, lifetime widening |
| Proofs | Unary and relational checks, payload evidence | Forged fields, wrong kind, wrong subject in every position |
| Composition | Conjunction, alternatives, trusted inference | Invented branch, invalid implication, duplicated consumable evidence |
| Capabilities | Bundled operations and consuming transitions | Unrelated target, unchecked construction, repeated token consumption |
| Mutation | Immutable snapshots and supported owned transitions | Replacement, mutation paths promised to be excluded |
| Async | Awaiting, supported borrowing concurrency, cancellation | Sending scoped evidence into unsupported static tasks |
| Extension | Downstream proposition modules and protected APIs | Downstream construction of another module's evidence |
| Tooling | Macro expansion and accurate diagnostics | Dependency rename failures, privacy leaks, false guarantees |

Compile-pass examples must accompany compile-fail tests so a rejected example cannot pass merely because the API is unusable. Use downstream fixture crates to test real privacy and trait boundaries. Include repeated naming inside loops, shadowing, same raw value named separately, and different values with the same domain type.

Runtime tests exercise checks and evidence payloads. UI tests verify diagnostics for relevant compiler versions. Miri is useful for any unsafe implementation or memory-safety-sensitive examples; it does not verify business policy truth. Benchmark compile time, binary size, allocations, and representative runtime operations before publishing performance claims.

Test the minimum supported version and current stable Rust. Add feature and target combinations once those features exist. Keep compiler-specific diagnostic snapshots separate from portable guarantee tests.

## Delivery stages

| Stage | Deliverable | Gate before proceeding |
| --- | --- | --- |
| 0 | Prototype fresh naming and private relational proofs | Wrong subjects and escaping brands fail; valid downstream use compiles |
| 1 | Small general core and immutable validation example | Authorization and a non-authorization example use the same naming mechanism |
| 2 | Proof composition and trusted inference | Composition preserves subject identity and consumption rules |
| 3 | Ergonomic capabilities and async example | Protected operations cannot target a different resource; documented async cases pass |
| 4 | Optional declaration macros and improved diagnostics | Generated privacy and cross-crate tests pass; expansion is auditable |
| 5 | Cargo helpers and selected custom lints | Each rule covers a demonstrated gap with acceptable false positives |
| 6 | Release preparation | Package name, MSRV, feature matrix, documentation, and performance claims are verified |

Stage 0 should decide whether lifetime branding provides adequate ergonomics, whether general evidence can preserve module authority, and what mutation contract the library can honestly support. Resolve those questions with small compiling examples before expanding the public API.

The first implementation task is a minimal crate with two separately named projects, one user, a trusted relationship checker, and an operation requiring its evidence. Pair that with an immutable validation example. These examples establish general GDP behavior before frameworks or compiler tooling enter the repository.
