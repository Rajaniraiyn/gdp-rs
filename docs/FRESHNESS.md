# Evidence and changing state

The [versioned example](../examples/versioned.rs) applies evidence to an atomic storage condition:

```sh
cargo run --example versioned --features macros
cargo test --example versioned --all-features --locked
```

The checker reads the owner and revision together. It issues evidence carrying that revision, branded to the store, user, and project. Another named backend cannot substitute for the checked store.

The consuming rename operation locks the same row, compares its revision, and writes while holding the lock. Success increments the revision. Another permission for the previous revision fails with `Conflict` and performs no write. This includes concurrent writers and ownership changing away and then back.

The mutex models a single-process atomic compare-and-write. A database adapter needs an equivalent atomic conditional update or suitable transaction. Comparing the version in one query and then writing unconditionally leaves a race. A zero-row result may need separate handling to distinguish conflict from deletion.

Every policy-relevant change must invalidate the revision. Independent ACLs, subscriptions, account state, or other rows need coordinated versioning or rechecks inside the atomic operation. Include those inputs in the checked revision.

Revisions must not wrap or be reused after deletion and recreation. The example uses checked arithmetic and fails before changing state on overflow. Production storage also needs revision and identity semantics across restarts.

`Store::change_owner` simulates a separately authorized administrative action; its caller authorization is outside this demonstration.

Consuming a capability prevents reuse of that token. The revision condition coordinates independently issued permissions. Network retries and multi-resource changes may require idempotency keys or transactions.
