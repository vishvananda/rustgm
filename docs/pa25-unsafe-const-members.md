# Unsafe generic const members

Clean original-core37 stops at slice/cmp.rs:19/file225:534. Slice PartialEq calls
unsafe SlicePartialEq::equal_same_length under T:[const]PartialEq<U>; a conditional
blanket implementation proves the helper trait. The readonly defining path admitted
only safe signatures, then demanded an exact helper-trait premise in its fallback.

A transient signature witness now binds the complete selected callable and its
opened signature. Safe requests retain the existing validated Fn/Rust signature
rules. Unsafe Rust requests additionally require the exact typed source Call,
result and SourceCallUse::call_safety row keyed by current body/node/signature.
Explicit-block permission stays distinct from UnsafeFunction, which also requires
the actual caller signature to be unsafe. The witness admits only that target,
opened signature and public unsafe Rust signature; there is no ABI retagging.
Source caller/report, current full call, actual trait header, selected method/UFCS,
conditional effect, implementation constness, borrow/cleanup and actual Code proof
remain required. Direct, access and Try requests keep separate coherent validators.

Ordinary const bodies now retain the same already discharged safety rows in their
existing local call-context owner. Ordinary runtime-only bodies keep their prior
path. Existing prepaid sorting, storage bounds and indexed lookup remain; no new
dense field, persistent cache, metadata/backend/storage/quota change occurs.
A lookup pays a fixed32-step upper bound for at most one million retained rows.

[PA25 ordinary minima](../pa25/tests/spec/unsafe-const-members/manifest.json) and
[PA31 source roles](../pa31/tests/spec/unsafe-const-members/manifest.json) each
predict17 outcomes:12 accept/5 reject, independently confirmed by Rust1.98.
All34 complete CLI outcomes qualify, including blanket/direct premises, safe and
runtime controls, composite wrapper owner, raw-pointer signature shape, unsafe
function caller, CTFE/runtime and scaling. Missing conditional effect, ordinary
implementation, missing unsafe context, foreign application and wrong argument
remain semantic rejections. The support Base call was corrected to explicit
<T as Base<U>>::read before any candidate execution; initial Rust E0790 and the
original inputs remain evidence. Predictions were never inverted. Raw-pointer
support reproduces only the original signature, not full slice comparison behavior.

Raw evidence under obj/parallel/unsafe-const-members/:

- declaration.json, ufcs-setup-correction.json, unsafe-caller-declaration.json,
  baseline2/ authenticate independent outcomes and immutable37 reproduction.
  Direct-premise, safe and runtime controls already pass in the baseline;
  unsafe blanket calls reproduce the exact original premise failure.
- fast2=1794703721225702094 passes four exact controls across two targets in
  219.68s: new legality/scaling, prior Try and borrowed fields. Strict selected/
  production/CLI Clippy, full format/audit pass. fast1 stopped on function length;
  direct-call validation was factored beside the existing protocol validators.
- native1794703991312596252 passes five groups/twelve actual programs in40.48s.
  The two new actual source runtime/CTFE consumers each print41/59; ten old
  native programs pass. Independent Rust runs both new consumers with41/59.
- replay/ authenticates3967 compiler sources and all packet hashes,34 complete
  ordinary/source outcomes and exact lowering/scaling. Its immutable dev-fast CLI
  is1ccf4aff5fc49aaa3057eea5c30f0d46d7566d70a0ba10a1492e1460efb2c1d0.
  Source8/32/128 inference/canonical work is3392/1358,13064/3928,51752/14184.
  No timing or whole-core speed claim is made.
- 315 harness tests,497 roots/13 binaries,44 stages/one component,128 pinned
  upstream fixtures and126 Reference files pass.

A new private guard checks actual explicit-block/unsafe-function witnesses and
rejects changed target, missing context and a copied typed operation without its
exact safety row. It and the prior Try guard await combined full CI (expected3087
private); no cold local private harness was run. Prior full43656293922 on8560c4dbb
(borrowed product05d1beedd) passed every stage,3085 release4.97s/debug45.56s.

Next publish one combined full successor, freeze clean38 and retry genuine original
core with unchanged722 sources/six reviewed pins/120s/quotas and postcheck.
Core37 remains the latest original observation until that succeeds. No original
metadata/LowIR/native consumer artifact exists. WholePA25–44, originalv13/v14,
Future/async, all older CTFE/bootstrap/native/closure/ABI requirements and the
released113-case PA25 component retain their prior status. New simple unsafe CTFE
payment does not close earlier aggregate/open/lifetime-bearing accepting consumers.
