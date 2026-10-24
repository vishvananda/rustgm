# Source panic builtin contracts

These preserved preimplementation hypotheses belong to LIB-013. They were
declared when the unchanged pinned core probe rejected its first builtin
declaration, `#[rustc_builtin_macro(core_panic)]`, at byte 50. See
[the local qualification](../../../../docs/pa31-source-panic-delegation.md) and
[the live blocker tracker](../../../../docs/library-build-blockers.md) for current
results: local delegation is development-qualified, and unchanged core now stops
at the `panic_internals` allowance on `assert_eq`. Foreign macro transport and
the full edition contract remain required.

The pinned Rust 1.98.0 implementation in
`compiler/rustc_builtin_macros/src/edition_panic.rs` delegates `core_panic` and
`std_panic` to `$crate::panic::panic_2015!` or `panic_2021!`; `unreachable`
delegates to its corresponding handlers. It forwards the original token stream.
`use_panic_2021` walks past expansions carrying the exact `edition_panic`
internal allowance, then uses the first remaining expansion's edition. This is
not always the physical caller edition or the builtin declaration edition.
`rustc_expand::ExtCtxt::std_path` constructs a definition-site `$crate` path.

`contract.json` manifests every source, its hash, expected acceptance and native
answer before execution. Ten single-source cases run in 2018, 2021 and 2024;
nine cross-crate contexts oppose provider and consumer editions, ordinary
wrappers and wrappers with the precise allowance. The handlers deliberately
return distinguishable integers instead of aborting, so execution identifies
the actual selected macro, defining crate and preserved nested input tokens.
They are tests of delegation, not a substitute panic runtime.

The original hypothesis files remain unchanged; `review.json` binds the
subsequent independent observations. Record disagreements without overwriting
the original hypotheses. The complete compiler fix must preserve
source-role authorization, typed capture/token provenance and stability checks;
source-only builtin roles must not enter the bootstrap macro prelude. Retain
existing forged/duplicate claim controls and test missing handlers at the actual
resolution boundary. A missing handler must not execute the authored builtin
stub or choose the other edition's handler.

PA31 remains planned. These contracts do not activate a grading suite or supply
text oracles. Original core must be retried after the complete fix, and every
new failure gets its own owning-assignment reproduction.
