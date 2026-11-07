# Analysis-only core census mode

Lane B of the [core census plan](../core-census-plan.md). A census reports every
failure it can isolate in the reached loops instead of only the first one. It is
diagnostic evidence, never a qualification: it always exits nonzero, writes no
view output, LowIR, artifact or metadata, and never counts as a passing build.

## Design

**Isolated loops.** Only loops that push one root work item onto an empty
declaration worklist and drain it are isolated. These are nominal headers, inherent
impl headers, trait impl headers and the per-definition loop in `Checker::run`. In
bodies, each function is isolated, in both the plain loop and the hidden-type
scheduler, and so is each initializer. Every other preparation and validation
pass (trait owners, associated types, destructors, early implementations,
deferred defaults, fields and sizes, local annotations, layouts and API exposure),
constant patterns and the pipeline completion passes still stop at their first
error. Those passes are multi-stage crate aggregations: one missing owner
invalidates the whole stage. Isolating them is not cheap and not known to be safe.

**Failed state.** Failure leaves in-flight state in place. Every `Active` marker,
including declaration states, impl-header active sets and nominal field and
contract states, keeps its existing re-entry guard, so a failed unit can never be
restarted or completed. The census poisons, all against the root-cause record:

- every definition activated by the failed drain that is still `Active`;
- the unit of the work item whose dispatch failed;
- the owner unit of each continuation (`finish`, or a non-`Begin` phase) left on
  the stack.

Not-started siblings stay unpoisoned. Poisoned definitions fail at the top of
`definition()`. A later root whose failing work item names a poisoned unit is
recorded once as `blocked` by that record and is not rechecked. So is a later
failure with the same location and message as an earlier record. The failed item
is the innermost failing unit: the owner of the failing work item, else the newest
still-active definition. When it differs from the loop root, the root is recorded
as blocked by it. A final check refuses a poisoned definition that became
`Complete`. In bodies, a hidden-type dependency on a failed function blocks the
waiting function and its whole suspended chain. A failed function is removed from
`typed_function_bodies` and its const completion is skipped.

**Leftover state.** The shared work stack and child scratch are cleared. Binders
and pending maps belong to poisoned owners. A failed body replaces the body
checker's per-body `State` with a freshly constructed one.

**Budgets stay sticky.** The declaration step budget, and any error whose message
uses the shared budget or capacity vocabulary (`budget`, `exhausted`,
`limit exceeded`, `capacity exceeded`, `exceeds ... limit`, `too many`), ends the
census with a `budget` record. A failure outside an isolated drain ends it with a
`crate` record, or `blocked` if its failing unit is poisoned. Limits, charges and
validations are unchanged. A local budget that is misclassified only stops the
census early.

**Moving past declarations.** After the per-definition loop, any declaration
record ends the census with a `stopped` record. The later declaration passes and
all bodies are skipped: they consume completed facts of every item. Bodies run
only after declarations complete with no record. Function failures do not block
initializers, which need only signatures. Any body record ends the census before
constant patterns, authentication, constant evaluation, the panic handler and lint
finalization.

**Output.** `dev/src/sema/census.rs` returns typed records in discovery order,
which is deterministic. Each record has a phase, category, item, file id and byte
offset (or unlocated), the expansion trace, the message and an optional blocking
record. The driver also counts checked, failed and blocked items per phase.
`--census-output PATH` is accepted only with `--source-core` and
`--emit-types`/`--emit-semantics`, and without `-o` or counting. Only the CLI
writes the escaped tab-separated file. It prints the per-phase summary to stderr
and always exits with status 1.

**Ordinary mode.** The census ledger is `None`. The only changes on the ordinary
path are `Option` checks on error paths and activation sites, and the
behavior-preserving extraction of the declaration dispatch and the body `State`
constructor. First-error diagnostics and outputs are unchanged.

## Checks

`dev/tests/census_mode.rs` (frontend group) drives the CLI on minimal `no_core`
sources. It covers three independent declaration failures, each user of a failed
struct or alias blocked once, four plain and two hidden-type-scheduled body
failures, a CTFE frame-limit `budget` stop after a failure, and a frontend
`budget` stop. A passing crate exits 1 with `terminal none` and no stdout. Files
are byte-identical across runs and both views. On every failing input, ordinary
mode prints exactly the first census record, at the same position. Private
`census_records` tests cover root-cause chains, repeats and budget
classification. `scripts/tests/test_original_core.py` covers the observer.

Focused `make fastcheck` receipts under `obj/qualification/fast/` (format,
architecture and strict Clippy passed in each):

| Receipt | Selection | Result |
| --- | --- | --- |
| `1794027132855203616` | `census_mode capped_cli expanded_cli` | 16 passed |
| `1794027233500598396` | private: census records, import scheduling, defining heads | 5 passed |
| `1794027486699276173` | `declaration_types nominal_declarations opaque_body_values generic_aliases opaque_inference` | 28 passed |
| `1794027548969935357` | `body_types hidden_values` | 10 passed |

The observer unit tests (8) and `maintainer_suites.py check` (461 roots) also
pass. Pending for scheduled CI: full release regular and native tests,
all-target Clippy, the harness and the original-core census itself.

## Census on original core

Do not run this until LIB-193 is joined; today it would end with one record from
`Checker::from_facts`. The integrator then:

1. Pins the candidate CLI and writes its producer binding, as for any
   original-core observation.
2. Uses a manifest copy that rebinds the `dev/bin/rustgm.rs` and
   `dev/bin/crates.rs` configuration-source pins. They changed only to add
   `--census-output`; no default, option or limit changed.
3. Runs `python3 scripts/observe_original_core.py --census --cli PIN
   --producer-binding BINDING --manifest MANIFEST --source-root PINNED_TREE
   --out obj/parallel/core-census-N`. The script applies its usual storage guard
   and 190 s hard limit.

`--census` adds `--census-output OUT/census.tsv` to the unchanged `--emit-types`
command. The run must exit 1 with empty stdout, the final census stderr line and
a complete file whose `end` count matches its records. Anything else is
infrastructure. `result.json` records `analysis_only: true`,
`qualification: false`, `analysis-only census; not a qualification`, per-phase
counts, category totals and the terminal record. The file hash is in
`raw_sha256`.

## Limitations

- First-error passes remain: trait owners, associated types, destructors, early
  implementations, deferred defaults, nominal fields and sizes, local
  annotations, layouts, API exposure, constant patterns and completion. A failure
  there ends the census with one record. The core census chooses which to isolate
  next.
- Reaching a poisoned unit through an inline guard that is not a work item can be
  recorded as `failed` rather than `blocked`. It is recorded once and never as
  success.
- Budget classification is lexical, so a misclassification only ends the census
  early.
- In bodies only a hidden-type dependency blocks a dependent; other callers check
  against the failed function's signature.
- Frontend records have file ids but no `file` path rows.
