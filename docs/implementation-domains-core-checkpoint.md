# Trait implementation domains and the next original-core boundary

Local `impl_restriction` support consumes parsed clause nodes, actual feature
and macro-token authority, resolved module identities and the real implementation
owner's ancestry. Visibility remains separate from permission to implement a
trait. Paths share the ordinary import fixpoint, including forward module aliases.
Parsed cfg-disabled headers still require the syntax feature. Ordinary source
retains its prelude and gains no source-provider role.

Seventy-four independent pinned-rustc observations pass: 38 acceptances and
36 rejections. The candidate passes the feature/origin/layout control, genuine
Functions artifact/import/restricted-export control and ordinary hot-layout
control. The domain control initially contained an invalid relocated fixture:
prepending a module put its crate-inner feature attribute after an item. An
immutable candidate proves that error and the corrected authored placement.

[CI 40214755330](https://github.com/vishvananda/rustgm/actions/runs/40214755330)
then passes the corrected domain control, formatting, architecture, strict
private Clippy and build in 174.05 seconds. Three unaffected passes are retained
from the preceding exact-source gate; this is not a fresh four-control run.
The original failed assertion, parser expectation and build-timeout receipts
remain preserved. No parser waiver or semantic rule was changed for that fixture.

Measured hot sizes stay unchanged: ItemKind 28, Item 64, Node 36, Source 184,
SourceCore 176, Unit 672, Package 256 and Report 3040 bytes. The ordinary syntax
gate examines exactly the measured item inventory with no feature-origin query
when there is no clause; an authenticated global request takes its empty scan
path. Actual ancestry requires three steps in the selected nested case; a failed
budget attempt is charged once. Imported unrestricted permission requires the
actual readable profile, canonical trait header, owner and inventory, rather
than a missing clause or name lookup. Publishing a restricted trait still fails:
transporting genuine restrictions is a remaining PA29 requirement.

[Original-core CI 40215218089](https://github.com/vishvananda/rustgm/actions/runs/40215218089)
uses immutable compiler
`fdf200a9d90ccec3bfdd106ce3671947136daa43358dd06bc4f472446e7649fa`.
It verifies all 722 original source inputs and opens 56 Rust files, advancing
from 42. The 6.423-second observation exits one without an infrastructure failure
at `library/core/src/mem/alignment.rs:26`, byte 981:
`duplicate value name '_'`. The two consecutive unnamed constants assert size
and alignment; both must retain their initializer typing and compile-time checks
without publishing a colliding lexical value name. This is LIB-027.

The file is 17331 bytes with SHA-256
`4eb6651741fc8d8d84ce274219cc72399949aa9eba2a4592a3f90d9293d1b587`.
The authenticated open trace records it at index 55. Source bytes and the compiler
remain unchanged; no core artifact or assignment is complete. Observer metadata
is retained under `obj/parallel/impl-domain-core-observer-v1`; the full original
trace, producer binding and opened-file inventory remain in the hosted artifact.
