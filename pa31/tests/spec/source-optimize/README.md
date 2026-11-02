# Source optimization intent

PA31 owns source declaration authority; PA40 closes the full attribute profile,
and PA29 owns authenticated imported intent. All three remain planned.

`contract.json` freezes 49 source cases and outcomes before reference execution.
The pinned Rust 1.98.0 compiler independently matches all 98 metadata/object
outcomes and executes both native companions with output `11\n`. The receipt and
compiler hashes are in `qualification.json`; source bytes and raw diagnostics
remain under `obj/parallel/source-optimize-v1/reference-v2`.

The primary implementation is the pinned Rust checkout's
`compiler/rustc_attr_parsing/src/attributes/codegen_attrs.rs`, OptimizeParser.
It permits exactly one parenthesized size/speed/none word (a trailing comma is
accepted), on functions, closures and methods with bodies. It requires
optimize_attribute and rejects duplicate annotations and unsafe wrappers.
`compiler/rustc_passes/src/check_attr.rs::check_optimize_and_inline` rejects
none with inline or inline(always), accepting inline(never), regardless of
attribute order. Generated meta attributes require their actual macro envelope;
forwarded whole items keep caller authority. Disabled cfg owners do not apply.

In `compiler/rustc_codegen_llvm/src/attributes.rs`, none adds optnone/noinline,
size selects minsize/optsize, and speed avoids the global size preference.
`compiler/rustc_mir_transform/src/inline.rs` excludes none callees from inlining.
The current RustGM/locked CPPGM execution profile has neither a Rust MIR inliner
nor per-function optimizing LLVM passes. Its conservative behavior remains
valid; source intent must be retained before future optimizers consume it.
Old metadata profiles must reject unencoded intent. No preference grants
const, unsafe, mutation, borrow, representation or call permission.

These are authored protocol tests, not copied upstream tests. The pinned primary
sources remain read-only. Candidate qualification is recorded separately from
reference success and never implies an original library artifact or PA release.

The final candidate passes eight private controls, strict Clippy/static audits,
58 production controls, 1,608 PA19/PA23 text cases and 48 CLI declarations.
Declaration CLI checks use the types view; the borrow opponent additionally
requires the loans view. The exact native-provider source lowers through RustGM
and the unchanged pinned backend and matches its original C observer and pinned
Rust with output `11\n`. The separate std println native input remains a broader
runtime requirement and is not claimed as a candidate success.

Widths 8/32/128 pay work 96/384/1536 and retained bytes
12096/48384/193536. Warm queries retain no storage; Header80, Node36 and Body456
stay unchanged. Original core advances to the lane-limit attribute in
portable-simd vector.rs107 with all 722 pinned source hashes unchanged.
