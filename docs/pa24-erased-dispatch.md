# Callable object execution

PA24 remains planned. Function items, function pointers and closures with empty
environments now execute through borrowed `dyn Fn` and `dyn FnMut` values.
Generic `FnOnce` consumers can use the ordinary callable implementations for
those references. Calling an unsized `dyn FnOnce` by value is not implemented.
Captured object targets still require checked conversion-specific entry evidence.

Object slots retain the complete parent trait application and an explicit
declared-method or language-call-trait identity. No synthetic method declaration
is inserted into the AST. A checked call selects its slot by the complete
argument tuple and delegated receiver permission. Reference prefixes retain
their required pointer loads; receiver permissions are validated before runtime
erasure. Each source use still passes its ordinary trait and lifetime proofs.

Concrete item, pointer and empty-environment targets share one forwarding entry
across their callable supertrait slots. Function pointers load the pointer from
their actual storage. The wrappers preserve the ordinary Rust ABI, including
aggregate arguments, the hidden aggregate result pointer and diverging results.
Upcasts retain the selected table relationships. Ordinary object methods reuse
the existing mechanical virtual-call emitter.

Runtime forwarding keys contain the complete object shape, slot and physical
reference-prefix depth. Reuse checks the physical ABI explicitly. The selected
source signature remains part of source-plan identity; lifetime differences do
not create duplicate native wrappers or grant another source use permission.
The first invariant run exposed one wrapper per call because LowIR signature
IDs themselves are not physical ABI interning keys. That failed observation is
retained in `obj/parallel/pa24/callable-dispatch/sixth/`.

## Qualification

`pa24/tests/callable-dispatch-outcomes.json` declares twelve accepted complete
programs before qualification. All twelve pass pinned rustc object generation,
the type/constant/LowIR views and independent CPPGM-versus-rustc native execution.
The matrix covers item and pointer targets, empty closures, mutable calls,
generic function items and consumers, nested reference consumers, tuple inputs
and results, inherited callable bounds, upcasts, higher-ranked references and
persistent pointer objects. Existing callable-object lifetime and ownership
negative controls remain in `erased_callables`.

The final focused batch, `obj/parallel/pa24/callable-dispatch/seventh/`, passes
strict all-target/all-feature Clippy, formatting, 123 affected source test groups,
eight native groups, three private invariants, the file audit (1,429 files;
zero findings), layout38 and upstream24+126. The native groups include all twelve
new executions and the existing object-method execution matrix. Successful scoped
cleanup removed 245 generated files, retaining inputs, diagnostics and receipts.

At 8/32/128 repeated calls, one table, one concrete target and one forwarding
function remain shared. Planning work is 106/274/946 steps, exactly seven
additional steps per additional call in this matrix. Separate complete input
applications retain different slots; differing physical reference prefixes share
only ABI-compatible forwarding. These are structural invariants, not a measured
wall-time or whole-compiler complexity claim.

The same batch restores all 1,381 PA20 text oracles byte-for-byte. The selected
constant cache repair restores 62 cases; seven other cases required normalizing
already-opened call-signature roots in the caller's lifetime scope. The signature
record is not a new binder. Nested callable children still retain their own
binders, and projection-free signatures avoid constructing replacement lists.
No references were regenerated. Combined release checks and the frozen
performance comparison remain separate publication gates.
