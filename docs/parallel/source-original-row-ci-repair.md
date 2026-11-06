# Original-row accounting placement repair

Main20c557e01's full run42169539998 fails its fast job127733857914 in two
unchanged fresh artifact consumers: generic original runtime/constant instances,
and a CLI client implementing imported required trait methods. Both fail with
`caller inputs exceed source work/storage budget`. The failed smoke log is
retained byte-for-byte beside a separately ANSI-stripped readable copy.
[Evidence](source-original-row-ci-evidence.json) binds those logs and the repair.
Unit checkpoint82ff80381 was not part of that failed main head and is preserved
as this repair's parent.

The new Originals owner unconditionally charged caller-input accounting before
checking whether any cleanup Requirements rows existed. It likewise charged
body-only reverse/shrink operations. An ordinary imported client can lack the
completed local source-header payer selected by that adapter. Existing body Vec
ownership operations then failed despite introducing no row validation, row
allocation or source policy query. This is a payer-placement regression, not an
exhausted legitimate budget or a stale positive assertion.

The repair confines the added paired-body/Requirements validation, reassembly,
reverse and shrink accounting to the actual Some-row path. Its complete coverage,
extent checks, full conservative overlap payment and pre-growth reservation are
unchanged. None uses the preexisting body-only storage movement without creating
a row, requiring a new source payer or allocating an optional row container.
No accounting limit, caller-payments API or semantic proof is relaxed. Absence of
a row remains data, never a NoDrop/const/loan/publication permission.

The exact two failing smoke consumers now pass with their original sources and
assertions. Format, architecture and strict selected lib/CLI/linking checks pass.
The first repair batch's outer280-second timeout interrupts post-run cleanup after
all inner commands complete; the timeout remains a failure receipt. The final
qualification separately checks the same smoke consumers, preceding cleanup/unit
controls and the new private opponent's metadata, without a cold private link.
The final outer wrapper exits0: all50 focused regular tests, all-test/all-feature
strict metadata, format/audit, harness and upstream checks pass. Private execution
remains CI-pending. The retry script reused its previous output directory, so
that initial E0616 failure survives as an explicitly labelled tool-transcript
observation rather than an original raw log; both actual CI failures remain in
the untouched raw fast-job log.

The added private opponent exhausts the actual payer through its existing charge
API, then moves real ordinary bodies with no cleanup rows. Actual Some-row input
with the same failed-payment state must fail before body consumption. An initial
metadata attempt tried to edit the private payer field and correctly failed
visibility checking; the settled test adds no field visibility or test-only hook. Existing missing/extra/extent
opponents remain. Source fields, original body/row pairing, actual const/scoped
selection, physical cleanup/loans, active Code cleanup and conditional-parent
history fences retain their existing authority. This repair adds no record or
metadata profile and makes no new native or whole-library claim.
