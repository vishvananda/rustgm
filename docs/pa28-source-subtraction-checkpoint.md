# Source subtraction and genuine ColdPath fallback

Commit7ca5df00c executes actual source unchecked_sub and sub_with_overflow
declarations through checked signatures, original instances, source API/const/
unsafe policy and actual Copy premises. The arithmetic declarations retain their
genuine bodyless const Rust signatures. Symbolic forwarding remains symbolic
until a closed integer instance selects the kernel. Ordinary same-name functions
keep their own source bodies. Bootstrap identities and old wire admission stay
fixed; unimplemented source intrinsic transport remains fenced.

ColdPath keeps its genuine safe const source body and actual retained
`miri::intrinsic_fallback_is_spec` marker. Its completed function header and
ordinary source original/instance/runtime/CTFE checks execute that real fallback;
no compiler-created noop body or name privilege is supplied. The marker query
uses the existing paid authenticated owner/effect ledger, not a new scan or map.

Unchecked runtime subtraction emits the actual Sub only. CTFE still faults on
unchecked overflow. The overflowing pair shares the existing target-width
subtraction/flag algorithm with ordinary checked arithmetic, returning wrapping
bits and the correct flag for signed/unsigned8/16/32/64/128 and isize/usize.
LowIR comparisons produce machine predicates; the tuple consumer converts the
flag to Rust I1 before storing its actual bool field. The earlier missing
conversion emitted an8-byte store at a1-byte field and caused a real native
SIGSEGV despite passing private/structural checks. Failed LowIR/source inputs
and all raw observations are retained. The backend and layouts are unchanged.

Final private33 passes153.71s, receipt1793276406430447314 SHA256
`1f54f4560b29f5296ad34939045e781c71caaf165efdf989a7af8ac8e5f180ec`.
All3167 merged compiler/test/harness hashes and all52 committed source/fixture
hashes match exactly. Root reuses that actual private/native evidence rather
than compiling the same private binary again. No new Root private/native
execution is claimed. Both private and production strict Clippy, formatting
and architecture pass.

The native-v2 proof has18 passing operations and SHA256
`02f73d161f8552d115b18d6faad0e90f43453ba56c01f830c5404ff3599a46ed`.
It includes the independent updated Rust/C observer, bounded-work measurements
and actual source-runtime, generic-forward and ordinary-lookalike LowIR/object/
C-object/link/execution. Direct12-width output matches pinned Rust, SHA256
`95071480423b95aa97cdea31f5c6c18310b4db30898663962f0afb15a53e7579`.
Generic candidate output matches that concrete model, while the same generic
Rust object hits a retained pinned-rustc ICE. Bool object generation also ICEs.
Neither is called a Rust language rejection or an independently qualified
same-source generic native success. Missing-Copy and nonempty ColdPath source
declarations are retained Rust-positive finite-profile opponents.

Repeated8/32/128 descriptor queries pay280/1120/4480 header steps with stable
87008 paid bytes. SubtractionSelection20, SourceIntrinsic1, Catalog1584,
NodeFacts80, Node36 and Body456 are measured; no hot fields or new maps are added.
Root production/CLI3 passes168.73s, receipt1793276800822119301 SHA256
`e9bb31cea1899529d6b3036783c5a8ffa42c93d0bf34fdb669922910fba44430`.
Authenticated evidence/fixture reuse, harness272/upstream and fresh probes pass
39.74s; SHA256
`41a0a9f512cd4b5e144a1158e569c2db5506042fbb5a8397001d0aa30378c96a`.

CLI42f10af2c2372c83fd333ad37492979a13b00773581762a5b526da063f23a8d9
still reaches file45:6972's open/exclusive/alternative pattern profile. All722
source hashes and target settings stay fixed under work14,500,624/attempted1GiB/
retained64MiB. The capped probe takes12.33s,374 opens; SHA256
`76c3270c41945d493ab09d62717c3968dc45c9e57f610f28f2dc6fd846fcb987`.
Strict retains the deferred expectation at68 opens; SHA256
`e004229a6145b11769aab1e5e9e536598493cb1dec35e684f0d4d5d85192d2fe`.
This is no original crate artifact or speedup measurement.

The [actual source chain](pa28-range-sub-intrinsics-plan.md) retains every original
checked_sub/RangeSub/char/panic dependency. The endpoint consumer's inherited
tracking requires real caller-location data during declaration CTFE; that shared
readiness correction is next. Generic refinements, scalar niches, broader numeric
intrinsics, source transport, full PA release checks and all original core/alloc/std
artifacts remain required. No whole assignment is released by this checkpoint.
