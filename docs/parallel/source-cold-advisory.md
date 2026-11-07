# Cold closure advisory and source position authority

The source consumer accepts actual closure, authenticated ConstClosure-child and
async-block body targets through the compiler's existing advisory function
contract. It keeps ordinary function bodies, word/unsafe-wrapper validation,
independent inline policy, and normal body/capture/const/unsafe/loan checking.
Wrong targets and duplicate words use the actual owner's unused_attributes
policy. Valid no-body trait/foreign declarations retain a named required profile.
No LowIR hint or optimizer promise is invented.

A missing stmt_expr_attributes request was a genuine false acceptance in the
initial candidate. Statement and block-tail closures are Rust-positive without
that request, so a closure-kind gate would be wrong. The consumer now borrows one
demand-only AttributePositions view of its actual configured Unit. Its paid
traversal reuses resolve/bindings/positions.rs::child, the same typed edge law as
source feature admission, with the existing nonallocating child visitor. A cached
owner query authenticates the same borrowed Unit and enabled configured owner;
unvisited or incompatible contexts are errors. The borrowed lifetime prevents
reuse after the source Unit dies. There is no parser/text scan or per-attribute
root traversal. Actual feature and token-origin macro allowance return before
allocation; ordinary item targets need no context traversal.

The existing local header producer reserve_source_contracts prepays the cold
Box, full node table, worklist growth/old-live overlap and every traversal visit.
The existing annotation reservation pays attribute input/attempt work. Failed
charges remain consumed. Both Bootstrap and source use this actual local owner;
this adds no semantic permission or invented imported payer. Existing source
report/lint-context authentication precedes the new context query.

Measured warm-host layouts: AttributePositions 32, ColdState 16, node position
slot 1 byte. Transient AnnotationState grows 48 to 64 bytes;
Declarations 7120 and Code 304 remain unchanged. No Body/Node/frame/wire fields or
persistent registry are added. The private 32/128-owner linear-work, exhausted
payer and foreign-Unit controls compile under final all-target metadata qualification;
execution remains for CI. Local private execution is deliberately not claimed.

The packet pa31/tests/spec/source-cold freezes 27 Rust 1.98 sources: 16 positive,
11 negative, with resolved rustc/driver hashes. Twenty-two cases have complete
matching candidate outcomes (11 full positives and 11 negatives). Five accepted
Rust cases remain explicit requirements: const closure callable semantics,
async block and async closure bodies, plus no-body trait/foreign advisories.
Those are not counted as semantic negative parity. The source native observer
checks real capture/control flow and primitive C ABI: `cold 23 71`.

Gate 4 retains the original false acceptance. Gates 1–3 and 5–7 preserve routine
compile/audit failures. Gate 8 passes format, audit, strict library/frontend/CLI
metadata and all three focused public tests. Final qualification and immutable
CLI binding are in the adjacent JSON. Final all-target strict metadata, Word
regressions, native observer, 282 harness checks, layout and pinned upstream
verification pass. Detailed receipts are under
obj/parallel/source-transport/source-cold.
No full assignment, whole original module, source metadata transport or original
core advancement is claimed here. Parent owns joined integration/core/full CI.
