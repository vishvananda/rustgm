# Typed body identity for proof pairing

Body contracts and their lifetime/ownership proofs now pair by `BodyKey`:
`Declaration(DefId)`, `Closure(ClosureId)` or `Anonymous(NodeKey)`. Diagnostic
provenance remains the actual source node. Several metadata declarations may
share that provenance; it does not make their contracts interchangeable.

Constructor contracts select the constructor DefId independently of their
nominal generic environment. Template, instance, ordinary body and loan proof
pairing uses the typed key. Explicit pairs continue to validate diagnostic
provenance and selected instance identity. The sealed instance proof test rejects
an anonymous-key proof paired with a declared body even at the same source node.
The public region text view is unchanged.

The worker base predates PA24's `Owner::Closure` body owner. Integration must map
`Owner::Closure { identity, .. }` to `BodyKey::Closure(identity)` and set the same
key on the closure contract. PA25's expression collector keeps anonymous keys;
selected declaration constant bodies retain their function/initializer identity.

Focused checks pass: strict all-target/all-feature Clippy, five sealed instance
proof tests, ten projection debt tests, 22 caller-outlives tests, eight quantified
proof tests and three proof-view tests. The larger nominal checkpoint additionally
passes its source/native controls. Source and logs for the shared guarded batch
are under `obj/parallel/pa26/nominals/`; no text references are regenerated.
