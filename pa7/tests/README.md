# PA7 fixture ownership

The manifest declares every required source and its independently expected
status before oracle generation. `rules`, `fragments`, `repetition`, `follow`,
`docs`, `scope` and `boundaries` cover PA7 responsibilities. Every case has a
complete-program rustc classification; explicit reasons distinguish the two
generated-syntax checks deferred to PA8.

Use `make test-pa7` for byte-exact committed text comparisons and
`python3 scripts/check_syntax.py pa7` for independent acceptance checks.
Only `make ref-test-pa7` writes generated sidecars after checking all outcomes.
No fixture may rely on later expansion, hygiene, type inference or lowering.
