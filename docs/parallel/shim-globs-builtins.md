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

## 2. The `compiler_builtins` loader flag

### Wire

A crate's loader attributes travel in one flags byte, profile tag 15
`CRATE_LOADER_FLAGS`, which leads only the outermost artifact:

```
[12-byte header][u8 15][u8 flags][the unflagged artifact's bytes from byte 12]
```

- Bit 0 is `#![compiler_builtins]`. Bits 1 to 7 stay open for
  `panic_runtime` and `needs_panic_runtime`, which are not added.
- The tag is written only when a flag is set, so every other artifact keeps
  its bytes. Encoding splices the two bytes in at byte 12 after the budget
  check.
- These are rejected by name:
  - a bit outside bit 0 ("unknown crate loader flag");
  - a zero flags byte ("crate loader flags envelope requires a flag");
  - a second tag 15 ("repeated crate loader flags");
  - a tag 15 on an artifact wrapped by the link or macro envelope ("crate
    loader flags belong to the outermost artifact").
- Older readers reject tag 15 by name. Given the candidate's flagged
  `nostd-builtins.rgm`, the pinned `rustgm-cand-1` (11b3fd7cd) fails with
  "invalid metadata base profile" (`obj/lane-b3/base-flag-probe.txt`).

`no-std-clients/builtins.rs` publishes a 62-byte artifact (sha256
`e7fd3e66…`) that begins:

```
52 47 4d 45 54 41 0d 0a 01 00 00 00 0f 01 00 11 00 00 00 63 6f 6d 70 …
```

The unflagged `plain-builtins.rs` artifact continues `00 11 00 00 00 63 …`
after the same header. The upstream shim publishes no flag. Its glob row
names owner 2, because `core` follows `compiler_builtins` in its sorted alias
dependencies: `0e 05 01 00 00 00 04 00 00 02 00 00 00 00 00`.

### Producer

- `#![compiler_builtins]` is a `Loading` decoration. `root_options` checks it
  as rustc 1.98 does:
  - it needs the internal feature `compiler_builtins` (`Feature` 76),
    otherwise "the `compiler_builtins` attribute requires its internal
    feature" (E0658);
  - "multiple `compiler_builtins` attributes";
  - "malformed `compiler_builtins` attribute input" for an argument (rustc
    E0565) and for an outer or unsafe form;
  - "the `#![compiler_builtins]` attribute can only be used at the crate
    root".
- `Package::loader_flags` reads the validated root word.
  `build_library_prepared` and `build_source_library_profile` attach the
  flags to the published interface.

### Consumer

`standard_imports::inject_compiler_builtins` runs after the prelude scope is
complete and before names are marked resolved. It follows rustc 1.98's
`CrateLoader::inject_compiler_builtins`:

- Only a `no_std` source client loads a real core, so only it can need the
  injection. RustGM's built-in core and std stand for a graph that already
  contains the crate.
- A `#![no_core]` or `#![compiler_builtins]` crate is never injected.
- Any artifact of the dependency inventory with bit 0 satisfies it. This is
  the integrator's model of rustc's check over every loaded crate.
- Otherwise, an `--extern compiler_builtins=` entry fails with "the crate
  resolved as `compiler_builtins` is not `#![compiler_builtins]`" (rustc:
  "the crate `compiler_builtins` resolved as `compiler_builtins` but is not
  `#![compiler_builtins]`"). Without one the client fails with "can't find
  crate for `compiler_builtins`" (E0463).
- An unflagged crate named `compiler_builtins` in the inventory but not on
  `--extern` gets the E0463 text. Rustc would find it by its name in the
  library path and reject it by the other name; both reject the client.
- The injected crate binds no name and adds no dependency row.

The upstream shim also needs `use compiler_builtins as _;` to resolve
through an `--extern` legacy artifact. In source-binding mode,
`Imported::alias_root` accepted only a root from the source catalog. It now
also accepts the imported root of an alias whose artifact has no source
descriptor. A path through `::compiler_builtins` and `extern crate
compiler_builtins as _;` already worked.

### Controls

Private tests (`dev/tests/invariants/loader_flags.rs`, included as
`resolve::standard_imports::loader_tests`):

- `loader_flags_lead_only_the_outermost_artifact_and_keep_every_other_byte`:
  - bytes 12..14 are `[15, 1]`, and removing them gives the unflagged
    artifact;
  - the round trip and re-encoding;
  - flags 2, 0x81 and 0;
  - a repeated tag;
  - a flag on the macro envelope's wrapped artifact;
  - every strict prefix;
  - an empty flags no-op, and a second assignment rejected.
- `no_std_clients_need_a_compiler_builtins_crate_in_their_graph`:
  - the builtins crate publishes `[15, 1]`;
  - a client without one gets the E0463 text;
  - a client with one in its inventory, also on `--extern`, is accepted and
    publishes no flag;
  - a `no_core` client is accepted;
  - a plain `no_core` crate named `compiler_builtins` on `--extern` is
    rejected by name.
- `the_compiler_builtins_attribute_is_one_gated_crate_root_word`: the
  ungated, duplicate, valued and nested-module forms.

PA29 amendments, declared on 2026-11-08 with
`obj/lane-b3/tools/amend_builtins.py` before any rustc or RustGM run of the
amended cases. Each manifest records them in its `amended` field.

- `no-std-client-outcomes.json` (13 to 17 cases):
  - `nostd-builtins` is compiled by RustGM;
  - every `no_std` client lists it in its inventory;
  - `nostd-missing-builtins-client` runs the 2024 client before the crate
    exists (rustc E0463);
  - `plain-builtins.rs`, a `no_core` crate named `compiler_builtins` without
    the attribute, is accepted and publishes no flag;
  - `nostd-plain-builtins-client` names it on `--extern` (rustc's "resolved
    as `compiler_builtins` but is not `#![compiler_builtins]`");
  - `ungated-builtins.rs` lacks the feature (rustc E0658).
- `variant-reexport-outcomes.json` (5 cases) and
  `shim-glob-outcomes.json`: the `compiler_builtins` stand-ins are compiled
  by RustGM, and every `no_std` client lists them.
- `extern-core-outcomes.json` (4 cases): the two `no_std` clients list
  `nostd-builtins`. The `no_core` client is unchanged.
- `shim-glob-outcomes.json` (15 to 17 cases):
  - `upstream-shim.rs` holds the exact bytes of rustc 1.98.0's
    `library/rustc-std-workspace-core/lib.rs` (sha256 `6e011f4f…`), recorded
    in `third_party/sources.json` as a source fragment;
  - `shim-glob-upstream-client` reaches core through it natively (`42 40`)
    with no `compiler_builtins` of its own, because the shim's `--extern`
    one is in its inventory.

The rustc 1.98.0 oracle matches all 118 declared outcomes natively
(`obj/lane-b3/oracle-builtins-1/oracle.json`).

### Evidence

- `scripts/check_macro_metadata.py --native` with candidate
  `obj/lane-b3/pins/rustgm-cand-2` (sha256 `72380b04…`, the same bytes as
  the fastcheck build of this tree): 118 cases, receipt
  `obj/lane-b3/candidate-b2/complete.json`.
- Older-profile byte identity against `rustgm-cand-1` (11b3fd7cd):
  - the 40 metadata artifacts that both candidates publish are
    byte-identical. The 5 new ones are the 4 `compiler_builtins` crates and
    the upstream shim;
  - 5 source-core providers × 8 source profiles give 40/40 identical runs
    (status, stderr, LowIR and metadata; 13 publish and 27 stop at the same
    profile limit), `obj/lane-b3/identity-2/identity.json`.
- Fastcheck `1794114135984961918` passes:
  - the three loader tests and the three glob tests;
  - `crate_linking::transported_rows::frozen_macro_metadata_outcomes_hold_for_every_case`;
  - `legacy_availability_remains_unknown_and_hot_records_do_not_grow` and
    `marker_shape_ordinary_storage_and_derive_owner_layouts_preserve_boundaries`;
  - the two `Imported` layout controls in `resolve::body_origin::layout_tests`.
- PA fastchecks:
  - `1794114352013180023` (PA1–PA12);
  - `1794114447058531070` (PA13–PA19 and PA21–PA24);
  - `1794114538180565247` (PA20, 1451/1451).
- `make test-upstream` passes: 128 pinned upstream fixtures and 126
  Reference files.
