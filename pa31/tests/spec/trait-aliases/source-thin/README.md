# Source Pointee equality through Thin

These two sources extend the independently qualified reduced Pointee provider
with its actual Thin alias and a generic associated-metadata consumer. The
contract was frozen before pinned Rust 1.98.0 metadata observations in editions
2021 and 2024. The unit-metadata pointer accepts and the slice pointer rejects
in both editions; all four predictions agree with observation.

This provider deliberately retains the previously qualified `Metadata: Sized`
contract. It does not model the complete upstream Metadata bound family, prove
RustGM execution, or establish a whole-core build. The candidate must check the
alias and its actual generic body through the ordinary semantic pipeline.

Raw observations and source hashes are retained in the integrator's
`obj/parallel/source-thin-v1`. No generated course references are supplied while
PA31 remains planned.
