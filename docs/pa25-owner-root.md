# Closure ownership and inline executable roots

A closure's expression source is its semantic body owner. Its `body` source is
its executable syntax root, which may itself be an inline constant. The existing
`BodyKey::Closure` identity remains unchanged. `semantic_source()` supplies the
expression key to CFGs, instance provenance checks and region contracts;
`Metadata.root` retains the syntax root across parked inference states.

`State.body` and declaration-time closure annotation contexts use that semantic
owner. The capture collector takes its executable root explicitly. Scope analysis
likewise accepts owner and root separately: scope boundaries start at the actual
root, while constant-body transparency compares against the semantic owner. This
preserves ordinary function/initializer/anonymous behavior, where owner and root
coincide, without adding a second stored root to every IR body or State.

Consequently `|| const { ... }` creates a distinct checked inline child instead
of flattening the const boundary into runtime closure execution. Its registry
parent is the closure expression and its own source remains the const block.
Typing, const eligibility and value demand use the existing isolated scheduler,
checked provider and iterative machine. Signature refinement, imported capture
terms, lifetime opening/closing and runtime-only closure call permissions retain
their existing paths. Generic anonymous specialization remains a separate
explicit boundary, including lifetime-parameter owners.

Pinned Rust object controls accept scalar/contextual/reference results and local
mutation/loops. They reject non-const calls, return across the inline boundary,
enclosing-local capture, and required faults in unused closures or closures
nested inside zero repeats. The actual sources and independent observations live
in `obj/parallel/pa25-inline-const/owner-root/qualification.json`.

A focused regression exposed one additional declaration-time consumer:
`local_context` had assigned annotated closure lifetimes to the executable root.
The migration changes that owner together with State, CFG and contract ownership;
it does not suppress or reassociate proof facts after checking. The failed first
gate is retained as `owner-root/first-owner-failure.log`.

The final source hash is
`1b5839591dd3e896437bba6a082579cf1d7918ec6312c5cc5c1741449bd5170f`;
the release binary hash is
`6e565a2ba3c611f6bfeb369d0ef54cd9675607a0e161caafc3841f6da2664b05`.
Qualification passed 65 focused source/inspection/closure tests, 13 constant
regressions, all 12 nested-evaluation tests including ten native programs, strict
release Clippy across all targets/features, the 1,318-file audit, layout and
upstream checks. All 45 declared inputs have independent pinned object outcomes
and recorded CLI outcomes; successful inventories repeat byte-for-byte. Three
remaining generic/lifetime-owner capability boundaries and three uncovered
positive cases explicitly differ from Rust acceptance.

The original uncontextual `|| const { &7u8 }` cases, with and without an enclosing
static return annotation, remain declared uncovered positives. The same is true
of a written `&u8` local annotation inside an inline body. An explicit
`fn() -> &'static u8` closure context passes both source and native checks.
The inferred case fails in `Refinement::outgoing_regions`: its Bound fold cannot
restore a child-owned output lifetime, rather than encountering a malformed
TermId. It shares the captured-closure output transport frontier. No inferred
lifetime is equated with static or an imported region in this checkpoint.

At 8/32/128 direct closure owners, indexed inline entries, preparations, type
publications, value publications, checked inline CFGs and machine frames are
exactly N. Registry probes are 15N, interpreter steps 5N, and retained logical
registry/body/code storage is 304N/2880N/488N bytes. These are charged logical
storage and measured work counters, not allocator heap or total scheduling cost.
Evidence is under `obj/parallel/pa25-inline-const/owner-root/`, including
`final-qualification.json`, `extra-qualification.json`, `work-counts.json`,
`inline-functional.log`, `close-native.log`, `close-lint.log` and
`inferred-direct.trace`. The prior immutable inline performance observations
remain unchanged; the earlier unexplained scalar RSS increase remains unresolved.
No full matrix, assignment activation or reference generation is implied.
