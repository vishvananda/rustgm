# PA25 deferred pattern publication correction

A named constant used in a match pattern initially reserves an uncompleted scalar
interval. The former batch algorithm evaluated every scope's requested constant
values before publishing any scope's completed ranges. Thus
`const N:u8=7; const C:u8=match 7u8{N=>1,_=>2};` could capture an executable CFG
with the placeholder interval for `N`, cache `C=2`, and later publish the correct
range too late. A runtime match naming `C` exposed the wrong result; rustc yields
`C=1`.

Deferred scopes now retain explicit typed body-readiness markers. A stable,
iterative dependency graph orders scopes by selected named-pattern owners. Each
scope resolves its requested values and publishes its ranges/coverage exactly
once before a dependent scope can request a value. An uncompleted body is
rejected before CFG construction, including const callees whose pattern scope
is not represented by an available owner dependency. A placeholder can never be
executed merely because ordinary body typing has finished.

Scheduling charges scope/request/edge visits, including repeated occurrences,
before growing dependency storage, with a report-wide one-million-visit bound.
The 8/32/128 repeated-user family has exactly `5*width+3` graph visits and one
constant execution. This is a graph-visit invariant, not a claim of total linear
scheduler work: the stable ready heap costs `O((V+E) log V)`, with additional
per-scope dependency sorting. Heap/sort comparisons are not in that counter.

Four portable integration tests cover eight independently compiled/native
positive cases: the original input, forward `N`, `main` before declarations,
nested matches, named range endpoints, a completed const callee, repeated queries
and a dependency chain. Independent object checks distinguish a Rust-valid but
unsupported cross-body callable from a true pattern-dependency cycle. The
former must report `body patterns require completed constant dependencies`;
it is not a semantic-negative claim. Dynamic cross-body pattern-readiness
continuations remain PA25 work, alongside discriminant readiness and the other
boundaries recorded in `pa25-retained-storage.md`.

Focused validation passed: the four new tests (including all native cases), the
six existing required-value/CFG tests, strict all-target/all-feature Clippy and
the architecture/file audit (1,202 files, zero findings). The new test was renamed
from `deferred_patterns.rs` to `interval_publication.rs` solely to satisfy the
repository filename-stem audit; its graph-visit test name was clarified and
counter documentation corrected afterward. Formatting, all-target Clippy and
audit were rerun; no semantic code changed after the passed tests.

Evidence: `obj/pa25-pattern-correction/{0..4}.log`; source-map SHA256
`881201e43158d21df85d9c159e770ccd71a204cdde7585890cff58c831bba866`.
Final shared resource receipt:
`obj/parallel/resources/ebc4cdf8495242e8a4b0d62ab66c0102.json` (return code 0).
The minimal production diff and integration support apply independently of the
retained-storage checkpoint and do not require its test helper.
