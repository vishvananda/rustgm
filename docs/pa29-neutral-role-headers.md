# Borrowed role-header validation checkpoint

This development slice starts at `e3db713f4d472bc086bf6aff8ef9d7c2b932d449`.
The original source findings and independent Rust/native receipts are preserved
in [the source-header interface review](pa29-source-header-interface.md), whose
opening describes the historical proposal rather than this implementation.

`sema::compiler_items::headers::Trait` borrows the original parameter, member
and predicate vectors. Its source constructor checks the actual resolver trait
declaration and retained associated owner/schema. Self must be the owner's
separate unsized type parameter at slot zero, with its actual binding and parent
map. Header validation does not establish a trait implementation, body proof,
Copy decision, const-call permission or executable original.

Sizing and ordinary role profiles consume this typed view. Copy, Clone and
LegacyReceiver prerequisite checks consume canonical clauses with their actual
qualifiers. Clone method validation uses the actual `function_header` facade;
the required bodyless method stays bodyless. Exact parent/member identity,
shared receiver, referent, reference binder, safe Rust ABI and result are checked
without syntax access in the shared signature validator. Additional members are
retained in the borrowed inventory rather than truncated to clone_fn.

The source adapter reads only existing item/list facts. It does not traverse a
type DAG, allocate a converted source predicate table, or scan other owners.
Source role validation constructs one view per role; the existing finite role
inventory and existing clause/member scans remain the work domain. The extra
Self checks use indexed lookups and a single binding. No persistent owner,
function-header, canonical predicate or IR value representation changes.

Five new private controls declare exact source vector sharing, complete member
inventory and absent required-method body; foreign Self owner/slot/parent/flags;
source versus transported typed-input qualifier parity and wrong member owner;
six malformed canonical method signatures; and cold/dense layouts. Existing
source-role, sizing, paid/unpaid and unsizing tests are the affected regression
set. Focused qualification below distinguishes retained passing results from the
exact final private recovery; no full release qualification is claimed.

This is the common validator prerequisite. An imported trait adapter is not yet
installed: actual importer declaration/parent/member remapping, complete
Self-subject classification, provided bodies and conditions must be validated
before any imported role is published. The typed-input representation control
tests structural parity only, not an import transaction or authority. Existing
trait-contract source inheritance classification remains unchanged.

Envelope 9 versions zero and one remain legacy-role contracts with their exact
old bytes. This slice assigns no wire tag/version and installs no SourceClient
mode. Real source-sysroot client authentication and replacement of bootstrap
identities are still required separately. Full original Clone/clone_from,
Destruct/Freeze, internal semantic attributes and all retained generic transport
families remain requirements; reduced header tests are not upstream-core
acceptance. The original identity function keeps its absent Copy bound.

## Focused qualification

Evidence is under `obj/parallel/pa29-source-headers/` in this worker checkout:

- `adapter-first`: format/audit pass; the independently declared new provider
  hypothesis rejects because the fixture omitted actual LegacyReceiver. Source,
  output and the failed hypothesis are retained.
- `adapter-second`: the fixture adds the real LegacyReceiver trait and reference
  impls. Pinned Rust 1.98.0 emits its object successfully, source SHA-256
  `f7c998ee2258ee18f14d59ef3096a8a81bc3ca74da08d5ce1e732484d967e3df`.
  The subsequent initial compiler import/style failures remain recorded.
- `adapter-third`: format, architecture/file audit, all-target/all-feature strict
  Clippy and registration pass. Its single batched build runs 20 affected source
  integration tests successfully and 12 of 13 selected private checks. The sole
  failure is a new test assertion incorrectly denying the retained implicit
  `Self: Copy` predicate; this is not a production failure.
- `adapter-recovery`: only that assertion changes, now preserving implicit Self
  and rejecting unrelated LegacyReceiver. Every production source SHA-256 is
  checked equal to the third snapshot before the recovery. The exact corrected
  private test and uncaptured layout test pass. Source integration, remaining
  private and strict/audit evidence is retained, not described as fresh.

Actual layouts are Owner 32, OwnerKind 20, DeclarationOrigin 12,
FunctionHeader 48, and stored TraitContract 136 bytes. Those definitions and
files are byte-identical to the e3 baseline; before/after containing layouts do
not change. The new transient Trait view is 120 bytes. Canonical Predicate stays
12 and IR Value stays 24. The view borrows complete source vectors and creates
no import catalog or persistent allocation, including repeated validation.

No worker full Cargo, active-text sweep, broad performance batch or repeated
native gate was run for this slice. The existing full-source independent/native
matrix remains required, with its earlier exact source/receipt hashes retained.
Central combined qualification and any reviewed source text changes remain
integrator-owned.
