# Forwarded literal and block const arguments

The original `core::ptr::swap_prefix` macro forwards a captured literal into
`swap_chunk::<$n>`. The parser now classifies that opaque fragment from its
existing captured AST before entering the ordinary const-argument expression
producer. It accepts actual literal, negative-literal and supported block roots;
it does not reparse text, manufacture an expression, or grant semantic proof.
The existing opaque import preserves capture identity, edition and spans.

The owning PA18 cluster contains eleven positive and eight negative programs.
Four private controls exercise those programs through actual source checking,
original/instance proof and lowering in both editions, repeated capture import,
and the verbatim original-core macro reducer retained under PA31. Three existing
frontend controls retain malformed-handle, repeated-identity and edition checks.
Wrong constant types and negative unsigned arguments still reject semantically;
unbraced arithmetic and type-position opponents remain rejected.

The primary pinned parser references are `rustc_ast/src/token.rs`'s
`can_begin_const_arg`, `rustc_parse/src/parser/path.rs`'s
`expr_is_valid_const_arg`, and `rustc_parse/src/parser/mod.rs`'s `check_const_arg`.
Forty independent metadata observations were recorded before implementation in
editions 2021 and 2024. Two hypotheses disagreed in each edition: captured
unbraced arithmetic rejects, while a captured unsafe block accepts. All four
disagreement rows and their initial predictions remain in the raw evidence.
These are metadata observations, not native executions.

Source checkpoint `180bb0a2de26f07c8ab03f2ec06bb53be8c18222` passed all seven
selected tests. Its first gate passed metadata, format, global architecture,
production strict Clippy and private strict Clippy, then exhausted the bounded
build deadline without running tests. An authenticated missing-only continuation
retained those five actual successes and completed the build, inventory and seven
tests in 160.33 seconds. Its receipt is
`obj/qualification/fast/1793095626671275676/receipt.json`; the retained static
receipt is `obj/qualification/fast/1793095227365293103/receipt.json`.
Earlier metadata and filename-audit failures remain retained separately.

At repetition widths 1/8/32/128, the actual argument-node counts are 1/8/32/128,
the captured literal is parsed once, and syntax-node counts are 21/70/238/910.
The new classifier uses bounded AST access and adds no cache or owner. These
structural observations do not claim a wall-time performance result.

The integrator authenticated all source fingerprints and retained worker raw
evidence before the normal merge. Its production gate passed39 frontend/CLI
controls, full PA5/7/8 text, strict Clippy, formatting and architecture in93.63s
(`obj/qualification/fast/1793096173262412371/receipt.json`). A separately frozen
native plan then checked all19 outcomes and executed all11 positive programs
with both pinned Rust and the locked CPPGM backend:22 executions and74 command
observations passed in50.48s. The scoped course writer generated exactly the19
new reference triplets and its provenance batch. Full PA18 text then passed
498/498 in3.04s; harness/upstream checks passed in13.77s. These observations are
under `obj/parallel/forwarded-const-integration-v1/`.

Immutable CLI `fac1e54d` also clears the original swap_prefix parser rejection.
The unchanged-core probe verifies all722 source pins, opens371 files and stops
after7.71s on a captured cfg string in sync/atomic.rs311 (LIB-044). There is no
complete core semantic/export result or crate artifact. The manifest's305 native
execution selections describe the full release inventory; this checkpoint ran
the11 new positive programs, not that entire native inventory.

Opaque expression paths, gated const blocks and broader expression promotion
remain separate grammar/representation work. Ordinary raw-path resolution is
unchanged. The PA31 reducer proves expansion/parser acceptance, not the full
swap implementation's borrow or native behavior.
