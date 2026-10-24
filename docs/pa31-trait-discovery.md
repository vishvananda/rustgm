# Trait method identity and scope discovery

Trait candidate discovery now carries the actual method identifier through
receiver calls and implicit or explicit associated paths. It indexes every
trait binding in a visited scope, including opaque trait names. Member names
have their own normalized contexts; filtering a trait by its binding name's
context would lose valid captured methods.

Non-root member partitions are separate from ordinary name tables. A contextual
query visits only its own identifier ancestry before copying candidates. Scope
memoization retains both the original method context and the context currently
walking scopes. Candidate reuse does not authorize access: actual trait-owner
adjustment and caller-dependent privacy are checked before applicability.
Parameter-bound priority, ordinary prelude behavior and complete applicability
proof remain unchanged.

## Actual scope boundaries

The parser's retained statement category identifies item and statement-macro
blocks before expansion can remove their contents. Those blocks and ordinary
modules are trait-lookup containers; value-only and binder scopes do not cause
definition-site rewinding. A contextual walk visits the initial container,
then follows the actual invocation ancestry or the macro's definition scope.
Prelude selection follows the reached terminal module.

Pinned rustc's `rustc_resolve::traits_in_scope`,
`ident::hygienic_lexical_parent` and
`build_reduced_graph::block_needs_anonymous_module` provide the source evidence.
`rustc_parse::parser::stmt::parse_stmt_mac` distinguishes brace/semicolon
statement macros from parenthesized expression tails.

The eleven [independent scope cases](../pa31/tests/spec/declarative-macros/method-scopes/README.md)
have 22 metadata/object observations and six native programs. Two initial
rejection hypotheses were falsified: parenthesized tails remain expressions.
Their original sources, contracts and failures are retained. A separately
declared continuation reuses those four language observations, executes the two
accepted programs and checks distinct brace/semicolon opponents. The evidence
contains 34 actual observations, with no duplicate language compilation in that
continuation. Even an empty statement-macro expansion retains its scope boundary.

## Qualification and costs

Qualified source is `956c9904fa51daabba3cac666793c72488975bf3`, integrated locally
as `708ee191`. All compiler files are byte-identical across those commits.
`obj/parallel/trait-discovery-static-v3/` in the root implementation worktree
passes format, the 2,327-file audit and all-target/all-feature strict Clippy.
The preceding filename-stem audit rejection and wildcard-import Clippy rejection
are retained; neither caused a lint allowance or weakened check.

`obj/parallel/trait-discovery-runtime-v1/` passes:

- All 1,460 nonignored private cases, including five new discovery controls.
- 202 affected integration cases and 20 native test functions.
- All 8,382 text cases across PA1–PA24.
- All 183 harness tests, maintainer registration and upstream provenance checks.

The complete receipt SHA-256 is
`15c5f816712226aa570729d6401bfae5d9af266e4cdf8b48c52414f5c0af820c`;
the checks SHA-256 is
`5400a681377e54b74c2ba956d0973fd9e7b14c0814c86588123c343aa284b141`.
The CLI SHA-256 is
`0e364a2129bf0b58da2509455bf28a8d7f1b6f307b5b70d5d7ef96e59a80960f`.
The snapshot binds 38,671 assignment files separately from compiler/tool source;
fixture hashes are verified before and after the batch. The 42 recorded stages
took 375.50 seconds, including 167.12 seconds for the debug build and 53.14 for
text tests. These are qualification durations on the shared machine.

Sibling-context widths 8/32/128 consume 104/416/1,664 charged steps, returning
only the matching partition. These are structural algorithm controls, not
end-to-end scoped-source or timing measurements. Ordinary `Methods`,
`Declarations` and `Scope` remain 312/7,064/32 bytes. The cold `ItemScopes` owner
grows from 56 to 88 bytes for its block-boundary set, plus storage proportional
to those boundaries. Combined performance qualification must account for that
cost and the original block-category scan.

Guard `caebd4b2a477452d8ffee63cf4406ec4` finished with status 0/error null under
an 8-GiB absolute cap and unchanged 150-GiB reserve. Cleanup
`1792865584684258506` removed 778 generated files, 1.42 GiB of unshared blocks.
After diagnosing the independent oracle disagreements, scoped cleanup
`1792865674514821416` removed 16 outputs, 0.02 GiB, preserving evidence.

## Remaining source support

The private controls use actual resolved legacy macro invocations with supplied
opaque query contexts. General scoped macro invocation remains disabled; these
controls do not claim that the complete scoped source programs compile yet.
Implementation-member conformance must adjust each actual impl identifier to
its actual trait container, map to a canonical trait member and reject duplicate
mappings. Associated types/constants, final privacy checks and foreign macro
transport remain required. Combined release/performance qualification, a core
artifact and assignment activation remain separate gates.
