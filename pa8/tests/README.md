# PA8 fixtures

The manifest declares 87 complete-program cases and every auxiliary source or
resource. `expansion/` covers generated syntax and worklist completion;
`hygiene/` covers definition/call-site identity; `resolution/` covers macro scopes,
exports and imports; `builtins/` covers typed and eager builtin values;
`include/` covers cached source insertion, directory ownership and byte resources.
Every case owns a committed `.ref`, `.ref.stderr` and `.ref.exit_status`.

All cases have pinned-rustc acceptance checks. The trailing-expression include
case uses default lints: rustc otherwise suppresses its incomplete-include error
with the harness's usual `--cap-lints allow`. No acceptance differences are hidden.
Environment cases explicitly set or remove their fixture variables in both
compilers' child processes. They never depend on the developer's environment.
File inputs use repository-relative requested paths for stable `file!()` output.

Fixtures exercise PA8 behavior even when their supporting item/type syntax was
introduced earlier. Type inference, ownership, library hooks and lowering are
not PA8 prerequisites. Typed implementation invariants separately test values,
provenance, parse/read counts and independently resolved definition identities.
