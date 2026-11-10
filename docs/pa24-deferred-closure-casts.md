# Closure casts wait for their actual caller inputs

Compiler ed07f0181 repairs premature cast preparation in an inferred closure.
The core reduction `let f = |n| read(n) as usize; f(1usize)` needs the real later
call to select `Read::Output`. PA24 is the earliest owner: no Index or source role
is needed. [Its packet](../pa24/tests/spec/projection-casts/README.md) freezes16
outcomes (13 accept,3 reject), their hashes and native classifications before
independent checking or text generation. [Four PA31 companions](../pa31/tests/spec/projection-casts/README.md)
retain genuine Index/callable roles, array/slice/selector forwarding, a generic
parent and a local constant. They claim semantic/object admission only.

The existing bottom closure continuation owns an optional cold boxed cast queue.
Only unresolved heads/projections enter it. It pays inference work and source
retention before allocation/append, then parks the original expression exits in
source order. The existing child input continuation supplies actual caller facts
before those exits replay full cast preparation. Scalar validity, function-item
pointer decay, metadata, coercion, trait, lifetime and ordinary body checks stay
in their original consumers. Work remains24 bytes; no ordinary body/closure
record grows, and no compiler quota, backend or cross-crate authority changes.

Production1794324253620726357 passes all16 outcomes, four full typed-lowering
companions and width8/32/128 with0/128 unrelated types. Fourfold width is bounded
by4x+256 inference/canonical work; unrelated types add no measured inference or
canonical work. The contract records the failed exact-counter-equality hypothesis:
reusable scratch/capacity can reduce total inference work, so the final invariant
forbids increases. Original failed inputs/results remain in the evidence folder.
Controls1794324981388913200 pass existing closure-pointer, explicit-pointer and
reference-unsizing rules. Native1794325104852985638 passes26 paired Rust/CPPGM
programs:7 new scalar/function-item cases,11 closure-pointer and8 pointer-metadata
controls. All16 independent Rust1.98 object outcomes agree, all13 positive Rust
programs run, and all4 PA31 companions pass metadata and object compilation.
The source companions first lacked genuine panic_bounds_check support; the
retained setup failure was corrected by declaring its ordinary tracked lang item.

Only16 new PA24 text oracles were generated. All509 PA24 outcomes,315 harness
tests/468 roots/13 binaries/44 stages,128 upstream fixtures and126 Reference files
pass. Existing oracle and upstream bytes are unchanged. The generated scalar
LowIR performs the actual u8-to-usize extension; the function-item case returns
and compares the real target address, also checked natively. Record and unpaid
trait results reject normally. The uncalled closure is independently E0282;
baseline and candidate retain the declared unresolved-projection rejection.

Private validation remains pending. The first attempt caught a duplicate test
module name, which was fixed. Final1794324687644228407 passed metadata, formatting
and architecture, then exceeded the five-minute feedback budget while Clippy
compiled/linked the cold private harness; no private tests ran. The intended five
controls cover queue order/no payload for known scalar heads, sticky exhausted
source payment, input owner identity, restored requirements and reference
frontiers. No passing-private claim or longer local budget is substituted.

Immutable dev-fast CLId2db4bd266e966df6f5f876af8f3e4eb38f04400e59c2678fd5b4f11f7607e7c
binds ed07f0181, an empty patch and3879 compiler/CLI/test Rust hashes. Thirty
A/A+twoABBA observations preserve complete ordinary typed-cast/generic-promotion
LowIR and74 ordinary survey outcomes. Candidate/base ratios1.102/1.177/0.985,
A/A0.924/0.934/1.017: tiny controls measured slower, survey essentially unchanged;
no speed claim. Evidence, failed attempts, producers, native receipts and raw
observations are under obj/parallel/projection-casts.

On unchanged722 original input hashes, semantic profile and quotas, the120s
observation FAILED at121.04s. Its partial stream retains307 failures/5967 item
attempts through function11290, stdarch x86/avx512f.rs file296 byte212064, SHA256
543de2845231d2a6159c6827005a24192ddb5aac644070bd42e1ccb3a60cc9aa.
It reaches parse_long_mantissa without its old failure; the first recorded failure
is now round_up, flt2dec/mod.rs file25 byte6502, projection normalization. The
incomplete prefix differs from the prior265/4086 observation, so raw failure
counts are not an improvement metric. No complete new census, checked-body
qualification or original core/alloc/std artifact is claimed. Reduce round_up's
actual inferred callback next, followed by repeated call-input/receiver causes.
Full predecessor42826374117 on7db87a632 passed every stage; the new successor
and private execution remain due. Optimized nested-proof qualification remains
pending. One worker continues in the existing tree with unchanged limits.
