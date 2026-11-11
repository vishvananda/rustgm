# Symbolic-length builtin source indexing

The builtin source-index classifier recognized fixed arrays and slices but
omitted `ArrayTerm`. Generic `[T; N]` accesses therefore remained source
Index/IndexMut calls and failed const-stable API checks in original core's
array each_ref/each_mut. The correction adds the missing existing constructor
to that same classifier. Real role/implementation/const-effect proof, exact
usize input and element Output checks remain mandatory. Existing IndexParameter
publication, mutable places, bounds, source contexts and execution stay shared.
No new policy exception, cache, representation, limit or backend changes.

Ordinary generic array read/write/reference controls remain at their existing
PA18 paths. The [actual source packet](../pa31/tests/spec/symbolic-index/README.md)
now declares14 independently checked inputs:10 accepts and4 rejections. Cases
separate fixed and symbolic lengths, shared/mutable/raw places, custom keys,
changed outputs and explicit calls. Actual generic const mutation/read produces47;
length2/index2 rejects with a bounds error even though the authored Index body
would return element0. The unchanged const/implementation opponents still reject.
No active PA18 fixture or course reference was changed.

Final focused receipt1794384185094759145 passes all14 language/body contracts,
three PA18 controls, actual CTFE47 and the source panic guard regression in51.67s
with fullfmt/audit/strict selected Clippy. Earlier1794383015766934024 passes the
existing73 source-index cases, canonical-work scaling, const-slice/ordinary-source
controls and primitive/mutable operator suites; its separate new native attempt
failed at the retained source panic-runtime boundary. All failed attempts remain
under obj/parallel/symbolic-index.

Both native companions remain **required positives**. Independent Rust/C prints
`symbolic-index 41 23 47 23 41 23 23 47` for each. RustGM source analysis and
actual CTFE47 pass; demanded native LowIR stops at the existing checked source
panic signature/caller/runtime routing requirement. The second reduction moves
wrapper reads into generic helpers and retains that same boundary. No candidate
native pass or negative language qualification is claimed. The original bounds
hook and guard policy were preserved. Source panic runtime routing remains
required for final core artifacts; PA31 and whole-library qualification stay open.

Existing native indexing controls pass1794384358973196561: all7 source observers
retain actual custom/explicit calls, source const values, unsized places and
ordered effects. This is regression evidence, not native qualification for the
two pending new companions. Upstream/hash/harness checks and the next immutable
original-core observation are recorded in the handoff and raw evidence.

Immutable CLI00d205dc7bab488fbdadc5064be2d421947f9bdc3dd573f1c5691175faa8fc3e
binds004ae71e9, empty tracked build diff and3901 Rust source hashes. Twenty
same-profile exclusive A/A+twoABBA control observations preserve complete output
equality; noisy timings support no speed/neutrality claim.

The unchanged120s original observer times out at121.46912s with198 partial records
throughfunction10445,file282:35667. Reached each_ref/each_mut failures are absent;
first now try_from_fn_erased98:31073, associated-type normalization at
`guard.array_mut.len()`. All722 hashes,375 opens,six bindings and original
profile/limits remain unchanged. ProgressSHA
330acd717cf9ce19fd5161ceac824416dbd12e218fc4926b9e2a5525a13ef505.
[Machine evidence](pa31-symbolic-index-evidence.json) binds these finite results.
No complete census/body count/throughput/original artifact is claimed. Full prior
42979017824 on8aa27348d passed every stage; new source successor qualification
remains due after publication.
