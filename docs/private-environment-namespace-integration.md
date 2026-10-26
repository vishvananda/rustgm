# Private environment and namespace integration

Compiler source `c489e44c` combines the qualified
[defining environments](private-defining-environments-checkpoint.md) and
[module namespaces](pa29-module-namespace-inspection.md). These are prerequisites
for source-library transport; they do not publish Provided bodies or restricted
trait metadata, complete an assignment, or produce original core/alloc/std.

[Combined CI 40229391850](https://github.com/vishvananda/rustgm/actions/runs/40229391850)
passes all eleven exact controls, production and private strict Clippy,
formatting, the global architecture audit, and the unoptimized private build.
The workflow also passes 215 harness tests, seven audit-cache tests, all 24
pinned upstream fixture checks and all 126 Reference file checks.

The production CLI build takes 30.44 seconds. The focused helper takes 117.86
seconds, including formatting 4.22, production Clippy 22.92, private Clippy
33.04, build 56.86 and exact execution 0.114 seconds. The independent Python
architecture audit takes 24.17 seconds alongside sequential lint commands;
only one Rust command runs at a time. Separate static receipts are joined
before building or running tests. These are development timings, not compiler
throughput measurements or assignment-release qualification.

The integrator authenticates all 2,614 source fingerprints, the eleven selected
names, both static receipts and the actual archived CLI bytes. Receipt SHA-256:
`b0df7e03474f56a8cff799868cd46765b5c76220513bacbcfed81eb3da732a55`.
CLI SHA-256:
`b540332f2c52398fc4c5cc0eedbad85b4d6036b23732e14cf5c57212be87645f`.
The CLI remains in the hosted artifact; verification does not extract or execute
another local binary. Raw receipts, outputs and binding verification are under
`obj/parallel/environment-namespace-combined-v1/verified-40229391850/`.

Assignment metadata retains its successful check from run 40228080113 through
an authenticated receipt and complete source-diff proof. That run's production
wildcard-import failure remains failed. The two subsequent formatting and
unused-import failures remain failed too; none is counted as private execution.
The final source uses explicit, formatted imports without lint exemptions.

The unchanged original-core frontier remains `mem/maybe_uninit.rs:1644`, at
`default fn spec_fill`, with 58 original files opened and all 722 inputs verified.
These private changes do not resolve that syntax/semantic blocker, so no redundant
whole-core probe or full text/native matrix was run. The preceding 8,493-case
text qualification remains historical evidence, not a claim of fresh execution.

Next work is the actual empty Contract adapter with old v0/v1 byte witnesses,
the explicitly versioned module/domain/advisory artifact and fresh consumer,
and specialization defaultness followed by its real graph and body dispatch.
