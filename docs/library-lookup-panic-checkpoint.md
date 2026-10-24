# Contextual lookup, complete source inputs and source panic delegation

Compiler snapshot `6bba6589` combines context-sensitive item/path/import/glob
lookup, operation-derived validation of complete source originals, and local
source-declared panic builtin delegation. It follows public `0bd4b4b6`.
The qualified compiler advances unchanged upstream core past its `core_panic`
declaration to the `panic_internals` allowance on `assert_eq`, line 41.
No whole core artifact, new assignment release or imported source-body execution
is claimed. PA1–PA24 remain implemented drafts; PA25–PA44 remain planned.

The lookup implementation preserves actual identifier contexts, definition
scopes and container-relative privacy; its glob and textual caches share
structural transformations without sharing caller authority. Complete source
validation derives required inputs from actual operations and signatures before
publication. The panic implementation authenticates declaration/attribute and
token provenance, including exact identifier spelling on warm cache hits.
See [lookup](pa31-item-lookup.md), [source domains](pa29-source-domain-index-checkpoint.md),
[panic delegation](pa31-source-panic-delegation.md) and
[witness repair](pa31-source-panic-witness-repair.md) for the component contracts.

## Functional qualification and retained failure

`obj/parallel/library-lookup-panic-release-v1` in the validation worktree retains
the first full gate at `3e39a17b`: 3,639 passes and four failures. All 13
integration targets passed. The private inventory caught three stale cold
`SourceCore` layout expectations and one real regression: exact spelling
authentication omitted the already-supported `attributes()` builtin metadata
word. That failed run is not reported as passing.

The recovery adds three production lines for the typed `Attributes` spelling,
a warm ordinary/raw-word mutation/restoration control, and the measured layout
expectations. `SourceCore` is 176 bytes, including 104 bytes of new cold contract
state; `Package` remains 256 bytes. Parser grammar and claim authority are
unchanged. The retained-target plan authenticated the exact source delta,
executables, inventories and preceding output before reusing unaffected tests.

`obj/parallel/library-lookup-panic-recovery-v1` completed all 65 functional
stages. Its 3,644 release/native results comprise **1,721 fresh passes**
(all 1,395 private tests and 326 frontend tests) and **1,923 retained passes**
from the other 12 unchanged integration targets. This is not a fresh rerun of
all 3,644 tests. The gate also passed all 8,382 text cases, 332 required exact
test names, affected independent source/native suites, seven transitive native
controls, formatting, architecture/file audits, strict all-target/all-feature
Clippy, harness/upstream/backend checks and three fresh adapters.

The private inventory took 3.68 seconds after its build; all active text cases
took 18.79 seconds. Compiler code generation remains the expensive part.
Development gates now run the full nonignored private inventory once after
building it, with required-name checks and passing output capture; focused
frontend/native selections remain separate. Do not duplicate the same private
execution inside a full maintainer gate.

Evidence SHA-256 values:

| Artifact | SHA-256 |
| --- | --- |
| Qualified CLI | `20eb54f33961841b6e496bec52dae3a41f51e10ee97dbdb4b03de68d8e728134` |
| Recovery functional receipt | `77b4135393c28106a90637edef22639b6b52b10c2b2f8babd0b1a88d9ad9a3aa` |
| Recovery checks | `bf595215e84a4a432b0cb715d2052cd4e85077e053c63968ec5116957e468fbd` |
| Retained-target bindings | `6a6f22c059ff9bf3626a113e0a976abcd15132f34aa52be759a43cb18926b463` |
| Original failed maintainer log | `0b3edfac502e074984eea6346e8f6084d65a9f48b64968e12f60499311b1d82f` |

## Performance and memory review

`obj/parallel/library-lookup-panic-perf-v2` measured 89 frozen rows. All 84
baseline/candidate pairs produced identical outputs, all 64 declared native
programs passed, and all 46 declared growth controls stayed within their limits.
The five newly supported panic rows require the old compiler to reject; they
have candidate-only observations and support no relative speedup claim.
The v2 plan changes only the recovery binding and expected qualification count
from v1, preserving every input, baseline, row and measurement threshold.

Measurements use A/A calibration, paired ABBA blocks, one-CPU affinity, immutable
executables and an exclusive resource lease. All observations remain retained.
No comparable case exceeded the 5% instruction review threshold. The largest
increase is 4.05% at macro depth 128/width 32. Complete source-original validation
costs 5,048,432 → 5,251,170 instructions (+4.02%); it now performs the mandatory
operation/signature/input validation. The same deterministic typed rows match.

Sixteen cases exceeded the RSS review threshold of both 512 KiB and 5%.
The largest is repeated library use at 128: 10,170 → 11,698 KiB (+1,528 KiB,
15.02%). Paired median wall changes range from −16.13% to +13.08%, with retained
calibration and individual blocks; this checkpoint does not claim zero cost or
uniform speedup. The fixed memory attribution batch examines repeated use 128,
owner depth 32, complete source originals and the repeated-Copy 128 control.
All eight A/B observations preserve exact outputs and inputs. Peak live heap
is respectively 154,443 → 154,443; 56,421 → 56,469; 218,008 → 218,104; and
333,183 → 334,659 bytes. The corresponding process high-water increases are
1,208/1,400/736/280 KiB; final compiler-mapping RSS increases are
1,208/1,400/740/276 KiB. These observations support compiler-page residency and
layout as the main contributor in these four cases. CLI text grows only
54,556 bytes: the RSS change is not that many bytes of newly allocated code.
They do not establish identical heap behavior everywhere or erase the sixteen
original RSS triggers.

The eight heap/stack/process mapping observations are retained in
`obj/parallel/library-lookup-panic-memory-v1`, results SHA-256
`aa79c5bba0cbd7f57a9588ea17937f39c9e9b1c88fdcae5599fadf13d4cf1499`.

Performance results SHA-256:
`8d00eec429df8505f2c52632489c13e183734c88b8b43ab3e0d5c88efabccbe1`.
Analysis SHA-256:
`6a314baee6c2d272dfbb462b5d67aa427850adfb0366996f8f73a473b7d2828a`.

## Real crate boundary and remaining work

`obj/parallel/whole-core-blockers-v5` uses the qualified CLI and the unchanged
Rust `88d9e12ae178fab0fb5cc050a94da85685d449ea` source/configuration. It exits 1
with `ERROR: file 3: 1250: internal macro allowance feature is not implemented`.
The same four original files were opened, with unchanged hashes. Result
SHA-256: `b333b6b448ed489df47726a86aac71040c8025d3ad32e664b2ce0dcd9a8c940a`.
This supersedes the earlier worker-only observation of the same boundary.

The [blocker tracker](library-build-blockers.md) keeps the remaining contracts:
complete per-token internal allowances and their actual feature consumers;
scoped macro publication, member lookup and foreign macro provenance;
real source artifact → import → fresh original/instance proof → native consumer;
and projected-Copy instance handoff with a genuinely nonempty alias scope.
The field-only candidate, LIB003 recovery and source-client descriptor are not
part of this compiler checkpoint. Ordinary generic source execution remains
disabled until its complete transaction qualifies.

## Qualification tooling

Tooling-only successor `441373fe` adds opt-in absolute output caps behind
explicit idle activation of controller schema 2. All 183 harness tests passed
in 6.555 seconds, including 30 resource-controller tests; suite registration
(377 roots in 13 binaries) and all 44 stage layouts also passed. The result in
`obj/parallel/output-caps-qualification-v1` has SHA-256
`1d795d48c44ef20cbf4dc75ff53aa0d8f346a53b97be2631f426ebb2bc157b20`.
This tooling gate does not rebuild the unchanged compiler. Compiler and
performance evidence above remains bound to `6bba6589`.

The default allowance stays 50 GiB and the filesystem reserve stays 150 GiB.
Smaller caps include all existing registered output, caches, pins and receipts.
They allow small gates to share the three slots when actual headroom permits;
exclusive measurements still exclude all other guarded work. See the
[idle rollout procedure](parallel-work.md#optional-absolute-output-caps-controller-schema-2).
