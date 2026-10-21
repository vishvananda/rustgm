# PA26 option_env library value checkpoint

PA26 remains planned. This checkpoint preserves option_env's library identity
through expansion, semantic analysis and ordinary aggregate lowering. Draft
outcomes are declared in `pa26/tests/environment-outcomes.json`.

## Contract

`Builtin::OptionEnv` expands eager arguments once and requests the existing
explicit environment provider. Its one string argument is validated before a
provider request. Missing values become None; present Unicode values preserve
all decoded bytes. Provider errors remain errors. Repeated env/option_env uses
share the loader's existing per-compilation snapshot, and disabled syntax or
discarded macro input makes no provider request.

`LibraryValue::EnvironmentOption` is a neutral support identity, carried by
`syntax::ast::Kind::LibraryValue` with one optional literal child. It carries no
semantic type or DefId. Macro source images remain diagnostic images: the
absent-value image reserves a nonempty origin range and is never reparsed.
Transfer remaps child identities directly; child walkers, serialization,
temporary scopes and lifetime extension retain the ordinary aggregate behavior.
Macro ancestry and physical source mapping remain in existing expansion facts.

The resolver registers the library macro on demand, preserving earlier builtin
DefId order. Aliases, qualified paths, prelude controls and macro shadowing use
ordinary namespace lookup. Its canonical metadata identity does not come from a
user item with the same name. The neutral value demands canonical Option
metadata even if a caller shadows Option, Some or None.

Semantic analysis selects the actual None/Some variant and fixes the result to
`Option<&'static str>`; absence does not leave the element type unconstrained.
Selected `Operation::Construct` facts feed ordinary `Rvalue::Aggregate` lowering.
The constant query fast path consumes the selected variant and optional field
through the ordinary constant product arena, retaining the selected field count.
There is no separate runtime Option implementation or generated source path.
Expression and statement contexts are admitted. Pattern, type and item contexts
are rejected; the pinned compiler independently rejects option_env in patterns.

## Qualification

All 15 declared source outcomes agree with pinned rustc object generation.
The seven focused tests pass, including four option_env native comparisons,
four nominal-library native comparisons, provider snapshot/error controls,
constant Some/None, statement and pattern-context controls. Strict all-target,
all-feature Clippy passes. The syntax-aware audit reports 1,164 files and zero
findings; layout checks, 24 upstream fixtures and 126 pinned Reference files
pass. Three earlier PA8 expansion views retain every reference byte.

The guarded command, logs, fixture hashes and independent outcomes are retained
under `obj/parallel/pa26/environment/` in `gates.json`, `gate-*.log`,
`qualification.json` and `earlier-text.json`. The first two final-gate attempts
stopped on Clippy match-style findings, corrected before this successful run.
No generated references or assignment activation are included.

## Remaining PA26 work

- Mutable indexing, assignment/comparison/dereference hooks and primitive
  operator library implementations beyond the operator checkpoint.
- IntoIterator/Iterator selection and for-loop control flow.
- Question-mark residual selection, error conversion and early-return cleanup.
- Additional built-in derives and formatting/runtime macro contracts.
- Complete method surfaces for the admitted library nominal types, including
  ManuallyDrop dereference, extraction and explicit destruction operations.
- Full cumulative release/native/text qualification and controlled performance
  measurements for a PA26 release, followed by reviewed reference publication.

These remain PA26 responsibilities even when a separately accepted PA30 ABI
slice begins. This checkpoint does not mark PA26 complete.
