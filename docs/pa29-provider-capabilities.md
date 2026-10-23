# Provider capability review before checked-body inspection

Historical proposal. The bounded implementation and actual checked-original
results are recorded in [the capability checkpoint](pa29-provider-capability-checkpoint.md).
In particular, the implemented const_destruct boundary preserves qualified
bounds; its feature request supplies no live-destruction capability.

Base: clean `8df70b06`, after borrowed validators `6049ded5`. This is a source
review and bounded implementation proposal, not admission of these constructs.
No wire descriptor or imported permission is enabled. B confirmed that policy,
role validators and source attribute hooks are free; callable const eligibility,
selection callables/forwarding, Engine/State and one-use payment remain B-owned.

The two unchanged targets remain separate:

- `semantic-provider-qualified.rs` retains full actual semantic declarations,
  attributes and clone_from body, with the explicitly declared API-stability
  removal from the independent matrix.
- `original-attributes-provider-qualified.rs` additionally retains original API
  and const-stability attributes. It cannot inherit acceptance from the first.

Both were independently object/native-qualified before this work. Source hashes,
all previous disagreements and Rust commands remain in the original inventory.

## Exact feature requests

`resolve/bindings/policy.rs::Request::named` currently rejects every unrecognized
root feature before declaration claims. This masks the attribute gaps below.

| Exact gate | Actual use/consumer | Existing mechanism | Missing boundary and proposed behavior |
| --- | --- | --- | --- |
| no_core | source-provider policy | explicit own provider/current identity, absent Bootstrap tables | Already admitted; unchanged. |
| lang_items | actual declaration claims | reserved LangItem identity plus completed role validators | Already admitted; unchanged. |
| rustc_attrs | internal declaration attributes | currently intrinsic, deny-explicit-impl and dyn claims | Already admitted as a request; each active attribute still needs a validated consumer. |
| const_trait_impl | const traits/impls and qualified bounds | admission walker; actual const flags; full Ordinary/Const/Conditional predicates; B's effect/payment consumers | Already admitted. Do not change paid/unpaid or callable eligibility. |
| auto_traits | actual unsafe auto Freeze | source admission, exact Freeze header, structural children/negative impls and formation proof | Already admitted; unchanged. |
| fundamental | actual sizing trio attributes | real sizing identities, deny-explicit-impl, physical sizing/metadata kernels | Add a capability request, but admit its active attribute only on the exact validated sizing roles under the restrictions below. Arbitrary uses reject. |
| const_destruct | clone_from's `Self: [const] Destruct` | exact Destruct role and qualified clauses already exist | Add request/use tracking for the actual Destruct bound, retaining all obligations. A request is not const-destruction eligibility; unsupported demanded Const Destruct still rejects. |
| staged_api | original stable/unstable attributes | retained attribute syntax only | Keep explicit unsupported until API-stability validation/propagation/use enforcement exists. |
| const_clone | original rustc_const_unstable API feature | declared const trait flags exist, but not API const-stability | Library feature request may later be recognized independently. Active const-stability use stays unsupported; it cannot become const permission. Not needed by the semantic target. |
| sized_hierarchy | original unstable MetaSized/PointeeSized API | exact language sizing semantics exist | API availability is separate. Keep full-attribute target pending until stability enforcement exists; do not reinterpret this as a sizing proof. |

The semantic target's unsupported requests are exactly fundamental and
const_destruct. The full-attribute target additionally requests staged_api,
const_clone and sized_hierarchy. No automatic feature deletion is proposed.

## Exact attribute capability table

| Exact attribute | Current active consumer | Existing semantics | Required missing consumer / decision |
| --- | --- | --- | --- |
| lang | reservations then compiler_items sizing/role validators | exact actual declaration and Self/member/signature checks | Preserve; familiar names are not evidence. |
| rustc_deny_explicit_impl | reservation set; declarations/traits/implementations | rejects explicit impls of that exact source trait; marker validation requires it | Preserve and require on scoped fundamental/specialization/coinductive sizing claims. |
| rustc_dyn_incompatible_trait | reservations::validate_unsizing_claims; dyn_compat/contracts | only exact actual Unsize is currently allowed/forced incompatible | Extend narrowly to actual Sized and Destruct. Retain the claim and consult it in dyn compatibility after the matching declaration family validates; same-shaped user traits do not inherit it. |
| fundamental | currently ignored by claim wildcard; feature request rejects first | source sizing trio has fixed intrinsic interpretation and forbids explicit impls when its actual deny marker is present | Record/validate only actual Sized/MetaSized/PointeeSized with the deny restriction and complete pinned header. This does not implement general fundamental trait/type coherence; other owners reject. |
| rustc_specialization_trait | currently ignored | no general specialization; the actual sizing roles are compiler-supplied and deny explicit impls | Retain only on the same fixed sizing roles. All explicit impls remain forbidden, specialization syntax remains unsupported, and no specialization permission is minted. Other owners reject. |
| rustc_coinductive | currently ignored | sizing uses checked intrinsic physical/metadata rules; ordinary arbitrary trait cycles still reject; source Freeze has its own structural cycle evidence | Retain only actual fixed sizing roles under the same complete profile. No generic coinductive trait rule or success-from-cycle is added. Arbitrary owners reject. |
| rustc_trivial_field_reads | currently ignored | This is dead-code diagnostic behavior, not a move/Copy/trait/CTFE permission | Validate its no-argument form and actual trait owner; initially scope to actual Clone. Keep diagnostic classification separate from semantic permission bits. Do not change field-read legality. |
| rustc_diagnostic_item | currently ignored | no body/trait privilege is derived from Clone/Copy/convert_identity spellings | Validate string/owner and retain real diagnostic binding/provenance. Any initial bounded name support must reject unknown active bindings explicitly; no factory or body replacement. Duplicate bindings must reject. |
| diagnostic::on_unimplemented | retained syntax; no custom trait-error rendering | ordinary selection and errors remain authoritative | Diagnostic-only; no trait proof. Preserve the declared source provenance and unsupported custom-rendering boundary. |
| stable / unstable | retained but no API stability enforcement | none | Reject active staged API contracts explicitly rather than treating them as semantic no-ops or API permission. |
| rustc_const_stable / rustc_const_unstable | retained but no const-stability enforcement | declared const eligibility/effect machinery is separate | Reject active use until exact availability and caller feature checks exist; never replace B's const debt proof. |
| inline / must_use / doc / allow | existing nonsemantic attribute handling | no inline, warning or documentation attribute supplies runtime/body authority | Preserve current diagnostic/hint behavior; do not include in semantic capability masks. |

Primary pinned compiler evidence corrects an earlier overbroad classification:
`compiler/rustc_passes/src/dead.rs:436–452` uses trivial_field_reads only to
exclude automatically derived impl reads from dead-code analysis. It is not
an authorization to read fields. `rustc_hir_analysis/src/collect.rs:956–1000`
separately retains fundamental, coinductive, specialization, deny-impl and forced
dyn-incompatibility facts. `impl_wf_check/min_specialization.rs:50–58` describes
specialization applicability; general specialization is not supplied here.
Stability attributes are a separate API/const-availability contract.

## Existing destruction and dyn limits

`sema/traits/selection/search/destructibility.rs::source_destruct_result`
explicitly rejects non-Ordinary Destruct with `source const destruction
eligibility is not implemented`. Keep this rejection unless an actual caller's
qualified premise or B's proved payment supplies the exact obligation. The
clone_from original may assume its written conditional requirement; a later
instance/caller must pay it. Do not broaden ordinary Destruct's universal rule
to Const or Conditional.

Forced dyn incompatibility is currently a one-off actual Unsize check in
`sema/dyn_compat/contracts.rs::collect`. The proposed typed attribute lookup
reuses that demand boundary and inherited contract traversal. It does not just
remove the reservation rejection: `dyn Destruct` and supertraits inheriting it
must actually reject. Ordinary same-spelled user traits retain their normal
compatibility rules.

## Proposed bounded implementation boundary

Add Fundamental/ConstDestruct request bits to the source feature set, preserving
its compact representation. The resolved actual Destruct bound consumes the
const_destruct use gate; a lookalike spelling does not. Unsupported const
destruction remains a real later error, not paid by this bit.

Collect known no-argument header claims once during the existing configured-owner
claim scan. Validate each against that declaration's complete actual LangItem
claims independently of attribute order. Retain a compact fixed per-role mask
in the existing cold provider reservation/catalog; no per-body/per-type fields,
global scan per use or second trait solver. Scope fundamental/specialization/
coinductive to the three sizing roles with actual deny-explicit-impl and their
complete shared header validation. Preserve current minimal providers that do
not request those attributes.

Keep diagnostic-only attributes in distinct typed claim/provenance data; do not
pack trivial-field-read or diagnostic-name state into proof permissions. API
stability remains a separate unsupported target. Measure cold reservation/
catalog growth and borrowed-header layout; Owner, OwnerKind, canonical Predicate
and IR Value must remain unchanged. Claim visits, retained slots and failure work
are bounded/charged in the existing pass, not rediscovered from source later.

Before any codec work, run the unchanged semantic target through ordinary
original proof and actual selected identity instances. Inspect only its actual
prepared Clone::clone_from/identity originals and paired Contracts. Full provided
body, exact Self/lifetime mappings, all qualified premises, promotion, escape,
loans, cleanup and original/instance proof stay mandatory. If the next failure
is a missing semantic consumer, preserve it and fix the owning phase; do not
invent an opcode or rewrite the source to fit the transport.

## Declared opposing controls

Freeze an independent source matrix before code changes:

- unchanged semantic-provider-qualified and full-attribute provider outcomes;
  the latter remains a separate unsupported API frontier;
- fundamental/coinductive/specialization on an unregistered lookalike trait,
  ordinary user trait, nontrait, or wrong actual language header: reject at
  explicit unsupported/invalid claim boundaries, rather than today's ignore;
- missing fundamental/rustc_attrs gate, nonempty malformed attribute, duplicate
  or conflicting claims, reordered attributes, and foreign role identity;
- attempted explicit sizing impl despite the new attributes: still reject;
- paid ordinary sizing/reference/metadata controls and unpaid type obligations;
- dyn Destruct/Sized and inherited incompatible principal: reject, with a normal
  same-spelled compatible user trait as the opposite;
- const_destruct request alone grants no const destruction or callable payment;
  retain actual paid/unpaid B controls and missing/wrong conditional premises;
- no-argument trivial-field-read marking cannot legalize an invalid move/read;
  malformed diagnostic item strings, wrong owner and duplicate identity reject;
- active stable/unstable/const-stability contracts remain explicitly unsupported,
  including when their library feature request is recognized;
- original semantic clone_from body and its given conditional Destruct clause
  remain byte-identical; invalid unused body/opposing lifetime still rejects.

Valid-but-unsupported arbitrary attributes, staged APIs and later source-role
families remain requirements for full core/alloc/std, not language negatives.
