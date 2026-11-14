# Genuine source unchecked integer arithmetic

Clean release26 (source e23bbc172,3962 Rust files, empty patch) clears the original
projected-root cleanup failure. Its unchanged original-core diagnostic checks
42310 declarations,25858 bodies and35120 item attempts with no function failures.
Completion stops at expanded num/mod.rs file12:1052023. An actual unsupported
intrinsic error-branch breakpoint identifies Declaration273, unchecked_mul, from
the pinned registry. This does not decode the concrete argument type. Postcheck
verifies722 original sources,375 opens, six configuration pins and all compiler
hashes with the unchanged120s/work/byte limits. No original library artifact exists.

The PA31 tests/spec/unchecked-arithmetic packet declares26 outcomes before
execution: eight accepting inputs and18 required rejections. It retains original
intrinsic declarations and the complete unsigned multiply wrapper byte-exact,
with pinned source revision, hashes and attribution. The executable usize wrapper
removes attributes and the separately owned precondition macro; its221 result
qualifies this finite reduction, not the full original method policy.

The240 literal expected results cover add/multiply/divide/remainder, all12 signed
and unsigned integer types and five valid pairs per operation/type. Manifest
values retain mathematical results, target-width bits and native sign extension.
Pinned Rust independently observes all240 constant exports and all240 runtime
calls, including i128/u128, plus wrapper221, ordinary99 and legacy subtraction7.
Seventeen opponents independently reject. The floating-point opponent instead
causes pinned Rust ICE101; retain that infrastructure failure as an unqualified
reference outcome, never as a passing rejection. RustGM must still reject this
integer-only operation. First oracle setup errors (missing core context and
no_core Neg support) remain under first-inputs/oracle1. The corrected setup keeps
all intended signed values and outcomes using same-width unsigned bit casts.

The implementation extends the existing scalar arithmetic descriptor for the
actual unchecked_add/mul/div/rem declarations and the existing unchecked_sub
identity. Reuse exact own Copy formals, actual const/unsafe declaration validation,
closed integer instances, typed direct-call witnesses and current caller/header
proof. Unmarked lookalikes retain ordinary bodies. CTFE uses the checked integer
kernel, rejecting overflow, zero divisors and MIN/-1, including remainder; it
never silently wraps. Runtime valid unsafe operations use existing LowIR scalar
operations. Imported/source-transport fences, frozen backend v3/17b024f69, all
quotas and storage.py remain unchanged. No dense type/body/value field is added.

The first native run passed every constant export but exposed two runtime i32
errors: -7/3 became1431655763 and -7%3 became0. The emitted native sequence used
64-bit division on unextended C argument registers. Retain all expected values,
source/C bytes, LowIR and disassembly. Explicit signed widening before narrow
source division/remainder, then result truncation, preserves the existing backend
and valid integer semantics. Wider inputs remain unchanged. This finite repair
makes no claim about all ordinary narrow-integer C ABI paths.

Standalone CLI library LowIR/metadata surfaces remain required. Baseline26 failed
runtime source declarations at its wider-metadata-profile fence and ordinary
controls at the initial source declaration profile; in-memory prepared/native
programs do not replace those original transport/artifact contracts. Whole PA25–44
remain planned; the separate113-case PA25 component is unchanged. Full43592623119
passed every stage on preceding e23bbc172 (3078 private release4.66s/debug46.06s).
A coherent successor original-core observation and full CI remain required.

Final focused receipt1794660904448918647 passes six exact tests in89.16s, with
strict selected/CLI Clippy, formatting and architecture checks. Final native
receipt1794661028311363989 passes three groups/nine actual programs in34.89s:
five new programs plus three shift observers and the156-product control. All240
runtime and240 constant observations agree with their independent literal oracle.
Earlier unknown-selection, function-size lint and two-value native failures stay
recorded; none was reclassified as a semantic pass.

Authenticated replay checks all3963 compiler/test source hashes, every declared
packet file, the byte-exact upstream cutouts and240 mathematical/target/native
values before and after the three new exact tests. It binds frontend harness
SHA25664e0b5f35542fe77cc89db34bb45fe206282a78a0eeeaf7be716a083f1f5d33e.
At8/32/128 successive constants, inference is1333/5197/20653, canonical work
2717/3269/5477 and CTFE steps680/2720/10880. This is bounded scaling evidence;
no timing speedup is claimed. All315 harness checks,495 roots in13 binaries,
44 stages/one component and128 upstream+126 Reference checks pass. No generated
text reference changed. Raw evidence lives under obj/parallel/intrinsic-next/.
