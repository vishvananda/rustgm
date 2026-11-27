# Target-feature source contracts

Authored Rust1.98.0 x86_64 reductions, pinned at
88d9e12ae178fab0fb5cc050a94da85685d449ea. Primary seams are
compiler/rustc_codegen_ssa/src/target_features.rs, compiler/rustc_middle/src/ty/context.rs
and compiler/rustc_mir_build/src/check_unsafety.rs; preserve NOTICE.
Predictions precede observation and all disagreements remain recorded. These
metadata-only tests never execute AVX and grant no native/backend capability.
Caller body features and ASM baseline-union-body features are distinct contracts.

`predictions-inline-always.json` declares five more reductions, frozen at
2026-11-09T12:01:50Z before any run. Pinned rustc rejects `#[inline(always)]` on
a target-feature function at the inline attribute, with no error code
(`compiler/rustc_codegen_ssa/src/codegen_attrs.rs:429-440`). Wfcheck forces that
query for every free and associated function
(`compiler/rustc_hir_analysis/src/check/check.rs:820-825, 1108-1109`), so
`inline-always.rs`, the unused generic `inline-always-generic.rs` and the
inherent `inline-always-method.rs` are all rejected. `inline-hint.rs` and
`inline-never.rs` are the accepted controls. Before this rule, RustGM accepted
the combination. Its probe ran the frontend at `868d91e1` on the same bytes: the
generic case was accepted, and the other four reached the library export fence
with their controls' text.
