# Required operator component joins

Operator-kernel remains unpublished and now has explicit partial implementation
status. Reconciliation of its existing fixtures found two direct gaps. Neither
Rust-positive input is excluded or relabelled as a language rejection. Whole
PA26 remains planned; broader constants, reference operations and derives keep
their already declared hook-closure/format-derive requirements.

`pa26/tests/spec/index-mut/captured.rs` mutates a canonical IndexMut receiver
inside an ordinary mutable closure and requires two calls to yield3. Pinned Rust
accepts the unchanged source. Root's qualified post-warning compiler rejects
it at328 with `adjusted or overloaded capture indexing is not implemented`.
The existing maintainer test explicitly qualifies Rust acceptance while
expecting candidate rejection. That passing test proves the old unsupported
boundary. C owns consuming the selected receiver through existing capture/place
facts, including source/native loan and evaluation-order qualification before
changing the candidate predicate.

`pa26/tests/spec/primitive-assignment/implicit-forwarding-boundary.rs` declares
both operands as u32, then uses `x += &y` without importing AddAssign. Its source
SHA-256 is `28fb4c45a426d3eaed57c14d6b7b4ef56bf024ce1265df53acc09e2e3193d2f2`.
Current pinned Rust metadata succeeds; qualified combined CLIaa6dbb86 rejects
`incompatible types` at100. The explicit-import companion already works. This is
canonical hook demand, separate from unsuffixed inference or unrestricted method
lookup. The integrator owns the existing resolver/declaration producer join,
with actual trait/receiver/loan proof and ordinary no-demand controls preserved.

The current independent and candidate streams are retained in
`obj/parallel/raw-composition-integration-v1/observed/`; captured evidence is in
`obj/parallel/lib047-qualified-v1/`. Neither observation executes invalid input
code. Their source bytes, tools and candidate hashes are bound. The read-only
701-case reconciliation packet under public
`obj/parallel/assignment-integration-review-v1/operator-kernel-completion-v1/`
also retains six broader required mismatches and212 residual rows. It is a
proposed inventory, not an active manifest or generated reference contract.
