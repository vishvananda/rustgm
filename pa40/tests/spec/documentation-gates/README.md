# Documentation feature gates

This unpublished PA40 packet qualifies the compilation requirements of `doc_cfg`
and `doc_notable_trait` against pinned Rust 1.98.0. `contract-v2.json` and
`cases-v2.tsv` are active: 73 original fixtures run in the 2021 and 2024 editions.
The contract records primary compiler-source hashes, fixture hashes, declared
outcomes and the native C observer. These are course-authored fixtures.

`doc(cfg(...))`, `doc(auto_cfg...)` and `doc(notable_trait)` require their genuine
crate feature or the actual attribute constructor's macro allowance. An Allow
lint cap does not waive these permissions. A captured whole item keeps caller
permissions; forwarded meta/literal payloads retain the introduced attribute's
constructor authority. Nested unstable cfg query names additionally require their
own genuine feature or token-origin allowance.

The documentation predicate is syntax and authority metadata. It cannot disable
an item, satisfy a const requirement, enable an unsafe call, authorize f16 or alter
target configuration. Reuse the existing cfg grammar in validation mode without
looking up target/custom values or applying a result to configured owners. This
profile covers that parser's boolean/name/value/all/any/not grammar. Broader cfg
version/compact-target syntax remains part of the owning cfg assignment work.

Malformed cfg predicates and string suffixes remain hard errors. In contrast,
malformed auto_cfg and notable_trait options produce optional documentation lints
under the pinned compiler. Their rendering, full target checks, documentation
output and imported metadata remain required. RustGM's uncapped active option
profile currently stops explicitly; required strict successes are preserved as
unfinished boundaries, not reclassified as invalid Rust. Whole PA40 and original
core/alloc/std artifacts are not established by this packet.

The C observer calls a function annotated with false documentation predicates and
an actual notable-trait implementation. It must print `7 9`; documentation must
not remove either exported callable. Its attribute-free control must execute the
same program. No Rust frontend other than RustGM may provide candidate lowering;
CPPGM remains only the LowIR-to-native backend.

The v1 independent oracle disproved a soft string-suffix prediction for doc(cfg).
The active v2 prediction retains the hard cfg string rule and adds independent
unstable-query and forwarded meta/literal cases. Earlier files are retained as
explicitly superseded investigation evidence, not passing predictions.
