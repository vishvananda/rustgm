# Typed body transport: Branch and checked Panic exits

Lane C checkpoint on `implement/body-transport-branches-20261107` (base
`beabc1161`). It is a transport slice, not a library milestone: no original
core artifact, LIB-193 repair or whole-assignment claim follows.

## Design

**Profile.** Source envelope tag 10 gains version 12,
`Source2024ModuleBranches` (CLI `module-branches-v12`). It grants exactly the
version-ten capabilities (nominals, conversion roles, call safety, definitions,
module compatibility, feature predicates, availability, module contracts,
unconditional callables) plus `branches()`. It adds no operation, nominal or
compiler grant and excludes the version-eleven compiler-operation profile
(tag 13, not CLI-routed). Terminator tags 0-2 keep their bytes; tag 3 is
`Branch { condition, yes, no }` as three fixed u32 ids and tag 4 is `Panic` with
no payload. Version 12 is a separate, explicit profile so that every earlier
version keeps its exact bytes and authority: a branch-free version-twelve
export differs from its version-ten export only at the version byte.

Earlier profiles cannot carry or silently erase the new exits. The producer
maps `ir::Terminator::Branch`/`Panic` only when the stage is version twelve;
`FalseEdge` (pattern decisions) still rejects rather than being flattened. The
shared encoder refuses the new rows for older generations, both staging paths
run `branch_profile::require_profile`, decoders accept tags 3/4 only under a
version-twelve descriptor, and the legacy envelope-9 codec and validator reject
them explicitly.

**Validation of untrusted rows.** Branch successors must name blocks of the
same original and the condition must name one of its values; an `Edge`
location is valid only for an actual successor of its block. The port plan
requires the condition's actual type node to be the canonical `bool` scalar.
`Panic` has no successor, unwind or cleanup edge: under the abort runtime
contract it ends the path without drops, and lowering reuses the existing
shared exit (one block calling the declared noreturn `abort`, then
`unreachable`) also used by provided bodies. SSA snapshot bodies (temporary
place bases) are proved by a single-chain availability walk; a two-way exit
there is refused as requiring a dominance proof. Malformed and truncated rows
fail decoding.

**Consumer proof.** A fresh consumer rebuilds `ir::Body` from the validated rows
(`rebuilt_operations::build`; the cached-original check compares Branch fields
and Panic, gated by the descriptor profile), then runs the ordinary original
and per-instance pipeline over the reconstructed CFG: typing of the condition
at each instance, move/initialization and merge-store ownership, storage,
regions and loans, drop requirements, promotion and const eligibility. No
provider proof is retained or trusted; an edge forged to skip the merge store
is a well-formed CFG that only the consumer's own initialization proof rejects.

**Remaining for complete body-IR transport.**

- Panic producers: no source construct emits `ir::Terminator::Panic`. Checked
  arithmetic, bounds checks, `assert!` and panic construction need their
  rvalue/projection forms; the imported `Abort` intrinsic needs cross-crate
  intrinsic identity.
- Enum discriminant reads and `match`: FalseEdge, guard tables and Guard locals.
- Trait-method calls through transported impls (associated originals, method
  selection rows).
- Drops: elaboration rows, destructor contracts and the `drop_glue` foundation.
- Closures: selected closure entries and captures.
- References and projections: region formals in free originals, early/captured
  callable frames, temporary field projections, and SSA snapshots across
  two-way exits. A dominator-based design (Cooper-Harvey-Kennedy idoms over
  reverse postorder, dominator-tree intervals, same-block statement order)
  was drafted but held back: no source body reaches it yet, and it is
  untestable without the private test harness.
- Constants: const items, promoted roots, static types, mutable statics,
  comptime bodies; `!`-typed headers.

## Evidence

Fixtures, frozen outcomes and the drop-glue refreeze are described in
`pa29/tests/spec/source-branches/README.md`; nothing was hand-edited or blessed.
Receipts are under `obj/parallel/body-branches/`. Final CLI SHA-256
`1b12b824166238091c70f89cb0079e93fffdfab8d3e8e32cb3b4442ee6c11ffc`.

- Independent Rust 1.98.0 (`rustc/receipt.txt`): all providers compile; native
  exits 7, 12, 22, 2, 13, 9, 31 and 5; opponents E0382, E0308, E0277.
- Candidate (`candidate/receipt.txt`): every frozen outcome matches. Version-ten
  export of a branching provider rejects; consumers re-run move, type and Copy
  checks per instance.
- Baseline `beabc1161` against candidate (`baseline-compare/receipt.txt`):
  version-ten exports of `spin.rs` and the source-nominals provider are
  byte-identical. The only old-profile change is the diagnostic for a two-way
  exit, now naming version twelve.
- Wire controls (`crate_linking::branch_transport`). Relabelling version 12 as 10
  rejects tag 3; a foreign successor, an unknown condition and a non-bool
  condition reject during validation; a deleted successor and every strict
  prefix fail decoding. A yes edge that skips the merge store, and a first
  `choose` exit naming the second condition (added after observation), are
  well-formed rows. Only the consumer's own initialization and value-dominance
  proofs reject them. Foreign and swapped owners are missing provider metadata.
  The tag-4 Panic row decodes and re-encodes under version 12 and is rejected
  under version 10.
- Forged Panic, natively. No source producer exists, so the `wait` looping edge
  is redirected to its dead block, which is relabelled Panic. The fresh
  consumer accepts the re-proved CFG and lowers the shared `abort` exit; the
  failing call ends with SIGABRT and the ready call still returns 5.
- Scaling: else-if chains of 6, 24 and 96 exits add exactly 367 wire bytes per
  exit. Each fresh consumer keeps exactly one two-way exit per arm, and LowIR
  grows less than fivefold per fourfold step. Provider and consumer each take
  under 0.15 s at width 96. The syntax nesting limit is 128.

Checks: focused `make fastcheck` passed assignment metadata, full format,
architecture audit, strict Clippy (CLI and `suite_linking`), the build and six
exact tests: the four new tests plus the source-nominal v10 and version-seven
regressions (`obj/qualification/fast/1794027869435931962`). Native `--ignored`
with `--prune-under obj/parallel/body-branches` passed the new native test and
the source-nominal native regression (`.../1794028014420886062`).
`make test-upstream` passed. Two earlier attempts failed in Clippy
(`too_many_lines` in `cfg_rows::emit`, `doc_markdown` in the test) and were
fixed (`.../1794027648586238692`, `.../1794027771525514850`). Pending for
scheduled CI: full regular/native suites, private library tests (no private
test was added; the private-lint cache here is cold), harness, all-target
Clippy and release builds.
