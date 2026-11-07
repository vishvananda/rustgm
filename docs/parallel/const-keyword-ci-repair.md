# Const keyword authority assertion after creation support

Full CI42323030622 on615e72459 passes2,788 private controls and every regular public
target, then fails exactly the old syntax-only const-closure fence expectation.
The genuine enabled Rust-positive fixture is unchanged. Creation support now
reaches the bootstrap const-call check and reports non-const calls in a constant
initializer. The authority test retains an exact current diagnostic requirement
in both editions; it does not relabel the original positive as Rust-negative or
claim invocation support. Genuine bootstrap const invocation remains C/D work.

The exact private authority control passes in focused scratch execution
1794016474547976334, alongside the three source-header accounting controls. That
whole gate fails three unmerged Field/query checks, so its partial outcomes are
kept separate. The parser/feature fixtures and production authority path used by
this assertion are unchanged. This publication changes only its obsolete test
expectation and explanatory documentation. Complete successor CI remains due.
