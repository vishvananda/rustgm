# PA19 trait and method validation

The PA19 release published 1,070 text-oracle cases: 683 accepted and 387 rejected. The semantic
lane contains 439 cases (436 accepted, three rejected); the LowIR lane contains
631 (247 accepted, 384 rejected). All accepted LowIR controls require clean
execution through both pinned rustc and the immutable CPPGM backend.

Subsequent PA20 normalization work removed the obsolete associated-type rejection.
Its exact source is now an accepted control in `dev/tests/projection_paths.rs`,
awaiting PA20's grading contract. Associated-constant value work similarly retains
that former boundary verbatim in `dev/tests/associated_declarations.rs` with
qualified native value controls. The active PA19 suite has 1,068 cases: 683
accepted and 385 rejected, including 629 LowIR cases. Counts below describe the
original release evidence; the current contract is in `pa19/README.md`.

## Source and expectation review

The corpus preserves static source controls from the existing maintainer tests
for trait/inherent headers, complete applications, caller bounds, conformance,
selection, qualified/implicit paths, visibility, receiver conversion and dispatch.
Native sources use their original check/fail wrapper. Feature directories replace
maintainer module prefixes; `maintainer_origin` records each source's origin.
The original PA17/PA18 inherent boundaries and PA18 trait-method boundary are also
present verbatim as accepted sources. No new upstream material is imported.

An initial declaration contained 1,033 cases, all of which matched independent
rustc and candidate outcomes. Supplemental review adds 37 cases, including the
nine entries from two separately classified inherent-header loops, static source
controls, generated callable-binder controls, migrated boundaries and explicit
phase/compiler differences. Six duplicate `(mode, source)` controls were removed;
inputs with different observable contracts remain separate.

One supplemental extraction prediction was wrong. The `dot-method-general-constant`
maintainer test accepts its source with rustc but deliberately requires RustGM's
PA25 boundary rejection. The extraction had inferred both outcomes from rustc's
acceptance. Reviewing the owning test corrected the declared RustGM outcome and
placed the source in `boundaries/general-constant-statements.rs`. The original
prediction, failing probe and correction remain retained; no compiler behavior
or oracle was changed to conceal this error.

Every case was qualified before reference generation. A second independent
candidate process produced identical stdout, stderr and exit status for all
1,070 cases. Full semantic output is an inspection view; the LowIR lane retains
all template, lifetime, ownership, loan, cleanup and instance checks.

## Explicit acceptance differences

Five sources accepted by rustc are rejected at later-assignment boundaries:
associated types and constants (PA20), higher-ranked trait bounds (PA21), trait
objects (PA23) and general constant statements (PA25). Each manifest entry has an
explicit independent status and reason.

`inspection/region-obligation-pending.rs` succeeds after declaration/body inference
and reports pending lifetime requirements. Pinned rustc rejects the invalid
returned borrow. The same source is required to fail in the LowIR lane under
`receiver-contracts/receiver-returned-borrow-cannot-escape.rs`. Inspection success
never authorizes execution.

The three `compiler-differences/duplicate-binder-*.rs` inputs contain redundant,
alpha-equivalent function-pointer trait bounds. RustGM canonicalizes equivalent
binder names and unused binders and accepts them. Rustc 1.98.0's default solver
rejects them with E0283. Supplementary probes of the same source bytes with that
same compiler, `RUSTC_BOOTSTRAP=1` and `-Znext-solver=globally`, accept all three.
That diagnostic experiment does not replace the default compiler oracle: the
manifest continues to require its rejection and names the difference explicitly.

The upstream [duplicate higher-ranked bound issue](https://github.com/rust-lang/rust/issues/140071)
is labelled as a compiler bug fixed by the next solver. It is related evidence,
not a claim that these three sources are a proven duplicate of that exact issue.
The course preserves canonical equality instead of manufacturing ambiguity from
redundant assumptions. Distinct lifetime correlations, local-to-static changes
and one-way outer bounds remain rejected by the neighboring binder/equality
controls. These three cases are excluded from default-rustc acceptance parity.

The nine differences above are all explicit. There are no native runtime
exceptions, codegen-only exclusions or skipped accepted LowIR programs in PA19.

## Reference, architecture and performance evidence

The handout defines `semantic-crate 1` records, deterministic ordering, count
output and the inspection/execution boundary before the references are generated.
Semantic records consume retained facts and share canonical type/argument/region
writers. They do not reparse source or rendered text. Candidate probes remain
isolated, and bound-selected methods retain their complete assumed application.

`pa19/reference.json` records source, manifest and compiler provenance. Oracle
triples were generated with `scripts/course.py pa19 --app dev/rustgm-ref --bless`.
The existing PA1–PA18 oracle bytes remain unchanged.

The preceding implementation checkpoints passed 929 debug tests, 117 affected
optimized tests, ten affected native groups and 3,789 earlier course cases. The
semantic-view checkpoint adds four focused debug checks and passes seven affected
optimized checks. Both checkpoints passed GitHub CI on main: runs 38735309406
and 38736772181. Their measured artifacts, performance limitations and source
hashes are recorded in [the buildout journal](pa18-pa21-buildout.md).

The final receiver checkpoint retains 2,132 frozen A/A and paired ABBA observations;
the semantic-view checkpoint retains another 468, including full rendering.
All binary/input/source hashes and output equality checks pass. Calibration noise
precludes a stable speedup or performance-equivalence claim. This course release
adds documentation, fixtures, references and stage registration; compiler sources
are unchanged from the measured semantic-view checkpoint.

Detailed declaration, extraction, correction, qualification, repeated-output and
native records remain under `obj/pa19-work/course-v1`. Full native evidence uses
the repository's ordinary harness and backend lock. The final release checks and
receipt are recorded in that directory; PA20 and PA21 remain unfinished.


Final installed checks pass all 4,859 PA1–PA19 course cases, all 1,070 independent
PA19 language checks and all 631 LowIR checks, including 247 native executions.
All 1,070 installed oracle triples match the independently repeated outputs
recorded before installation. Formatting, Clippy all/pedantic, the 578-file
zero-finding audit, 27 harness tests, 24 upstream fixtures, 126 vendored Reference
files and the backend qualification pass. The independent-check report now calls
these nine differences acceptance differences, covering both phase boundaries
and the documented default-solver discrepancy.

The final receipt is `obj/pa19-work/course-v1/release-validation-v1.json`.
