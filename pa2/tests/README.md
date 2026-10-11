# PA2 fixture ownership

139 required fixtures are listed in `manifest.json`. The owning handout defines
the exact literal view and its contextual boundary. Every fixture has three
generated `.ref` sidecars; do not hand-edit them.

The spec clusters cover integer radix/range/suffix boundaries; float rounding,
subnormals, infinity and double-rounding traps; Unicode characters; bytes;
ordinary/raw strings; byte strings; C strings; booleans; and stream behavior.
Six unchanged Rust UI cases are imported with checksums and attribution in
`third_party/sources.json`.

The `rustc` manifest field explicitly classifies independent checks:

- A value kind selects runtime byte/bit comparison with rustc.
- `accept`/`reject` requests an independent expression compilation outcome.
- `fragment` is a stream contract, not a complete Rust expression.
- `upstream` is checked as a full program by `scripts/check_rustc.py`.

For PA2 expression checks only, rustc linting is allowed: narrow overflow and
floating infinity are decoded facts here; contextual lint policy comes later.
Invalid escapes, suffixes, radix digits and magnitudes beyond u128 remain errors.

Put required regressions here, in the earliest assignment owning all their
features. Register every input in `manifest.json`; no undiscovered fixtures.
