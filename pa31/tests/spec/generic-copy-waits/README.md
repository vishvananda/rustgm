# Generic Copy waits

Eight no_core source-core libraries for constants evaluated while declarations
are still being checked, whose const fn copies a value of a generic nominal:

- `impl<T: Copy> Copy for Choice<T>` with its const `Clone`, copied inside a
  called const fn and inside the anonymous length block itself;
- core's `NonZero` shape, a wrapper whose field projects through
  `T: Zeroable`, copied inside `Choice<Wide<u32>>`;
- a wrong-length opponent (E0308), a non-`Copy` element whose second use reads
  a moved value (E0382), a `Copy` impl with a non-`Copy` field (E0204) and a
  `Copy` impl whose where-clause names the waiting constant (E0391);
- the `NonZero` shape declared after the waiting const fn, which destructures
  the projected field: rustc accepts it, and this analysis records the gap.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. The lang-item prefix is the exact pinned Rust core fragment
from the census batch 2 controls, with its attributes and comments. It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/waited_copy_selections.rs` checks every case.
See `docs/parallel/census-batch6.md`.
