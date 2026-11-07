# Original stdlib unit graphs

Each set holds the unchanged raw Cargo 1.98.0 `--unit-graph` outputs for core,
alloc and std. `scripts/stdlib_targets.py` refuses any bytes whose SHA-256
differs from its observation. The graphs keep the planning machine's absolute
paths. Package IDs bind only to the frozen local inventories and registry archive
pins, and the executor relocates every path explicitly to the caller's roots.
They are planning inputs for the target unit executor, not evidence of any
RustGM compilation.

- `course-profile/` is the default set, behind
  [the course-profile observation](../../stdlib-course-profile-outcomes.json).
  `scripts/stdlib_plan.py` planned it with Cargo's dev profile set explicitly to
  RustGM's fixed profile: opt-level 0, debug 0, debug assertions off, overflow
  checks on, panic abort. Its declaration preceded planning. Every graph equals
  its historical graph in all fields except the profile.
- `dev-profile/` is historical evidence, behind
  [the original observations](../../stdlib-graph-outcomes.json). It was planned
  with Cargo's default dev profile, whose `debuginfo=2` and
  `debug_assertions=true` RustGM cannot express.
