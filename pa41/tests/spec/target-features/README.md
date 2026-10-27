# Target-feature source contracts

Authored Rust1.98.0 x86_64 reductions, pinned at
88d9e12ae178fab0fb5cc050a94da85685d449ea. Primary seams are
compiler/rustc_codegen_ssa/src/target_features.rs, compiler/rustc_middle/src/ty/context.rs
and compiler/rustc_mir_build/src/check_unsafety.rs; preserve NOTICE.
Predictions precede observation and all disagreements remain recorded. These
metadata-only tests never execute AVX and grant no native/backend capability.
Caller body features and ASM baseline-union-body features are distinct contracts.
