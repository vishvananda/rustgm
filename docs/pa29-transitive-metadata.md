# PA29 transitive metadata inputs

This development checkpoint separates artifact availability from source name
visibility. `--extern ALIAS=FILE` supplies a direct dependency binding;
`--dependency-metadata FILE` supplies a required transitive artifact without
introducing an extern-prelude name. Both feed the existing typed dependency
graph, identity and visibility validation. The metadata wire format is unchanged.

The driver shares its canonical-path table across both input kinds, decoding
each path once under the existing combined 1,024-argument and 16 MiB budgets.
An artifact may be supplied through both kinds of input without duplicate
decoding. Source-core option validation now validates the provider mode without
loading metadata; the compilation context owns the single metadata load.

The seven predeclared cases in `pa29/tests/transitive-metadata-outcomes.json`
use two libraries with the same crate name and different identities. A client
sees its renamed direct dependency while the first library's name and the
second library's private dependency alias remain inaccessible. Inventory-only
inputs grant no name. An explicit direct alias restores access to the first
library. Missing or malformed artifacts fail through ordinary graph/decoder
checks. Repeated and reordered inputs retain byte-identical LowIR and metadata.

## Focused qualification

The exact original sources pass independent Rust 1.98.0 object/rejection
checks. Both positive native clients return zero with empty output under Rust
and the RustGM/CPPGM multi-unit path. Candidate controls additionally cover
missing inventory, truncation, the combined argument budget, unsupported output
mode and a missing option value.

All-target/all-feature strict Clippy, formatting, architecture/file audit,
harness/registration and upstream validation pass. All 8,382 active text cases
pass with the debug candidate in 43.95 seconds. This is a development check;
the complete release maintainer/native gate remains required before publication
and will be combined with the next coherent compiler checkpoint. The registered
Rust wrappers were typechecked; their exact Python qualification entry point
was run directly, avoiding another integration test build during development.

Candidate SHA-256:
`714bb3cddb5c0957faf29adc0c673f6a2b0bf857d3d3564dc5aa3dd4f46fad11`.
Receipts are under `obj/parallel/pa29-transitive-metadata/development-v1/`
and `development-v2/`. The first attempt retained a harness error: driver
usage failures return status 2, while metadata/semantic failures return 1.
Only those three harness expectations changed. Exact source hashes justify
retaining the passed Rust build, lint and audit checks; the complete corrected
candidate harness and text sweep then passed under a fresh guarded run.
The final completion receipt SHA-256 is
`59098efbcce942659c8efce683fc4f5983a817829cb125b01d2b257ac9b81c97`.

This boundary grants no generic-body export, source-sysroot role or target build
execution. Those remain separate requirements before compiling original
core/alloc/std and real dependency graphs. PA29 remains planned.
