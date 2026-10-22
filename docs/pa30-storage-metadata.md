# Named storage across crate boundaries

This maintainer checkpoint exports real static identities, linkage, permissions
and completed initializer values. Imported definitions have no synthetic syntax
or initializer bodies. This is ongoing PA29/PA30 work, not assignment activation.

## Wire and semantic contract

Schema 1 profile 5 adds named storage to the existing shared type/record graph.
Profiles 0–4 retain their existing bytes. Static identities occupy the value
namespace and preserve crate identity, canonical paths and exact symbol policy.
The wire table distinguishes owned initializers from foreign declarations;
foreign safety and mutability remain separate. Reachable private backing is
transported without making its name public.

Initializer nodes contain typed scalar bits, postorder record fields or shared
static references with a real target identity and canonical field-ordinal path.
The importer validates types, record arity, backward value edges, static targets,
field paths, reachability, namespace collisions and aggregate quotas before
allocating local definitions. It registers every imported storage identity before
constructing completed values, permitting addresses to refer to later entries.

Owned imported immutable contents are available to ordinary constant evaluation.
Foreign contents and mutable statics remain unavailable there. Runtime accesses
retain the original global symbol and safety/mutability checks; imported storage
is not replaced by an unrelated local initializer or copied global. Raw access
uses the central static-read permission check on each demand.

Discovery uses visited definition/value/location tables. Encoding memoizes actual
typed values and emits postorder nodes through a bounded worklist. Validation
uses shared type facts and iterative reachability; import remaps the graph once.

## Required evidence

`pa30/tests/storage-metadata-outcomes.json` declares 16 unchanged sources and a
C provider. Two libraries are compiled independently; eight clients exercise
scalar values, transitive mutation, address identity, record fields, constant
references, foreign storage and raw mutable addresses. Six negatives cover
private names, immutable assignment, unsafe accesses, mutable constant reads and
foreign constant reads. Each rejection checks its actual owning diagnostic.

`pa29/tests/storage-metadata-outcomes.json` adds eight independently qualified
sources for hidden field backing, alias identity, private-name rejection,
self-referential static cycles and 8/32/128 shared roots. Those cases have no
foreign interface dependency and belong to PA29. The two native clients retain
all original assertions and add a local source macro plus the existing runtime
check helper: the first probe assumed an imported standard-library `assert!`
macro, which this frontend does not yet provide. Both versions and that failed
probe remain in `obj/parallel/pa30/storage-metadata/extra-declared-v{1,2}/`;
the self-contained version was independently object/native-qualified before
candidate execution. No library declarations or assertion expressions changed.

`scripts/check_storage_metadata.py` verifies pinned Rust outcomes, repeatable
library metadata and LowIR, reversed import ordering and literal native execution
through the configured CPPGM object backend and C linker. It is registered in
the ordinary and ignored-native crate-linking suites. Run it through the resource
and storage guard, like the other maintainers' commands.

Codec tests independently exercise truncated/trailing artifacts, hidden field
backing, invalid targets/paths/types, forward value edges, unreachable storage,
scalar-only graphs, exact foreign names, open lifetime rejection and collisions
in the value namespace. No text oracle is generated for this planned assignment.

The final focused source/native receipt is `checks-fifth/` beneath
`obj/parallel/pa30/storage-metadata/`: all 24 declared sources and ten native
clients pass through the permanent checker, including independent Rust objects
and native executables. The PA29 shared-root test also checks one private backing,
unchanged type-graph size and linear artifact/value-node bounds at 8/32/128 roots.
These are graph/storage assertions, not wall-time or process-memory claims.
`checks-fourth/` records strict release all-target/all-feature Clippy, formatting,
file audit (1,687 files, zero findings), harness and layout success. Its incorrectly
named upstream script and exact-test selector failed before executing their
checks; `checks-fifth/` uses the actual upstream verifier and all four exact test
names. The earlier codec qualification remains valid. Failed receipts are retained.
Combined active-text, release-suite and performance qualification are still
required before publishing this compiler checkpoint.

## Still required

This checkpoint does not close arbitrary aggregate/array initializers, anonymous
allocation and byte-image graphs, general raw-pointer relocation metadata,
references to storage owned by a dependency, public reexports, generic bodies,
variadic declarations or the remaining aggregate ABI metadata. Reinterpreted
byte offsets are not treated as canonical field ordinals. Existing unsupported
boundaries remain explicit while these families are implemented.
