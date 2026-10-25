# Unnamed free constants and visibility diagnostics

This is an unqualified compiler candidate composed over public `0343cf76`.
The verified original-core frontier is `library/core/src/mem/alignment.rs`,
physical file 55, byte 981: its second `const _` was incorrectly published into
the value namespace. Observer CI40215218089 verifies all 722 upstream files,
opens 56 and produces no artifact. The original source hash is
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

The compiler candidate selects six private groups: namespace identities and
inactive syntax, retained typing/CTFE and actual exports, visibility levels and
mixed failures, source authentication/budgets/costs, the existing budget-note
control, and the new unavailable-origin retention control. Exact CLI warning
checks stay bounded. Seven new PA6 and one PA8 references are pending; the
integrator owns their selected generation after code review/pass. Existing
references and receipt history stay unchanged.

Policy gains one byte, from 10 to 11, in the existing optional dense node-policy
table. This adds one byte per retained policy node whenever a known lint activates
that table, not merely per public unnamed constant. Actual capacities, aggregate
payload, Effects/cause/ErrorData/enclosing layouts and width 1/8/32 initializer
work will be printed by the selected controls; they are unmeasured until the
candidate gate executes. The no-policy path retains no table. No broad packing
rewrite, allocator hook or persistent counter is introduced.
