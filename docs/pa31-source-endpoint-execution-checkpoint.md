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

## Joined compiler and original-core observation

Joined commit15237701b5b4c8eb1612a9ba4e00c42aa8118863 includes the unsigned-niche
component. Its deduplicated61 private controls pass209.94s, receipt
`1793284011578454967`, SHA-256
`3f5b6dfe83af787d88e54762bf4eb011328808fdfbefbbcf96f7fc0558c9bc88`.
Production/CLI3 passes36.53s with production strict Clippy, receipt
`1793283992948983832`, SHA-256
`cee14ffacf996bedc4de81e767f74700bab8347af4ca7da9689e93cecf779498`.
Both launchers pass and all3182 compiler/test/harness hashes match the integrator.
The cold private build timeout and production finalization timeout are retained
as failures; neither substitutes for those completed successor gates.

The joined endpoint native5 passes, receipt SHA-256
`c44d74476ab35f70c681db211bd7013f809e3946732ecca13e123a6624e34a4e`.
The scalar-niche/native cleanup matrix passes all12 operations, receipt SHA-256
`87c62e6ad59503a1afe043ffb092a4082562a0fc0f7f3605fcdd14a253105ce4`.
Root reuses the private binary's proof by exact source and fixture equality;
there is no redundant integrator private rebuild.

Current CLI1accafdd51e9516a62a8a2f997a57fac3e745aa70dbe6221252a9fec7261460c
clears the original range-pattern failure and stops at file268:2225's parsed
field-type derive boundary in12.94s. Opened-order268 is original stdarch x86
mod.rs, at the `2` in `__m128i(2 x i64)`, whose types macro creates an array
field. LIB-092 owns typed constant-child transfer in derives. The374 opens
remain a loading count, not compiled artifacts. All722 pinned hashes and target
settings are unchanged, with the same explicit work14,500,624/attempted1GiB/
retained64MiB profile. Capped receipt SHA-256 is
`3de3aad3d7527cd0e19291e22bc7f280499570d964a986201cf4c7c06c2e80f4`;
strict separately retains the deferred expectation at68 opens, receipt
`7f668ce5b0e88d078ea1b07474b55ba7523edaf91c661a33b4ab6777492c6450`.
The53.24s helper batch also passes272 harness controls,385 registrations,
44-stage layout and upstream verification, receipt
`a7f66cdd3e3713271067e37efc4f723fdfee29e6b0155880dfe532062a2d202e`.
This is a changed failure point, not a measured performance improvement.

Generic endpoints,
disconnected alternatives, original checked subtraction/char conversion/panic,
refinement metadata, original core/alloc/std artifacts and whole-PA release
checks remain required. No new assignment is marked implemented.

Full public0d4dde55a qualification CI40893498057 passes every lane: strict
lint/architecture/harness, host build graph, release/course, native, debug and
independent Rust. Push40893343278 also passes. This does not activate another
assignment or produce an original library artifact.
