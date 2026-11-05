# Callable associated-family checkpoint

Generic associated families now retain their actual captured declaration
environment through forwarding. Method conformance compares completed rigid
normal forms and proves lifetime choices separately. Complete Self, type, const
and member identities remain required; projected results never imply equality
of independent inputs. The existing five scoped-projection safeguards pass.

Actual callable scope kinds survive inference, normalization, proof copying and
grafting in the existing 12-byte binder row. Callable-local type conditions defer
under RFC1214 without adding caller assumptions. Explicit lifetime relations,
including independent local lifetime ordering, still require proof. The full
PA21 run caught an overly broad deferral of those relations; its original
negative and oracle remain unchanged and now also run in the focused public gate.

At a real call, a complete matching expected result can propose that callee's
fresh late lifetime arguments. This keeps full rigid identity, rejects conflicting
slots and nested-local escapes, and leaves every argument, formation, result and
loan proof in place. Shared subgraphs are visited once per binder depth with the
existing inference budget. Ordinary noncallable family queries take the existing
path without a new conditions allocation. Header completion retains the existing
trait-name graph to avoid opening irrelevant quantified bounds before real
semantic member selection.

The 37 new sources comprise 19 forwarding and 18 callable-formation cases. Pinned
Rust independently verifies every complete-source outcome. Three actual native
forwarders read 17 through shared/inherited borrows and write 29 through a mutable
borrow. Callable-formation fixtures use complete LowIR proof; semantic inspection
alone does not reject borrow errors. Initial generation refused four incorrectly
staged negatives and wrote no references. All nine successful callable-formation
programs have explicit native pass outcomes for their empty main functions.

Current qualification passes 10 exact public tests, strict selected Clippy,
formatting and architecture in 110.24s. The subsequent 56.04s exclusive checkpoint
passes all 801 PA21 text cases, 280 harness tests, upstream provenance and 25
independent native checks, including 15 executable comparisons. The three reviewed
old semantic references lose only a redundant callable-local type-outlives proof
row; original sources and independently accepted outcomes remain unchanged.
References are generated through the existing scoped runner. Private callable
scope-copy/graft controls compile under all-target strict Clippy; their execution
and full successor qualification remain CI work.

Immutable baseline `292c96f2` and candidate `a7f6b568` retain 72 A/A-calibrated paired
ABBA observations and byte-identical common outputs:

| Input | Candidate median | Paired median B/A |
| --- | ---: | ---: |
| 8 GAT methods | 39.75ms | 1.0233 |
| 32 GAT methods | 131.19ms | 0.9874 |
| 128 GAT methods | 478.25ms | 1.0146 |
| 4096 ordinary items | 825.79ms | 0.9397 |

All observations remain, including a 13.67% positive GAT block and A/A ratios
between 0.854 and 1.366. These controls establish no broad speedup or absence of
regression. Failed candidates' executable/metadata artifacts are retired while
their source inputs, diagnostics and receipts remain.

The [qualification record](callable-family-evidence.json) binds the actual gates,
compiler hashes, reference review and observations. AsyncFn implicit callable
evidence, actual future execution/cancellation and general body/policy transport
remain required. No whole assignment after 24 or original core/alloc/std artifact
is complete.
