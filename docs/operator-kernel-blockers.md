# Required operator component joins

Both direct operator-kernel gaps are now source/native qualified. The finite
kernel is implemented; publication, complete inventory and release checks remain
pending. Whole PA26 stays planned, and broader source/reference/derive obligations
remain in their declared units.

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
The original baseline CLIaa6dbb86 rejected it at100 while pinned Rust accepted.
The [qualified typed-demand repair](implicit-assignment-demand.md) now accepts
and executes that unchanged positive. Its old candidate rejection and Rust-only
exception are removed. Exact owner, signature, source authority, scalar latency,
lexical dot visibility, loans and source order remain checked. Baseline layout
Traits192/Resolution2056 shrinks to168/2032; Language44/Body456 remain unchanged.
All17 original/control predictions and visibility3 independently match pinned
Rust, and the genuine native programs pass. Component publication is still pending.

The current independent and candidate streams are retained in
`obj/parallel/raw-composition-integration-v1/observed/`; captured evidence is in
`obj/parallel/lib047-qualified-v1/`. Neither observation executes invalid input
code. Their source bytes, tools and candidate hashes are bound. The read-only
701-case reconciliation packet under public
`obj/parallel/assignment-integration-review-v1/operator-kernel-completion-v1/`
also retains six broader required mismatches and212 residual rows. It is a
proposed inventory, not an active manifest or generated reference contract.
