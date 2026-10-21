# Parallel frontend publication checkpoint

The validated compiler snapshot is `7e80bff036c19c2029c92b9952a2ca777e2c23eb`.
The subsequent `a867246a` change documents corrected performance measurements
without changing compiler code. PA1–PA23 remain active; PA24–PA30 remain planned.
This checkpoint does not release closures, constant evaluation, library hooks,
unsafe operations, intrinsics, crate metadata or foreign interfaces as complete
assignment contracts.

The implementation now includes isolated noncapturing source closure bodies,
pointer reification and full/partial contextual lifetime signatures; typed
Option/Result/ManuallyDrop metadata and operator/index dispatch; lexical unsafe
contexts, raw access, mutable statics and runtime union storage; and checked,
bounded constant CFG evaluation with declaration readiness and discriminants.
The optional frozen ELF object backend profile is qualified independently of
the existing executable backend. Later worker checkpoints and generic closure
templates are outside this snapshot.

## Reviewed reference changes

Eleven stderr sidecars changed. Independent pinned-rustc checks reject ten
invalid programs. The remaining PA12 array-length case is valid Rust and keeps
its existing documented unsupported boundary; only its diagnostic offset
changes. The other changes diagnose non-const calls and invalid uses of enclosing
generic parameters instead of the earlier blanket implementation boundary.

Three unchanged, now-supported fixtures moved from PA12/PA19 into the PA25 draft
in `2395c0bd`. Each passed independent rustc object and native execution checks
and execution through CPPGM. The PA12/13/18/19 reference receipts were generated
and reviewed against the exact compiler and fixture hashes. No fixture contents,
success outputs or exit-status sidecars changed in this reference update.
The active fixture count is now 7,781.

## Validation

The frozen validation checkout passed:

- Formatting, release all-target/all-feature Clippy with warnings denied,
  the architecture/file audit (1,211 files, zero findings), audit unit tests,
  harness, layout/discovery and upstream/reference provenance checks.
- Full release Cargo tests: 1,912 passed. Native maintainer tests: 139 passed.
- All 23 active text suites: 7,781 exact comparisons.
- Independent source checks for PA6–PA23: 7,081 checks.
- All configured native assignment suites: PA11–14 and PA16–23. Their receipts
  distinguish executable comparisons, codegen-only controls and rejection
  checks; a codegen-only control is not counted as runtime coverage.
- Both the original backend contract and the optional ELF object profile.

The validation binary SHA-256 is
`5842e2dc6ae2a7bc950cf184eadf9a727ee5de8628b2444732c79d25bd86c91f`.
Compiler source and binary hashes were checked before and after qualification.
Local evidence is retained under
`rustgm-validation-work/obj/parallel/publication-7e80/`: `prepared.json`,
`diagnostic-qualification.json`, `reference-review.json`, `source.json`,
`candidate.json`, per-stage logs and `complete.json` (62 successful final gates).
The final resource receipt is `a341ab6467c44235ae72682d41ffac2b`.
Earlier failed comparisons remain available alongside the corrected results.

Performance evidence and limits are recorded in the individual feature notes,
including [partial contextual lifetimes](pa24-partial-context.md). The corrected
candidate has a small measured generic instruction cost; noisy wall times do
not establish tight performance neutrality. Immutable binaries, input hashes,
A/A calibration, ABBA observations, instruction counts and RSS remain retained.

The shared controller limits qualification to two heavy jobs, two Cargo jobs
and test threads per job, 12 GiB RAM and eight CPU cores per process tree.
Measurements run exclusively. Generated files are capped at 50 GiB per worker
with a 150 GiB filesystem reserve; scoped cleanup preserves receipts and current
comparison pins while removing disposable executables and objects.
