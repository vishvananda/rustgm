# PA31 cumulative library inventory budget checkpoint

Qualified 2026-11-04. This closes LIB-141's observed inventory resource frontier;
it does not complete PA31, release another assignment or produce an original
core/alloc/std artifact. Root works alone in the existing warm btrfs checkout.

## Cost ownership and allocation

The preceding immutable CLI ad230642 stopped at a fixed inventory-only 1M work
limit: 836965 steps plus the 238065-row owner-table preflight reached1075030.
Its charged43128704 bytes were below64MiB. The failing preflight was linear;
that observation did not establish quadratic behavior.

Capture and completion now borrow the actual SourceCore/LocalMacros contract
payer, after its existing identity validation. Earlier source work and inventory
work/attempted allocation accumulate under one immutable configured snapshot.
The meter grants no feature, source role, const eligibility or API permission.
Standalone staged libraries without that owner keep their previous finite
1M/64MiB fallback. The original-core profile still declares work14500624,
attempted1073741824 bytes and retained67108864 bytes, with an explicit Allow cap.
The limits are unchanged; their inventory coverage is now cumulative.

Owner arrays reserve their scope/declaration/import capacity on the actual axes.
Declaration filling pays its own pass; capacity reservation does not pretend to
visit uninitialized declaration elements. The optional const column is reserved
when needed. The used-policy set prepays and reserves a captured-policy upper
bound once, so inserts cannot hide hash-table growth. No source reparsing or
new per-definition policy carrier is introduced.

A conservative live draft reservation shares retained accounting with macro
witnesses and memo storage. Completion drops capture-only tables, then retains
the completed policy/declaration/import/const payload in one cold State counter.
Later macro payload checks include it. This is deterministic logical storage,
not RSS; transient draft charging remains deliberately conservative. All first
work/attempted/retained failures stay sticky and failed attempts remain paid.
Repeated completion is rejected before it can reset an existing live reservation.
Pending allowance-only Library creation remains unpublished until completion and
source reservations both succeed.

Only cold source contracts grow: State112, Contracts120, SourceCore200 and
Source208 bytes. Package256, Node36, Body456, Library152, Inventory112 and scope
Frame24 retain their exact dense layout checks. The borrowed payer stays in the
capture draft and is absent from the published inventory.

## Qualification

The final joined gate passes59 exact private controls and5 production/CLI cases
in182.27s, including strict Clippy, format and architecture audits. It includes
all new cumulative/sticky/retained/repeated-completion boundaries, actual source
and local owners, standalone fallback, shared-feature scaling/4096 functions,
module/import/variant ownership, const/default separation, language-role and
non-const-member controls, and exact ordinary/cold layout assertions.

All three packet programs are accepted independently by pinned rustc1.98.0
(88d9e12ae). Their candidate and predecessor semantic outputs match byte-for-byte;
an ordinary executable's LowIR also matches. These source controls do not bypass
the existing stability-metadata export fence or claim library artifacts.
Upstream attribution/checksums and the full harness pass. All fifteen frozen
configuration constants match predecessor output (SHA256
86c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d),
and all six configuration source pins remain unchanged.

Predecessor full CI41820005554 passed2659 private tests and all regular integration
suites, then failed two stale diagnostic oracles. Pinned Rust independently rejects
PA19's inherent overlap with E0592 and PA23's auto-trait member with E0407.
The course generator regenerated only their stderr sidecars and scoped provenance
batches; source bytes, empty stdout and exit1 stay unchanged. The final affected
text gate passes PA19 1193/1193 and PA23 415/415 in42.35s with its10 registration/CLI
controls, format, architecture and strict production Clippy. Successor full
release/native/debug/component qualification remains required in CI.

## Measured behavior

The retained96 observations use immutable A/B executables, frozen inputs, A/A
calibration and paired ABBA blocks, checking every output hash. Median B/A ratios
are0.958 ordinary,1.017 for256 source functions,1.000 for1024 and0.973 for4096.
A/A medians are1.072/1.036/1.143/1.045, with material individual-block noise.
No speedup is established. Candidate names-view medians are7.37ms ordinary and
24.64/56.64/204.37ms for256/1024/4096 functions; these are finite frontend scaling
controls, not full upstream library compile-time or RSS claims.

## Original core and next requirements

CLI041e30260fe1a2b67364e64ea43f4bc71e721a0a5023fad6145efbf40ead5fde
clears the old guard and reaches a real owner diagnostic in18.34134s/375 opened
sources: `ERROR: file 138: 554: const library stability requires a const function,
const trait or const trait implementation`. All722 original input hashes, target
options, lint cap, source limits and six configuration pins remain unchanged.
All original files are verified;375 opens is not a successful core artifact.

The first-open map points to iter/adapters/cycle.rs21, whose byte554 begins
`const impl<I: [const] Clone> Cycle<I>`. Confirm the exact retained owner while
reducing it. Current Role::of admits const impl stability only for trait impls,
while this header is inherent. LIB-142 belongs to PA25 const item/call semantics,
PA19 method headers and PA31 library ownership, with PA29 transport retained.
Required work includes actual enclosing-impl/member identity, generic const-bound
proof, const eligibility inherited from a real const block, ordinary/nonconst
opponents, cfg/macro provenance, distinct regular/const API policy and metadata
fences. Merely accepting the annotation must not grant const-call permission.
Whole core, alloc and std builds remain required; assignment statuses do not change.

## Reproducible retained evidence

Current immutable CLI is041e3026; immediate predecessor isad230642. Olderc1510a1d
and the unqualified9e63b840 draft CLI are retired with receipts; sources, logs,
observations and newest warm caches remain. No new worktrees or root build outputs
are created. Builds/checks stay within five minutes, two threads,16GiB absolute
output cap and150GiB reserve on btrfs.

- `obj/qualification/fast/1793794181669061999/receipt.json`: `9b4b6a9da057342823a57dede685dbcbf3ee04fa1eeab6b502e2b42ff6fb0030`.

- `obj/qualification/fast/1793794692686155932/receipt.json`: `52c9799dbcc4648384bbf3ced8a136472a014ec81c1902386102d1d8d7795364`.

- `obj/parallel/inventory-budget-v1/qualification-v4/receipt.json`: `0d8e163ac6fd3a3e11bd8593856d274a4721a49bca48ffcb5d0c1d8fceb76677`.

- `obj/parallel/inventory-budget-v1/configuration-v1/receipt.json`: `3ce94a5ee686bc5665f48dd45d3d34d256824b82276854a510908cfbcd5df99c`.

- `obj/parallel/inventory-budget-v1/measurement-v1/receipt.json`: `622d049e36fbf1f9501381c11a0d4c365e7edbfa78cd64d14fa25d68e474c852`.

- `obj/parallel/inventory-budget-v1/reference-v1/receipt.json`: `6bf65cc4d3bfbb817304b405f88680fad671d61d33b6ae16e3a6efedf80899a7`.

- `obj/parallel/inventory-budget-v1/retirement-v1.json`: `e9ba5757c3421915853a8c6999d3d53c622a7fa5db69cf6e232cb68928dd1a2b`.
