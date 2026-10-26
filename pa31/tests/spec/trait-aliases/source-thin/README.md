# Source Pointee equality through Thin

These two sources extend the independently qualified reduced Pointee provider
with its actual Thin alias and a generic associated-metadata consumer. The
contract was frozen before pinned Rust 1.98.0 metadata observations in editions
2021 and 2024. The unit-metadata pointer accepts and the slice pointer rejects
in both editions; all four predictions agree with observation.

This provider deliberately retains the previously qualified `Metadata: Sized`
contract. It does not model the complete upstream Metadata bound family or
establish a whole-core build. Candidate private controls now carry the positive
through ordinary body, ownership, concrete instance and LowIR checking; the wide
pointer fails with an incompatible-type diagnostic.

`native-provider.rs` preserves the complete positive ancestor and adds a C ABI
observer entry. `native-contract.json` freezes the source and C observer hashes
and predicts `7\n` before execution. Pinned rustc passes object creation, linking
and actual native execution. The candidate now produces the same `7\n` through
the unchanged CPPGM object backend and that exact C observer; both executions
exit successfully with empty stderr. The older `thin-associated-equality.rs` program retains
its original `println!` bytes and remains outside candidate execution until that
macro path exists; it is not rewritten to make this check pass.

Raw observations and source hashes are retained in the integrator's
`obj/parallel/source-thin-v1`. No generated course references are supplied while
PA31 remains planned.
