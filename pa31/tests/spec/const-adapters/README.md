# Actual source callable adapter contrasts

This draft binding packet contains six required positives and four language
rejections. The [ordinary PA25 packet](../../../../pa25/tests/spec/const-adapters/README.md)
owns the underlying const closure language semantics. These counterparts add
actual source `FnOnce`, `FnMut`, `Fn` and `Destruct` role declarations, preserving
the existing PA25 `const-closures/source-execution-v2.rs` support prefix exactly.
All sources are authored reductions; no original core input is modified.

Pinned Rust independently accepts all six positives and rejects all four
opponents during object compilation. In immutable RustGM CLI `8b73f15a`,
`self.rs`, `binary.rs` and `forward.rs` reproduce core's invalid canonical
trait-selection diagnostic. A temporary diagnostic-only build confirms its
underlying cause: const closure defining obligations require a scoped
continuation. That instrumentation was removed; the semantic fence remains.

The direct conditional call, ordinary adapter and const factory pass source
type analysis. Their generic body inspection views still reject separately;
they do not establish native or complete library compilation. Ordinary-bound,
ordinary-closure, wrong-output and runtime-creation opponents retain their
independent rejection contracts. The runtime-creation input was refined to keep
the actual callable hierarchy unchanged; both Rust and RustGM now diagnose
creation outside a const context. Earlier exploratory bytes and observations
remain under `obj/parallel/const-adapter/`.

`manifest.json` declares every source before correction.
`independent-review.json` binds independent outcomes, actual diagnostics and
candidate limitations. It is an observation receipt, not a course text oracle.
Independent commands use the PA25 packet's pinned flags plus `--crate-type=lib`.
Candidate analysis uses `--source-core minicore@adapters --crate-name minicore
--crate-disambiguator adapters --emit-types`. There is no generated text oracle,
CPPGM execution claim, component release or whole-PA activation in this packet.
