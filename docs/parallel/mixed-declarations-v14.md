# Complete existing declaration families in mixed metadata

The mixed v14 profile promises existing v13 body transport together with
nonexecutable authenticated compiler operations. Its producer still tested the
Nominals mode enum before classifying enums, variants, tuple/unit constructors,
const parameters and named constants. V14 uses CompilerOperations mode, so all
four complete legacy provider families failed despite v13 accepting them.
After correcting that gate, the same providers exposed another omission:
required Fn methods lost their existing conditional constness because callable
effects were enabled for Nominals mode but not body-bearing CompilerOperations.

Both decisions now follow the existing body_nodes capability. Old v13 behavior
is unchanged; operation-only v11 still has no body-node capability and gains no
new declaration or callable authority. The existing typed producers, schema,
result columns, decode checks and source/native guards are reused. No wire
version, dense field, new cache, backend or resource policy changes.

The [PA43 manifest](../../pa43/tests/spec/mixed-declarations/manifest.json)
freezes four complete original PA29 providers with the exact already-qualified
Future/Output header and genuine cleanup declaration appended. All eight
ordinary clients/opponents preserve their old bytes. With and without an
operation, all four providers now serialize/decode/re-encode under v14. Fresh
CLI consumers retain tuple/unit constructor, enum, const-parameter and named
constant behavior; private/type opponents reject for their prior causes. Fresh
typed helper consumers preserve actual Output, unsafe and lifetime duties. No
helper gains an executable Original or native future state.

Pinned Rust1.98 checks four separately declared standard-core semantic/native
companions, which each return42, and accepts the helper consumer while rejecting
unsafe, wrong Output and escaping-lifetime cases. These companions model the
ordinary subset used by the unchanged clients; complete legacy provider
semantics retain their existing PA29 independent contracts. The actual helper
is never executed. Four emitted RustGM/native consumers also return42, while
both previous helper-before/after ordinary consumers retain41.

Focused1794674660553215232 passes six exact frontend/linking tests in157.65s,
strict production/selected/CLI Clippy, complete formatting and architecture.
Native1794675128987314929 passes two exact groups/six actual programs in44.57s.
The first test draft incorrectly inspected private descriptor rows; tests now use
public body views and actual fresh consumers. Fast2's real rust-call constness
failure motivated the second capability repair. First diagnostics remain.

Final replay binds3964 compiler/test source hashes, all packet/reference/basis
hashes and an immutable candidate. It repeats both new tests. At8/32/128
const-generic functions, inference/canonical/encoded bytes are115/111/9864,
451/449/23852,1795/1777/79876; each function retains an ordinary body alongside
exactly one operation. No timing/speed claim. All315 harness/495 roots/13 binaries/
44 stages/one component/128 upstream+126 Reference checks pass.

Immutable same-profile dev-fast baseline63c31d0bf has CLI SHA256
696cf424d29248241bce8dbb8812ca66a866a368c3d25b5fe1958def6692f89b.
Legacy A/A and candidate B/B bytes agree for every full provider:

- struct-constructors: bf18cd23541dccb038bc6c2904559fe9945272068f071dc27fe313086525f2bd
- enum-match: 0e862f298202835cda9552cb3da8ff26c0a38368a27ce5daab6db3aca3ca5e2c
- const-parameters: 55a2de3e8e053bcaf7da9d2a4cf6aa1c2e4cf3fd0960fc838f3fc0d72b627da0
- named-constants: ec9702823ce195786df20ebb67fc4341b2f9396d4d6b5a0060869aae678e929b

Raw evidence/pins are under obj/parallel/mixed-declarations/ and
capture-match/declaration-baseline-dev-fast/. The recorded earlier release29
baseline independently observed all eight v14 provider failures and four v13
A/A successes. It is not a timing comparison with the dev-fast candidate.

Three prior reduced const-Deref library positives now clear declaration kinds
but remain blocked by `source role form is outside the complete first transport
family`; no metadata is produced. Never substitute the successful standalone
CLI/native cases for these artifact requirements. Clean original core30 is the
next genuine build after the joined explicit-Deref and declaration repairs.
Previous full43611467168 passed every stage on170c (3079 private release8.39s,
debug93.73s); one successor on this joined checkpoint remains required. Every
original v13/v14 metadata/LowIR/native, future state/poll/drop and older handoff
requirement stays open. PA25–PA44 remain planned as whole assignments.
