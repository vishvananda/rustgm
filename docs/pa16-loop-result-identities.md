# Reference results in repeated calls

Clean release19 clears the anonymous-array output proof and reaches a loan
conflict in original core's IterMut::next_chunk, at227:111229. The unchanged
120s observer checks42310 declarations,25858 bodies and35120 item attempts,
with zero failed/blocked functions. Completion is still0; no original library
artifact exists. All722 original hashes,375 opens, six pins and fixed limits
verify. The diagnostic names `self.post_inc_start(1).as_mut()` inside the loop.

The [PA16 packet](../pa16/tests/spec/loop-results/README.md) reduces this to a
safe callback returning a mutable reference with an independent lifetime. Storing
that result inside a loop rejects on release19; a shared-result control accepts.
The [PA27 packet](../pa27/tests/spec/loop-results/README.md) retains generic raw
pointer/wrapper/write variants. [PA31](../pa31/tests/spec/loop-results/README.md)
uses genuine source language declarations, actual pointer Copy implementations
and the original NonNull::as_mut body with a by-value as_ptr reduction.
Every outcome and input hash was declared before its independent execution.

A CFG value identity recurs when a loop executes a call again. The previous
result's referent loan must not become a loan through the fresh pointer result.
Rust1.98's [borrow dataflow](https://github.com/rust-lang/rust/blob/1.98.0/compiler/rustc_borrowck/src/dataflow.rs)
likewise retires borrow identities associated with an overwritten root place.
RustGM now indexes temporary-root loan masks once and retires them when that
actual temporary is defined again. Before replacement it checks a shallow write
to the temporary's own storage; a live storage borrow still rejects. Original
input-place loans and their lifetime/reborrow constraints remain independent.
Both ordinary statements and ordered cleanup statements use the same operation.
The sparse index has at most one entry per existing loan, within the existing
500000-loan and work limits; no type contract, dense declaration field or quota
is added.

Raw dereference roots also carry no inherited reference-loan promise, as required
by PA27. Borrowing a reference stored after reconstruction still creates ordinary
reference-place loans. Pointer storage reads, unsafe permission, source Copy
proofs, alignment, initialization, ownership and constant validity retain their
existing checks. This does not change the backend or storage wrapper.

Focused receipt1794616733767643096 passes six exact tests across three targets
in62.66s, with full formatting/audit and strict selected/CLI Clippy. It proves
25 complete outcomes: three safe accepting cases and five safe opponents,
six ordinary raw positives/two opponents, and seven source positives/two
opponents. Existing loan/reborrow, raw-address and anonymous-output controls pass.
The8/32/128-owner test runs actual concrete loan proofs, not an empty template
inventory. Frozen inputs independently match pinned Rust.

One additional ordinary unbounded manual Clone/Copy wrapper remains a REQUIRED
accepting contract in PA27's manifest. Bootstrap mode rejects its trait
implementation before loan checking. Its input and acceptance are unchanged;
the actual source-role counterpart passes complete proof. This finite repair
must not be described as closing that separate declaration boundary.

Native receipt1794616907890211530 passes all four independent Rust/C and
RustGM/CPPGM/C comparisons in75.13s, with strict selected/CLI Clippy, fullfmt
and audit. All four frozen programs accept; their raw reference/candidate
outputs remain under obj/parallel/loop-results-native/. Immutable release20
and actual original verification are next. The stronger C observer owns both input cells:
two actual result generations must preserve the first3 and mutate the second7
to9, with exact output `9 3 9`. The original native9 companions remain required
alongside it. Neither these programs nor a passing census replace original-core
metadata/LowIR and genuine native consumers. All earlier valid requirements and
whole planned assignments remain open.

Full43530351089 on published65d222d27 passed3074 private tests (including the
new scalar-output reproof unit) and every regular maintainer target. It stopped
on one old PA24 text reference: an unchanged invalid unrelated-lifetime result
now reports the same lifetime cause at192 instead of168. Independent Rust still
requires the missing b:a relation. Review and scoped regeneration of that exact
stderr retain the rejecting status and original input. Native/debug/independent
CI continuation did not run. Full43523008382 on5e9216115 remains fully green.


The first native attempt omitted the declared debug_assertions=false Rust flag,
so its ordinary reference object could not link pointer-check panic symbols.
The corrected exact source profile passes without any runtime shim. The first
scaling setup selected a template-only inventory for nongeneric functions and
checked zero bodies; the replacement concrete proof checks every owner. Raw
failed gates remain. No expected input outcome was inverted.

Scoped generation adds eleven PA16 text triples and changes only the reviewed
PA24 stderr among28035 existing sidecars. FullPA16 passes340/340; the exact PA24
unrelated-result/input-escape/projected controls pass3/3. Their original bytes
and rejecting outcomes remain unchanged. Raw receipt: inline-region-arrays/
loop-references/receipt.json.


The immutable ownership harness replay records8/32/128 loans,176/704/2816
points and6744/26976/107904 charged steps: exactlyn/22n/843n. All315 harness
tests,489 registered integration roots,44-stage/component layout and128+126
pinned upstream/Reference checks pass. The temporary source index has3952 Rust
files including its two real additions; no private harness was rebuilt locally.
