# Nested projected-size proof reuse

`P<T>` stores `<T as Tr>::Out`; `impl<T: ?Sized> Tr for T` returns T.
The earlier implementation repeatedly rebuilt completed projected-size suffixes
in fresh builtin proofs, starting 2^d-1 normalizations. Its outer own counter
also omitted nested proof work, so the local budget did not bound that repeated
work. The prep, accounting and reuse changes form one publication checkpoint;
the accepted-to-budget-failure intermediate is not published alone.

A private saturating nested ledger occupies existing tail space in inference
contexts and selection queries. Each completed child contributes its total once;
repeated builtin calls charge only their new delta. Reported own counters keep
their meaning. The original one-million-work allowance remains, and actual
nested owners are guarded at 93 with a named propagated depth error. Live preload,
saturation, error propagation and fixed-layout checks pass.

Immutable physical properties descend only along the current alias's reachable
projection-free unshifted graph. Actual completed physical sizes return through
selection evidence, normal forms and the cursor to the owning size query. Both
transfers require exact implementation availability and environment identity or
placeholder-handle identity; provisional or free-region subjects are excluded.
Structural properties never satisfy a trait goal. Source formation and selected
trait/lifetime evidence remain mandatory. Distinct typed map namespaces keep
structural data out of completed trait lookup without growing hot records.

Private accepted-result observations at depths 1, 2, 4, 10, 13, 31 show exactly d
normalizations, exact nested-ledger conservation, and quadratic total work. The
affine-work target was disproved and remains a separate performance follow-up.
The measured fixture envelopes through 31 are `80*d*d` for true and `180*d*d`
for false results. The false path retains its separate metadata/tail checking.
These finite controls do not prove an asymptotic bound for all Rust.
All eight independently accepted source reductions are in PA31's
nested-projection-work packet, which lists each frozen input hash. Source depth 32
is accepted after reuse; the withdrawn nominal-depth 32 rejection is not imposed.

Remaining known gaps: the higher-ranked canonical-selection family
`Field<U>(for<'a> fn(P^d<&'a U>), u8)` at depth 2 and above still rejects valid Rust. Body
inference cursor consumers bound child work locally but do not yet sum it in
their owning work ledger. Dispatch normalization needs the same ownership audit.
No frontend worker thread or CLI stack change is included. PA31 stays planned,
and original core/alloc/std artifacts remain required.

## Qualified development evidence

The final development CLI is `f5b5db49798fb73c7141c414b3ab97a0bc9d727c1e33afbe2c85584a553724fe`.
Production fastcheck `1794273825146180434` passes strict Clippy, formatting,
architecture and the exact binder regression. Private source/status joins
verify 130 distinct passing cases: 124 unchanged cases from `1794273169797051159`,
the additional focused cases from `1794272971116434729` and the corrected work
case in `1794273595158026494`. The two earlier batches remain failed because their
new work assertions were wrong; no failed receipt is relabelled. The compiler
source hashes match, and the temporary golden hook is independently reconstructed.
The final hook reproduces all 8,356 golden rows byte-for-byte, SHA-256
`6d634d2fe14901c83dc3099fa2eefb8971c4b5d14ee18f4c36b7f67e8adcd870`.

| Depth | Normalizations | Placeholder true work | Placeholder false work |
| ---: | ---: | ---: | ---: |
| 1 | 1 | 79 | 81 |
| 2 | 2 | 271 | 349 |
| 4 | 4 | 1,053 | 1,863 |
| 10 | 10 | 6,519 | 14,667 |
| 13 | 13 | 11,007 | 25,713 |
| 31 | 31 | 62,505 | 157,005 |

Both 74 twins and 336 survey inputs retain complete status/stdout/stderr equality.
All 313 harness tests, 465 registered roots, 44-stage layout, 128 pinned upstream
fixtures and 126 pinned Reference files pass. No upstream bytes changed. Generic
depths 31/32 and closed 12/31 pass on the default 8 MiB development CLI stack.
A process-stack bisection for generic 31 brackets its minimum at 448–512 KiB,
with 64 KiB resolution; low-stack signals are measurement observations only.

Immutable same-dev-fast binaries `43818e28` and `f5b5db49`, frozen inputs, A/A
calibration and two paired ABBA cycles retain 50 raw observations with complete
output equality. Candidate/base geometric ratios at depths 4/8/12 are
0.838/0.233/0.0252; ordinary 74/336 controls are 0.999/1.003. A/A ratios are
1.017/1.050/1.034 and 1.009/0.978 respectively. These are development-profile
measurements, with no optimized or whole-core speed claim. The original release
baseline build took 7m20s; a fresh matching optimized candidate build, its stack
probe and release A/A+ABBA remain explicitly pending outside the five-minute
local feedback loop. Focused passing evidence permits development integration;
this is not completion of that performance/release contract.

The unchanged 722-input original-core census takes 79.14s, opens 375 sources and
checks 42,310 declarations with none failed/blocked; bodies remain 0. Its TSV is
byte-identical to census25 (`ae73984639563236d2aa464043008f8763f4e6783567da25d887d252263cefb1`),
still stopping at AtomicPtr::from_mut's closed projected-field layout requirement.
All source hashes, profile and limits are unchanged. It is analysis-only
discovery, not an original library artifact or a qualification pass.

Raw receipts, exact source bindings, all failed predictions, fixture oracles,
stack observations and timing records remain under
`obj/lane-b10/{cp2,cp3,prep-single}` in the existing pa26-work tree. The full
scheduled/manual CI gates remain separate.
