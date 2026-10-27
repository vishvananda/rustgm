# Required operator component joins

Operator-kernel remains unpublished and now has explicit partial implementation
status. Reconciliation found two direct gaps; captured indexing is now qualified,
while implicit assignment demand remains. Neither
Rust-positive input is excluded or relabelled as a language rejection. Whole
PA26 remains planned; broader constants, reference operations and derives keep
their already declared hook-closure/format-derive requirements.

`pa26/tests/spec/index-mut/captured.rs` mutates a canonical IndexMut receiver
inside an ordinary mutable closure and requires two calls to yield3. Pinned Rust
accepts the unchanged source. The earlier post-warning compiler rejects
it at328 with `adjusted or overloaded capture indexing is not implemented`.
The old maintainer test expected candidate rejection despite Rust acceptance.
The [qualified selected-receiver join](pa26-selected-index-captures.md) closes
that boundary:27 independent controls,21 native pairs and the actual integrator
CLI/backend/native run pass. The candidate predicate is now positive and its
Rust-only exception is removed. Loan, mode and evaluation-order checks remain.

`pa26/tests/spec/primitive-assignment/implicit-forwarding-boundary.rs` declares
both operands as u32, then uses `x += &y` without importing AddAssign. Its source
SHA-256 is `28fb4c45a426d3eaed57c14d6b7b4ef56bf024ce1265df53acc09e2e3193d2f2`.
Current pinned Rust metadata succeeds; qualified combined CLIaa6dbb86 rejects
`incompatible types` at100. The explicit-import companion already works. This is
canonical hook demand, separate from unsuffixed inference or unrestricted method
lookup. The integrator owns the existing resolver/declaration producer join,
with actual trait/receiver/loan proof and ordinary no-demand controls preserved.
Seventeen frozen controls now independently match pinned Rust outcomes; three
valid native controls prove reference-forwarding results and overload order12.
The current candidate still fails nine required positives, while scalar,
no-demand, disabled and explicit-import controls pass. Baseline layout measures
Traits192/Language44/Resolution2056/Body456. Inputs and predictions are committed
in `pa26/tests/implicit-assignment-demand-predictions.json`; exact39-command
receipt and baseline streams remain under
`obj/parallel/implicit-assignment-demand-v1/before/`.

The current independent and candidate streams are retained in
`obj/parallel/raw-composition-integration-v1/observed/`; captured evidence is in
`obj/parallel/lib047-qualified-v1/`. Neither observation executes invalid input
code. Their source bytes, tools and candidate hashes are bound. The read-only
701-case reconciliation packet under public
`obj/parallel/assignment-integration-review-v1/operator-kernel-completion-v1/`
also retains six broader required mismatches and212 residual rows. It is a
proposed inventory, not an active manifest or generated reference contract.
