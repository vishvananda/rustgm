# RustGM Programming Assignment 14: Moves and initialization

Status: **implemented-draft**. Prerequisite: PA13. The profile is Rust 1.98.0,
edition 2024, Linux x86-64. This assignment publishes the ownership proof already
required beneath the earlier executable subset. Later lifetime, loan and
destruction proofs remain separate.

## Required implementation

Build sparse move paths from the typed body CFG. Classify each place read as
Copy or Move using canonical type obligations, then prove definite initialization
and assignment permissions across branches and loops. Never reconstruct places
from syntax spelling or from an inspection view. Lowering consumes the checked
body and retained facts; it cannot repeat or bypass this analysis.

A non-Copy read moves its selected value. Other disjoint fields remain usable;
reading the whole product requires all tracked fields to be initialized. A
mutable field can be reinitialized only while its parent structure exists.
Writing one field cannot manufacture an uninitialized or wholly moved parent.
Whole-value replacement reinitializes its tracked subtree. Physical layout and
zero size do not make a nominal type Copy.

An immutable local permits its first initialization once on each path. At a
join, definite initialization intersects predecessors and possible writes unite
them. A write after a possible earlier initialization is invalid even if another
path left the variable uninitialized. Storage entry/exit reset the local's state;
a local freshly declared inside a loop differs from a reused outer local.
Only reached blocks contribute, but PA13's imaginary ownership successors are
included even when executable lowering omits them.

Named value bindings move or copy. `_` does not move its place merely because
it appears in a let/match; any necessary projection evaluation still applies.
Ordinary indexing keeps an array indivisible for move tracking: a non-Copy value
cannot be moved through `a[i]`, including a literal index. Copy fields can be
read through indexing, and an initialized mutable array can be assigned through
it. PA13's selected constant indices and rest patterns can describe statically
disjoint moves. Overlapping selections share initialization bits. An empty
subarray reads/moves no elements, while eligibility to access its storage and
internal guard protection remain separate checks.

Struct updates read only fields not explicitly supplied. Temporary aggregates
also have move paths. Enum tag tests require the initialized input specified by
PA13; a surviving tag does not validate a partially moved payload. Guard views
are read-only, delay owning captures until success and cannot supply a non-Copy
move. Their existing protected-place/liveness proof runs with initialization.

## Copy and Clone obligations

Cache closed-profile properties by canonical type. Scalars, function values,
raw pointer values and shared references have their builtin Copy/Clone rules;
mutable references do not. Checking a pointer/reference property does not walk
its referent or infer properties from layout. Tuples and arrays require eligible
elements, including arrays of length zero. Nominals require the appropriate
builtin derive hooks and all field obligations; `Copy` requires `Clone`.
Duplicate derives and invalid field obligations are diagnosed. Packed derived
`Clone` additionally requires Copy fields, including zero-sized fields.

These hooks establish properties; they do not implement general trait solving,
user impls, `.clone()` calls or procedural derive execution. Ordinary array
repeat expressions evaluate their operand once. Length zero still consumes a
non-Copy operand; length one needs no Copy bound; greater lengths require Copy
unless the operand is a named constant evaluated independently for each element.

Opaque incoming reference values can be classified for moves/copies when their
existing type facts suffice. This is not a reborrow, loan, outlives or alias proof.
Explicit reference operations and pending region constraints retain their
PA15/PA16 boundaries. Unsafe static writes, user destructors and generalized
trait behavior are not silently admitted by an ownership view.

## Text contract

`dev/rustgm --emit-ownership SOURCE` emits a deterministic inspection only after
all ownership checks succeed. `--count-ownership` runs the same checks and emits
its count line. The optional `-o OUTPUT` follows the earlier atomic-output
contract. Diagnosed failure returns 1 with no partial stdout. Infrastructure
failures never become language rejections.

The view begins with `ownership-crate 1`. Bodies retain program order and begin
`body N source=FILE:NODE`. Each body contains:

- `  path mN type=TYPE root=lN|vN`, optionally followed by ` parent=mN` and one
  of ` tuple-field=N`, ` nominal-field=N`, ` array-index=N` or
  ` subarray=START..END`; every path ends ` subtree=START..END`.
- `  use vN copy|move` for each selected read, in value order. Other values have
  no use record.
- `  block bN reachable=true|false` in block order, including imaginary proof
  reachability.

`lN` and `vN` refer to the typed body's locals and temporary values. Type/field
identities are the canonical semantic IDs established by the earlier stages.
Move-path IDs follow deterministic construction order. A subtree is a half-open
bit interval, not a byte offset or a range of path IDs. It includes the shallow
structure and selected descendants; unmentioned fields remain represented by
the parent's shallow state. Array overlap normalization may share subranges.

Each body ends `end-body`. The final line is
`eof bodies=N paths=N copies=N moves=N`; count mode emits the same line without
`eof `. Counts include the same checked bodies and selected uses, even when
later executable code omits an imaginary path. Do not reparse this view as IR.

## Efficiency and limits

Use flat worklists and compact intervals. Retain only demanded field/index/rest
paths; do not enumerate an array from its declared length. Reuse cached Copy
queries and retain negative results without poisoning independent queries.
Initialization uses must-initialized/may-written bitsets and queues only changed
successors. Charge actual visited words, statements and joins. The current
profile permits at most 4,000,000 state words (64 MB of must/may state) and
32,000,000 charged work units per body; exceeding a bound diagnoses failure.
Selected array partitions also cap retained boundaries at 1,000,000.
Internal guard liveness has its own bounded proof. Deep traversal and arena
cleanup must not recurse on source depth.

The inspection API retains per-body facts for rendering. Executable verification
runs the identical proof while releasing each body's inspection records before
lowering. Follow `spec.md`, safe Rust, Clippy all/pedantic and the file audit.

## Tests and completion

The manifest declares 124 cases: 112 ownership views (58 acceptances and 54
rejections) and 12 executable LowIR controls. All have independent pinned-rustc
checks with no acceptance exceptions. The twelve executable cases assert full
values through the qualified CPPGM backend and rustc, including branch/loop
reinitialization, storage, partial moves, struct updates and guard capture.

Run `make test-pa14`, `make test-rustc`, `make test-native-pa14` and the root
lint/audit/harness checks. Only explicit `make ref-test-pa14` regenerates the
committed sidecars. The [validation record](../docs/pa14-validation.md) records
independent outcomes, sparse-state invariants and scaling measurements.
