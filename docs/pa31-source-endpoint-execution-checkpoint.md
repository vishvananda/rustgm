# Source range endpoint execution checkpoint

The authored RangePattern consumer now compiles and executes through the locked
LowIR backend. Its deliberately noncanonical u8 MIN=3, MAX=9 and identity
sub_one distinguish actual source selection from hardcoded primitive endpoints.
The native C observer matches pinned Rust: `7 3 7 -2 7\n`. These bodies are not
the original integer/char implementations; those remain required.

## Compiler changes

Declaration-time anonymous constants preserve their actual caller identity and
Const effect. Solved tracked-call demand schedules only genuine Location,
NonNull and PhantomData definitions/fields, then checks the real nominal data
producer before resuming publication. Location reentry revalidates the source;
PanicInfo remains separate. No unrelated body or nominal sweep is introduced.

A cold Publication cursor resumes selected implementation conformance by the
actual touched obligation. Method/constant implementations use the ordinary
per-owner conformance kernels and sparse owner buckets; functions-only seals
remain strict and associated types retain their explicit boundary. Binary
annotations prepare the written generic default through the existing default
work item before querying the unchanged Rhs=Self operator contract.

Dispatch transports sealed CallerSite callable types through the same proved
Bound as the callee operands, validating before and after rewriting. Source
owner, value and location stay fixed. Missing, stale and forged identities
still fail. Empty-site bodies keep the ordinary fast path.

Work follows touched calls, obligations and actual owner buckets. The retained
cursor avoids repeated prefix scans across dependency suspensions. No common
body/IR field is added. Erased scalar children remain sparse; a pre-existing
global lifetime-fact delta may conservatively retain an anonymous root during
suspension. That storage optimization is deferred as LIB-091, preserving proof
rather than weakening completion.

## Executed evidence

- Private42, format, full architecture and strict private Clippy pass in115.92s:
  `obj/qualification/fast/1793283393572117030/receipt.json`, SHA-256
  `0affb6cb8e4ef01dd55342cd4b0806b9d4ac49b4c712bee939a6f3468f35d6cc`.
- Actual LowIR validation, object generation, independent C object, link and
  native execution pass: `obj/parallel/caller-readiness-v1/native-v1/receipt.json`,
  SHA-256 `a40b68a9334f164127b727ee19690cffff84b9a6fd430d00f33db9ef72f2a18d`.
- The independent positive Rust metadata/object/native rows are reused only
  for identical frozen source/observer/toolchain. Their retained predecessor
  batch failed a separate disconnected-positive fixture, whose corrected
  independent observation is recorded separately. No overall failed batch is
  described as passing.

The ordinary-implementation opponent still rejects. Its actual source
on_unimplemented diagnostic replaces the generic const wording; its assertion
now accepts that source-defined message. Pinned Rust independently rejects the
same unchanged fixture with E0277. No reference output or fixture was changed.

Production/CLI qualification and a fresh unchanged-original-core observation
remain pending for this checkpoint. The unsigned-niche component must first
join this descendant and qualify the deduplicated union. Generic endpoints,
disconnected alternatives, original checked subtraction/char conversion/panic,
refinement metadata, original core/alloc/std artifacts and whole-PA release
checks remain required. No new assignment is marked implemented.
