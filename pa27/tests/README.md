# PA27 fixture ownership

The released `unsafe-places` component publishes `units/unsafe-places.json` and
the reference sidecars of its 262 cases; see the
[release record](../../docs/pa27-unsafe-places-release.md) for its deferred
list. Every other family here is unpublished.

The nine `spec/capture-native` cases declare `prelude: tests/prelude/check.rs`.
The compiler reads that file's bytes followed by the fixture's on standard
input, as one translation unit; `prelude/check.rs` defines the `check` function
those bodies call and is a byte copy of the maintainer native harness prelude.

PA27 is closed at its finite contract: `manifest.json` is exactly the union of
the released unit manifests, written by
`python3 scripts/finite_contracts.py compose pa27` and never by hand. It has no
reference receipt of its own; the released units own every sidecar. The draft source
programs that no released unit claims are listed in `manifest.draft.json` and
stay unpublished.

Required families: required unsafe contexts; alignment; raw borrows; union reads.

Put required regressions here, in the earliest assignment owning all their
features. Register every draft input before qualification, then publish the
reviewed manifest and generated references together, through a unit release.
The draft does not add a case to the closed contract.
