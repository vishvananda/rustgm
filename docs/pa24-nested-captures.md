# Nested capture formation

The active closure collector reads only its retained immediate-child scheduler
indices. A split descendant slice supplies borrowed child capture plans without
cloning inference graphs or scanning siblings. Child paths are reconstructed from
parent binding terms and canonical field/reference steps; child inference IDs
never enter the parent. A per-child prefix cache shares imported ancestry and
records sparse `(child source, child path) -> parent path` mappings before outer
capture precision or move-storage truncation.

Forming a borrowed mutable or unique child environment requires a mutable outer
call, even when the child is never invoked. Moving a child field requires an
outer move only when the parent value is not Copy. An outer move closure changes
its storage independently of that call permission. Parent-local roots retain
ordinary local formation rather than acquiring invented environment fields.

Runtime child formation resolves mapped physical parent places and checks exact
formation types before ordinary reads/borrows. A covered inner wildcard checks
initialization again, so moving the captured field before inner formation still
rejects. An uncovered exact parent mention has no runtime storage: its outer
creation already checked initialization and contained-reference validity.
Initialization-only suffixes retain typed projections without runtime or
liveness Project actions. Normal ownership, loan, lifetime and cleanup proofs
remain mandatory.

The declared matrix contains 25 Rust 1.98 default-lint outcomes: 18 accepts and
seven rejects. Every accepted program executes through both rustc and the pinned
CPPGM backend with value checks. Cases cover shared/mutable/move/precise captures,
three levels, shadowing, parent locals, empty children, unused mutable formation,
unique borrowing, conflicts, repeated Once calls and wildcard formation duties.
The direct-capture group and 45 existing closure/guard regression functions also
pass, including their selected native controls.

Nine independently qualified scaling inputs retain linear work:

| Input | Sizes | Capture graph work | Canonical work |
| --- | --- | --- | --- |
| Nested depth | 2 / 8 / 32 | 12 / 36 / 132 | 78 / 258 / 978 |
| Distinct fields | 4 / 16 / 64 | 41 / 149 / 581 | 156 / 528 / 2016 |
| Repeated field | 4 / 16 / 64 | 23 / 59 / 203 | 87 / 183 / 567 |

Evidence is under `obj/parallel/pa24/nested-execution/`. `final-first.json` records
the passing focused tests, strict all-target/all-feature Clippy, audit 1337/zero,
layout and pinned-upstream checks. Earlier observations remain: depth 64 reached
the existing syntax-nesting limit of 128, and strict Clippy required a test-only
String formatting repair. The parser limit was preserved; depth controls use
2/8/32 instead. Combined release and immutable performance qualification remain
integrator work.

Generic captured execution, stronger callable adapters, opaque receiver storage
views, reference coercion capture records, and the existing packed/raw/pattern
leaf boundaries are separate work. This checkpoint changes no activation or
generated reference.
