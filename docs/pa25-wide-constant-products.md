# Wide floating constant products

Clean release21, built from f5f3e2961, clears original core's180/PI constant
division. Its unchanged120s diagnostic checks42310 declarations,25858 bodies
and35120 attempts without a failed/blocked function. Completion then stops at
f128.rs981, `const HI: f128 = f128::MAX * 0.5` inside midpoint. The23.0756s
observation ran above the14/20 timing gate. Postcheck verifies722 unchanged
inputs,375 opens, six pins,3956 compiler hashes and unchanged resource limits.
There is no original library artifact. Full43546470095 passes every stage on
the preceding division source, including3075 private tests and its ratio unit.

The [PA25 packet](../pa25/tests/spec/wide-constant-products/README.md) owns ordinary
constant multiplication. [PA31](../pa31/tests/spec/wide-constant-products/README.md)
retains the unchanged source-div foundation, exact original Mul declaration and
macro, actual wide Copy/Clone/Neg implementations and the original HI/MAX
expression in a reduced const function. The full original midpoint cutouts and
Mul bytes are retained separately with source revision/ranges/hashes. The
reduction does not claim the remaining midpoint runtime body is implemented.

All outcomes were declared before execution. Independent Rust1.98/C executions
observe98 results across four matrices. Exact finite bits, signed zeros and
infinities compare directly; arithmetic NaNs compare only by classification.
Cases include full significand products, both ties-even directions, minimum
subnormals, signed underflow, overflow, opposite exponent extremes, actual source
const UFCS calls and the original MAX/2 value. Six feature/type/const-call/integer
fault/source-role/const-implementation guards reject. Release21 reproduces the
missing operation on all four accepting matrices. No outcome was inverted.

The shared exact-rounding kernel now consumes a full finite product. Four by
four32-bit schoolbook multiplication performs16 limb steps and keeps at most
eight limbs, preserving all226 significant product bits for f128 before rounding.
The existing rational path supplies nearest/ties-even, normal/subnormal carry
and saturation. Binary scale stays in the output exponent, so exponent gaps do
not allocate proportional scratch. Common typed format decoding and special
values live in sema/constants/interchange_arithmetic; multiplication and division
remain separately selected operations. No host f64 intermediate, external
frontend, dependency, dense record, alignment change or quota increase is added.
The2048-word literal bound and inherited CPPGM attribution remain unchanged.

Fastcheck1794633042585911598 passes four exact tests in134.60s: all172 old/new
matrix results,12 rejecting guards, both8/32/128 scaling packets and the literal
control. PA2 remains142/142 with unchanged text references. Selected/CLI strict
Clippy, full formatting and architecture checks pass. Native fastcheck
1794633256850046765 passes all eight actual RustGM/CPPGM/C byte observers in39s,
covering the new products and old divisions. They use the existing byte-address
integer/pointer ABI, with no wide runtime instruction, C ABI or panic shim.

The authenticated replay verifies3956 compiler/test hashes and all fixture/bit
hashes before and after the three exact tests. Product inference work81/246/906
is55n/8+26; canonical work11/35/131 isn+3; actual output128/512/2048 bytes is16n.
The division controls retain their exact82/244/892, n+3 and16n counters. All315
harness checks,490 integration roots/13 binaries,44-stage/one-component layout,
128 upstream and126 Reference checks pass. Raw evidence is under
obj/parallel/wide-constant-products/. The first replay helper assumed a newer
bits_path field in the old manifest; its setup failure remains in consistency.log.
Corrected consistency-v2.log and scaling-replay-v2/ pass without changing inputs.

The new private product unit covers full128-bit limb carry and both formats'
ties/extreme exponents. Its execution awaits full CI;
the known cold private harness is not rebuilt locally. Existing ratio scratch
and literal tests retain their original paths. Runtime wide arithmetic, casts,
patterns, metadata/wire and wide C ABI remain separate required boundaries.
The backend stays frozen v3; capability/configuration truth is unchanged.

The previously pending20/21 rounding comparisons now pass all48 complete output
equalities under the14/20 load gate. Additional19/20 type-view controls pass48.
Both use four A/A and two ABBA blocks per input. Original19/20 full-LowIR callback
exports remain required positives: baseline19 rejects their exposed callable/
reference scalar metadata before timing. That failure and original declarations
are retained; the type views do not qualify or replace those artifact contracts.
The largest controls show possible roughly7–15% overhead, with small/noisy timings
and ordinary-control reversals; no speed improvement or whole-core performance
claim is made. Raw calibration, commands, hashes and observations are retained.

Next freeze one clean compiler and observe unchanged original core. Only a
completed analysis permits the genuine module-bodies-v13 metadata/LowIR attempt,
followed by actual original consumers. Whole PA25/31, the separate113-case
const-machine scope and every earlier retained positive remain incomplete.
