# Forwarded const arguments

These PA18 cases preserve real macro capture grammar through const-generic
arguments. The matching captured literal AST supplies integer, boolean,
character and signed syntax; expression/block fragments retain their actual
literal or block root. No rendered spelling is reparsed or promoted into a type.
Ordinary constant evaluation, argument kind/type checks and instance proof still
apply. Repetition and double forwarding retain the opaque fragment identity.

Pinned Rust 1.98 observations were frozen before implementation and run in both
2021 and 2024. All actual results agree across these editions. Literal/signed
expression fragments, plain block fragments, captured expression blocks and
unsafe blocks are accepted; ordinary type contexts, malformed separators,
unbraced arithmetic and invalid constant values reject.

Two independent hypotheses disagreed with observations and remain retained:
`opaque-expression.rs` predicted acceptance, but unbraced `1+2` requires braces;
`opaque-unsafe-block.rs` predicted rejection, but the genuine block is accepted.
These observations determine the manifest outcomes, not replacement fixtures.
The original predictions/raw receipts are retained in the maintainer evidence.

`opaque-runtime-expression.rs` remains a rejection. Opaque expression paths are
not promoted into ambiguous type argument paths by this checkpoint; their
complete const-argument classification remains required separately. Explicit
ordinary/raw paths retain their existing namespace resolution behavior.
