# Source float-reliability cfg queries

The frontend now implements all four float/math backend-reliability cfg gates
under their actual `cfg_target_has_reliable_f16_f128` feature. The
[frozen packet](../pa41/tests/spec/float-reliability-cfg/README.md) separates query
permission, genuine macro origin, compiler-owned target truth, source float-type
permission and metadata publication. The fixed target values stay false. No
backend or floating-point operation changes.

The implementation adds one appended bit to the existing cold u64 feature word
and one known macro consumer. All prior ordinals remain fixed. Existing source
owners, configured-demand witnesses and macro-frame memo queries supply the
authority. CfgGate/KnownFeature remain one byte, Features eight bytes, and the
ordinary package/source-provider layout controls remain unchanged. No additional
AST pass, per-query header walk or parallel target-fact table is introduced.

Final compiler/test snapshot evidence:

- Twelve exact private regressions, strict Clippy, full format and architecture
  audit:226.83s. This includes all148 declared outcomes, target override denial,
  packed layouts, prior atomic/sanitizer gates, false-demand provenance and
  repeated queries at allowance widths8/32/128 and1/32 calls. Receipt
  `obj/qualification/fast/1793757615581395393/receipt.json`, SHA-256
  `c49d7820e3a1416795dc6e3f698f52e2170d46e027ab306d751693dcd8ae0f72`.
- Nineteen production regressions plus PA8 126/126, strict Clippy, format and
  audit:120.32s. Receipt
  `obj/qualification/fast/1793757862581615123/receipt.json`, SHA-256
  `34049494a29cd740c9530080464b1b0ebee8732f2b3f2e365f462632af096a0b`.
- Pinned Rust148 outcomes and its actual native `1 1 1 0`:4.01s. Receipt
  `obj/parallel/float-reliability-v1/reference-v2/receipt.json`, SHA-256
  `b264afd565e1725a35bd55ad9a4569d7574cfeadfa3f7cdbdc9570e8bc8e0ad1`.
  Two additional independent source-executable metadata outcomes:0.06s,
  `reference-native-v2/receipt.json` in the same scratch root, SHA-256
  `8e2f52973afb8bf3d94c646f4fb00e072c8cb18161033465128ae5f4c33a149c`.
- Joined qualification:36.14s, all150 candidate edition/type outcomes, the
  predecessor's actual feature rejection and ungated-use bug, identical15-value
  configuration snapshots, actual source-executable native `0 0 0 0`, retained
  SourceCore library-export denial, upstream/harness checks and original core.
  `obj/parallel/float-reliability-v1/qualification-v5/receipt.json`, SHA-256
  `2778441bd90d40cbbe653ef186fe13abc0c5da0ff8a41cba00a3e72caade8f86`.

Immutable CLI:
`obj/parallel/float-reliability-v1/cli/d98d15849940e39490e8406ce5b99380aff9788b960be56f43246fbf0e941fa4/rustgm`.
The unchanged snapshot's constant-view SHA-256 is
`86c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d`.

The new manifest explicitly requalifies only `dev/src/expand/config.rs` from
`c111892059dd18b420ee63d29d35cb2a6eb0d051df235cf2dbe176a70ab2ef23` to
`a93164ee91e2f95e0a7edb1f35906496ccf1978b6d332083ebaecedd5a445d5c`.
It retains the predecessor manifest and configuration-binding history. All722
original input hashes, the other five configuration-source pins, actual target
facts, options, capAllow and finite limits remain unchanged. This is not an
unchanged configuration-implementation claim and uses no ad-hoc `--cfg` input.

Original core clears LIB-131 and reaches LIB-132: `abi_unadjusted`, requested at
original `library/core/src/lib.rs:108`. The ordinary error is still unlocated.
The immutable-binary debugger probe verifies the actual error branch and exact
blocked name; raw disassembly, stack and source/binary hashes are preserved.
Probe18.87s, `obj/parallel/float-reliability-v1/feature-probe-v1/receipt.json`,
SHA-256 `396aeb9392b5e6a4bc0a0f8df39d0654993711c4a49a67cc6cb5af76497c3b8f`.
The core observation takes19.38s/375 opens; result SHA-256
`71759e99fbe20ec28586a74b0b5293842d29b5b7f858cc54fa6b720c5ef34681`.

PA28/PA31 own the next ABI feature and its genuine source convention semantics;
PA29 must preserve calling-convention identity in metadata. Do not authorize
`unadjusted` by treating it as Rust or C without an independent ABI contract.
Later library inventory costs, full Drop/Pin, nominal/value/body-policy transport
and original core/alloc/std artifacts remain required. No whole PA is released.

Failed setup evidence remains in reference-v1, qualification-v1 through-v4 and
the earlier native foundation. They include a misplaced inner attribute, a
missing executable entry, an attempted overwrite of an immutable pin, an actual
closed metadata-export profile, and missing source Copy roles. Successful
qualification uses unchanged compiler/test sources; no required outcome was
inverted to conceal those failures. Final fixture/source-binding, upstream and
harness review passes13.25s; receipt
`obj/parallel/float-reliability-v1/final-review-v1/receipt.json`, SHA-256
`e7f6a51454f5f16220ed754fe55b9dd8f361e5e38fec497e6e8654256215ef91`.
Full CI41755882002 on the predecessor51b7c2d61 is terminal success across every
gate. The successor's full run remains separate from the focused results above.
