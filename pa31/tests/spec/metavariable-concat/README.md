# Metavariable concatenation frontier

LIB-039 owns the pinned `${concat(...)}` template operation at core's unchanged
`fmt/num.rs:149`. PA7 owns its template representation, PA8 identifier emission
and hygiene, and PA31 the independent `macro_metavar_expr_concat` feature gate.
This feature is independent of `macro_metavar_expr`; accepting the declaration
alone does not establish generated-name resolution.

The positive source invokes the macro and resolves the generated function. The
opponent removes only its feature request. Both pinned editions accept the first
and reject the second with E0658. The current qualified compiler rejects both
at template parsing with `expected macro token`. Predictions, input hashes and
six raw observations are retained in the contract and integration evidence.
No runtime or original-library artifact is claimed. The complete template must
remain valid before later cfg processing; skipping unknown tokens is not a fix.
