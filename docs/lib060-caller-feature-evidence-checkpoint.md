# LIB-060 sealed caller feature evidence

The native demand queue now retains the actual sealed caller index separately
from its diagnostic source. Materialization borrows that caller's authenticated
environment and threads its shared Arc through the ephemeral instantiate context
to `callee_premises::specialize`. A callee's own marker supplies no caller proof.
Root/export requests retain explicit missing caller evidence.

Checked templates validate the canonical function, closure or anonymous owner and
exact source before returning their existing environment. Ordinary bodies use
their actual completed function/initializer and source identity. An ordinary
declaration with no completed environment is known empty only when direct
parameter, predicate and semantic-parent rows are all absent. An existing empty
row still requires its completed environment. Missing/imported/adapter authority
remains an explicit requirement; this change demands no additional bodies or
source scans.

The caller token is four bytes. The exact original cold Work sum measures 36
bytes and the current sum measures 40. Every queue push, including initial roots,
globals, adapters and body dependencies, prepays `Growth::vector::<Work>` before
publication through the existing 64 MB instance budget. Attempted work remains
charged on failure. Warm pushes within retained capacity allocate nothing. Store
472, Environment 64 and NodeFacts 80 bytes remain unchanged; no persistent body,
node, environment or source-parse field is added.

Private fastcheck `1793219078636480706` passes all four selected controls in
130.18s, including strict private Clippy, formatting, architecture checks and the
existing cleanup-reuse invariant. Genuine ordinary caller evidence succeeds;
missing, unrelated, root-only, incomplete, wrong-root and missing-index evidence
reject. Checked-template owner/source opponents reject. Cold capacity coverage,
warm reuse and a rejected growth attempt before allocation pass.

Receipt `1793218414559938347` is retained as a failed prediction: three authority
controls passed, while the claimed equal old/new Work widths failed. The original
prediction and corrected measured-width prediction remain distinct. Earlier
static failures are retained too. This snapshot qualifies the focused private
boundary; production/CLI and actual source/native matrix integration remain
pending on the joined compiler. It does not complete LIB-060, transport, original
core compilation or a release. Source-linked receipts and the handoff are in
`obj/parallel/sealed-caller-feature-v1/`.
