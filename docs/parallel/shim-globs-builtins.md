# Shim glob rows and the `compiler_builtins` loader flag

Lane B batch of 2026-11-08, branch `implement/shim-glob-builtins-20261108`
on main `85dbaaff0`. It implements the integrator's two decisions after the
[core sites and re-exports batch](core-sites-reexports.md):

1. one glob row per `pub use dependency::path::*`, so that crates whose
   `core` is `rustc_std_workspace_core` resolve through the shim as rustc
   does;
2. rustc 1.98's `compiler_builtins` injection with a crate loader flag.

The [nightly private-test repair](nightly-private-repair.md) comes first on
the branch and has its own record.

## 1. Glob rows

### Wire

The macro envelope (tag 14) gains version 5 and row kind 4 `GLOB`:

```
[u8 4][placement path][u32 owner][target path]
```

A path is a `u16` count followed by strings, each a `u32` length and its
bytes. The owner is 1 plus the index of the target's crate in the
publication's own sorted alias dependencies, as for a re-export row. Owner 0
is rejected. A glob row has no name and sorts with the other rows by
(placement, empty name), so it comes before the named rows of its placement.
Glob rows of one placement sort by (owner, target). Version 5 is written only
when a table has a glob row; every other table keeps its version-1 to
version-4 bytes.

`shim.rs` (`pub use core::*;` with `core` its only dependency) publishes one
row, `04 00 00 01 00 00 00 00 00`: kind 4, an empty root placement, owner 1
and the empty root path of `core`. Its whole artifact is the same for 6, 24
and 96 public target names. `ambiguous-shim.rs` publishes three rows at its
root: (1, []), (1, [`left`]) and (1, [`right`]).

### Producer

- After imports finish, `resolve::crates::record_module_globs` keeps each
  public glob import whose final source is a module that no local
  declaration defines, as (import index, module).
- Every name such an import binds is covered by its glob row, so no public
  name, macro or re-export row repeats it (`export::dependency_globs::Globs`).
  A glob-made ambiguous name stays covered only when no other public glob of
  that module made it.
- `dependency_globs::rows` publishes the placement from the public module
  traversal and finds the target's owner and canonical path through the
  dependency catalog: the owner's own declaration rows, whose ancestors must
  all be public, or a legacy artifact's module definition. A glob in a
  private module binds nothing outside the crate and publishes nothing.
  These are rejected by name: a glob in a module reached only through a
  re-export, a target whose owner reaches it through a non-public module,
  and an owner that is not a direct dependency.

### Consumer

`import_crates` expands glob rows after it binds every artifact's
definitions and macro re-exports and before `bind_crate_aliases`
(`resolve/crates/shim_globs.rs`):

- Each artifact is expanded after the artifacts its rows name; a cycle is
  rejected by name.
- The owner is found by crate identity through the artifact's own
  dependency list and the inventory. The target and placement are found
  through each crate's public module identities (`placement_scope`), never
  by a name or alias.
- Every public binding of the target in the type, value and macro
  namespaces is bound at the placement, unless the placement binds that
  name itself (an explicit binding shadows).
- When two glob rows give one name different targets, the name becomes
  ambiguous. Only a use of it is an error, by name.
- Each row is charged 1 unit, plus 1 per binding of its target module that
  it visits, against `MAX_COMPONENTS`.

Older readers reject version 5 by name: the pinned base binary rejects the
candidate's `shim.rs` artifact with "unknown exported macro metadata
version".

### Controls

Private tests (`dev/tests/invariants/shim_expansion.rs`, included as
`resolve::crates::shim_globs::tests`):

- `glob_rows_stay_fixed_and_expansion_work_grows_linearly_with_the_target`:
  6, 24 and 96 target names. The shim bytes are identical across widths,
  bytes 12..14 are `[14, 5]`, and expansion work grows by exactly 18 and 72.
- `explicit_shim_names_shadow_and_two_glob_targets_are_ambiguous_only_when_used`.
  It also checks that `pub(crate)` and private-module names are not carried.
- `forged_glob_rows_and_relabelled_tables_are_rejected_by_name`:
  - owner 0, and an owner past the alias list;
  - a version-4 label ("unknown exported macro row kind") and a version-6
    label;
  - a version-5 label on a table without a glob row;
  - every strict prefix;
  - targets the owner does not export (`sealed`, `missing`), and a
    placement the shim does not export (`left`). These decode and then fail
    in the consumer by name.

PA29 `shim-glob-outcomes.json` has 15 cases under `spec/shim-globs/`. They
were declared before the frozen rustc oracle and before any RustGM candidate
run.

- Cases:
  - `core.rs` and its rustc-only `compiler_builtins`;
  - three shims, each declaring metadata version 5;
  - three native clients (2015, 2018 and 2024), each printing `42 40`;
  - `macro-client.rs`;
  - opponents: `hidden` (E0425), `sealed` (E0433) and `shadow` (E0425,
    E0433), each with RustGM's named diagnostic;
  - `shadow-control`;
  - the unused- and used-ambiguity clients.
- The rustc oracle matches all 112 declared outcomes natively
  (`obj/lane-b3/oracle-shim-globs-2/oracle.json`).
- Declared divergence: rustc reports a used cross-crate glob ambiguity
  through the deny-by-default future-incompatible lint
  `ambiguous_glob_imports`. The oracle runs with `--cap-lints=allow`, so
  rustc accepts the call and selects one target. RustGM rejects it with
  "ambiguous value name `shared`", as for a used glob ambiguity within one
  crate (E0659).
- Amended control: the first `shadow-client.rs` returned
  `core::option::Option<u32>` from a public function. Through the plain
  shim it resolved, but the export failed with "generic nominals are outside
  the metadata profile", a limit unrelated to globs. The file now uses
  `Option` only privately. The oracle was rerun before the next candidate
  run, with unchanged outcomes.

### Evidence

- `scripts/check_macro_metadata.py --native` with candidate
  `obj/lane-b3/pins/rustgm-cand-1` (sha256 `ef762b54…`): 112 cases, receipt
  `obj/lane-b3/candidate-1/complete.json`.
- Older-profile byte identity against the pinned base
  `obj/lane-b3/pins/rustgm-base-6db8f5124` (sha256 `be666823…`):
  - the base qualified the 97 earlier cases
    (`obj/lane-b3/base-1/complete.json`);
  - all 33 metadata artifacts they publish are byte-identical with the
    candidate's;
  - 5 source-core providers × 8 source profiles: 40/40 identical exports
    and LowIR (`obj/lane-b3/identity-1/identity.json`).
- Fastcheck `1794111025875476358` passes:
  - the three glob tests;
  - `crate_linking::transported_rows::frozen_macro_metadata_outcomes_hold_for_every_case`;
  - `legacy_availability_remains_unknown_and_hot_records_do_not_grow` and
    `marker_shape_ordinary_storage_and_derive_owner_layouts_preserve_boundaries`;
  - the two `Imported` layout controls in `resolve::body_origin::layout_tests`.
- PA fastchecks:
  - `1794111080165525747` (PA1–PA12);
  - `1794111151969177038` (PA13–PA19 and PA21–PA24);
  - `1794111234521497884` (PA20, 1451/1451).
- `make test-upstream` passes.
