# Source-owned builtin macro claims

The first source-provider feature-admission probe found executable wrong code:
`#[rustc_builtin_macro] macro_rules! line { () => { 99u32 } }` produced the
stub value 99. Pinned Rust returned the actual call-site line 6. The explicit
builtin-name, raw-name and enabled cfg_attr variants had the same mismatch.
The original bytes, Rust/native results and failed candidate observations are
retained; successful metadata inspection alone did not establish semantics.

This repair admits only configured source-owned claims for the thirteen existing
`expand::builtin::Builtin` contracts. The private resolver facade requires the
actual enabled attribute owner, explicit source-core provider and rustc_attrs
gate, a real MacroRules declaration, and checked bare/explicit argument syntax.
Its output is the existing typed Builtin variant. Unsupported active claims,
wrong owners, missing gates, duplicates and malformed arguments fail before any
stub can execute. No generic attribute whitelist or new feature gate is enabled.

The ordinary macro-rule compiler still validates the real stub. Independent Rust
checks reject empty/malformed/invalid-fragment rule bodies, but accept valid
nonmatching rules because invocation uses the registered builtin. Expansion
stores Body::Builtin in the ordinary Definition with its actual Some(NodeKey).
Paths publishes the normal real DefId; textual scopes, lexical shadowing,
imports, exports, source location and configured/generated owners use existing
machinery. The synthetic builtin_definition path is not used for source claims.
The names pipeline uses the same facade and requires the actual Macro DefKind
and declaration node before completing source reservations.

No persistent record/enum fields were added. Existing measured sizes remain
Package272, Resolution2160, SourceCore72 and Reservations240. The private claim
kind is temporary. There is no new persistent cache or source-independent
permission; retained configured attributes are inspected by the two resolver
consumers, with constant thirteen-way contract matching only on actual claims.
Source layout/callable proof and intrinsic CTFE/runtime boundaries are unchanged.

## Evidence

All paths below are relative to `obj/parallel/pa31/feature-admission-v1` in the
worker checkout. `evidence-hashes.json` records the manifest/result checksums.

- `declared.json` / `independent/results.json`: 32 initial feature/attribute
  hypotheses, all independently matched pinned Rust outcomes. They retain
  unsupported valid gates/uses and invalid accepted cases separately.
- `declared-v2.json` / `independent-v2/results.json`: nine original source cases,
  eight Rust native positives and one forged-name rejection; four baseline
  native mismatches (99 instead of 6), three correct ordinary/shadow controls,
  and one separate exported-macro metadata boundary.
- `declared-grammar.json` / `grammar-independent/results.json`: nineteen syntax
  hypotheses. Four initial predictions were superseded honestly: three assumed
  Rust would ignore invalid stub rules, and the assert invocation lacked actual
  no_core panicking/Not prerequisites. Fifteen predictions matched unchanged.
- `first/results.json`: eight focused resolver/provider unit tests and one debug
  CLI build passed.
- `closing/results.json`: seven candidate native programs now pass, including
  the four unchanged wrong-code reproducers, and all nineteen corrected grammar
  classifications match. Strict stopped on one redundant error-map closure.
  The equivalent borrowed closure reference and removal of an unnecessary test Result return are the only subsequent code edits.
- `candidate/provenance.json`: immutable debug binary SHA-256
  `461dbe121530ffb87ab3fb6cd2ffbd07024f734625e4f4057f0a2d736a7f4101`,
  before that lint-only edit. No performance or final-source binary claim.

## Required boundaries retained

Nonempty builtin helper-attribute lists are valid Rust and remain an explicit
unsupported contract; an empty list works. Other actual builtin families such
as asm, format_args and derives need their owning implementations rather than
stub execution. The unchanged exported-alias native source passes Rust but its
candidate LowIR boundary is `exposed declaration is outside the scalar metadata
profile`; this is a PA29 prerequisite, not failed macro lookup.

The original whole-core source still first fails on the requested root feature
at byte4062/line88. No additional core modules have been compiled by this repair.
The source-admission census separately distinguishes requested gates, configured
inactive consumers, supported constructs and unsupported active uses. Missing
never-type/expression-attribute gates and misplaced nested feature requests
remain required admission corrections. Actual source Copy/callable/Tuple,
richer sizing/layout proof transport and unsupported source builtin families
remain prerequisites; ordinary upstream APIs must come from upstream source.

## Closing qualification

`tail-v2/results.json` records strict all-target/all-feature Clippy, all four
macro_rules tests and all 57 crate_expansion tests passing. The first tail
retains its test-only unnecessary-Result lint failure. The second audit stopped
only on repeated filename prefix source (eight files). The physical files were
renamed to claimed_macros.rs / claimed_macro_bindings.rs with explicit module
paths; no semantic change or repeated native/CLI compilation was needed.
`audit-close/results.json` records formatting, syntax-aware audit (1,854 files,
zero findings), layout and exact grouped inventory checks passing. No full
worker release, active-text regeneration or performance measurement is claimed;
those gates belong to the combined integrator cut.
