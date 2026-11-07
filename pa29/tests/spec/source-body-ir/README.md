# Source body IR transport packet

`source-body-ir-outcomes.json` freezes two providers, fourteen clients, their
hashes, edition, target, independent outcomes and the wire controls before
candidate execution. Both providers reuse lines 1-58 of
`spec/source-nominals/provider.rs`: the authored sizing, receiver, Clone and
Freeze declarations of PA31's exact-source-linkage and Pin foundations. Their
inherited attribution and the repository NOTICE apply. Primitive Clone/Copy
impls, Packet and the match functions are authored controls.

`provider.rs` exports Copy-bounded generic decisions over primitive scrutinees:
`grade` (inclusive integer range), `band` (two ranges), `exact` (u8 literal),
`either` (or-pattern plus literal), `letter` (char range) and `flag` (bool
literals). Each decision lowers to closed primitive Binary comparisons and
two-way exits. Twelve clients instantiate them at i32 and bool, including both
inclusive edges; `mismatch.rs` and `copyless.rs` are wrong-scrutinee and missing
Copy opponents (E0308, E0277). `wire.rs` holds only `exact`: one Binary node and
one two-way exit anchor the wire controls, and `exact-hit.rs` consumes it.

Pinned Rust 1.98.0 (`RUSTC_BOOTSTRAP=1`, `-Cpanic=abort`) compiles both
providers and every positive client; the C observer returns 3, 41, 44, 32, 36,
21, 23, 24, 38, 27, 30 and 14, and 21 for the wire provider. Exploratory
candidate probes of similar scratch sources chose the pattern forms; a negative
range pattern was dropped because rustc requires the `neg` language item for
it. No frozen file was run by the candidate before freezing.

The candidate profile is `module-bodies-v13`. The wire controls (allocated but
unreconstructable codes, operators and operand kinds, unknown forms, malformed
schemas, an arity overflow, a forward operand, a bitwise result type change, a
FalseEdge relabel, a version relabel, every prefix and foreign owners) are
mutations of these exports; their expectations are recorded in the outcome
file and the owning checkpoint.
