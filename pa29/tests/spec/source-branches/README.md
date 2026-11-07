# Source branch transport packet

`source-branch-outcomes.json` freezes three providers, twelve clients, their
hashes, edition, target and independent outcomes before candidate execution.
Every provider reuses lines 1-58 of `spec/source-nominals/provider.rs`: the
authored sizing, receiver, Clone and Freeze declarations of PA31's
exact-source-linkage and Pin foundations. Their inherited attribution and the
repository NOTICE apply. Primitive Clone/Copy impls, Packet, Pair and the branch
functions are authored controls.

`provider.rs` exports `pick` (Copy-bounded two-way choice), `swap` (non-Copy
moves on both paths), `choose` (else-if chain) and `settle` (loop exit through
a conditional break). Seven clients instantiate them at i32, bool, u8 and
`Packet<i32>`. `moved.rs`, `nonbool.rs` and `copyless.rs` are use-after-move,
non-bool argument and missing Copy opponents (E0382, E0308, E0277). `wait.rs`
has one Branch, one infinite loop and one dead Unreachable block; it anchors
the wire controls. `spin.rs` has no two-way exit and anchors the byte-identity
and Panic-gate controls. `wait-failed.rs` diverges with genuine rows and is
never executed with them.

Pinned Rust 1.98.0 (`RUSTC_BOOTSTRAP=1`, `-Cpanic=abort`) compiles all three
providers and the nine positive clients; the C observer returns 7, 12, 22, 2,
13, 9, 31 and 5. The first independent attempt rejected an unbounded `pick`:
the no_core foundation declares no `drop_glue`. Rather than import the const
Destruct/drop-glue contract, `pick` became Copy-bounded and the non-Copy control
became `swap`; the packet was refrozen before the candidate's first source
outcome. Exploratory candidate probes of similar scratch sources preceded the
freeze; no frozen file was run by the candidate before freezing.

The candidate profile is `module-branches-v12`. The wire controls (relabelled
version, foreign successor, non-bool and unknown conditions, a lost successor,
every prefix, an edge skipping the merge store, foreign owners, and a forged
Panic exit) are mutations of these exports; their expectations are recorded in
the outcome file and the owning checkpoint.
