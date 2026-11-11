# Callable outputs before structural consumers

Original core's IndexRange::fold consumes
`self.try_fold(init, NeverShortCircuit::wrap_mut_2(f)).0`. Frozen ordinary probes
reduce the missing result constraints to generic FnOnce/FnMut/Fn returning a
tuple through `drive(f).0`. Nominal, opaque, const and source-language roles are
unnecessary to reproduce it. Explicit result/type arguments, function pointers
and custom Make::Output controls already pass. PA24 is the earliest owner.
Actual core's const/source FnMut/Try companions remain required.

The initial [47-case packet](../pa24/tests/spec/callable-results/README.md) declares
35 accepted executable programs and 12 rejections before implementation or oracle
generation. All outcomes independently agree with pinned Rust1.98.0, edition2024,
aborting panics and checked overflow. All35 positives independently execute actual
CHECK assertions. The packet includes all three call traits, tuple/named/array/
callable consumers, associated results, opaque Self/named/free adapters, references,
moves, quantified inputs, mutable forwarding and width8/32/128 with128 unrelated
types. Rejections preserve field/input/result/kind, move, lifetime and quantifier
requirements. No input is silently reclassified as an unsupported negative.

Complete a call's arguments, then reuse the existing owner-indexed argument
refinement to progress only that owner's ready trait requirements. Keep suspended
queries, notifications and full evidence for ordinary final proof. Existing
inference certificates supply readiness without interning or scanning unresolved
suffixes. Inspect that summary once, rather than rescanning wide result prefixes
after every goal. Register projections introduced by selected outputs with the
existing body normalizer before structural consumers. No hot fields, global
cache/type scan, compiler limit, LowIR or backend change is introduced.

The first focused run passed45/47 cases and the width/noise invariant. It exposed
one associated output needing normalization and a tuple out-of-range access that
reported an internal identity error. The corrected run
1794358768672527277 passes7exact tests in121.64s: all47 complete LowIR outcomes,
width/noise, scoped callable assumptions, projected parameter patterns, erased
callables and existing source-closure/actual Iterator bodies. Strict selected
production/CLI/test Clippy, full formatting and architecture pass.

Inference work at widths8/32/128 is6229/21541/82789, trait selections33/105/393;
canonical work stays25. Projection, unsizing and method counters remainzero.
The128 unrelated types add exactlyzero body work at every width. A new private
certificate/no-interning/budget guard remains pending in full CI; the cold private
harness is not rebuilt locally after its known five-minute limit.

Native receipt1794359073388122651 passes all35 new Rust/CPPGM programs in55.37s.
The preceding native batch1794358997144576224 passes both existing native groups
(15 projected-parameter and17 source-closure programs), but fails the new group's
filename-derived crate name. The corrected test removes `.rs` from that name;
no production change or source-fixture change was needed.

The full PA24 comparison then exposed three real generic captured-reference
regressions in existing positive fixtures. Keep those sources, outcomes and
references. Four additional capture contrasts were frozen before repairing that
interaction: owned/shared/nested generic captured-reference tuples accept, while
a local capture escaping to static rejects. Independent Rust object/native checks
agree for all four and recheck the three existing positive controls. The current
packet at that snapshot owns51 outcomes (38 executable positives,13 rejections).

Type analysis passed the regressions, but demanded lifetime proof found two
independent callee universals without the source closure's necessary outlives
relation. Retain those checked source conditions in the existing declared
callee-premise payload after validating the exact predicates. The source caller
must prove that whole payload; only the checked-caller route converts it into
callee assumptions. Shared typed source argument substitution and complete Self
matching preserve capture/output slots and nested binders. Body entry consumers
reuse those same kernels. Only the empty quantifier from callable sugar is lifted
through the existing binder operation; nonempty quantifiers keep their scoped
consumer. Contextual reference conversions retain their ordinary ordering.

Corrected receipt1794361457829517119 passes9exact tests in119.97s: all51 complete
LowIR outcomes, width/noise, all22 generic and21 nested receiver outcomes and the
five existing controls above. Strict selected production/CLI/test Clippy, full
formatting and architecture pass. Native1794361591493889968 passes4groups in65.06s:
38 new,17 generic receiver,13 nested receiver and15 projected-parameter programs.
Complete text/provenance checks remained pending at that snapshot; initial
immutable6069289c was not a qualified replacement.

One additional projected-Self capture was then declared and independently
object/native checked before its correction. It passes type analysis but fails
complete lowering on50a2fb65 with a structural source-contract mismatch. As the
existing body entry path requires, normalize the substituted formal and actual
Self before matching. Formal normalization keeps its own source coordinates and
proof; rebase its private witnesses, restore matched caller slots, then append
every debt to the existing payload. The final packet owns52 outcomes
(39 executable positives,13 rejections). Final public receipt1794362197862994056
passes the same9exact controls, including all52 outcomes, in137.82s. Native
1794362417748714215 passes4groups in54.25s:39 new and45 existing programs.
All selected strict checks and the unchanged width/noise work bounds pass.

PA12 already owns the unchanged `reject/tuple-field.rs` reduction. Its Rust
rejection remains1; tuple access now checks the selected arity before child lookup.
Only that predeclared diagnostic reference changes from an internal identity
error to `tuple field index is out of range`. Scoped generation and complete
comparisons pass all579 PA24 and125 PA12 cases. All1581 existing PA24 sidecars
retain their exact bytes;374 of375 PA12 sidecars remain unchanged. The52 new
triples and scoped provenance ship together. Reference review and original
diagnostics are retained under obj/parallel/callable-results/.

The final immutable dev-fast CLI is
8b73f15a8334129dc7b9849347d05dc7d7ca7fac3c6286dc871b45c3fbcb1361,
bound to committed31cfbe08b0c9589c2bc55b93ac895076f72087c3, an empty patch and3896
Rust source hashes. Retiredbad606/50a and analysis e9/6ba executables are absent;
their observations and source bindings remain. Cleanup1794363184111643302 removes
those four pins; tidy1794363184280435301 removes100 disposable outputs.

Harness1794362838437194980 passes315 tests,468 roots/13 binaries/44 stages,
128 upstream fixtures and126 Reference files. Controls1794362959289190435 passes
three exact nominal/source-opponent/required tuple-factory checks in50.51s.
The three separate valid tuple-factory cleanup owners remain required.

Forty exclusive A/A+twoABBA observations retain complete output equality for
ordinary import and explicit-result LowIR, scalar callable-result types and the
unchanged source-role known-representation types. The first batch completed20
valid observations before its entry-free scalar input was accidentally selected
in executable mode. Those20 were retained; only the remaining two groups ran
with their declared type modes. Candidate/base ratios in that order are
1.012/1.006/0.969/0.939, with A/A0.712/0.867/0.923/1.042. No speed or neutrality
claim follows from these noisy unoptimized observations. Width/noise counters
above supply the changed-path scaling invariant; the broad matrix remains CI work.

The unchanged120s original-core observation still FAILS at121.2428s. Its retained
prefix has203 failures and4328 item attempts through function8870,file239:53421,
node3300 in str/pattern.rs; progress SHA256
12f5ce98e668bbf2cebbb2128cdceecf534bcac52e9be6b34a4eafd22d2bb7d2.
All722 input hashes, six bindings, profile and limits remain fixed. No file93
IndexRange failure remains in the prefix. The first errors are wrap_mut_1/2 in
ops/try_trait.rs file95:16817/17146, both invalid canonical trait selection.
Preserve their actual const/source FnMut/Destruct/Try roles while reducing the
next blocker; choose the earliest owner from the resulting minimum. Different
census prefixes establish no speed or body-count claim. No original core/alloc/std
artifact exists. Private/full successor qualification and the earlier optimized
nested-proof measurements remain pending.
