# Bounded source feature admission

This checkpoint qualifies the source-only preparation `120e712b` on performance
base `46d8b82a013536682a3ec9b87070d60e2a9bd7ec`. It separates a root feature
request from permission to use a particular construct. It does not compile
upstream core. The original unchanged whole-core observation remains byte4062,
line88, on `asm_experimental_arch`; no new whole-core run is claimed here.
Other unsupported root requests still reject.

The prior forty-command source/native qualification belongs to its immutable
46d8 compiler. The focused receipts below qualify this successor separately.
No compiler or test command ran while the integrator's measurement hold was
active. All subsequent probes and checks used the existing resource/storage
controller; failed admission attempts started no compiler processes.

## Requests and actual uses

The private resolver policy recognizes `never_type` and `stmt_expr_attributes`
as two additional capability bits. `asm_experimental_arch` is a recognized
request with no capability bit. Recognizing that request grants neither an
assembly binding nor lowering: the exact source-builtin claim facade still
rejects unsupported asm/global_asm claims before fallback stub expansion.
Ordinary same-named macros retain their ordinary behavior.

Only enabled retained attributes on the actual crate root supply requests.
Nested or generated misplaced feature attributes remain inert, matching Rust's
warning-only treatment; they cannot mutate the root set. A later actual never
or expression-attribute use must still satisfy the root capability. Disabled
cfg/cfg_attr owners supply no request or use permission. Configured attribute
indices must name their actual enabled owner.

The completion validator carries an ephemeral typed parent role. Direct
function, function-pointer and closure never results are stable; nested never
types require the gate. A callable-bound output is not a direct function
result. Stable item, statement and method-receiver roles do not automatically
become expression roles. Statement and tail attributes are stable, whereas
attributes on grouped values, array bounds and const-value blocks need the
expression gate. Loop/while/labeled-loop bodies retain the surrounding
statement-versus-value role. Inner attributes on if branches are rejected even
when the expression gate is enabled.

Actual installed include/expansion edges determine roles. There is no new
include restriction: PA8's common `parse_include` already rejects type,
pattern, trait, impl and foreign-item contexts. Positive ownership controls
use real item/expression includes. Invalid type-include hypotheses and their
original sources remain in the evidence rather than becoming supported cases.

Existing source declaration, intrinsic, sizing, original/template-body,
specialized-call and caller proof duties are unchanged. Feature recognition
creates no trait, implementation, callable or evaluation permission.

## Publication, bounded work and layout

Loading validates its completed configured package once. Final expansion
validates after its worklist completes and before publication; its internal
loader does not separately perform loading-completion validation. These are
different published trees, not repeated scans per include or readiness retry.

Ordinary compilation returns on absent `Package.source_core` before allocating
traversal storage or inspecting any node. Source mode walks each actual Unit
root once, skips disabled subtrees and retains actual parent roles. Inline
modules are not separately replayed. Include templates are provenance headers;
installed copies are visited beneath their actual parents. Macro token bodies
and retired intermediate trees are not additional active trees.

Six private invariants pass, including rejected-use progress, forged owner
indices, ordinary zero-work behavior and exact unique configured-node visits.
Observed work is:

| Axis | Sizes | Active node visits |
|---|---|---|
| Nested module depth | 8 / 32 / 64 | 23 / 47 / 79 |
| Independent item expansions | 8 / 32 / 128 | 53 / 173 / 653 |
| Separate module instances | 8 / 32 / 128 | 68 / 236 / 908 |

Width and shared/distinct include controls also pass at 8/32/128 with exact
unique-node equality and linear bounds. A shared include has one template;
distinct includes have one per source. All 18 frozen scaling sources compile
under pinned Rust 1.98. The original 128-module-depth attempt hits the existing
frontend syntax limit before admission; that failed receipt is retained and
the boundary is asserted, not reported as feature-validator coverage.

Measured `Package` is 240 bytes, `SourceCore` 72, and both `Features` and the
local `Position` one byte. The initial 272-byte expectation came from the older
pre-performance source-binding receipt; the flat-table base had already
reduced it. This change adds no AST, Package, Resolution, SourceCore or other
persistent fields. Local counters charge actual nodes, edges, retained
attributes and template headers, including progress before errors.

## Qualification and preserved corrections

Evidence is under `obj/parallel/pa31/feature-uses-v1/`:

- `declared*.json`, `inputs*` and `independent*` retain 84 pinned Rust source
  observations, commands, hashes and all superseded hypotheses. Corrected
  classifications include stable tails, gated const-block inner attributes,
  invalid type includes and inert nested requests. The original method-receiver
  source requires missing `legacy_receiver`, so its rejection is not evidence
  for an attribute-gate rule. Two original local-binding native hypotheses
  lack source Copy; two later preparation inputs had invalid extern spacing.
  Those failures are retained separately from the corrected native programs.
- `focused-v3` passes six private admission invariants and four grouped
  `feature_requests` tests. The debug CLI build passes. A subsequent test-only
  removal of unnecessary Result wrappers preserves the test bodies and passes
  final strict checking.
- `tail-v2` passes fmt, all-target/all-feature Clippy, three source-builtin
  invariants, four macro_rules and 57 crate_expansion tests, registration, and
  the 1901-file audit with zero findings.
- `native-close`, `native-group-close` and `builtin-close` pass six new paired
  Rust/candidate native programs and seven retained builtin wrong-code native
  controls. The grouped-expression positive needs the actual gate; its
  otherwise-identical ungated source rejects. The independently observed
  generated trailing expression is stable and remains classified accordingly.
  Tool and backend-lock hashes and exact outputs are retained.
- `published-fixture-provenance.json` binds unchanged fixtures/references from
  public 488109dd. All 8362 PA1–24 comparisons pass byte-for-byte. The outer
  prepared script initially interpreted `run_stage`'s success=True as a
  nonzero failure. All 24 per-stage receipts are successful; `text-summary.json`
  records this bookkeeping correction without rerunning the compiler or
  regenerating references.

The immutable debug candidate is `candidate/rustgm`, SHA-256
`bc1d9f7396381870f052282298f0cbe8f81435454012478dfbbc77e823379881`.
Its source map and backend hashes are in `candidate/provenance.json` and the
native summaries. This is behavior/diagnostic evidence, with no timing claim.
Full combined release qualification and publication remain integrator work.

Actual source Copy/Clone, callable/Tuple declarations and const callable proof,
prelude/internal-macro bindings, generalized sizing debts and richer source
feature contracts remain required upstream-core prerequisites. The existing
source layout contract does not gain direct runtime intrinsic lowering here;
public upstream wrappers must use their real associated constants. Remaining
PA26/PA30 requirements stay required. Ordinary library bodies are to come from
upstream sources, not further convenience-method factories.
