# Ordinary source nounwind in the abort profile

Commitfdf2fedd8 admits ordinary `rustc_nounwind` functions using their actual
calling convention and the fixed target's panic policy. C is non-unwinding;
Rust/rust-call use the same immutable abort fact that owns cfg. User cfg cannot
replace it. The exhaustive ABI match requires a new explicit rule when an unwind
ABI is implemented. PA39's actual boundaries, cleanup and transport remain open.

The existing source function, word-form and internal-feature producer remains
authoritative. Ordinary body/const/unsafe and reified function-pointer proof is
unchanged; this marker creates no intrinsic identity or extra call permission.
No source body is synthesized, and no field, map, allocation or per-call owner
scan is added. Package256, Node36 and Body456 remain unchanged. Marked and
unmarked ordinary controls produce byte-identical LowIR in this abort profile.

The authored source contract is `pa31/tests/spec/source-nounwind/contract.json`.
Its Rust/C const and runtime bodies and ordinary function pointer produce
`0 1 0 1 0 1`, both independently and through the frozen CPPGM backend. Six
opponents preserve actual feature, word/target, unsafe and const errors.
C-unwind is independently Rust-positive and stays an explicit unsupported ABI,
not a claimed language rejection. Fifteen pinned Rust operations pass; SHA256
`160fc4570e8dababcd6cb6a72d3d90623a01da63de88e59a785b5e8dd9351c3b`.
The first oracle omitted LegacyReceiver needed by the bool Clone receiver.
Its inputs/raw outcomes are preserved and supply no semantic qualification.
The corrected fixture adds that genuine source foundation only.

Five focused checks pass145.28s. Adding the missed CI cold-table alias control
then passes all6 in113.55s; receipt1793275639546294780 SHA256
`a9039f9cdb421f93bfaab99327232997be33324272ab19e88a7629059441d8d6`.
This includes the existing source caller-location and actual Abort signature
controls. The CI failure was one stale Reservations2224 expectation; the observed
and independently measured size is2288. Other hot sizes and ordinal checks remain
unchanged. Full CI40866229757 had2244 other private passes before stopping.

Production/CLI3 passes100.26s; receipt1793275890922734498 SHA256
`dcf3d80351d235cf50972a6a5b121ed261f981ea738f50b3506f13192c67edf2`.
Both strict Clippy modes, formatting and architecture pass. Fresh LowIR validation,
CPP object generation, separate C observer/link and actual native execution pass
in0.36s; SHA256
`8d758a9a858c92e6bb3a1f4f31cae60e9e349b262f0a9775d0dc62501e283997`.
Harness272, pinned upstream verification, source/fixture bindings and fresh
strict/capped original-core probes pass34.51s; SHA256
`8a1bcc52f23eb220e6c1c4268b696988e453e092af3eebe6e593bd50aa23203a`.

Current CLIf36deced80d0243e5e1af8c98d99fbd0cdedab1d07b9bcee7ab1f5311d278bc8
still reaches the open/exclusive/alternative pattern profile at file45:6972.
The capped observation takes10.93s,374 opens,all722 original hashes and target
settings unchanged; SHA256
`257231cf2ecf3c9efffda86a9afb258324ddbcff17a27922fced3edf800046bc`.
Strict retains the deferred expectation at68 opens; SHA256
`a823b5ef95211d577e49589f8b0a7ccb7b982d7f15c6a8055fa149b5a9a0014a`.
The finite profile remains work14,500,624/attempted1GiB/retained64MiB. These are
discovery observations, not original crate artifacts or a speedup measurement.

The unchanged pinned unlikely function is retained by exact interval/full-source
hash in the contract. Its actual ColdPath dependency belongs to the arithmetic
worker's source implementation; no stripped marker or replacement helper is used.
The endpoint consumer, complete original helper graph, generic refinements,
niches, metadata, all PA release requirements and original core/alloc/std builds
remain required. No whole assignment is released here. Full qualification of
this new checkpoint remains pending.
