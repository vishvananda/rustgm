# Conditional native library dependencies

The [source packet](../pa29/tests/spec/conditional-native-links/README.md) adds
actual conditional native-library selection. A crate request alone grants no
foreign-call safety, symbol definition or LLVM intrinsic implementation.

Expansion evaluates each link predicate with the compilation's actual cfg inputs
only after the complete owner survives removal, including a later cfg attribute.
The selected condition remains a typed configured effect. Demanded query gates
reuse the existing owner/capture/ordinal/origin witnesses, including retirement
and fresh-instance checks; no provenance fence is removed. Actual link_cfg crate
permission remains distinct from internal macro allowances. Malformed library
metadata fails even when its library condition is false. Foreign declarations
remain visible and retain their original unsafe contracts.

Selected dylib/static records live in a bounded cold foreign registry plan,
separate from the symbol-name pool and LowIR. The driver buffers an explicit
--native-links-output manifest for the external object/cc bridge. A missing
manifest fails before publishing LowIR/metadata. Native-library flags never become
shell commands. The backend validates schema, target, names, counts and combined
bytes, then links actual selected dependencies with separately provided objects
and search directories. The C++ compiler and its code generation are unchanged.

An optional versioned metadata envelope transports only a provider's own dynamic
library dependencies; ordinary artifacts preserve their prior bytes. Final
manifest construction visits each imported artifact once, without copying
transitive prefixes into every provider. Nested/truncated/over-budget envelopes
fail. Static publication still requires real archive bundling and fails explicitly;
modifiers, other native kinds, broad imported profiles and fresh dynamic
producer/consumer/native qualification remain required. This is not a PA29
release or an original core/alloc/std artifact.

Qualified evidence:

- Final private6 / strict Clippy, format and architecture: 34.17s. Receipt `obj/qualification/fast/1793774030917322662/receipt.json`, SHA-256 `c16f1012ba79fe0b70d88f8cf24cefcd0bde9a1af5fc3dd23d2e8707479e6eeb`.
- Production9 / strict Clippy, format, architecture and PA8 126/126: 168.45s. Receipt `obj/qualification/fast/1793773654342835809/receipt.json`, SHA-256 `d8ad7332f9963089e2ce9d0ec97392e256c4bf76c20fde0d329497eaf26ce898`.
- Native integration1 / actual Rust main, selected archive and independent Rust controls: 37.04s. Receipt `obj/qualification/fast/1793773954287570618/receipt.json`, SHA-256 `e3243bb8c12380184fc9e559fab8c1b209f11bc6abb12b9690495b1abe836bdb`.
- Independent94 source outcomes plus two native programs: 4.19s. Receipt `obj/parallel/conditional-native-links-v1/reference-v4/receipt.json`, SHA-256 `e7b2e2cfb880f9729595c76e93472eed3107802f72307cc7d0348667f9173bf1`.
- Joined CLI/configuration/backend/harness/original-core qualification: 63.34s. Receipt `obj/parallel/conditional-native-links-v1/qualification-v2/receipt.json`, SHA-256 `759ffa58238aa731ba2499caa070436725a00d899cadaab69cf6f03ab484bd30`.

The production control compiler sources are identical to the final snapshot; the
only later Rust change gives the native integration's two programs an explicit
common crate name. The final native and private gates cover that test correction.
All source predictions preceded qualification. Oracle V1 exposed missing archive
setup; V2 fixes setup, while V3/V4 requalify portable repository-relative paths.
No source bytes or required outcomes were changed to conceal a compiler failure.

Immutable CLI SHA-256 `02117cec8c9e98e38c71a24455986438e2bb9c6fa92662d94001f888707ea2d9` clears
LIB-136 and reaches **LIB-137, link_llvm_intrinsics at original core/lib.rs139**.
The unchanged original-core observation takes 29.18s, opens375
sources, and verifies all722 original hashes. Explicit Allow cap, profile and
finite resource limits remain unchanged. The two changed driver configuration
source pins are explicitly requalified against all fifteen equal predecessor
configuration constants; the other four pins remain unchanged.
Core receipt `obj/parallel/conditional-native-links-v1/qualification-v2/core/result.json`, SHA-256 `04342d01da7e63f8c2cb3ddc50529ad9aa5ce6feacde9886bad96abfb97a754e`.
A bounded debugger probe, with fresh disassembly confirming its stack offsets,
identifies the next request: `obj/parallel/conditional-native-links-v1/feature-probe-v1/receipt.json`, SHA-256 `4dffe868da0e0eeb08035f901ab0123c7d02f66d0170938434da407c4af1089e`.
This does not establish later semantic checking, exported bodies or an artifact.
The first identical-binary/input observation took16.41s; the portability
requalification took29.18s under shared host load. Both receipts
are retained, and these observations do not establish a timing regression or gain.

The ordinary 128/512-function immutable A/B check retains40 observations,
frozen source hashes, output equality, A/A calibration and four paired ABBA
blocks per size. Paired median B/A ratios are
0.987/
0.944; calibration max/min
ratios 1.386/
1.436 show substantial
noise. No reliable speedup is claimed. Receipt `obj/parallel/conditional-native-links-v1/measurement-v1/receipt.json`,
SHA-256 `e9dc8f6daffff756729123df6e5096bb8f95d5b46c97d76ea0a41f8dc06fc25b`. Ordinary layouts gain no per-function row; cfg uses are sorted
at most once per affected owner, with no growing crate-wide prefix.

Full predecessor qualification [41781814852](https://github.com/vishvananda/rustgm/actions/runs/41781814852)
on1f5a2b8d0 passes every gate. The earlier failure was the CI disk reserve,
not a PA25 mismatch. CI now omits host debug symbols while preserving assertions
and retires the correct completed dev cache, retaining release outputs and the
8GiB reserve. Successor full qualification remains pending when this checkpoint
is committed. Root stays alone in the warm btrfs tree with five-minute/two-thread/
16GiB/150GiB guards. Successful generated native outputs are pruned; inputs,
diagnostics, immutable current pins, receipts and raw observations are retained.

Next implement genuine link_llvm_intrinsics feature/use, signature and lowering
contracts; an LLVM name cannot be treated as an ordinary C symbol. Preserve the
remaining const, Drop/Pin, ABI/SIMD, macro, body and imported type/value-policy
requirements. PA25–PA44 and the original core/alloc/std builds remain unfinished.
