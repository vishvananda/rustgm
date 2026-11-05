# Current CPPGM backend checkpoint

All three tools now pin cppgm-extended
`b7c6b8e72595a524e1b3f147212d69c02b257dd8`, replacing the mixed905bd7e4/3438f5ae
pins. Existing g++ O3/no-strict-aliasing build stamps match; dependency checking
with unchanged configuration reports the tools up to date. The C++ checkout's
tracked sources and existing untracked files were preserved.

`backend.lock.json` binds tool bytes, and `backend.distribution.json` binds the
new public `backend-pins-v2` archive. The deterministic bundle includes all three
tools, their manifest, inherited NOTICE and Apache license. The v1 distribution
remains available for historical pins. CI and clean installations verify the
archive/lock and every tool before installing immutable pins.

The reviewed [LowIR contract](https://github.com/vishvananda/cppgm-extended/blob/b7c6b8e72595a524e1b3f147212d69c02b257dd8/pa8/lowir.md)
adds useful capabilities:

- `convert trunc exact`: implemented here for the proven0/1 comparison carrier
  in unsigned atomic min/max. `compare_u32` produces the comparison and exact
  narrowing together. Ordinary casts stay ordinary; the wrapping mask subtraction
  stays wrapping. Typed conversion identity adds no instruction/owner flag field.
- `v128` and vector operations: available backend vocabulary for PA28/41 intrinsic
  and target closure. Actual Rust SIMD selection/feature/ABI/transport proof is
  still required; this change qualifies existing C128 copy boundaries.
- `equivalent_entry`: available only with actual equivalent behavior, ABI and
  absence of required distinct address identity. It is not granted by matching
  rendered bodies or names.
- Debug types, lexical scopes and local locations: available for PA44's debug
  closure, using real source/type facts. Wider floating literals/runtime support
  can support future PA25/28/41 work; current frontend fences remain explicit.

C++ `typed` alias facts and `nowrap` assumptions cannot be copied wholesale into
Rust lowering. Rust raw-pointer/union aliasing, wrapping arithmetic and overflow
checks require their own producer proofs. The backend's O0 development profile
remains explicit; updating compiler tools does not silently enable O2.

The SIMD check exposed an existing bridge mismatch: `build_objects` ran the
standalone executable-profile reader before the host-object driver, rejecting
valid SSE/SSEUP classes. Object units now go directly through the locked
`cppgm++ -c` host-object parser/validator, avoiding that wrong and redundant parse.
Every object input must be an existing `.lowir`; suffix checks also protect the
shared emitter from routing into the C++ frontend. The scalar standalone
executable bridge keeps its existing parser/linker path.

## Verification

The final fast gate passes3 selected public tests,1193 PA19 text cases, strict
selected Clippy, formatting and architecture in45.84s. It includes the exact
carrier/ordinary-cast opponent, all28 atomic extrema source contracts and
8/32/128-caller bounded-work controls. The existing shape checker now requires
exact comparison narrowing while retaining its wrapping-mask/dataflow assertions.
Six actual-source extrema native cases match rustc on the updated tools.

Clean bundle installation, executable/object/C callback/global and exact-carrier
handshakes, all27 active backend atomic cases, source-attribute fresh serialized
native clients, and the Deref/Pin/Future observers pass. Existing SIMD copy/ABI
sources with an explicit ordinary entry match pinned Rust exactly:
`1.25 -2.00 3.50 4.00 5.50 -6.25 7 -8 3.50` plus LF. No source body or LowIR
is replaced. The initial SIMD attempt's narrower-reader rejection is retained;
the corrected object-profile path passes.

Immutable old/new object compilers also pass native observers on two frozen
common LowIR inputs with exclusive A/A-calibrated paired ABBA observations.
Paired B/A medians are1.066/1.117, with A/A ratios0.668–1.475 on these very short
millisecond samples. They establish no speedup or reliable regression estimate.
Raw observations are retained. The first benchmark harness had duplicate main
symbols; the corrected observer link renames only the source entry in verification
objects. Timed inputs and compiler outputs remain unchanged. This is not a
whole-program performance or generated-code quality qualification.

Final evidence under warm btrfs scratch:

- `obj/qualification/fast/1793841174654758876/receipt.json` (45.84s).
- `obj/parallel/backend-refresh-v1/qualification-v2/receipt.json` (19.21s):
  38e6565c6d71eb97f649913f4fcaff8f9e2e307216fab86d7ec11ae7623f8dc0.
- `obj/parallel/backend-refresh-v1/simd-v2/receipt.json`:
  5d8f9ca119ee7630c61599e850a9abc044e248e922e187ecadf6cc430f58583d.
- `obj/parallel/backend-refresh-v1/native/receipt.json` (same compiled Rust test
  binary/backend, before the adapter-only correction):
  8246e71b3d59a80b8df707b0da56b32a0daf8bddf688690a6389c73081746d58.
- `obj/parallel/backend-refresh-v1/measurement-v2/receipt.json`:
  edde4eb594a409f5f09e89764459eda3a7db7c5d2011ecfba3fca8b565d5dd2a.

Earlier full Pin CI41939550904 is terminal success. A full combined successor
remains pending. No whole assignment or original core/alloc/std artifact follows
from this backend checkpoint.
