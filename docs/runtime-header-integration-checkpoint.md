# Source Abort, shared mapping and associated-family integration

Compiler `ac6966fe0a08b797b3b3d3c7c452a0361c2af1ac` integrates the qualified
[source Abort](source-abort-runtime-checkpoint.md),
[private signature mapping](private-signature-mapping-checkpoint.md) and
[Pointee declaration producer](pointee-source-declarations.md), together with
the associated-family alias correction below. No assignment or original
core/alloc/std artifact is completed by this checkpoint.

## Actual behavior

An associated family's input prefix now includes every clause emitted by a
written trait alias, including associated equalities and Copy requirements.
Previously the alias branch skipped the prefix update, allowing these real
input obligations to disappear. Associated-result aliases now enter the same
bound worklist and canonical expander as other alias bounds. Their clauses
remain promises about the projected result; they do not extend the input
prefix. The existing implicit result Sized requirement is retained.

Three controls inspect the actual prepared family and exercise a real function
signature containing `X::Item<T>`. A caller satisfying the full alias succeeds;
a caller missing Copy fails through projection selection. A result alias
retains its actual trait, associated equality, Copy and Sized clauses on the
result. There is no second alias graph or permission table. Complete alias
execution/export remains fenced while header, formation, region, object,
default/const and imported consumers are unfinished.

Source Abort consumes the actual reached-call and terminal-edge witness before
emitting the existing LowIR Unreachable operation. It skips subsequent cleanup
only on the terminating path. Normal paths retain ordinary destruction. The
shared private signature memo and source formal certificate preserve the actual
consumer Catalog, Store, owner and inherited inputs. They do not establish
defining environments, imported original/instance proof or public Provided
execution. The Pointee producer retains authentic Metadata/DynMetadata
declarations; normalization and automatic implementation remain separate work.
Its single transport guard is preserved at the shared collector entry.

## Qualification

[Hosted run 40347068121](https://github.com/vishvananda/rustgm/actions/runs/40347068121)
passes exactly 32 private controls in 0.215 seconds. The complete focused helper
takes 150.13 seconds, including production strict Clippy 22.38, private strict
Clippy 42.60 and unoptimized build 72.27 seconds. Formatting, assignment
metadata, global architecture, harness, upstream and audit-cache checks pass.
All 2,749 compiler/test/tool fingerprints match the input commit, actual checked
source before/after execution and the integrator's current source. No source
formatting adjustment remains outside that committed input.

Three new frozen Rust metadata predictions were observed once in run
40344203033: full input accepted, missing Copy rejected and result alias
accepted. All agree with pinned Rust 1.98.0; there are zero infrastructure
failures. The successful successor authenticates those source/diagnostic hashes
without repeating Rust executions. Original predictions and raw streams remain
retained under `pa31/tests/spec/trait-aliases/family-inputs/` and `obj/`.

The integrated Abort, Not and storage/Clone producers each emit source and
LowIR byte-identical to their separately native-qualified versions. Root
verification recomputes all six hashes from the actual archived files. This
retains their exact native observations without claiming a fresh combined
native execution. Abort's normal program prints
`interposed\ndropped\n7 9 11\n`; reached Abort prints only `before\n` and
terminates abnormally. A particular signal is not required. The backend and
pinned upstream sources are unchanged.

Actual integrated sizes are Catalog 776, Reservations 872, Validated 40,
Contracts 928 and State 224 bytes for the Pointee declaration owner. Package
256, Body 448 and Store 472 remain unchanged. The private mapping State is
240, Cursor 136 and Reader 24 bytes; warm1000 remains 38,000 charged steps
across all nine width/unrelated-body combinations. Formal collection charges
2,187/2,572/3,892 steps for 1/8/32 formals, including the integrated collector
guard. These are logical work/storage observations, not a wall-time speedup.

The source-proof predecessor observer records trait-selection Source 16
(alignment 4), Option Source 16, Evidence 136, Frame 1312, Node 160,
Normalization 520, ProjectionQuery 912 and SearchQuery 656 bytes. These are
the measured baseline for the next metadata consumer, not predicted after sizes.

Receipt SHA-256 is
`5114a998dff9a769762f7414c7ab043728f913a63220a818d085273819c3a94b`;
binding SHA-256 is
`49c73359336438e0e345c92152d135a9314e435e85ea4ec636016cf221dd5d48`.
Root verification and small archives are retained under
`obj/parallel/abort-family-integration-v1/qualified-40347068121/`.
No local compiler build, executable download, release build or broad matrix ran.

The failed static-only run 40342898497 and 29/32 run 40344203033 remain failed
and retained. The latter exposed the missing associated-result alias consumer
and two incorrect test assumptions: implicit result Sized and the existing
missing-Copy diagnostic. The accepted/rejected source predictions did not change.

The next work remains complete aliases, source Pointee metadata normalization,
transmute/panic/formatting prerequisites and actual imported provided bodies.
The last unchanged-core observer still stops at Thin in ptr/metadata.rs with
65 files opened and all 722 pinned inputs verified. There is no core artifact;
opened files are not compiled units.
