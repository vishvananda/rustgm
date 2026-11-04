# LLVM declarations and scalar bit operations

Active declarations: `contract-v3.json` and `cases-v2.tsv`. All 43 authored
source outcomes are declared before qualification in Rust 2021 and 2024, using
pinned Rust 1.98.0, Linux x86-64 and an explicit Allow lint cap. V2 adds signed
execution to the original corpus; V3 adds mixed-sign results and a type alias.
Earlier source bytes and outcomes remain unchanged.

The primary Rust feature-gate implementation checks the first `link_name` of an
actual foreign function or static, at the item's span. Macro permission belongs
to that item's origin. An allowance on a forwarding macro cannot authorize an
item captured from its caller; a generated item may use metadata supplied by the
caller. These are independent of the foreign function's unsafe/const contract.
Inactive declarations and ignored attributes on Rust definitions remain distinct.

The unsigned, signed and mixed-sign native programs each declare 14 `ctpop`,
`bitreverse` and `bswap` operations over 8/16/32/64/128-bit integers (`bswap`
starts at16 bits). Each performs56 comparisons, including zero, all ones, the
high bit, and values obtained at runtime from the independent C observer.
Their expected output and the ordinary control's output are `1\n`.
The grouped maintainer `llvm_calls` integration executes all four programs with
both RustGM/CPP and independent Rust.

Rust names/types retain their ordinary meaning; LLVM uses signless integer
widths. Alias normalization and equal-width argument/result views must not be
mistaken for unsupported operations. The typed kernels use logarithmic bit-group
transforms over the existing LowIR vocabulary; the C++ backend is unchanged.
See [LLVM's bit manipulation reference](https://www.llvm.org/docs/LangRef.html#bit-manipulation-intrinsics).

Other LLVM operations remain declared requirements, including floating point,
zero-poison/immediate operands, vectors, target-specific operations, imported
identity and address profiles, and original library consumers. Unknown unused
declarations may be retained; a reached unsupported operation or LLVM storage use
must fail before it becomes an ordinary external C symbol. Backend signature
diagnostics are not classified as Rust source-language rejection. No whole PA28,
upstream stdarch implementation, or original core/alloc/std artifact is established.
