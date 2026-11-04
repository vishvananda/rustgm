# Actual Pin source declarations and constructor bodies

PA31 owns these local source identities and ordinary callable headers; PA43
retains pinning guarantees, receiver validity, projection, suspension and
cancellation. PA26/27 own the actual library/unsafe consumers and PA29 their
authenticated transport. No whole assignment is released by this packet.

`contract.json` freezes 25 complete authored inputs and independent Rust
predictions. `cases.tsv` is the candidate manifest. Pinned Rust 1.98.0
88d9e12ae178fab0fb5cc050a94da85685d449ea confirms every outcome, including
native object/C-observer execution. These are authored protocol controls;
the foundation inherits the existing sizing/Copy/Clone/Destruct provider and
its attribution. Repository NOTICE/third-party terms remain applicable.

The primary sources are `compiler/rustc_hir/src/lang_items.rs`,
`library/core/src/pin.rs` and
`compiler/rustc_hir_analysis/src/check/wfcheck.rs` in the pinned read-only Rust
checkout. The actual `pin` role targets a struct without an extra generic,
field or representation restriction. `new_unchecked` targets an inherent method
without imposing additional signature restrictions on unused declarations.
The real library constructor is const/unsafe and has an ordinary source body.

Cases distinguish zero/type/lifetime/const/two-parameter struct declarations,
multiple fields, wrong nominal targets and duplicate claims. Unused and
impl-owned lifetime controls preserve distinct declaration/body/public frames. Constructor
controls distinguish inherent versus free/trait targets, genuine body checking,
ordinary unsafe and const call permission, duplicate moves, conflicting mutable
borrows and escaping locals. Rust-valid unused alternate constructor owners and
signatures remain valid; an implicit pin consumer cannot infer its own required
Pin owner/signature from that acceptance. An ordinary similarly named struct
remains ordinary. Neither language role grants Copy or suppresses field drops.

The native companion's actual constructor records5, then constructs its real
Pin value. Scalar extraction returns7, a mutable-reference field writes9, and
Pin owning a Trace drops that real field and records3 before returning8. Rust
and the unchanged C observer print:

```text
record=5
scalar=7
record=5
borrowed=9
record=5
record=3
owned=8
```

Candidate source tests run all25 cases through analysis, original body proof
and demanded lowering; their native input is qualified against Rust separately.
Three private controls check reservation/completion, actual headers, the completed
empty-lifetime scheme, ordinary field destruction and the unchanged metadata
fence. Metadata compilation alone is not private execution evidence.

The real standard-library constructor parent requires `P: Deref`; its positive
and missing-bound opponent are retained in the required receiver packet. The
ordinary source native here proves role/body semantics, not that broader API.

Pinned receivers remain required: actual Deref/LegacyReceiver proof and exact
Self/argument/lifetime identity must precede acceptance. Do not strip a nominal
wrapper by spelling or role and thereby skip those obligations. Implicit pin
construction must authenticate the actual selected constructor and operands.
Unpin/unsafe projection APIs, future/state-machine formation, async destruction,
cancellation and imported contracts remain required. Original core still stops
at LIB-154 AsyncDrop until its genuine signature and consumers exist.
