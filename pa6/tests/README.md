# PA6 required fixtures

`manifest.json` declares outcomes before reference generation. `spec/names/`
exercises namespaces, lexical scopes, imports, privacy and normalization;
`spec/modules/` contains isolated source packages with explicitly listed
auxiliary module files. Only primary inputs have text sidecars. Auxiliary files
are discovery-checked and included in reference receipts.

Run `make -C pa6 test` for exact text/stdout/stderr/status comparison and
`python3 scripts/check_syntax.py pa6` for the independent pinned-rustc checks.
The variant-as-type fixture explicitly separates name resolution from PA9 type
validity. No raw fragments are excluded from independent checking.

The module-primitive fallback cases preserve canonical bare primitive types when
a same-named module is visible, including an imported module. Real nominal and
alias shadows and qualified module members remain ordinary resolved bindings.
Pinned metadata accepts both new sources; their text sidecars are pending the
integrator's reviewed scoped generation.
