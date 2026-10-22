# PA25 completion gaps after caller-paid premises

This is a requirement inventory, not a reduced release scope. PA25 remains
planned. Its owning README requires const functions, statics, promotions and
symbolic allocation provenance, including const restrictions, pointers, recursive
calls, limits and generics, plus the entry-free constants inspection contract.
The historical checkpoint documents describe their own snapshots; their old
"still required" paragraphs are not evidence of current implementation gaps.

## Demonstrated language blockers

Two original independently Rust-positive families remain in this lane after the
caller-paid checkpoint. Their sources and declared positive outcomes are intact:

- `pa25/tests/spec/open-type-initializer-context/generic-pattern.rs`: the unused
  rigid `T` in `S::<T>::C` does not affect the constant's scalar value, but the
  current source-computation shape/materialization boundary still rejects this
  generic original. Supporting it must preserve original body/const eligibility,
  actual parameter identity and every eventual selected-instance proof. It may
  not make arbitrary generic dependent patterns concrete or relax ordinary
  Associated admission. This is the next generic-template/value consumer gap.
- `pa25/tests/spec/inline-query-context/closure.rs` and `paid-lifetime.rs`: the
  nested closure/inline associated reference result requires the checked parent
  output contract. The current parent namespace/result transport rejects that
  conditional interface. It must transport/prove the real conditional child and
  caller obligations, retaining private storage/escape upper bounds. Clearing an
  OutputLink, assuming its conclusion, or equating lifetimes by shape is invalid.

The original `open-type-initializer-context/paid.rs` is now implemented, with
source/object/native evidence in the caller-premise handoff. The lifetime-negative
`inline-query-context/unpaid-lifetime.rs` already reaches actual lifetime rejection;
it must not remain described as a positive implementation frontier. The integrator
owns its stale draft diagnostic correction.

These are known failures from declared controls, not a proof that all other
language families are complete. Reconcile the whole owning source inventory on
the integrated compiler before activation; do not infer completion from an old
122-case subset or from searching for unsupported error strings.

## Required families with existing implementation evidence

| Required family | Current evidence and remaining release obligation |
| --- | --- |
| Const functions, control flow, recursion and restrictions | `dev/tests/cfg_execution.rs` and `required_values.rs` qualify iterative checked execution, values/native programs, recursion and deterministic limits; inline/associated matrices cover generic selection and unused/skipped invalid bodies. Keep complete instance identity and selected callee proof. The two blockers above remain part of generics, rather than exclusions. |
| Statics and promotion | Original controls include `associated-execution/static-suspension.rs`, `inline-written/static.rs`, `static-local.rs`, `promotion.rs`, and `lifetime-initializer-results/{static-reference,promoted-reference,dangling-reference}.rs`. Reconcile all current storage/forward/static-context and optional-promotion controls on the integrated cut. No additional current PA25-positive failure is established by this lane's receipts; this is not a blanket completeness claim. |
| Symbolic provenance and allocation validity | Existing allocation/static/projection/literal identities, permissions, liveness, initialized storage, bounds and escape checks are shared typed machinery. The separate C lane now supplies later byte/relocation/enum/range cases. Reconcile its integrated outcomes, preserving each earliest owning PA. Raw intrinsics or future-stage unsupported operations must not be confused with missing PA25 reference/static behavior or silently exempted from the full provenance responsibility. |
| Bounds and limits | Interpreter call/instruction/storage limits, dependency cycles versus recursion, retained query/dispatcher limits and inspection graph limits have private/source controls. Caller payment now charges raw predicates, preparation, solve work and retained rows. Keep both resource-error and ordinary valid cases, plus shared-DAG/scaling controls. A compiler signal/panic/timeout is never a successful limit diagnostic. |
| Slice/text and patterns | Named-pattern and discriminant continuations, indexed compressed reads, literal backing and C's rest/Subslice/Subarray implementation exist. Reconcile the integrated receipts. The pinned Rust1.98 text-pattern const-function hypotheses reject for unstable const PartialEq; those retained negatives are not missing required positive functionality. |
| Constants inspection | `docs/pa25-constants-view.md` specifies checked entry-free emit/count, no partial output, deterministic graph identity/ordering, deferred generic declarations versus completed roots and bounded iteration. Inspector tests cover values, static aliases, large repeats, unused-invalid input and limits. Final integrated vocabulary must reflect all currently supported payloads; source-only values remain deferred and never masquerade as materialized typed roots. |

Release work remains separate from these implementation blockers: finalize the
handout, complete owning manifest and independently checked expected modes,
current constants-view grammar/counts, generated sidecars, PA13/PA16 prerequisites,
full original source/native inventory, all active earlier comparisons and frozen
performance review. PA25 currently has no released `tests/manifest.json`; generating
that contract and activating status are integrator gates, not fulfilled by this
worker's draft outcome files. The new PA20 byte differences also require central
review against current public907dcd0, not automatic acceptance from an older base.

## PA29 contracts essential for upstream core

The upstream core→alloc→std goal makes generic/const-body transport a material
cross-crate implementation dependency. Existing closed records/arrays/C layout
and reexport work is useful foundation; it does not supply these remaining contracts:

1. Preserve external defining identities independently of exported names. Remap
   every nominal/field/variant, trait/impl/member, lifetime/type/const parameter,
   projection application and full Instance across each artifact. Retain unused
   binder/argument slots, genuine nested binders and canonical dependency identities.
2. Export/import trait and inherent member selection inputs: declared/quantified
   predicates, associated equalities/defaults and validated impl conditions. Caller
   requirements remain debts; metadata does not mint a callable or const permit.
   Canonical core language items must retain their exact defining identities.
3. Transport checked generic and const-function/initializer body templates and
   their real contracts, normalization debts, ownership/drop facts and source-free
   provenance. Reprove actual imported instances and caller requirements locally.
   Do not ship rendered text, reparse source, clone proof graphs per demand or
   serialize compiler-local paid/source handles, epochs, service reservations or
   cached positive permissions.
4. Give imported nested closure/anonymous owners a real defining-body identity and
   diagnostic provenance. Existing NodeKey-based source caller publication cannot
   be satisfied by inventing a source node. Closure environments/entries and full
   enclosing contexts must remain distinct from public callable ABI.
5. Preserve cross-crate data and code symbols, reference/slice metadata, relocation
   identity, function and vtable targets and target ABI facts. Extend unsupported
   generic/enum/body/value families explicitly with bounded graph/cycle validation;
   keep existing wire profiles byte-compatible. Image bytes alone cannot serialize
   symbolic provenance. Upstream constant bodies often need types/layouts from
   dependencies even before runtime monomorphization.

Root owns the current metadata implementation and global roadmap. This list is a
handoff dependency interface for the forthcoming generic-body lane; it introduces
no new shared schema, bypass or exemption for the remaining PA25 language work.
