# Source provider capabilities and actual original bodies

This focused successor to `11295e30` implements the bounded capability proposal.
The previously qualified neutral header views at `6049ded5` remain a separate
checkpoint. No artifact version, source-client mode, assignment activation or
cross-crate execution permission is added here. Full core/alloc/std and the
complete generic transport matrix remain required.

Combined release and immutable-binary performance qualification are recorded in
the [integrated checkpoint](source-capabilities-dyn-returns-checkpoint.md).

## Implemented boundary

Configured-owner collection retains exact sizing-role attributes in a one-byte
mask for each of the twenty existing language roles. Fundamental, specialization
and coinductive claims require the actual Sized/MetaSized/PointeeSized identity,
its real deny-explicit-impl restriction and complete validated header. Arbitrary
owners reject explicitly. These flags add no general specialization, coherence,
cycle-success or implementation permission.

Forced dyn incompatibility now consults the completed actual Sized and Destruct
roles, including inherited contracts, alongside the existing Unsize restriction.
Unknown or incomplete role claims cannot answer those queries successfully.
Diagnostic items bind real Clone/Copy/identity declarations; trivial-field-read
metadata is diagnostic-only. Neither replaces a method body, permits a move or
pays a trait/caller obligation. Active API/const-stability contracts remain
unsupported, separately from the semantic provider.

The const_destruct request is recognized without capability. A written conditional
Destruct premise remains admissible and qualified. This request neither pays that
premise nor implements const destruction. The earlier draft incorrectly gated
the bound; that draft was removed before qualification. Live-drop eligibility
and unsupported execution remain separate owning consumers.

## Real originals observed

The unchanged `semantic-provider-qualified.rs` passes ordinary construction,
promotion, complete original proof and exact paired Checked lookup. The observer
requires original `Function { execution: None }`, matching declaration/signature,
body/Contract/proof source and key, and the exact retained Environment Arc.

| Original | Actual typed body | Retained contract and cleanup |
| --- | --- | --- |
| Clone::clone_from | Two reference parameters and two dereference projections; Borrow, Constrain, actual clone_fn FunctionItem, Call, Store through `&mut Self`, unit return. Eleven statements in one block. | Two universals, exact Self parameter, fourteen lifetime obligations, four possible cleanup sites including replacement. The actual Environment retains `Self: [const] Destruct`. |
| identity | One rigid parameter, local Read, return. Three statements in one block. | Zero universals, three lifetime obligations, one retained proof, four possible cleanup sites. Actual sizing premises remain in its Environment. |

The original failed observer assumed all trait inputs belonged in
`Contract.type_assumptions`. That field intentionally contains caller-outlives
clauses. The correction inspects `Template.environment.trait_clauses()` and
matches the actual Self, qualified Destruct declaration and empty arguments;
it adds no trait premises to a region Contract. The failure is retained.

The complete actual local/projection/value/statement/CFG/cleanup dump is at
`obj/parallel/pa29-source-headers/capabilities-recovery-second/observer-layout.stdout`,
SHA-256 `c69a348c7b141e376f8b2a348085731dd5b42253737f8d250dce7bab437f44b7`.
These are actual proved original bodies, not serialized proof or instance/CTFE
permission. No imported body recipe is inferred from source spelling.

## Layout and bounded ownership

| Model | Before | After |
| --- | ---: | ---: |
| Source Catalog, optional boxed provider only | 504 | 528 |
| Reservations, optional boxed provider only | 512 | 600 |
| Owner / OwnerKind | 32 / 20 | 32 / 20 |
| DeclarationOrigin / borrowed Trait view | 12 / 120 | 12 / 120 |
| FunctionHeader / TraitContract | 48 / 136 | 48 / 136 |
| canonical Predicate / Value | 12 / 24 | 12 / 24 |
| Package / Resolution / Declarations | 240 / 2032 / 7048 | 240 / 2032 / 7048 |

The fixed role masks and diagnostic slots add 112 bytes across the two cold
provider records, with no per-role allocation. Ordinary sources still have no
provider reservation/catalog or imported catalog. Claims reuse the existing
configured-owner pass; claim and successful retained-row visits increment its
existing step counter. Role lookup loops have a fixed twenty-role bound. No
whole-body scan or new proof graph is introduced.

## Focused evidence and retained failures

Evidence root: `obj/parallel/pa29-source-headers/`.

- `capabilities-rust-first/receipt.json`: all seventeen independently declared
  pinned Rust object outcomes match, including valid-but-unsupported arbitrary
  semantic attributes. SHA-256
  `8f1b6398796597f24c7c328f0daffa00e9e7409ccc198652e3e2fd35743b5410`.
- `capabilities-conditional-rust/receipt.json`: preserved failed bound-only
  hypothesis (missing Freeze), plus the Rust live-drop gate pair. The separate
  corrected Freeze source passes in `capabilities-conditional-second/receipt.json`
  (SHA-256 `6a7611168432d3addc488fbc68c4bcc11ddc8c691abbc9aa44977ee7d74dc73d`).
- `capabilities-first`: format/audit passed; two match-arm lints and a private
  test visibility error stopped strict checking before execution.
- `capabilities-second`: format/audit passed; the new observer's needless
  borrow stopped strict checking before execution.
- `capabilities-third`: all-target/all-feature strict, registration, twenty
  source integration tests, the seventeen-case capability matrix, qualified
  bound preservation, pending-header rejection and cold layout tests passed.
  The observer's region-only-field assertion failed and remains recorded.
- `capabilities-recovery`: private iterator `.is_empty()` used an unstable
  ExactSizeIterator API; no tests executed. The correction uses the existing
  stable `.len() == 0` spelling.
- `capabilities-recovery-second`: fresh format, lib/tests all-feature strict,
  one debug private build and exact actual-original/header-layout tests passed.
  No production source differs from the passing third source batch; only the
  private observer changed. Earlier source/audit/capability results are retained,
  not described as freshly rerun.

Original semantic and full-attribute providers and their independently qualified
hashes remain unchanged. The full-attribute target still has the declared staged
API/const-stability frontier. No full release/native/text sweep or broad timing
run was repeated in this worker.

## Next executable transport boundary

The observed bodies require real references/binders, reborrow/coercion, typed
trait-member calls and assignment/cleanup, with all original inputs and
operation-derived obligations. An identity-only wire path must not drop the
provided clone_from body or assume legacy Copy/Sized identities. The accepted
source-client protocol remains provisional until these actual body/header
forms, source role authentication and complete installation transaction are
encoded and reviewed. Actual staged API and library-feature inventories are
another required full-attribute boundary. Existing artifact profiles and bytes
remain unchanged.
