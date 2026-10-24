# Library build blocker tracker

Updated 2026-10-24. This is the work queue for compiling the original pinned
`core`, `alloc` and `std`, not a count of completed internal checkpoints.
PA1–PA24 remain implemented drafts; PA25–PA44 remain planned. No additional
assignment or whole upstream library has been completed.

The source pin is Rust `88d9e12ae178fab0fb5cc050a94da85685d449ea`, edition 2024,
x86-64 Linux, panic abort. Use the existing immutable shared source tree and
record target configuration, compiler identity and opened-file hashes on each
whole-crate attempt. Never patch upstream source to move a blocker.

## Observable crate milestones

| Milestone | Current result | Next acceptance evidence |
| --- | --- | --- |
| Original core loading and expansion | Blocked. Last recorded original-root attempt stopped at scoped `pub macro assert_matches` in `library/core/src/macros/mod.rs:170`. | Re-run the unchanged root; retain the actual next error and opened source set. |
| Original core semantic checking and body export | Not reached end to end. | Check every required declaration/body and produce the real crate artifact. |
| Original core object and consumer | No successful build. | Link and execute an independent consumer using its actual generic implementations. |
| Original alloc and consumer | Not reached; depends on core and cross-crate transport. | Build unchanged alloc and execute allocation, reallocation and destruction. |
| Original std and consumer | Not reached; depends on the preceding crate graph and platform contracts. | Build unchanged std and execute a default-std consumer. |

The last recorded core probe predates the current local integration candidate;
it must not be described as a fresh result. Its exact diagnostic was
`ERROR: file 3: 6527: expected path segment`. The retained record is
`rustgm-validation-work/obj/parallel/library-stability-whole-core-frontier-v2/`.
Local integration snapshot `91113764` passes 127 private, 151 frontend and
8,382 active text tests. Its release/native qualification is running. Those
counts do not establish a new crate milestone.

## Prioritized issues and assignment ownership

"Owner PA" identifies the compiler/course responsibility. "Test PA" is the
earliest assignment owning **all** features in the complete reproduction. A
source-library reproduction using `no_core`, lang items or compiler-private
attributes belongs in PA31 even when its underlying algorithm has an earlier
owner. Add an earlier-PA regression too when the failure can be reproduced
without those later features. Do not move fixtures mechanically by subsystem.

| ID | Priority and evidence | Blocking issue | Owner PA / test PA | Worker | State / dependencies |
| --- | --- | --- | --- | --- | --- |
| LIB-001 | P0; last observed whole-core blocker | Scoped declarative macro publication, item/path/import/glob lookup and definition-site visibility | PA31 internal feature; PA8 shared machinery / PA31, plus applicable PA7–PA8 regressions | Integrator | In progress; syntax/context/index groundwork exists, complete semantics still disabled. |
| LIB-002 | P0; prerequisite of LIB-001 | Hygienic field, method and associated-member identity; captured names and container-relative privacy | PA31; PA12/PA19 shared consumers / PA31, with earlier ordinary controls | C + integrator | Investigating isolated field slice; shared context-adjustment API must be agreed first. |
| LIB-003 | P1; actual reduced source failure | Projected Copy proof sees one lifetime premise in inconsistent declaration/body namespaces | PA21 scoped proof + PA31 source const-trait defaults / PA31; add PA21 control where expressible | B | Implementing an authenticated body query view; preserve distinct lifetimes/applications and final proof obligations. |
| LIB-004 | P1; actual reduced source failure, readiness cause not yet observed directly | Early const read treats source `u8: Copy` as absent although its impl is present | PA20 implementation readiness + PA25 early CTFE / PA31 source-binding reproduction | B, coordinated with C | Diagnosing prepared-header versus validated-impl scheduling; needs exact state/wakeup evidence. |
| LIB-005 | P1; independently qualified reduced program | Anonymous constant-expression `const_eval_select` ownership and cache validation | PA25 execution + PA28 intrinsic semantics + PA31 source identity / PA31 | C | Source-ready at local `379006ad`, unqualified; preserve handoff while C prioritizes LIB-002. |
| LIB-006 | P1; core-to-consumer prerequisite | Complete substituted signatures, operation types and required inputs for real exported generic originals | PA29 / PA29 data invariants; PA31 source-provider fixtures | A | Implementing one complete actual producer/consumer path; domain indexes alone do not close this issue. |
| LIB-007 | P1; core-to-consumer prerequisite | Deterministic generic/source metadata, authenticated remapping and fresh imported body proof | PA29 + PA31 source-role admission / PA29 ordinary multi-crate tests and PA31 source-provider tests | A + integrator | Blocked on LIB-006 and imported proof interfaces; descriptor/codec dispatch remains disabled. |
| LIB-008 | P1; next whole-crate acceptance gate | Close remaining active core source requirements and emit a real core artifact | PA32, with each failure assigned back to its language owner / PA32 original crate plus reduced owning-PA cases | Integrator | Blocked on LIB-001/002; discover concrete additional issues from unchanged-root runs, not guesses. |
| LIB-009 | P2; crate-graph prerequisite | Source dependency identity, edition/build configuration and reproducible build orchestration | PA34 / PA34 | Integrator | Partial interfaces exist; complete original graph qualification remains required. |
| LIB-010 | P2; not yet reached as a whole crate | Original alloc, allocator/runtime contracts and collection consumers | PA33; PA27/PA28 operation prerequisites / PA33 plus reduced owning-PA cases | Unassigned until core advances | Blocked on LIB-007/008/009; split observed alloc failures into concrete issues when reached. |
| LIB-011 | P2; not yet reached as a whole crate | Original std, platform linkage and abort-profile consumers | PA36; PA30/PA34 platform/build prerequisites / PA36 plus reduced owning-PA cases | Unassigned until prerequisites advance | Blocked on the original core/alloc graph; no unsupported percentage-complete estimate. |
| LIB-012 | P2; externally useful acceptance | Original no-default-features serde_core, then alloc/std serde_json graphs | PA35 / PA35 | Integrator | Run the core-only graph as soon as LIB-007/008/009 permit it; do not wait for std. |

## Reproductions, tests and closure criteria

Paths below include unpublished worker fixtures. Their presence in a working
branch is not a passing test or a released assignment contract.

- **LIB-001:** `pa31/tests/spec/declarative-macros/` contains predeclared source
  contracts and independent observations for concise/braced declarations,
  forward/imported aliases, repeated introduced items, mixed transparency,
  captured path segments and privacy. Maintainer suites `declarative_macros`,
  `hygiene`, `textual_scopes` and the PA7/PA8 suites retain legacy behavior.
  Close only after the real positive/negative source corpus and native answers
  pass, including exact-context import wakeups, then rerun original core.
- **LIB-002:** extend that PA31 fixture cluster with predeclared captured/literal
  field names, repeated opaque containers, method/associated-name collisions
  and private-member opponents. These new cases are **required, not yet
  qualified**. Reuse PA12 aggregate and PA19 method controls. Close only when
  actual member selection and privacy consume the shared typed context rule;
  a context-aware storage table alone is insufficient.
- **LIB-003:** retain
  `pa31/tests/spec/trait-default-effects/projections/provider-moved.rs` and
  `provider-partial-container.rs`, their public-context counterparts and
  independent contracts. Add actual body-alias/foreign-environment tests,
  independent `'a`/`'b`, complete trait/family arguments, nested binders,
  unresolved inference and failed-budget controls. The private
  `body::defining_cleanups` and trait-selection tests must pass alongside the
  retained native contexts. Do not fix ambiguity by merging arbitrary matches.
- **LIB-004:** keep the unchanged
  `pa31/tests/spec/const-eval-select-ctfe/reference-anonymous-selector.rs` and
  its independent Rust acceptance. The bounded `read_frontier` invariant now
  observes a canonical `u8` read and false Copy decision despite the actual impl.
  Add prepared/validated status and cold/warm/declaration-order controls. Close
  only with correct dependency resumption and a successful complete program;
  returning Deferred forever is not a fix.
- **LIB-005:** `anonymous-empty-selector.rs` and its C consumer already have
  independent positive results. The local successor adds four private owner,
  cached-result and foreign-Code controls, reuses lifetime/readiness checks,
  and declares a 64th native context. Those successor checks remain unrun.
  This does not close LIB-004 or general reference/conditional callback support.
- **LIB-006:** use the actual
  `pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs`
  collector and private `qualified_source_rows`, `transport_domains` and
  `regional_forest` controls. Add same-arity wrong signatures, changed reference
  types, wrong binder relocation, omitted inputs and unused-body failures.
  Require complete operation-derived coverage, atomic failure and independent
  width/depth/shared-DAG work and storage bounds.
- **LIB-007:** extend PA29's multi-crate contracts and the PA31 source-header
  transport family. Require complete provider → artifact → fresh consumer
  checking → native execution, input reordering, duplicate/missing dependencies,
  tampered owners/roles, const permissions, lifetime/loan/cleanup proof and
  deterministic text views. Support indexes or serialized bytes alone do not
  close this issue. Exported macro provenance is a separately required subtask.
- **LIB-008–012:** each actual build attempt supplies a versioned source/config/
  dependency lock, exact diagnostic, opened-source inventory and artifact/native
  result. Add a reduced owning-PA fixture for every newly reached failure while
  retaining the unchanged whole-crate acceptance test. An earlier parser error
  must not make a later semantic negative appear to pass.

Every completed fix includes its owning handout update, independently declared
positive/negative outcomes, relevant maintainer tests, and manifest-discovered
text-oracle tests when that assignment's contract is published. Generate and
review oracle text; never hand-edit it or accept an empty/planned suite as a pass.
Text fixtures belong in the earliest PA owning all features, and private test
roots must remain registered in the existing maintainer suite groups.

## Work and qualification rules

Keep the three existing GPT-6.1-Sol/high workers. A worker can carry multiple
independent issues, but keeps coherent commits and an explicit handoff for each.
Agree shared APIs before concurrent edits; use separate worktrees. Current
primary lanes are A: LIB-006/007; B: LIB-003/004; C: LIB-002 with LIB-005 frozen;
integrator: LIB-001/008/009/012 and integration.

Source work proceeds concurrently. Heavy jobs remain serialized at current disk
headroom under the existing 150-GiB reserve, 50-GiB/worktree allowance and CPU/RAM
limits. Use focused batched checks during a fix and one combined release gate
for a coherent integration. Reuse unchanged qualification and independent
observations with exact source/tool bindings; retain failures and clean outputs.

Update this tracker at each handoff/integration and each original crate attempt.
Report three separate outcomes: **issue fixed**, **crate boundary advanced**,
and **assignment released**. Closing an issue requires its complete reproduction
and guards to pass. Advancing a crate requires a fresh unchanged-crate result.
Releasing a PA requires its entire cumulative contract; no partial activation.
