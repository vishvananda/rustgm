# Source SIMD representation and ABI reducers

These are authored reductions of the Rust1.98.0 representation contract at
88d9e12ae178fab0fb5cc050a94da85685d449ea. The real x86 SIMD declarations come
from library/stdarch/crates/core_arch/src/{x86/mod.rs,macros.rs}; retain NOTICE.
The native source keeps the genuine source language-trait foundation already
qualified by the source-operator fixtures and appends ordinary nominal SIMD
structs and identity functions. It does not replace the original stdarch graph,
its derives, or SIMD intrinsic bodies with a library factory.

Predictions are frozen before observations. Version2 corrects token spacing in
separate native/layout files; original version1 source and hypotheses remain
unexecuted. Metadata, object and C-observer outcomes are independent Rustc
observations, not evidence that RustGM can execute these inputs. The candidate
must later lower the actual source through the checked pipeline and locked
LowIR object bridge; hand-authored LowIR cannot stand in for failed source.
