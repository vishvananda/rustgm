# Closure proof allocation

One body entry-selection pass now shares a lazy private-region cursor across
direct captured calls and object conversions. Previously every nonempty retained
normalization proof scanned the body's complete accumulated obligations and
proofs to find its next free identity. Repeated projected calls could therefore
repeat a growing scan. The cursor scans once, then advances by each newly owned
proof interval. Projection-free bodies never initialize it.

Every normalization debt, caller equality, source condition and nested quantified
scope remains retained. Rebase still precedes substituting actual caller slots;
private witnesses cannot collide with them. The cursor is local to the selection
pass, which owns all newly appended proof intervals and appends no independent
body variables. Standalone proof consumers create a fresh cursor. Allocation
scan visits now contribute to the existing canonical-work counter.

The private invariant begins with existing body variables and a previously owned
proof interval, then appends 8/32/128 proofs containing a private witness, a
caller-slot equality and a separate quantified universal interval. It verifies
disjoint ownership, unchanged caller identity and retained scopes. Recorded
allocation/canonical work is 9/33/129 and retained proof count is 9/33/129.
These are structural work counters, not a complete compiler benchmark.

Focused checks pass strict all-target/all-feature Clippy, formatting, the file
and architecture audit, twelve entry invariants, 29 affected source groups and
all active PA1–PA23 text suites without reference changes. Evidence is under
`obj/parallel/pa24/proof-cursor/`. The first private-test filter selected no tests;
`second/cursor.log` records the corrected explicit test and one passing execution.

The first proposed public scaling input is valid Rust but reaches the same
preexisting associated-projection rejection in both the frozen `be2e9cbc`
compiler and this candidate. Its declarations and failed observations remain in
`inputs/` and `second/`. It is not counted as a passing compiler scaling case.
Public changed-path measurements remain pending the projection repairs and
combined integration; no wall-time speedup is claimed for this checkpoint.
