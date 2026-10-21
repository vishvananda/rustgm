# PA25 fixture ownership

Planned: no released grading contract or generated reference outputs yet.

`migrated-outcomes.json` declares accepted outcomes for the unchanged former
PA12 discriminant-statements/constant-function and PA19 generic-constant-statements
boundaries. All independently compile with pinned rustc. Their original paths and source
hashes are retained in the draft declarations; their obsolete rejection oracles
are removed from the earlier assignments. Native qualification executes both compilers before these fixtures can enter
the released PA25 manifest.

The unchanged former PA9 `constant-call` boundary is also retained, with a
declared rejection: required constants cannot call non-const functions, including
when unused. Its type inspection mode preserves the original input contract.

Required families: const restrictions; pointers; recursive calls; limits; const generics.

Put required regressions here, in the earliest assignment owning all their
features. Keep draft inputs in explicit outcome files until release; then register
every input in `manifest.json`. No undiscovered fixtures may enter the graded suite.

`inspection-outcomes.json` declares the entry-free library, compressed-repeat,
unused-invalid-initializer and unused-invalid-body inspection controls. The exact
proposed row grammar and ordering are in
[the constants view draft](../../docs/pa25-constants-view.md). Emit and count modes
consume the same complete checked pipeline and graph inventory. These draft
fixtures have no generated oracles and do not activate PA25.

`inline-owner-outcomes.json` registers the two unchanged scalar const/static
owner controls migrated from PA28. The current full draft inventory and its
remaining coverage questions are in [PA25 readiness](../../docs/pa25-readiness.md).
