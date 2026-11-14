# Defining generic const callable bodies

Clean original-core candidate31 clears Ord::max, then stops at
`cmp.rs:1641`, min_by_key's `f(&v2)`: `const invocation has no completed callable
formal order`. An unused free generic function with no early lifetime arguments
had no callable registration until a client demanded its function value.

Retaining a call in an actual defining const function now demands the existing
callable-template producer. It stores the complete inherited/own formal order
once; subsequent calls use the existing indexed registration. The readonly
validator consumes that order and exact retained Fn/FnMut receiver borrows,
preserving source body/statement, call-trait role, caller effect, opened signature,
complete argument/result, actual generic substitution and borrow capability.
Actual loan/move proof still rejects calling an owned FnOnce twice or borrowing
an immutable variable for FnMut. No second formal-order reconstruction, new
cache, persistent AST/body/value field, wire, backend or storage-policy change.

Concrete function-item Fn/FnMut calls also need a real borrowed receiver during
CTFE. The reached call checks its actual Code entry and evaluated statement,
validates the complete safe opened signature and operands, then reads through
checked storage. FnMut requires the actual Writable permission; both forms
require the loaded function item's exact identity/type/signature. Storage waits
remain pending. Actual const function entry, source/API policy and proved Code
remain mandatory. Captured closure calls keep their existing separate proved
entry and environment path.

The [ordinary PA25 packet](../pa25/tests/spec/generic-const-calls/manifest.json)
and [actual source PA31 packet](../pa31/tests/spec/generic-const-calls/manifest.json)
freeze15 predictions each:12 accept/3 reject. They cover unused FnOnce/FnMut/Fn,
higher-ranked reference inputs, repeated mutable calls, ordinary effects, moved
values and immutable receivers, the unchanged original min_by_key item, concrete
CTFE/runtime calls and8/32/128 scaling. All15 source outcomes qualify through
prepared lowering and standalone CLI. Pinned Rust1.98 independently agrees with
all ordinary outcomes. All15 ordinary candidate commands stop earlier at the
existing core::marker::Destruct import boundary, so none counts as semantic
parity/rejection; every original positive and negative requirement remains.

Evidence under `obj/parallel/generic-const-calls/`:

- `declaration.json`/`baseline/` freeze13 original predictions and reproduce the
  missing registration and old FnOnce-only check with clean immutable31.
  `additional-declaration.json` freezes the shared/runtime native companions
  before execution. Original method/foundation bytes have separate hashes.
- fast3,1794680741839406928, passes four exact tests in132.15s, including prior
  conditional-closure and Ord default controls. fast4,1794680934117603013,
  qualifies the final packet/scaling in48.66s. Strict selected/production/CLI
  Clippy, full formatting and architecture checks pass. A new private retained-
  borrow/actual-receiver forgery test awaits full CI; no cold local harness.
- native2,1794681053414857447, passes two groups/eleven actual programs in41.79s:
  the new shared/mutable41s and actual constant/runtime `41 59`, plus eight
  prior const/runtime callable observations. `reference-native/` independently
  executes the three matching standard-core observations.
- `replay/` authenticates all3964 compiler source hashes, both manifests and all
  packet inputs/foundation. It checks15 complete source CLI outcomes, records15
  unqualified ordinary imports, and replays lowering/scaling. Post-lowering
  inference/canonical work at8/32/128 is1762/2037,6994/6959,27922/26623.
 315 harness tests/495 roots/13 binaries/44 stages/one component,
 128 upstream and126 Reference checks pass. No timing/speed claim.

Fast1 exposed the CTFE FnOnce-only guard and failed the unchanged16GiB artifact
cap; superseded executable pins were then explicitly retired/pruned with hashes
and receipts retained. Fast2 caught a typed permission-token mismatch, corrected
to require Writable. The first native selector named a nonexistent test and
failed before execution; native2 uses the actual exact test name. No quota,
reserve, expected outcome or original-source input was relaxed.

The next qualification is a clean immutable original-core retry and one full CI
successor after the previous run terminates. Original v13/v14 metadata/LowIR/
native consumers, conditional target payment, wider receiver adapters, ordinary
bootstrap, executable futures and every older required unsupported contract
remain open. PA25–PA44 stay planned; the separate113-case const-machine component
is unchanged.
