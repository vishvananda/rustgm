# Source shift and bitwise-or execution requirements

These six inputs retain the frozen source bytes from worker checkpoint
`4572314fc5573ff614bb9dfee8c0a20ef2a5b214`. Their sizing/Clone/Copy foundation
and Shl/BitOr trait blocks come from the pinned Rust sources; primitive impl
reducers are authored. Preserve the Rust attribution and repository NOTICE.
The original trait source is `library/core/src/ops/bit.rs` at revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`.

`fence-contract.json` lists every input, its source hash, actual independent
Rust outcome and temporary candidate diagnostic. The two valid Rust programs
remain required positive implementation cases. Their current unsupported result
does not count as implementing operators or passing the final PA31 contract.

The source compiler must resolve the genuine trait, Self/Rhs/Output, method,
implementation and const effects before scalar emission. Ordinary overloads and
explicit calls retain real source method behavior. Until that hook exists, the
candidate rejects reached source shifts and bitwise-or instead of borrowing
primitive execution from the older bootstrap path. Missing source traits also
reject. Assignment operators have distinct trait identities and remain separate
required work. Earlier ordinary/bootstrap operator behavior is preserved.

The integrator authenticated twelve retained metadata observations across
editions 2021/2024 and their twenty-four raw streams. This checkpoint runs no new
native comparison and generates no text references. PA31 remains planned.
