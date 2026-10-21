# Projected closure signatures and specialized output proofs

PA24 remains planned. Written projected inputs and nested quantified function
pointers now retain the closure body's actual lifetime scope. This checkpoint
adds no frontend delegation, code generation rule, proof permission or text oracle.

## Scope and proof changes

The closure execution signature describes the current body binder. Its parameter
and result roots are normalized in that existing scope; a function type genuinely
nested inside a root still opens its own binder. Previously normalizing the whole
body signature opened an extra binder, so a projection proof could accidentally
compare an outer lifetime with a fresh nested universal and require it to be
static. Both initial and resumed closure checking use the same root traversal.

Conditional output solving exports only the ordinary scalar interface debts.
Projected verification alternatives remain checks on the complete proof after
those exact debts have been assumed. The parent must discharge every exported
pair. A disjunction cannot invent a new interface assumption, and a conjunction
with an unproved branch still fails. Membership output inference remains an
explicit unsupported operation.

Specialization now uses the existing formation reproof for noncapturing closure
contracts as well as captured ones. It retains every original obligation and
projection proof, requires input/result formation, and proves the specialized
body independently of the original parent link. This introduces no additional
assumptions for the noncapturing path. Captured source execution conditions
continue to require their existing checked-source protocol.

## Qualification

The independently declared original `written-input` and `nested-binder` fixtures
are unchanged. `two-nested` adds two separate quantified callable binders under
one projected capture. `nested-input-escape` now reaches the actual unentailed
lifetime rejection instead of an implementation boundary. Opposing borrowed,
distinct-clause and distinct-output controls remain in the same owning family.

Private graph controls test scalar debts plus Any/All verification and prohibit
verification-only assumption invention. A source-derived specialization control
injects an unpaid universal-to-static debt and requires the complete proof to
reject it. The final focused checks pass: the projected family has 12 source controls
(eight accepted, four rejected) and all eight accepted native programs pass.
The 604 other private tests passed in the first full private run; after correcting
the setup of the new adversarial test, that test passes as well. Formatting,
strict all-target/all-feature Clippy, file audit (1,475 files, no violations),
layout and upstream checks pass. Integrated equality and reference-object source
and native controls also pass, as do all PA1–PA23 text suites without changes.

Written-input work is 8,049 / 31,257 / 124,089 at widths 8 / 32 / 128;
nested-binder work is 10,180 / 36,604 / 142,300. Both grow affinely with width.
All six independently declared scaling programs compile to Rust objects and
execute successfully with both Rust and CPPGM. These are functional and work
scaling results, not isolated wall-time performance claims. The combined release
and immutable-binary performance gates remain separate publication requirements.

Development evidence is retained under
`obj/parallel/pa24/conditional-projections/`. The first attempt exposed the
incorrect extra body binder; the second reached the separate specialization
boundary; the third passed all three public views. The fourth batch passed
strict Clippy and caught a crowded test filename in the architecture audit.
The test was renamed by its obligation without an audit exception.

The fifth batch exposed an incorrect test setup: an explicitly typed closure has
no inferred output port. The corrected adversarial test deliberately injects its
parent bridge and states that fact. The sixth batch caught a zero-test filter for
the separately integrated reference-object family; the corrected seventh batch
ran both source and native groups. All failed receipts remain available.

The six scaling controls completed before a later, unrelated PA25 inventory
script failed by treating a library input as a program. Their completed receipt
is retained in `scaling/complete.json`; that inventory failure does not change the
source or expected outcomes of this checkpoint.
