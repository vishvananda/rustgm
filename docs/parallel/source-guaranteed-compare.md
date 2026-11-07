# Actual source pointer guaranteed comparison

The source intrinsic `ptr_guaranteed_cmp` now has a checked direct CTFE operation
and ordinary native fallback consumption. The complete pinned intrinsic function
is retained in pa31/tests/spec/source-guaranteed-compare/original-function.txt.
The declaration remains existing intrinsic profile 53; no registry, hot field,
value variant, wire profile or backend operation was added.

The do-not-const-check word now retains its general defining-body meaning for
actual written const functions: the same local provider, completed header,
function and source body must authenticate it. It grants no executable const
permission. Ordinary marked calls still reach the existing non-const Code fence.
Existing panic-role signature/indirect contracts remain checked. The old unused
ordinary indirect-marker private negative was independently disproved by pinned
Rust and replaced by an actual positive; its execution is pending CI. Broader
trait/default/comptime marker profiles remain required.

The transient intrinsic descriptor authenticates the actual completed declaration,
full one-type closed Sized instance, safe Rust signature, current source header,
caller Code, opened call signature, evaluated pointer arguments and const-call
effect. The evaluator consumes existing canonical WordAddress/Origin/layout data.
It never reads pointed storage. Integer bits, original static identity and strict
in-bounds distinctions, allocation alignment residues and conservative promoted,
local and function provenance follow pinned rustc_const_eval. Unsupported valid
symbolic profiles and imported operation proof retain explicit fences.

Native execution materializes the real fallback body. Its first observer exposed
an existing missing raw-pointer operator erasure: the original PartialEq body
recursed into itself. The shared typed primitive comparison path now consumes
same-type raw-pointer ==/!= only after its real source trait/where obligations.
Const/conditional contexts retain the raw-pointer comparison prohibition. Ordinary
explicit method calls retain their actual bodies. Existing ABI pointer lowering
still owns data/metadata representation, and pointer promotion already declines
folding. No ptrcmp-specific equality shim or fabricated call/body is used.

Independent Rust controls distinguish the original fallback (`1 0 1`), changed
ordinary fallback (`99 99 1`) and altered real Eq implementation (operator `1 0`,
explicit method `0 0`). The same three actual CPPGM object/C executables pass.
The initial native recursion/segfault receipt and exact LowIR remain retained.

Final bounded receipt: obj/parallel/source-transport/guaranteed-compare/final2.
Format/architecture, all-target/all-feature strict metadata (95.98s), six new
public controls (17 provenance cases, one-past/live reads, actual expired-read
negative, 15 independent hard negatives), existing cold3/Word3/weak4, three native
observers, 282 harness tests/458 roots, and upstream127/Reference126 pass. No
private test execution, release/full-PA qualification or new original-core
observation is claimed.

Measured transient Selection is 12B and Word is 32B. Declarations7120, Body456,
Value24, Code304 and Frame248 remain unchanged. Ordinary nonconst paths allocate
no qualifier metadata. Fixed descriptor work is charged through the existing
source owner; reached calls charge64 plus existing canonical/layout work. Failed
layout adapters conservatively consume the remaining admitted work bound rather
than refunding unreported failure work. Existing canonical caches own their normal
allocations; no second cache or permission ledger exists.

The reference packet preserves all original register_tool request/declaration
inputs and their real Source setup failures as PA40 requirements. A separately
predicted and Rust-qualified authored support profile omits that unused request
and declaration while retaining the complete original intrinsic function/tool
attribute. Only this explicit support profile is counted as candidate passing.
Full original standard-module closure, broader pointer symbolic/metadata cases,
conditional const payment, source transport and general CTFE cleanup remain
required. Global tracker/main publication and unchanged-core observation belong
to Root.
