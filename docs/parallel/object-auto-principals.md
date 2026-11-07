# Census batch 1: auto-trait object principals

The first original-core census failed `core/src/any.rs:162` and `:169`
(`impl fmt::Debug for dyn Any + Send` and `dyn Any + Send + Sync`) with "object
requires exactly one principal trait". Lane A's diagnosis was correct. In
source-core mode, `builtin_traits.identify` has no library scope. Core's own
`auto trait Send`/`Sync` therefore resolve as ordinary `Written::User` traits,
and the second one counts as a principal. A no_core probe reproduced this at
the `Send` bound.

## Change

- `prepare_object` asks the source-declaration authority. A bound is an object
  auto marker only when two things hold: the declaration's own header is
  `auto trait` (`OwnerKind::Trait { auto_: true }`), and its source catalog role
  is Send, Sync or Unpin (`freezing::object_marker`). Names are not used.
  Generic arguments are rejected, and quantified bounds keep their scope.
  Builtin-mode objects take the unchanged `BuiltinTrait::Auto` path.
- Two non-auto traits still give "object requires exactly one principal trait"
  (E0225). If either trait is a custom auto trait with no represented marker,
  the error is now the explicit "object auto traits without a represented marker
  are not implemented". This changes no accepted input.
- Source auto goals on an object root are answered from the object's own written
  or inherited bounds (`dynamic_has_auto`). Before this change the structural
  step refused every abstract root, so `need::<dyn Any + Send>()` failed even
  once the type formed.

Freeze and UnsafeUnpin have no established object meaning. They are not
promoted to object markers.

## Evidence

The fixtures are in `pa31/tests/spec/source-object-autos/`: 14 inputs, 28
checks against pinned rustc, all frozen before observation. The six v1 no_core
setup failures are kept. Candidate results:

- Main `219c13b80` (CLI `7051deee`) matched only 7 of 14 cases, and several of
  its rejections had the wrong cause.
- The candidate (CLI `a071a3a3`) matches 13 of 14. The remaining case,
  `v2/custom-auto.rs`, gets the typed refusal described above.

Focused fastcheck `1794032899929518480` passed in 180.9 s: format, audit, strict
Clippy, 9 exact tests (the new principal_auto_sources tests plus the
auto_sources, dynamic_source, automatic_sources and source-Freeze regressions)
and PA23 text.

The census was analysis-only, used unchanged inputs from Lane A's pinned tree
with the same cap and limits, and took 22.2 s with 375 opens. It checked 1,199
declarations, with 5 failed and none blocked. Both `any.rs` records are gone,
and there are no new object-related records. The 5 failures are the earlier
quantified-clause failures in `array/drain.rs` and `filter_map.rs`. The census
still ends at the `flatten.rs:316` budget stop. Main has drifted from Lane A's
census CLI (for example Lane C's `2947cc817`), so the +2 checked count is not
attributed. The only manifest pin rebound is `dev/bin/crates.rs`, which adds
an unused `--source-profile` choice; its diff is recorded.

Evidence is in `obj/parallel/object-autos-v1/`: `reference-v1`/`v2`,
`candidate-main` and `candidate-final`, `census-join` (manifest and producer)
and `core-census`.

## Remaining

- Custom auto traits beside a principal are refused, as described above.
- Bodies have not been reached. `impl dyn Any + Send` method bodies coerce to
  `&dyn Any`, which drops auto bounds, and that coercion is not yet checked
  against source autos.
