# Unnamed free constants and visibility diagnostics

Source `72cf2304` is qualified by six focused controls, the reviewed eight new
text oracles and all 8,485 existing text cases. The preceding original-core
frontier was `library/core/src/mem/alignment.rs`, physical file 55, byte 981:
its second `const _` was incorrectly published into the value namespace.
Observer CI40215218089 verified all 722 upstream files and opened 56. That source
file's hash is
`4eb6651741fc8d8d84ce274219cc72399949aa9eba2a4592a3f90d9293d1b587`;
no upstream input is modified.

The shared item indexer retains real `DefKind::Const` identities, canonical
item keys and distinct initializer scopes while omitting only a free unnamed
constant's lookup binding. The expansion publication pass retains introductions
and scope ownership and emits no declaration-name wakeup for that constant.
Named constants, associated named constants, privacy, wildcard patterns and
anonymous imports preserve their existing paths. Associated unnamed constants
are rejected from the actual associated owner. The PA5 parsed underscore span
is unchanged; PA6 owns the new namespace controls and PA8 the generated copies.
PA25 retains ordinary type/CTFE checks and PA31 the actual alignment reducer.
The unchanged valid-Rust alignment reducer currently reaches the unsupported
`assert!` macro at byte 129. Its exact unresolved-macro failure is retained as an
explicit prerequisite; this slice does not implement assertion expansion or
claim successful typing/evaluation of that reducer.

All initializers remain in ordinary declaration/body/constant work. Invalid
unused initializers, including in unused functions and expanded macros, must
still fail. Disabled owners and unexecuted macros create no declaration,
visibility cause or initializer demand. A retained public placeholder is not
exported through the namespace-based scalar exporter. The source header profile
continues rejecting unsupported constant inventory; there is no wire expansion
or access rewrite to hide a declaration.

Public or restricted visibility on a free unnamed constant warns under the
actual `unused_visibilities` lint. The existing owner-produced lexical policy
handles allow/warn/deny/forbid and weakening. A rare typed cause is retained in
the existing optional expansion Effects owner after the resolver allocated the
real definition. It authenticates its defining image, canonical item key,
owner/scope/module, complete item snapshot and token context. One shared
consumer formats warnings for both semantic success and terminal expansion or
resolution errors. It uses the existing canonical warning sink and Error carrier;
no library I/O, second lint parser, whole-header error scan or hot node field is
added. Diagnostic identity uses physical source plus actual syntax context, so
a copied captured token warns once while two literal invocations remain distinct.

The one-byte private Error retention state distinguishes Complete, Budget and
UnavailableOrigin. Existing budget note bytes and the primary error are
preserved. An origin authentication failure receives its own bounded note, and
later merges cannot replace that cause with a budget label.

Three frozen independent metadata batches run 76 observations across editions
2021 and 2024: CI40216397961 (38), CI40218068532 (28), CI40219038737 (10), all with
zero infrastructure failures. All status predictions match. Six warning-count
hypotheses were wrong and remain retained: public placeholders warn, private-name
errors retain that warning, and a duplicated captured item warns once. The
committed contract corrects those predictions while preserving the original
plans and raw diagnostics. These observations are not runtime qualification.

The compiler selects six private groups: namespace identities and
inactive syntax, retained typing/CTFE and actual exports, visibility levels and
mixed failures, source authentication/budgets/costs, the existing budget-note
control, and the new unavailable-origin retention control. All six pass in
[CI40223575234](https://github.com/vishvananda/rustgm/actions/runs/40223575234),
with formatting, global architecture audit, strict private Clippy and the
unoptimized build. The helper takes 163.40 seconds. Its receipt SHA-256 is
`e5522a088e26d036c2d53337649c15bb186551a84931ed544353ee93400ff7a4`.
The qualified CLI is
`123138617b80b8bdf39ad7c9bce4ec0df48618e6e649efbadf3755e983ac817c`.

The integrator authenticated that receipt and all current source fingerprints,
then ran the unchanged prebuilt CLI: all 8,485 existing text cases match exactly,
and eight new outcomes agree with their independent contracts. Manual review
checks actual constant owners, scopes, bindings and macro nodes before official
scoped generation of seven PA6 and one PA8 references. Text execution takes
54.32 seconds; generation, byte comparison and 215 fresh harness tests plus
upstream checks take 14.07 seconds. There is no local compiler build, reference
churn outside those eight cases or repeated full-private/native sweep. Evidence
is under `obj/parallel/unnamed-constant-local-review-v1/`.

The earlier Clippy limit, cold-build timeout and three initial test failures
remain recorded. The successful repair restores authenticated macro introductions,
checks all three real associated constant owners and retains the unsupported
alignment assertion prerequisite. The focused CI workflow's later harness step
fails solely because these root-owned references were pending at its frozen tree;
the integrator's fresh harness run after generation passes.

Policy gains one byte, from 10 to 11, in the existing optional dense node-policy
table. This adds one byte per retained policy node whenever a known lint activates
that table, not merely per public unnamed constant. Actual capacities, aggregate
payload, Effects/cause/ErrorData/enclosing layouts and width 1/8/32 initializer
work were printed by the partially failing CI40222311370. The one-lint control
retains 7 policy nodes, capacity 7: 77 payload bytes versus the previous 70, an
aggregate increase of 7 bytes; the outer optional table is 24 bytes. Effects is
368 bytes, Causes 24, each Record 120, ErrorData 128 and RetentionNote 1. Report
is 3040 bytes, Engine 4088, Node 36, NodeFacts 80, Scope 32 and resolution
Definition 36. Macro Body/Definition remain 96/136. Widths 1/8/32 retain the same
number of real initializers, visits and validations as their width, with 34/41/65
definitions and one name binding. These are layout/work observations, not an
allocation count or performance qualification. Three of six groups passed;
follow-up repairs authenticate warning introductions and correct the three
source-associated-owner assertion while preserving the explicit assert gap. The
successful repair confirms these same costs. The no-policy path retains no table.
No broad packing rewrite, allocator hook or persistent counter is introduced.

The fresh unchanged-core observer
[CI40224012971](https://github.com/vishvananda/rustgm/actions/runs/40224012971)
uses the qualified CLI, verifies all 722 original inputs and opens 58 files in
6.075 seconds with no infrastructure failure. The next error is
`ERROR: file 57: 63133: expected punctuation`, physically
`library/core/src/mem/maybe_uninit.rs:1644`, at `default fn spec_fill`.
That file is 63,963 bytes, SHA-256
`4189eaa95d41706cd506ef9dfdb81478640a53ccacc4d88f96329ad3a27e90fb`.
The result hash is
`ccfa0777d16846e5645a3242262f233e9c179c47b7bbbd2ec53f2e0cd60406c8`;
the opened-source map hash is
`6d6b9377d7734e40757dbf1aa798a93c0d4b9136e9fbaab2241ebbf107844bfb`.
No core artifact or assignment release is claimed. Specialization defaultness
and the separate assertion-expansion prerequisite remain required.
