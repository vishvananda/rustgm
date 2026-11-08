# Scoped body proofs, variant re-exports and `--extern core=`

Lane B batch of 2026-11-07/08, branch `implement/scoped-proofs-shim-globs-20261107`
on main `519f8eda3`. It implements the first three Lane B items of the
[census plan](../core-census-plan.md) as far as the integrator's decisions
reach:

1. "invalid scoped proof evidence" when a generic body selects a
   region-binding impl;
2. variant re-exports in a source-core export (core's `prelude::v1`);
3. `--extern core=` naming a crate's direct `core` dependency, the
   shape-independent part of the `rustc_std_workspace_core` shim.

The shim's glob row and the `compiler_builtins` injection are recorded in
section 4 with the integrator's decisions. They are not implemented in this
batch.

## 1. Body selections with quantified conditions

A generic body that normalizes `Option<Self::Item>` in a method signature's
callable scope selected an impl whose condition is quantified
(`for<'a> F: FnMut(&'a T)`). The selection's proof carries that condition's
own binder scopes. `Cursor::advance` appended it with `append_at` below a
non-root scope, which rejects quantified evidence there ("invalid scoped proof
evidence").

- `Obligations::append_streamed_at` (`types/lifetime_proof/stream.rs`) grafts
  a child that has its own scopes or blocks below the query's scope instead.
- `graft_owned` (`types/lifetime_proof/graft.rs`) is the grafting body without
  the fresh-template check. It maps every child scope below the root, and
  attaches owned intervals and scoped blocks in allocation order. Each
  foreign interval stays excluded through `resume_at`. A foreign interval
  that crosses an owned block, or that lies outside the envelope, is an
  error. `graft` keeps its template check and calls it.
- Private controls: `stream/interleaved_tests.rs` (a child with an inner
  binder below a callable scope keeps its gap, depth and escaping-equality
  failure; a crossing gap is rejected; root and flat children keep their
  attachment). `elided_sugar_parameters.rs`'s two recorded gaps, `body-caller`
  and `body-fixed-output`, now pass, so its `GAPS` list is gone (amended
  control: the cases now expect success).
- PA31 `pa31/tests/spec/scoped-body-selections/`: 17 fixtures, declared
  2026-11-07 before any rustc or RustGM run (declaration copy
  `obj/lane-b2/declarations/scoped-body-selections.declared.json`). The rustc
  1.98.0 oracle (`obj/lane-b2/oracle-v1/scoped-body-selections/`) matches
  all 17. Eleven are accepted, and four opponents are rejected: `mismatch`
  and `adapter-mismatch` (E0308), `unsatisfied` and `escaping` (E0599).
  `nested_condition_universes.rs` checks them against RustGM.
- Two recorded gaps share a separate, pre-existing cause, present on main:
  a body-selected impl parameter that is determined to name a caller lifetime
  (`caller-region`, `ordinary-region`). The trial stays `Progress::Inferred`
  and fails as "trait obligation requires an unambiguous matching assumption
  or implementation". (Amended 2026-11-08: both now pass. The cause and the
  repair are in [caller-region premises](caller-region-premises.md).)

## 2. Variant re-exports

`pub use crate::option::Option::{self, None, Some}` failed at export ("source
header names an unreserved actual declaration"). Following the integrator's
decision, there is no new tag or row:

- The producer (`authored_export/namespace_emission.rs`) emits the
  value-namespace binding as an ordinary `ModuleBindingRow` whose target is
  the kind-11 variant declaration. It drops the type-namespace alias only when
  the same scope has the same variant's value binding, with the same access.
  Otherwise it fails with "type-namespace variant re-export requires its value
  constructor binding", for example a glob whose `Some` constructor a local
  function shadows.
- The consumer (`generics/implementation_domains.rs`) re-derives the
  namespace from the target's installed parent (`constructor_namespace`). An
  enum parent binds the value constructor. A struct parent fails with "source
  struct constructor bindings require their wider constructor profile",
  which Lane C extends (integrator decision of 2026-11-08). Both sides also
  reject a variant re-export below version 13 by name. The earlier
  variant/enum version-13 gates already fire first: "source declaration kind
  is outside the initial source profile" on export, and "source enum headers
  require the version-thirteen body IR profile" on restage.
- Recorded deviation: a type-namespace use of a re-exported variant, such as
  `prelude::v1::Some { 0: x }`, is not supported. The PA29 opponent
  `variant-type.rs` (`fn take(_: None)`) is E0573 in rustc and "unresolved
  type name" in RustGM.
- `reexported_variants.rs` (private) checks:
  - the v13 round trip;
  - four `Import` rows plus the prelude `Glob` rows, all value-namespace
    `Variant` targets;
  - forgeries: namespace flipped to type, source relabeled `Declaration`, a
    duplicated type-namespace alias row, the enum parent relabeled as a
    struct;
  - older-profile rejection on export and restage;
  - strict prefixes around the renamed row;
  - the shadowed-constructor rejection;
  - a 6/24/96 scaling control: one value row per re-exported variant, with
    consumer work linear in the width.
- PA29 `variant-reexport-outcomes.json` (`spec/variant-reexports/`): five
  cases, declared 2026-11-08 before any rustc or RustGM run of those files.
  An exploratory RustGM probe of a similar provider ran first, under
  `obj/lane-b2/variants`. The oracle matches all five. 2021 and 2024 clients
  match `Some`/`None` from the injected prelude and renamed `Present`/`Absent`
  through a module path, natively (`42 40`). A rustc-only `compiler_builtins`
  stand-in is built against this provider and passed by `--extern`.

## 3. `--extern core=`

`Dependency::new` now admits the alias `core`; `std` and the path keywords
stay reserved (`extern_core_aliases.rs`). The `no_std` injection
(`standard_imports::provider_root`) reuses an existing `core` extern-prelude
entry instead of failing as "already bound". That entry must be a public,
direct, unambiguous binding to an imported crate root; otherwise it fails
with "no_std source client's `core` alias is not a dependency root". The
edition prelude is then read from that crate. Without the alias, the provider
root is bound as before.

PA29 `extern-core-outcomes.json` (`spec/extern-core/`): four cases, declared
2026-11-08 before any rustc or RustGM run. The opponent was redesigned once
before any run: the first draft aliased a second source provider, which the
one-provider preflight rejects before the injection. All four match the
oracle and RustGM:

- the no-std 2024 and 2015 clients with `--extern core=` run natively;
- a `no_core` client reaches `core::base` through the alias;
- the opponent aliases that `no_core` client, which has no prelude: rustc
  reports "cannot resolve a prelude import", and RustGM reports "no_std source
  client's core provider lacks its edition prelude".

## 4. Integrator decisions for the remaining items (2026-11-08)

Source clients export through `export_with_originals`: the legacy core table,
`PublicNames` and the tag-14 macro wrapper, never a version-13 descriptor. The
shim's glob row and the builtins flag therefore cannot be version-13 shapes.

- **Shim glob (approved).** Tag-14 macro table version 5 adds row kind 4
  `GLOB`: `[u8 4][placement][u32 owner][target module path]`.
  - The owner ordinal is the REEXPORT rows' (1 + index in the sorted alias
    dependencies; 0 rejected). Rows are ordered by placement, and a table
    without such rows keeps its exact bytes.
  - The consumer expands only externally visible names, across the type,
    value and macro namespaces. An explicit shim binding shadows a glob name,
    and different targets are ambiguous only when used. Each expanded name is
    charged once. Older readers reject version 5 by name.
  - Forgeries: owner 0, an owner past the alias list, a target module the
    owner does not export, and every strict prefix.
  - Rejected: splitting the glob across `PublicNames` and tag 14.
- **`#![compiler_builtins]`.** A new crate-loader flags record in the legacy
  core table's profile chain. It is written only when a flag is set: one byte,
  bit 0 = `compiler_builtins`. Unknown bits are rejected by name; later bits
  such as `panic_runtime` and `needs_panic_runtime` join it. The tag number is
  Lane B's to choose and report.

Neither is implemented here. Task 5 (`self-where`) is not started.

## 5. Checks

All fastcheck receipts are under `obj/qualification/fast/`.

- Task 1: commit `09ac9f056`, with its PA31 oracle and fastcheck receipts
  from 2026-11-07.
- Task 2: `1794099670121023368` passes the variant invariant, both hot-record
  invariants, `domain_artifacts` and the 93-case macro-metadata integration
  test.
  - `scripts/check_macro_metadata.py --native` qualified 93/93
    (`obj/lane-b2/macro-cand-t2`).
  - The oracle-only run matched 93 (`obj/lane-b2/macro-oracle-v1`).
- Task 3: `1794100036577839467` passes the alias test, two
  `client_selection_policy` tests, both hot-record invariants and the
  97-case integration test.
  - The oracle-only run matched 97 (`obj/lane-b2/macro-oracle-v2`).
  - The native run qualified 97/97 (`obj/lane-b2/macro-cand-t3a`) with pin
    `obj/lane-b2/pins/rustgm-cand-t3a`. All 32 earlier `.rgm` hashes are
    unchanged.
- Text oracles on the final tree, using the dev-fast CLI:
  - PA1–PA12 pass (`1794100525587030098`);
  - PA13–PA19 and PA21–PA24 pass (`1794100602812737308`);
  - PA20 alone passes 1451/1451 (`1794100726914337637`).
- `make test-upstream` verified 127 pinned upstream fixtures and 126
  Reference files.
- Older-profile byte identity (`obj/lane-b2/identity/identity.json`): base
  pin `rustgm-base-09ac9f056` (`2533373…`) against candidate
  `rustgm-cand-5d4b113e8` (`7d89660…`), over five providers and eight
  profiles. 39/40 exports are identical (12 successful, 27 identical
  rejections). The only difference is the variant provider at version 13,
  which the base rejects and the candidate exports. All 31 earlier PA29 `.rgm`
  hashes also match a main-built receipt.

## 6. Shared files touched

The only shared file touched is `types/lifetime_proof/{graft,stream}.rs` (task
1). `sema::traits` is untouched.
