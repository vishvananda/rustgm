# Architecture-specific feature requests

`contract-v1.json` and `cases-v1.tsv` declare56 authored programs in Rust2021
and2024, pinned Rust1.98.0, Linux x86-64, panic abort and an explicit Allow lint
cap. All matrix and native inputs have declared SHA-256s. Independent Rust
qualification includes112 source outcomes and native compile/execution.

The17 non-x86 language requests are known pinned Rust features. On this course's
fixed x86-64 target they have no attribute consumer, enable no CPU features and
need no ordinary binding/library-policy owner. Unknown and misspelled requests
remain errors. `naked_functions_target_feature` is a separate selected-target
contract; it is deliberately excluded from the foreign-architecture class.

An active attribute always uses the selected target's feature table. Every
foreign request has a rejecting foreign-attribute counterexample. Shared `aes`
is instead interpreted as a valid x86 feature. All foreign requests together
cannot supply the missing avx10, clflushopt, movrs, rtm or amx gate; an actual
matching x86 request can. Cfg-inactive declarations remain inactive. The matrix
also retains const, raw-pointer safety, owner and unrelated macro-allowance
checks. Existing literal-span permission and imported-policy requirements remain
owned by the earlier target-feature packet.

The native program uses the15 original core requests, a real baseline-sse2
call and fixed target cfg queries. Independent Rust and RustGM/CPP must print
`7\n9\n`. `native-control.rs` has the same body and attribute without those root
requests. Qualification must compare its LowIR with the annotated program and
the immutable predecessor under the same explicit crate identity. This proves
ordinary scalar execution and request/configuration separation; reached SIMD,
assembly, target-specific intrinsic instructions and wider transport remain
PA41/28/29 requirements. No backend extension or whole assignment is established.

Primary sources at88d9e12ae178fab0fb5cc050a94da85685d449ea are
`compiler/rustc_feature/src/unstable.rs`,
`compiler/rustc_target/src/target_features.rs` (`rust_target_features` selects
the architecture table), and
`compiler/rustc_codegen_ssa/src/target_features.rs` (attribute validity, gate,
implied features and ABI checks). The existing typed x86 table remains the
consumer; request recognition must never merge foreign architecture tables.
