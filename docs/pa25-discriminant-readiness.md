# PA25 resumable discriminant execution

The PA12 source `const fn f()->isize{3}enum E{A=f()}fn main(){}` previously
reached the checked executor, requested `f`'s body, then surfaced the synchronous
query driver's internal `constant execution body is not ready` error. Independent
Rust 1.98.0 object qualification accepts that source.

Discriminant completion now suspends the existing query graph, runs its exact
execution/declaration requirement through the declaration worklist, and resumes
the same graph. Pending state retains the current entry cursor and duplicate-tag
set. Completed entries are published once and are never restarted on a wait.
The all-implicit classification is maintained when entries are appended, so a
resume cannot rescan the complete entry table. The implicit-only path remains
linear and avoids constructing an evaluation query graph.

Source enum definitions explicitly retain uncompleted variant ordinals. Tag
publication removes the selected ordinal, and layout remains unavailable until
all ordinals have completed. Both canonical enum casts and service-free variant
reads reject an unready selected tag. ABI layout rejects an uncompleted enum
before recording a physical representation. This prevents a const callee from
reading a provisional zero tag or caching a provisional layout while a caller's
discriminant query is suspended. Indexing and first publications are bounded and
counted; repeated publication does not increment the count.

The supported end-to-end source/native cases are:

- Ordinary and forward const-function discriminants, including implicit successor
  tags, loops, closed generic calls and repeated calls to the same checked body.
- Const callees containing named constant patterns, with forward/reordered
  declaration controls.
- Casts of already completed enum variants from another discriminant's callee.

Two independently Rust-valid cases remain explicit boundaries: a named-constant
pattern directly in the discriminant root retains the existing general-pattern
message, and a callee casting a forward uncompleted enum tag reports
`enum cast requires a completed discriminant`. The latter also safely rejects
an actual self-tag cycle; the diagnostic alone does not classify Rust validity.
The independent outcomes distinguish that invalid cycle from the valid forward
case. Ordinary enum/constant dependency cycles retain the existing iterative
cycle diagnostic. Duplicate tag values retain their duplicate-value diagnostic.

Validation passed under the shared controller: formatting/check, one direct ABI
readiness unit, three new source/outcome/native tests, eight existing
pattern/required-value regressions, strict all-target/all-feature Clippy and the
architecture/file audit (1,204 files, zero findings). Eight positive sources
match independently compiled/native values. Five independent object controls
cover the valid boundaries, true cycles and duplicate values. The scaling
families at 8/32/128 variants assert exactly one indexing/publication per
variant, `N+1` finish-entry visits and one CFG for a reused callee, or `2N` visits
and `N` CFGs for distinct demanded callees. These counters describe indexing,
publication and finish-entry visits, not all interpreter or hash-table work.

Evidence: `obj/pa25-discriminant-readiness/{0..6}.log`, `probe.json`, source-map
SHA256 `567273bf2f1cb3ac12d2b38231b8a6b39f1c6a0fcf550a2ccbd58efd91e1f547`.
Successful resource receipt:
`obj/parallel/resources/942c3022c65d4b67bfd8dd94174fbfee.json` (return code 0).
An earlier batch was cooperatively interrupted with exit 130 after review found
the resumed all-implicit scan; it is retained as infrastructure interruption,
not a test outcome. The revised bounded implementation passed all listed gates.

This checkpoint does not complete PA25. Dynamic pattern/tag continuations,
initializer indexing and unused-initializer validation, associated/slice/text/raw
execution, deterministic constant graph views and owning release fixtures remain
separate work. The integrator owns migration of the now-supported PA12 fixture
and assignment activation/reference publication.
