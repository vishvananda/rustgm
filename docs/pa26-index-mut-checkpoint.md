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
allocation. Isolated closure bodies keep their own facts. Method receiver
selection also propagates mutable demand through an indexed place when its
selected receiver requires a mutable reference. This refinement preserves the
raw-pointer stop: a mutable method called through an indexed raw pointer still
permits the pointer to be read with shared Index.

The existing selected-call and receiver-borrow pipeline performs lowering. The
method's returned reference becomes an ordinary place, retaining normal
mutability, ownership and lifetime checks. There is no new IR dispatch or raw
pointer permission bypass. Plain assignment and primitive compound assignment
retain right-before-left runtime evaluation; index receiver borrowing still
precedes index-argument evaluation. Mutable square-index receivers use an
immediate borrow; ordinary method receivers retain their two-phase reservation.
Thus an index argument cannot inspect an already mutably borrowed container,
while an ordinary method argument can use the shared receiver before activation.
Native side-effect controls distinguish the
shared getter from the mutable getter and verify this ordering.

The 44 declared Rust 1.98.0 object-qualified cases are in
`pa26/tests/index-mut-outcomes.json`: initially twenty-five accepted cases,
sixteen language rejections and three explicitly valid Rust boundaries.
The later assignment checkpoint admits the unchanged custom compound case,
bringing this draft to twenty-six accepted cases and two boundaries.
The later [selected capture join](pa26-selected-index-captures.md) admits the
unchanged captured mutation source through actual IndexMut receiver selection
and native execution. The original unfinished-boundary observation remains
historical evidence; explicit mutable-parameter closures remain supported.
Custom compound assignment is supplied by the later checkpoint in
`docs/pa26-assignment-checkpoint.md`; primitive compound assignment through a
mutable index was already implemented here. Neither boundary was described as
a Rust language rejection. Direct library-trait dot-method
lookup, such as `w.index_mut(...)`, remains a separate required PA26 surface
boundary; the paired inherent-method reservation control executes correctly.

Twenty independently compared native programs cover replacement, borrowing,
primitive compound mutation, nested indexes, generic inherited output, tied
lifetimes, mutable raw addresses, mutable-reference dereference chains, shared
raw-pointer dereferences, side-effect order, unsized slice outputs and mutation
through an explicit closure parameter, immediate index borrowing, ordinary method
reservation, mutable and read-only methods, and reference/raw-pointer method
receiver chains. Tests also assert that IndexMut's output
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
`79802320e2f74e2e88ef1b90d708aa2a` for the initial checkpoint. The immediate-borrow
and method-receiver follow-up passes `receiver.json` under resource receipt
`6b485321153b47acbfd97bb6f8df1dfd`; its earlier Clippy failure identified one
call-operator helper caller requiring the explicit, unchanged two-phase policy
(failed receipt `881828adcb714bf08b12a9247188236f`). Independent pre-fix controls
also retained the two incorrectly admitted index-argument conflicts and the
direct trait-method boundary. Initial functional checks caught missing child
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
