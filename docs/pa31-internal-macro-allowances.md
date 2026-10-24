# Macro-specific internal feature allowances: proposed local successor

This is the LIB014 interface and preimplementation contract, based on the
qualified local panic source checkpoint `2c31704b` and docs `4dfce465`.
No production implementation, new oracle observation or candidate support is
claimed here. The original core frontier is macros/mod.rs line 41 byte 1250,
`#[allow_internal_unstable(panic_internals)]` on `macro_rules! assert_eq`.
The owning behavior is PA31 library permissions using PA8 token provenance.

## Pinned rule and honest scope

Pinned `compiler/rustc_span/src/lib.rs::Span::allows_unstable` inspects only the
raw token context's outer expansion's actual feature list. It does not union
ancestor lists. `compiler/rustc_middle/src/middle/stability.rs::eval_stability`
consults this fact when checking cross-crate unstable APIs; same-crate ordinary
unstable APIs are already allowed. These facts differ from edition_panic's
explicit call-site walk. An unannotated inner macro's literal does not borrow
an ancestor's allowance, while a captured expression already bearing its own
allowed context keeps that context through an unannotated forwarding macro.

Current `resolve::stability::Inventory` is local-only and rejects dependency
stability contracts. There is no existing foreign regular-API use checker to
patch honestly. This first successor may admit and retain the exact authentic
local list and expose the per-token fact; foreign API/metadata availability
remains explicitly unsupported until its actual checker exists. Nothing here
changes root feature requests, compiler capability bits, const eligibility,
trait payments, selected callback authority, intrinsic declaration roles or
ordinary user permissions. Source provider authorization remains unchanged.

## Stored representation and producers

Add a small closed `InternalFeature` enum for the implemented local profile:
`EditionPanic`, `PanicInternals`, `CoreIntrinsics`. A crate-visible re-export through
`resolve::bindings` lets later semantic consumers name `InternalFeature` without
exposing the private contract owner. A private `InternalFeatures(u8)`
retains exact parsed membership in the existing cold `Allowance` witness; it
never calls `policy::Features::insert` or `Library::request`. Unknown names,
repeated attributes, duplicate entries and unsupported forms retain explicit
profile diagnostics rather than being mislabeled Rust rejections. The initial
single-attribute profile preserves the already retained repeated-attribute
Rust-positive opponent. A broader allowance inventory is read-only work.

`panic_contracts::allowance(owner, attribute, forest)` returns this set from the
actual typed list. Existing full Attribute, retained Forest-tree, active NodeKey,
feature-authorization and defining-provider checks remain mandatory. Parsing,
new witness storage and final publication continue charging the existing
SourceCore Contracts owner before work/growth, with sticky exhausted budgets
and no usable partial row. No Package pointer or hot record field is added.
Measure cold Allowance/Declaration/Entry/SourceCore/Definition sizes; do not
infer the actual change from padding arithmetic.

`Allowance::allows(InternalFeature) -> bool` and
`Declaration::allows(InternalFeature) -> bool` expose membership read-only.
The stability attribute collector still requires its exact owner/Attribute
witness; a matching name or feature set cannot substitute for that witness.

Both current presence-to-edition shortcuts must change together:

- `Engine::define` derives Definition.edition_panic from
  `contract.allows(EditionPanic)`, not allowance presence.
- `Engine::panic_definition` authenticates that same membership against the
  retained definition bool; Frame.skip remains this exact edition-only fact.

Thus panic_internals/core_intrinsics alone cannot skip an edition frame. The
existing edition suffix cache and its full Frame stamp remain edition-only;
there is no general permission ancestor cache or permission union.

## Per-token query and consumers

Proposed interface on the existing report:
`Report::allows_internal_feature(site: Location, feature: InternalFeature)
 -> Result<bool, Error>` (crate-private). Location must be the actual use token
span from the owning typed consumer; callers must not replace a captured use's
span with an enclosing call, diagnostic origin, macro name or definition span.
The API validates bounds and current source identity. It returns false with
no allocation/work for ordinary packages without source bindings. An admitted
source provider charges the existing Contracts owner before lookup, including
warm hits, then obtains the actual raw `Package::syntax_context` and its single
`Marks::outer` expansion. ROOT returns false. It does not normalize to ItemContext
or LocalContext and does not walk expansion parents.

Factor the existing exact definition/frame authentication into one borrowed
cold `FrameView` shared by Engine panic selection and Report's read-only query:
Package, actual definitions, actual invocations and the current Resolution.
Authenticate real context/index, invocation Call NodeKey, active source and
original span, actual definition NodeKey/module/scope, provider and retained
Header witness before returning that definition's feature membership. Unknown
or foreign source/mark/frame/definition/owner rejects; a genuine unannotated
macro returns false. No new Report/Package/Body/Code/Value/Type/frame fields or
second solver is needed. The existing indexed invocation vector and declaration
map give O(1) feature queries after token-context resolution; no attribute scan
or ancestor union is introduced. Charge any traversed retained-origin links in
the existing context helper before work if that helper needs a counted variant.

The immediate semantic consumer is the existing edition selection, now using
its exact typed bit. The new Report query is exercised by actual source-token
controls and is ready for the future completed foreign API checker; it does
not constitute permission to execute an unstable API by itself. Future API
checking must use its genuine Rust use-site span; mixed literal/captured path
segments are separately declared exploratory evidence, so no guessed terminal
identifier rule is built into this interface.

`Stream::transcribe_with` already applies marks only to Definition pieces;
Invocation pieces preserve actual raw context. Opaque captures retain actual
registry payload identity and their own per-token contexts. Those producers
remain unchanged. The new query consumes them rather than assigning a grant
at the enclosing invocation or granting all captured tokens a literal's mark.

## Declared source and private controls

`pa31/tests/spec/source-panic-macros/internal-allowances-contract.json` declares
21 new inputs before any oracle invocation: four local own-provider header/list
cases; one valid-but-initially-unsupported repeated-attribute case; and sixteen
reference API provenance cases using actual `core::intrinsics::likely` and
`core::panicking::panic`. Predicted outcomes are twelve accepts and nine rejects;
accepted programs declare native stdout `7\n`. The known-false panic branch
avoids executing panic but must still undergo feature checking. Foreign API
cases remain candidate-pending even if Rust accepts them. One mixed captured
terminal-path case explicitly tests the actual reference check-span rule;
a disagreement must be retained, not corrected in place.

Retain earlier permission owner, missing-feature, unsafe/malformed, unknown and
empty-list opponents, and the entire 168-observation/35-native oracle corpus.
Planned new private controls use actual parsed/transcribed tokens to distinguish
literal versus captured expression/path/tt, exact outer inner/outer permission,
already-granted forwarded captures, independent root callers, and typed feature
membership versus edition skipping. Warm wrong full Attribute/Forest/source,
wrong frame/definition/owner, cross-report same-numbered identities, exhausted
work/storage and no-allowance/no-source fast paths remain required. Width and
nested forwarding controls must report actual query work and cold payload;
ordinary paths cannot allocate a speculative feature map or a second index.

Pinned macros/mod.rs also lists assert_matches, formatting, lint, rustc_attrs,
contracts and other allowances, including multiple attributes. Their existence
is inventory evidence, not admission or implementation. Full source core,
foreign macro transport (LIB007), general edition closure and later semantic
feature owners remain open. No new heavy job is authorized by this document.
