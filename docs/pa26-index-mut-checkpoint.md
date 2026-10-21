# PA26 mutable indexing checkpoint

Canonical `IndexMut<Idx>` inherits the existing `Index<Idx>::Output` identity.
The resolver appends the new hook without changing existing hook order and
loads Index before allocating IndexMut. Neutral output metadata distinguishes
an owned declaration from an inherited declaration and its actual trait owner.
IndexMut declares no second Output member. Its ordinary trait header requires
`Self: Index<Idx>`, and its method contract is
`index_mut(&'a mut Self, Idx) -> &'a mut Index::Output` with an unsized output
permitted. Normal conformance rejects a missing supertrait, an invented Output
member or an incompatible reference signature.

Stores, primitive compound assignments, mutable borrows and mutable raw-address
formation request a mutable place before operand inference. Sparse body-local
facts propagate that demand through nested fields and indexes. At a dereference,
the consumer waits for the operand's reference/pointer type: an indexed mutable
reference requires mutable indexing, while copying a raw pointer from a shared
index does not. A later mutable-reference consumer refines the original pending
method/call slots in place. It keeps the same canonical Output type and emits
no duplicate selected call facts. Ordinary shared reads need no mutation index
allocation. Isolated closure bodies keep their own facts.

The existing selected-call and receiver-borrow pipeline performs lowering. The
method's returned reference becomes an ordinary place, retaining normal
mutability, ownership and lifetime checks. There is no new IR dispatch or raw
pointer permission bypass. Plain assignment and primitive compound assignment
retain right-before-left runtime evaluation; index receiver borrowing still
precedes index-argument evaluation. Native side-effect controls distinguish the
shared getter from the mutable getter and verify this ordering.

The 33 declared Rust 1.98.0 object-qualified cases are in
`pa26/tests/index-mut-outcomes.json`: nineteen accepted cases, twelve language
rejections and two explicitly valid Rust boundaries. Capturing closures remain
an unfinished PA24 prerequisite, so their mutation is retained as a boundary;
a noncapturing closure taking an explicit mutable parameter executes correctly.
Custom compound assignment still needs the pending AddAssign family. Primitive
compound assignment through a mutable index is implemented. Neither boundary
is described as a Rust language rejection.

Fourteen independently compared native programs cover replacement, borrowing,
primitive compound mutation, nested indexes, generic inherited output, tied
lifetimes, mutable raw addresses, mutable-reference dereference chains, shared
raw-pointer dereferences, side-effect order, unsized slice outputs and mutation
through an explicit closure parameter. Tests also assert that IndexMut's output
is exactly the canonical Index member and that its receiver/result are mutable
references tied by one callable binder. Negatives cover immutable receivers,
shared references, missing IndexMut, overlapping loans, receiver/index conflicts,
supertrait/conformance errors and escaping mutable references.

Strict all-target/all-feature Clippy passes. Fourteen focused test groups pass
across the operator, call-operator, static-permission and syntax-transfer targets,
including earlier operator natives and diagnostic controls. The architecture
audit reports 1,267 files and zero findings; the 38-stage layout check, 24 pinned
upstream fixtures and 126 Reference files pass. The native command is:

```sh
cargo test --manifest-path dev/Cargo.toml --test language_operators \
  --test call_operators --test unsafe_statics --test syntax_transfer \
  -- --include-ignored --nocapture
```

Evidence is under `obj/parallel/pa26/index-mut/`, with declared fixtures, hashes,
independent object outcomes, `complete.json`, native outputs and resource receipt
`79802320e2f74e2e88ef1b90d708aa2a`. Initial functional checks caught missing child
scheduling in a newly guarded compound-assignment arm; its runtime lowering was
unchanged. The initially positive capture hypothesis reached the documented
PA24 boundary and remains explicitly qualified as such. Earlier failure logs
are retained separately from the final passing gate. Mutation tests live under
the existing operator target to preserve ownership and filename limits.

This is a functional checkpoint, not a PA26 release. Assignment/comparison/Deref
library hooks and primitive implementations, iteration, residual/early-return
question-mark semantics, additional derives, formatting/runtime contracts and
remaining nominal methods still require implementation. Cumulative performance
and release/native/text qualification remain necessary before activation. No
references or assignment activation changed here.
