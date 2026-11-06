# Actual report and legacy cleanup-row accounting

This is the LIB177 repair on main d6218dfd1, following the narrower None-row
repair. Full42176016850 exposed genuine ordinary Some-row consumers that lack
local source header accounting. Both original CI failures and the earlier
None-row failure remain historical; Some means a pending cleanup row, not a
SourceCore declaration or a header-accounting owner.

The actual Report now reaches Builder::new/Originals. A cold boxed Rows owns the
same moved Requirements Vec and borrows that Report through its valid interval.
Before selecting a payer, the constructor rejects an absent root, requires the
actual lint context and authenticates any existing local/imported Catalog against
the Report. SourceCore/source-client modes cannot lose their required Catalog.
SourceCore always uses the existing strict completed source-header payer. Its
missing header or failed payment is a hard error with no ordinary/legacy retry.

Ordinary/client structural row work uses the actual Report producer's existing
bounded diagnostic-payload accounting when present. Attribute-free ordinary
reports instead own the existing OriginalIndexBudget (1M work/64MiB), scoped to
this real row construction. Its counters move with the Rows owner, remain sticky
on errors and are not reset by pops/shrinks. This is cost accounting, not a trait,
const, loan, feature or publication permission. No source Header or Registry is
created to make bookkeeping succeed; no existing limit is raised.

Some coverage/extents, allocation prepayment and conservative old-live/new-vector
overlap charges remain. Box72 bytes are paid before allocation. None still moves
only existing bodies and introduces no payer, row allocation or traversal.
Measured Originals32 bytes (formerly48), Rows72 and Payer48 match the actual
retained layout log. Body/Plan/Proved/wire/backend records are untouched.

The six targeted semantic tests pass: temporary-edition destruction/borrow
opponents; selected generic destruction; two2048-scope scaling inputs; const
function destruction; generic artifact runtime/constant consumers; and fresh CLI
implementations of imported required methods. Source bytes/outcomes were frozen
before observation. The final clean-wrapper batch passes these plus all44 prior
cleanup tests, primitive-payment3 and cursor-fence1:54 regular test functions.
Format/architecture, strict lib/CLI/frontend and all-test/all-feature metadata,
harness and upstream checks pass. Private hostile execution is CI-pending; its
metadata covers missing/foreign/stale/empty reports, missing Source Catalog/header,
exhausted Source/report/legacy owners, no-row movement, and malformed row extents.

The first runtime batch's280s outer timeout interrupts cleanup after all six
inner semantic steps and its source map finish; that failure is retained as124.
The cached final batch has a clean outer0. No native or full-library result is
claimed by this repair; prior native Source observations are not repeated here.

[Evidence](source-row-payer-evidence.json) binds exact compiler/fixture/log/receipt
bytes. Root owns the Makefile smoke addition for the frozen temporary-editions
case. CTFE preparation remains in its separate durable note; no destructor
scheduler was added. Active Code cleanup, conditional-parent history, precise
policy provenance and imported type/body transport fences remain required work.
