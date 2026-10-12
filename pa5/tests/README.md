# PA5 fixtures

Every `.rs` input is listed in `manifest.json` with independently declared
reference and rustc outcomes. The clusters cover items, signatures, generic and
qualified types, pattern forms, attributes/configuration, syntax rejection and
explicitly deferred name/module checks. All are original course fixtures.

The rustc checks compile complete crate inputs with the pinned target/profile;
configuration arguments are forwarded exactly. Selected cases additionally
assert configured-owner booleans, including that a disabled parent prevents
visiting a malformed nested cfg predicate. These are declared expectations,
not values read back from the generated oracle.

Use `make test-pa5` for byte-exact sidecar comparison. `make ref-test-pa5` is the
explicit reference-generation operation, subject to outcome checks and review.
`dev/tests/crate_syntax.rs` supplies independent typed-structure and resource
invariants. No PA5 fixture requires LowIR or the backend bridge.
