# Canonical lifetime binders in local aliases

The local clause consumer now expands nested quantified aliases through the
existing structural substitution and universal-binder machinery. This extends
the [canonical clause checkpoint](trait-alias-clauses-checkpoint.md). Complete
alias semantic consumers, imported transport and the original core artifact
remain open; the declaration pipeline's alias execution/export fence stays.

Each expansion application retains one canonical surrounding binder, including
unused slots. Its key includes actual alias, subject, complete arguments and
surrounding arity. Entering another quantified edge substitutes below that
edge's binder, then merges real subject/argument roots into the canonical
surrounding scope. No ancestor binder path or inherited clause list is copied.
Clause publication composes the remaining local quantifier once. The visited
set is query-local and contains no proof answers or cross-Store identities.

`Quantifier` now shares its validated opening with typed input roots as well as
clauses. Parent slots are checked before inner slots can collide with them;
ordinary and empty binders retain capture depth, and nested callable locals
remain local. The same existing occurrence and substitution walkers provide
charged entrypoints. The quantified-clause interner prepays actual cold storage
and collection growth; failed validation and failed payment retain attempted
work before propagating errors. No persistent Store, Facts, checker or node
field is added.

Three genuine-source controls check nested aliases through three surrounding
scopes, unused formals, associated equalities and captures inside function
pointers; the real function-bound worklist adds its own fourth scope. Identical
inputs in quantified and unquantified contexts stay distinct. Repeated shared
graphs exercise widths8/32/128 without flattening. A typed input control checks
invalid parent slots and rejected allocation before opening publication. The
five existing alias groups, ordinary captured-owner conformance, four existing
composition controls and counted-formation failure rollback remain selected.

[Run40268504525](https://github.com/vishvananda/rustgm/actions/runs/40268504525)
passes all17 exact tests, strict production/private Clippy, formatting, global
architecture, harness, upstream and audit-cache checks. The helper takes177.44s;
the actual tests take0.03s. All2,712 source fingerprints match before/after and
the imported host formatting. Receipt SHA is
`0f00173e6a0eac051fc856e0ef23c081f807bfcaf4c86df5b417ab169516e33e`;
binding SHA is
`59203b78a286fba741efe7adc5f6e6ada39a3a385b5fe627c1053c84a87c49f5`.
No native/reference matrix, release build or full private/text sweep was run.

| Quantified shared width | Counted work | Cumulative logical charged bytes |
| --- | --- | --- |
| 8 | 610 | 29,181 |
| 32 | 2,392 | 121,205 |
| 128 | 9,448 | 480,853 |

These are logical work/storage charges, not physical heap or wall-time speed.
Unquantified widths retain work394/1,574/6,246; their cumulative charges become
13,285/55,037/217,821 bytes because the exact surrounding context is retained
in frames/keys. Ordinary programs still allocate no alias owner. Existing
composition tails64/128/256/512 take385/769/1,537/3,073 steps, with validation
128/256/512/1,024 and one new canonical type per tail element.

Alias lifetime-to-lifetime schema relations, complete original/normalized
projection consumers, object principals, source sizing/default well-formedness,
const qualification and genuine imported contracts remain required. No opened
core file is counted as a completed compilation and no PA is released here.
