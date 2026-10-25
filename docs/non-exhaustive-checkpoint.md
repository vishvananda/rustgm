# Local non-exhaustive contracts

The [merged focused gate](https://github.com/vishvananda/rustgm/actions/runs/40185018653)
qualifies source `92e0d894`, integrated at `9f40b453`. All four selected
integration controls pass in 0.03 seconds. Assignment metadata, formatting,
architecture audit, strict integration/CLI Clippy and the unoptimized build
complete in 129.31 seconds; all 212 harness checks also pass. This repairs the
exact closed-record test that failed in periodic full CI. A new full periodic
run is separate from this focused evidence.

The real local consumer validates word form on structs, enums and enum variants.
Construction, destructuring, exhaustiveness and uninhabited empty enums retain
ordinary same-crate behavior. Duplicate attributes and misplaced struct-field
attributes use the existing lexical `unused_attributes` policy. In particular,
field placement warns by default, rejects under deny, and accepts a field-level
allow beneath a crate-level deny. Other invalid owners/forms and genuinely
unknown attributes retain explicit rejection.

The final contract in `pa29/tests/spec/non-exhaustive-local/contract.json`
binds nineteen exact source files to 38 real Rust 1.98.0 observations across
editions 2021 and 2024. The initial prediction that field placement was a hard
error was disproved in both editions. Those original predictions and raw
diagnostics are retained, together with the two later deny and two allow
observations; the original 34 were not repeated. The separate hosted 34-case
observation independently confirms the same two prediction disagreements.

Collection dispatches from the existing configured owner/attribute traversal;
it adds no extra scan of the crate. Retained facts authenticate the actual source
image, owner, declaration, effect, span and target kind through existing phase
interfaces. One optional pointer lives in the existing cold attribute owner;
the marker map is allocated only when needed. No field is added to Declarations,
Node, Body or Definition. Source/map/query work and storage are bounded.

Closed record export reads these typed facts instead of rescanning raw attribute
spellings. Cross-crate non-exhaustive metadata remains unsupported and the
original precise export-profile rejection is preserved. No external construction
or exhaustive-match permission is inferred from absent imported metadata.
PA29 remains planned; no references or backend behavior changed.

Retained evidence is in the integrator worktree under
`obj/parallel/non-exhaustive-passed-40185018653/` and the C worktree's
`obj/parallel/pa31/non-exhaustive-*-independent-v1/` directories. The first
integration attempt's phase-dependency audit failure is also preserved; the
consumer now reads through `Unit::retained_attributes`.
