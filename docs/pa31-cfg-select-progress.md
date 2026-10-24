# Actual source `cfg_select` checkpoint

The locally qualified compiler now passes the original core `cfg_select`
declaration at `macros/mod.rs:234–235`. Its fresh unchanged-core observation
stops at `format_args`'s `rustc_builtin_macro` attribute on line 994, byte 35014.
The compiler returns 1 with an explicit unsupported-builtin diagnostic. All four
opened original files match the pinned Rust source provenance. This is concrete
loading progress, not a successful core crate build.

## Exact source and integration boundary

Qualified source is `d8ed72868bb73ccd8fd481720ed0f1f5cdb64144` on
`pa31-cfg-selection`. The accepted scoped-declaration source is
`f2b8a4936adbea4f147d57978333ebf7e80aaf71`; its docs-only successor
`ad64923fac6e799cb64f51ad05f39275807d4c64` is the minimal patch base.
Do not merge the entire historical worker branch.

The implementation commits are `ff2d9d10`, `63622c5f`, `0af5dede`,
`9f1a005e`, and `ac5aaebb`. The shared ordinary-Clone dependency and its final
receipt binding are `56e06952` and `d8ed7286`. The production role-header file
is byte-identical to the independently qualified `98e599c7`; the three corrected
controls and their fixtures are byte-identical to `6c481b2d`. The initial generic
const-use opponent from `98e599c7` is deliberately excluded. The checked-in
[shared dependency contract](../pa31/tests/spec/source-functions-transport/ordinary-clone-shared-contract.json)
binds the final candidate and independent positive/concrete const-use negative.

Root already owns the 61 frozen `cfg_select` inputs and their contracts through
its fixture commits. They need no duplicate import or oracle execution. The
worker copied those exact commits, preserving all original disagreements.

The compact handoff is `obj/parallel/pa31/cfg-select-handoff-v1/handoff.json`,
SHA-256 `2456c64283df6c034de74e4faba58ca0140822ab925575134ee6c4753f596857`.
It binds every source and qualification receipt. Its recommended patches are:

- `cfg-select.patch`, 24 compiler/test files, SHA-256
  `fbae4cf28284fa27ef7e2805b7bd496f1ba31098c6b31257ca9803d42e723c50`.
- `shared-clone-dependency.patch`, only the missing exact shared dependency,
  SHA-256 `c8e911972df42debcebc5e9be07f5427e6ec15a8b7551c552e1229742e093277`.

The combined `compiler-tests.patch` and fixture-only patch are retained for
review, but the separated patches avoid duplicating an already integrated A
fix. The four shared fixture files are `minimal-provider.rs`,
`ordinary-clone-concrete-provider.rs`, `ordinary-clone-concrete-const-use.rs`,
and `ordinary-clone-shared-contract.json` under
`pa31/tests/spec/source-functions-transport/`.

## Qualified behavior and authority

The actual authenticated declaration supplies `Builtin::CfgSelect` on either
scoped or legacy macro syntax. Ordinary same-spelling macros keep their authored
behavior. The canonical macro index, real declaration node/module/scope,
published header `Rc`, actual defining edition, and source-provider identity
must agree. An exhausted global macro-contract budget rejects warmed lookups.
The diagnostic identity is declaration data and cannot grant execution.

All predicates and branch attributes are checked, including after a matching
or wildcard branch. Every RHS is parsed in the actual invocation grammar;
inactive names and macros are not semantically evaluated. Only selected syntax
is configured and atomically installed at base zero in its existing source
image. The actual registry inputs, token origins, editions and payload identities
are reused. Failed grammar, payment and target checks leave the unit unchanged.

Unbraced RHS framing reuses its expression parse in expression context. Other
output categories need a separate grammar check, which is counted. Direct
brace-delimited statement tails reuse the existing Expansion tail consumer.
Parenthesized/forced expressions and imported opaque expressions retain their
expression category, including at an outer semicolon. No synthetic unit, source
reparse, invented opaque payload, definition permission mark or empty executable
stub is introduced. General scoped invocation remains unsupported.

## Evidence

`obj/parallel/pa31/cfg-select-source-v4` has fresh format, 2,323-file audit,
all-target/all-feature strict Clippy, 183 harness tests and upstream checks,
then one debug build. All **1,449 nonignored private cases**, **12 affected
frontend functions**, and **51 native programs** passed. The native count is
39 unchanged independently positive `cfg_select` cases plus 12 retained scoped
controls. Nine new selection/installation/authority/layout controls and three
final shared Clone controls are explicitly required in the private inventory.
The frozen source corpus remains 206 retained observations, including 41
independent native programs; no oracle was rerun for this recovery.

The 61 `cfg_select` sources classify as 39 actual candidate native positives,
18 intended language-negative controls, one original staged provider whose
builtin expands but retains its separate final stability frontier, and three
original incomplete-provider hypotheses retained with their earlier upstream
missing-Copy disagreement. Those three are not counted as candidate acceptance
or as negative coverage of `cfg_select`. The separately declared complete
providers pass unchanged after the qualified ordinary-Clone dependency.

The C4 CLI SHA-256 is
`0836f6af2d4ad8dfb58d88475d486bca99d223ba614e22627587ee7cbf31f33d`.
Its guard token is `6eb33e5a17ca4ca79039099235ec8604`, terminal 0/error null;
cleanup `1792862648962910956` removed 102 disposable outputs. The success-required
core observation is `obj/parallel/pa31/cfg-select-whole-core-v2`, guard
`bb4b0802cbb04883ba6f4efceddcb966`, wrapper 0/compiler 1/noninfrastructure;
cleanup `1792862733614681666` removed zero files.

V1 type/borrow wiring failure, V2 four style failures, and V3's exact two frontend
failures remain intact. V3 passed 1,446 private cases but had no native phase.
Its separate provisional core observation is retained and superseded by the
qualified compiler's observation, without changing the earlier result.

## Work, storage and remaining checks

Actual branch widths 8/32/128 produce parser calls 8/32/128, paid work
100/412/1660, and storage preflights 29,584/121,840/490,864 bytes. Each retains one
selected expression node. These are owned parser/work counters and conservative
allocation preflights, not heap/RSS observations. Actual full-source width and
selected-depth cases also compile and execute through the C ABI.

Measured cold `Claim` stays 2 bytes. `Reservations` grows from 600 to 616 bytes
for its fourth diagnostic slot; baseline and candidate DWARF are hash-bound.
Actual Rust sizes remain Body 96, Definition 136, Package 256 and SourceCore 176.
GDB's enum-only Body query reports zero on both binaries and is explicitly not
an actual Rust layout measurement. There are no new hot record fields or opcodes.

Combined release, all active PA text cases (including PA8 statement-category
regressions), paired ordinary-path performance and candidate-only source scaling
remain integrator requirements. Existing width 8/32/128 and selected depth
8/16/32 fixtures are independently and natively qualified performance inputs;
no timing claim is made here. Scoped execution, foreign macro transport and
full source stability remain open. LIB-017 now requires actual formatting
builtin/diagnostic/allowance identities, typed formatting syntax, real source
`fmt::Arguments`/argument construction and their normal call/borrow obligations;
recognizing its declaration alone would be an intermediate milestone.
