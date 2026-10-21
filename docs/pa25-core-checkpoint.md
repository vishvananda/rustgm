# PA25 checked execution core

This checkpoint implements source-to-native required constant calls. PA25 stays
planned until its remaining contract and qualification work is complete.

The semantic layer owns typed expression requests, closed instance identities,
exact readiness requirements and resumable declaration continuations. The
pipeline installs a checked-CFG provider. The interpreter reads immutable typed
operations and shared constant values; semantic analysis does not import the
body builder, borrow checker or interpreter. Eligibility covers skipped code;
execution only visits selected control flow.

Execution uses an iterative frame stack, typed scalar arithmetic, symbolic
mutable allocations and projections. Function recursion is distinct from named
initializer value cycles. Instructions, frames and storage have deterministic
limits. Typed code snapshots and retained early generic templates each have a
64 MiB charged storage bound. Work counters record actual steps, frames,
resumptions, peak frame/storage slots and charged retained code/template bytes.
These counters describe execution work, rather than measured heap usage.

Each source function has one inference/CFG construction. Early concrete bodies
move into ordinary compilation. Late concrete demands borrow the already built
body from a temporary owner store; all originals return to their initial order
on success or evaluation failure. Published generic templates have one immutable
`Rc` owner shared with runtime instantiation. A template demanded before runtime
publication retains one immutable typed copy for execution and one for handoff:
the consumers own differently timed arrays, with both copies charged before
retention. No source is reparsed or inferred for that handoff. Reentrant service
and ownership-store borrows return explicit errors.

The borrowed loan verifier is the existing defining-body proof algorithm,
extracted behind a per-body interface. Interpreter snapshots follow region,
ownership, cleanup and loan verification; calls retain the selected closed
instance and its caller obligations. The phase audit admits the interpreter's
actual typed value/operator/source-identity dependencies and the pipeline's
installation edge, while keeping semantic analysis independent of execution.

Focused qualification lives in `dev/tests/cfg_execution.rs` and
`dev/tests/required_values.rs`. It independently checks Rust 1.98.0 const values
and declared negative source outcomes, then executes accepted emitted LowIR with
the configured CPPGM backend. Source invariants check one CFG construction for
early, late, repeated and generic-instance demands, alongside real work counts.
The maintainer source suite is separate from the future released PA25 manifest.

Still required: associated general initializer specialization, complete
slice/text projection execution, the deterministic `--emit-constants` graph
view, PA27 pointer/union integration, manifest-listed assignment fixtures and
independent references, broader earlier-stage qualification and frozen-binary
performance evidence. No assignment activation or oracle regeneration belongs
to this core checkpoint.

Core qualification passed formatting, strict all-target/all-feature Clippy,
1,164-file architecture/layout audit with zero findings and all four audit
unit tests. The explicit focused run passed both machine tests (24 independent
const-value cases plus deterministic limits) and all three source tests (ten
independent native programs, seven independently declared object rejections,
and source CFG/work invariants). Exact source hashes and separate gate logs
are retained in `obj/pa25-core-checkpoint/`. Full release/integration gates and
performance observations remain separate qualification steps.
