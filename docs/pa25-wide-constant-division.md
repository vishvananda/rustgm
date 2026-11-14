# Wide floating constant division

Clean release20 clears the original iterator loan failure and stops in
num/f128.rs799 at `const PIS_IN_180: f128 = 180.0 / consts::PI`. Its unchanged
120s diagnostic retains42310 declarations,25858 bodies and35120 attempts,
with zero failed/blocked functions. Completion remains0; no library artifact
exists. All722 input hashes,375 opens, six pins and fixed limits verify.
The27.7973s observation ran above the timing gate and supports no speed claim.

The [PA25 packet](../pa25/tests/spec/wide-constant-division/README.md) owns ordinary
constant division; [PA31](../pa31/tests/spec/wide-constant-division/README.md)
retains genuine Div/Neg/Clone/Copy declarations and original primitive operation
bodies. Every input and outcome was declared before its independent execution.
Four matrices observe74 results, including direct division, ordinary const-fn
calls and actual source Div const calls. Finite values, signed zeros and
infinities compare exactly. NaNs compare by classification because Rust does
not promise preservation of their arithmetic payload/sign.
[The primitive documentation](https://doc.rust-lang.org/core/primitive.f128.html)
records that distinction. Four ordinary feature/type/const-call/integer-fault
opponents and two source role/const-implementation opponents remain rejecting.

The implementation shares the existing bounded rational rounding kernel below
literal and constant consumers, in support/ieee_rounding. Decimal scanning stays
in literal; target division consumes already selected Primitive/FloatBits values.
Both formats use exact significands and nearest/ties-even rounding. The binary
scale remains in the output exponent, with early underflow/overflow, so opposite
exponent extremes do not create exponent-sized limb buffers. The literal's
2048-word cap remains. Quad values never pass through a host f64. FloatBits and
Scalar representations are unchanged; no dense node/declaration field or quota
is added. Original CPPGM attribution is preserved with the shared numerical code.

Only f16/f128 constant division is added. Other wide arithmetic/casts/patterns,
wire transport, runtime lowering and wide C ABI retain their explicit boundaries.
The native observers request byte addresses through the existing integer/pointer
ABI; they do not require wide runtime instructions. The frozen backend and its
capability/configuration truth are unchanged. No frontend operation is delegated
to another compiler or library implementation.

Focused receipt1794620649316462287 passes five exact tests in204.59s, all74
bit observations, six opponents, old loop/anonymous/literal controls and full
PA2 142/142. Strict selected/CLI Clippy, full formatting/audit pass. Receipt
1794621066380636715 passes all four actual RustGM/CPPGM/C byte-image executions
in37.95s against the independent Rust/C observations. Raw source setup failures
are retained: the first prefix lacked matching const_clone stability and the
actual Neg role; its next revision lacked Neg member stability. The corrected
third prefix passes without changing expected outcomes or original cutouts.

Scaling receipt1794621384701268646 passes exact8/32/128-initializer programs
in72.29s. Independent Rust/C execution agrees on every byte, including opposite
exponent extremes. Inference counts82/244/892 are27n/4+28; canonical11/35/131
isn+3; the verified actual output image is16n bytes. The current source/harness
hash replay retains these observations under wide-constant-division/scaling-replay/.
All315 harness tests,490 registered integration roots/13 binaries,44-stage and
one-component layout,128 upstream and126 Reference checks pass. No old text
reference changes; the142 PA2 outputs are exact.

The new private scaled-ratio boundary unit awaits CI. Its numeric separator
spelling was corrected after local public qualification; no local private harness
is rebuilt. Existing private long-decimal scratch tests still use the same public
literal path. Initial local audit gates requested more precise module filenames;
Clippy requested fixed-size slice iteration and a test semicolon. Raw failed
gates remain, with no semantic positive inverted or rejected opponent removed.

Full43546470095 passes every stage on this division sourcef5f3e2961, including
3075 private tests and the new ratio boundary unit. Clean release21 clears the
original division site and reaches midpoint's MAX*0.5 constant; the
[product continuation](pa25-wide-constant-products.md) records that next repair.
Module-bodies-v13 metadata/LowIR and genuine original native
consumers remain required. Whole PA25/31 and every earlier retained requirement
stay open, including both borrowed-result native consumers and the ordinary
unbounded manual Copy-wrapper positive. The113-case const-machine component
is unchanged. Paired same-profile20/21 observations have frozen literal/scalar
controls now pass48 complete output equalities under the unchanged14/20 gate.
Raw calibration/paired observations remain; no speed improvement is claimed.
