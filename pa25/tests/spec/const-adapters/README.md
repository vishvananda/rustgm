# Conditional const closure adapters

This draft PA25 packet reduces the callable language semantics used by core's
`NeverShortCircuit::wrap_mut_1/2`. Its manifest declares six required positives
and four language rejections. The initial twenty fixtures across this packet and its
[PA31 source counterpart](../../../../pa31/tests/spec/const-adapters/README.md)
were independently checked before any compiler correction. The expanded source
packet now has 31 cases; its defining-contract checkpoint preserves these ten
ordinary requirements. All six PA25
positives also execute successfully with pinned Rust.

The ordinary adapter and const factory already pass RustGM's complete LowIR
path. The four conditional adapters remain positive requirements: current
bootstrap lookup/preparation fails before their actual closure obligations.
Matching rejection statuses behind those same setup failures do not qualify
the rejection controls. The PA31 counterpart reaches the original source-backed
const eligibility proof without this earlier setup problem. That finite source
proof now passes; actual source factory runtime/CTFE execution remains required.

`manifest.json` lists every source and its expected outcome.
`independent-review.json` records actual independent and candidate observations;
it is not a course text oracle. There are no new generated course references,
CPPGM native claims, released component or whole-PA activation here.

Independent commands use Rust 1.98.0, `RUSTC_BOOTSTRAP=1`, edition 2024,
`--cap-lints=allow`, `-Cpanic=abort`, `-Coverflow-checks=yes`,
`-Cdebug-assertions=no` and `-Copt-level=0`. Check each source with `--emit=obj`;
link and execute every declared positive, expecting status zero and empty output.
The authored reductions use installed core for their ordinary language roles.
Original upstream source and attribution bytes remain unchanged.
