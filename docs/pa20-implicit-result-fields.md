# Implicit protocol result fields

Clean original-core38 stops at slice/index.rs:1035/file226:34591 in the tuple-of-
bounds SliceIndex get method: try_into_slice_range(...)? .get(slice). Its implicit
residual call returns a generic aggregate with a borrowed field. The exact result
lifetime instantiation has no corresponding expression node and was not prepared
before readonly CFG construction asked for its drop properties.

The frozen38 debugger trace reaches aggregate::Definitions::raw_shape through
NeedsDrop::demand, Builder::create_owned, implicit_call and question_mark's residual
conversion. The new sema producer prepares storage for each actual implicit protocol
call result, using its solved opened signature. It reuses the existing bounded,
memoized field preparation during the existing call pass. Ordinary explicit results
remain prepared through touched expression types. There is no declaration-field
fallback, readonly mutation, layout substitution, new cache/dense field, quota,
backend, storage, ABI or wire change. Lifetime/formation/trait/safety/borrow/cleanup
and actual executable Code obligations remain separate.

[The PA20 ordinary control](../pa20/tests/spec/nominal-residuals/manifest.json)
and [the PA26 question-mark counterparts](../pa26/tests/spec/nominal-residuals/manifest.json)
together declare 17 Rust outcomes, mirrored by the
[PA31 source roles](../pa31/tests/spec/nominal-residuals/manifest.json).
Rust 1.98 independently confirmed 13 acceptances and four rejections. All 17 complete
source outcomes qualify, including direct/delegated values, references and slices,
associated-output methods, no-question control, actual borrowed execution and
scaling. Wrong output/associated output, escaped lifetime and moved value remain
semantic rejections. The ordinary no-question generic/associated control qualifies;
other16 ordinary question-mark attempts remain UNQUALIFIED at their required
provider-catalog boundary, including independently rejecting inputs. They are
retained PA26 requirements, never expected bootstrap failures.

PA20's active manifest registers the no-question control. PA26's draft packet
records the other 16 inputs and their observer. The no-question primary retains its original free generic
control and adds the paired actual associated-output delegation, making PA20 its
owning minimum. Its independently accepted object and ten-line main-only LowIR
oracle are checked; codegen-only classification explicitly claims no runtime feature
execution. The actual source native observer separately executes the borrowed fields.
The full ordinary contract and original SliceIndex body/native behavior remain open.
All4878 previous PA20 oracle files remain unchanged; only one scoped reference batch
and three new sidecars are generated after the explicit course declaration.

Raw evidence under obj/parallel/nominal-residuals/:

- declaration.json and baseline/ freeze17 independent predictions and immutable38
  comparisons. Direct values/borrows, value delegation and no-question control pass;
  borrowed/slice/associated delegation after ? reproduces the exact missing-instance
  error. trace.gdb/trace.log identify its readonly drop consumer without altering
  the frozen compiler. course-declaration.json declares the primary/auxiliary split
  and associated control amendment before execution/oracle generation.
- fast2=1794706508201717071 passes four exact new/old controls in168.93s with strict
  selected/production/CLI Clippy, full formatting/audit. fast1 stopped at missing
  active PA20 inventory registration before compiler tests. Source ownership remains
  PA20/31; no assignment or audit gate was weakened.
- native1794706785316415956 passes four groups/eleven actual programs in43.94s.
  New generic borrowed success/residual paths print41/59/23; ten prior programs
  pass. Independent ordinary Rust executes the counterpart with41/59/23.
- replay/ authenticates3967 compiler sources/all packet hashes and34 CLI attempts:
  18 qualify (17 source plus the ordinary control),16 unfinished ordinary attempts
  stay required. Exact lowering/scaling pass.8/32/128 inference/canonical/field-
  instantiation work is11270/4316/507,41798/12692/1587,163910/46468/5907.
  Current immutable dev-fast CLI is
  a688464caf5a5604d324c7106c7590cd092023e8904594b9f5b06a07af6f7004.
  No speed claim is made. Initial replay's final layout check correctly required
  the missing new course reference; generating the declared primary closes it.
- course-primary*, course-verify.log and course-primary-verification.json preserve
  independent Rust object success, reviewed IR/status, exact candidate comparison,
  one scoped codegen-only native comparison and unchanged old oracles.315 harness
  tests,497 roots/13 binaries,44 stages/one component,128 upstream fixtures and126
  Reference files pass. PA20 now has1627 primary cases (614 semantics/1013 LowIR);
  its608 declared runtime executions are unchanged.

The private initializer adapter repair afe0fc9e2 compiled in full43661946931, then
strict private Clippy found a missing semicolon in the new Try corruption guard.
That single statement is corrected here; assertions and expected results are
unchanged. Prior43660831620's eight missing Signature arguments remain repaired.
Both full attempts stopped before test execution; combined private3087 qualification
remains required. Latest fully green43656293922/8560c4dbb passed3085 private release
4.97s/debug45.56s and every other stage. No cold local private harness was run.

Next publish one full successor, freeze clean39 and retry unchanged original core
with722 sources/six reviewed pins/120s/unchanged quotas plus postcheck. Core38 is
still latest until that run. No original metadata/LowIR/native consumer artifact
exists; all older originalv13/v14, Future/async, CTFE/bootstrap/closure/ABI/native
requirements, wholePA25–44 and the released113-case component retain their status.

Clean release39 on publishedb3ab20376 verifies the original tuple-of-bounds get
fence clears.3967 hashes/empty patch, CLI599bd507…bba9cd2c,7m38s build. Original
v14 next rejects ops/try_trait.rs:415/file95:16949 in NeverShortCircuit::wrap_mut_1:
`const call operators require their actual source function owner`, from captured
F invocation inside its returned const move closure.17.839266604045406s/status1/
no timeout/RSS744164KiB; no metadata/0-byte LowIR. Postcheck passes722 originals/
375 opens/six reviewedpins/all3967/cleanpin/fixed120s/unchanged limits. Diagnostic
only/no speed/artifact/native-consumer claim. One full43665225882/b3ab passes
fast and all-target/private lint/architecture/harness, and runs release/course;
private3087 execution and final full status remain pending.
