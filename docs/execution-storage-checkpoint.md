# Selected constants, iteration and stored values checkpoint

This checkpoint combines checked caller-specific constant selection, integer
`MAX` constants, canonical user-defined iteration and `for` lowering, typed
byte-addressed constant access, and named static storage transported through
crate metadata. PA1–PA24 remain active; PA25–PA30 remain planned. The remaining
language/library families and assignment release contracts are still required.

The production cutoff is `b870a3ee`; the tested source snapshot is `ec5bbca4`.
The default-feature release executable has SHA-256
`d087014fdf6b53f5c1d98f808d4e3e9b77a59640d6dffd209aeabc958dd79be7`.
The default Cargo build reuses the all-features executable byte for byte because
this package has no production feature split. Later edits regenerate reviewed
references and correct a test comment; they change no compiler behavior.

## Functional qualification

The first complete release Cargo run passed 2,556 tests and exposed five test
expectation/coverage failures. Review established the following corrections:

- Two exact layout assertions came from different worker bases. `Declarations`
  is 8,472 bytes: the earlier iterator baseline gains an eight-byte optional
  computations box, an eight-byte work counter and one 32-byte `write_bytes`
  intrinsic slot. The resolver's trait registry gains an eight-byte optional
  iteration catalog. The named-storage map is inside boxed linkage and does
  not explain the `Declarations` increase.
- One symbolic-initializer invariant now reaches a concrete selected lifetime
  after legitimate alias canonicalization. It retains its original fixture,
  adds a genuinely open generic fixture, and preserves executor admission,
  publication and cache-authority assertions.
- The unchanged zero-sized `usize::MAX` fill and intra-object byte-access
  sources now succeed. Pinned Rust object emission and all three candidate
  inspection/lowering modes independently confirm those outcomes. Numeric
  pointer and overflow opposites retain their rejection contracts. An expired
  zero-sized transient pointer is a constant-evaluation observation, not a
  native execution claim.

After those test-only repairs, all 810 library tests and 106 memory-suite tests
pass. The other twelve integration groups retain their passing full-run
evidence, yielding **2,561 qualified regular tests**. Production source hashes
are unchanged across the repair. The full ignored/native run additionally
passes **263 tests**. Baseline test inventories remain present with their
original regular/ignored classification.

Release Clippy with every target and feature and warnings denied passes.
The file audit checks 1,714 files with zero findings. All 90 harness tests,
assignment layout/discovery, pinned upstream/reference checks, and executable
and object backend handshakes pass.

Full affected independent checks pass for PA6 names and PA20/PA21 semantics
and execution. PA20 checks 836 LowIR inputs, including 508 executable
comparisons; PA21 checks 450 LowIR inputs, including 264 executable comparisons.
Existing declared stage-specific acceptance differences remain explicit.

## Text-oracle review

All 8,360 active cases were compared. Initially 8,352 were byte-identical and
eight required review; no exit status changed. The complete source and output
hashes match the corresponding worker review bundles.

PA6's `for_binding` adds demand-loaded canonical iteration metadata. Every
original source name, scope, definition and reference remains identical and in
order. PA20 has six semantic-view changes replacing private selected aliases
with their proved caller or static arguments. Its remaining change preserves
an invalid lifetime rejection and points at the associated-type use at byte
115 instead of the trait argument at byte 92. Declaration contracts, original
types and the opposing unpaid-bound rejection remain intact.

Only these eight cases were regenerated, after independent qualification,
through explicit selected `course.py --bless` invocations. Scoped receipts
under PA6/PA20 `reference-batches/` preserve the existing full-stage receipts.
Every generated output matches the individually reviewed bytes; layout and
provenance checks pass. No fixture or expected exit status changed.

## Performance qualification

The comparison uses immutable published and candidate binaries and 43 frozen,
independently Rust-object-qualified inputs. Thirteen inputs execute in both
versions with identical output/metadata bytes; thirty exercise newly supported
behavior and have no baseline speedup claim. The inputs cover shared argument
observation, scalar controls, C metadata, integer constants, user iteration,
selected callers, static roots/imports and byte-access scaling.

The initial independent source pass found an incorrectly unpadded static name
in three authored import controls. Corrected source declarations and their
independent qualification are retained separately, before candidate observation.
The first candidate preflight also passed a LowIR-only library flag to constants
inspection. Both compilers rejected that command with status 2. The corrected
preflight preserves the other 42 successful comparisons and changes only that
one invocation; independent library qualification and fixture bytes are intact.

All 43 measurements and output/metadata checks pass. Across the thirteen shared
inputs, instruction deltas range from -0.45% to +0.73%; median maximum-RSS
deltas range from -522 to +144 KiB. No predeclared instruction, memory or
scaling attribution threshold is triggered. Each complete 8/32/128 scaling
family has an incremental instruction ratio of 2.79–4.05 for fourfold growth.
These results support bounded scaling for the measured families, not every
possible source program.

Shared paired wall medians range from -2.71% to +3.43%. A/A spread reaches
16.54%, and some candidate-only calibrations are substantially noisier, so
these wall figures do not establish a small end-to-end speedup. All observations
are retained without filtering.

The declared protocol retains six A/A observations,
five paired ABBA blocks, three executions per observation, eight RSS readings
and instruction profiles. No favorable rerun or candidate-only speedup claim
is permitted. The exclusive resource lease excludes registered RustGM heavy
jobs, but cannot exclude unrelated projects on the same machine.

## Retained evidence and remaining work

The integration checkout retains `obj/parallel/execution-storage-checkpoint-v1/`
and `-v2/`, including the original failures, source maps, complete logs,
inventory, candidate pin and reviewed reference generation. Main retains
`obj/parallel/execution-storage-outcome-review-v1/`,
`execution-storage-independent-v1/` and `execution-storage-performance-v1/`
and `-v2/`. Generated native/object artifacts are cleaned; sources, diagnostics,
receipts, hashes, raw measurements and current pins remain.

Future independent batches can scope cleanup to their declared evidence roots,
`obj/rustc-syntax/`, `obj/native/` and any backend output they create, avoiding
a scan through unrelated historical evidence while preserving cleanup/reserves.

The next separately frozen batch adds the cleanup requirement algorithm,
pointer offsets, selected public callable storage, iteration method visibility,
projected/nested constant results and byte-array fragment transport. None of
those later results is evidence for this executable. Constant-evaluation,
collection iteration, unsafe validity, intrinsic, metadata and foreign-interface
completion remain separate obligations before PA25–PA30 can be released.
