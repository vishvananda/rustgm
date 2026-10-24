# Enum-alias member lookup (LIB-002)

Enum variants reached through a type alias now use the retained context of the
actual final path identifier. Both alias completion and qualified nominal paths
pass that existing suffix node to the shared semantic consumer. It adjusts the
context against the actual enum's introduction, probes the existing contextual
binding table and applies the resulting definition-site privacy. The captured
alias name and the path's first segment cannot replace the variant's identity.

No stored path, body, type, definition or member layout grows. The change reuses
typed syntax and the existing context adjustment/index; it does not reparse a
path, scan every enum or introduce a new cache. Ordinary methods return from the
enum-kind check before requesting this adjustment. Origin traversal and context
adjustment retain their existing costs; this is not a universal constant-time
claim for arbitrary forwarded provenance.

The four predeclared [source hypotheses](../pa31/tests/spec/declarative-macros/enum-alias/README.md)
match pinned rustc in all eight metadata/object observations and two native
programs. The positive cases print `7`; the physical-caller and mismatched
suffix cases reject. The original contract is unchanged; `review.json` binds
the observations, sources and rustc executable. Evidence remains in
`obj/parallel/enum-alias-oracle-v1` in `rustgm-pa29-work`, results SHA-256
`ecfd472a86eb9fd851d528115b22d0638ee6cfa0614b4c9ec6691237b4ba84a8`.

Two private controls use real expansion/enum owners with explicitly supplied
opaque provenance. They distinguish a captured prefix from its introduced
variant, test root/opaque warm queries, and oppose definition-site privacy with
the physical caller. They exercise the actual semantic selection function.
They do not enable scoped macro source admission; complete fields, methods,
associated items, publication and foreign provenance remain required.

Development qualification passed formatting, file/architecture audits, strict
all-target/all-feature Clippy, harness/upstream checks, all 1,396 nonignored
private tests, all 295 nonignored frontend tests and 8,382 active text cases.
The one private and 31 frontend ignored controls were inventoried, not executed.
Combined release/native and changed-path performance remain integrator gates.

`enum-alias-candidate-v2` retains the single debug build and private/frontend
execution; `enum-alias-text-v1` completes the never-executed text phase against
that exact binary. It authenticates/reuses the successful preceding stages,
without another compiler build or private pass. The text receipt SHA-256 is
`e8776fe761ac370d31fa84d9b0b18ca0eb0ebd2440810d9e9c42cc173ac8eadc`;
the tested compiler/source manifest is
`47ed25a3498abf4c7986182b4871be1b9141660e171e47316f5d11305a0c6c1e`.
Later documentation and independent-review records do not alter compiler inputs.

Two infrastructure failures remain recorded: v1 lacked its local audit virtual
environment, and v2's Makefile text target tried to acquire a nested storage
lock for its unconditional release-build prerequisite. The first recovery
prepared the pinned audit dependencies; the second called `course.py` directly
with the existing debug CLI and explicit scoped evidence root. Neither failure
is a language rejection, and no expected output or compiler source changed.
