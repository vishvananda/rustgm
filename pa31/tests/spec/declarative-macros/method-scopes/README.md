# Trait lookup scope boundaries

These eleven original Rust programs distinguish item scopes from ordinary value
scopes for scoped method identifiers. PA31 owns the macro hygiene; PA19 owns the
shared candidate-selection machinery. Both immutable contracts and the reviewed
independent observations are retained. No graded PA31 manifest is activated.

Pinned Rust 1.98.0, edition 2024, x86-64 Linux, panic abort and overflow checks
produce 22 metadata/object observations and six native programs. The first
contract incorrectly predicted rejection for two parenthesized expression tails.
Their acceptance is preserved; the second contract adds distinct brace/semicolon
opponents and native execution for those already accepted sources. It reuses the
four unchanged language observations, rather than compiling them again.

A method literal can see a trait in the initial caller module and another at its
macro definition, producing ambiguity. A local item or statement macro creates
an intervening block module: lookup can visit that block then rewind to the
definition without searching its caller parent. The boundary survives an empty
expansion. Parenthesized tail and initializer expressions do not create it.
A captured method identifier keeps ordinary caller lookup.

`review.json` binds all source contracts, tools, profiles and actual observations.
The original hypotheses are not edited to agree with the implementation. The
maintainer consumer tests use real resolved legacy macro invocations with supplied
opaque query contexts; they do not claim end-to-end scoped macro execution.
