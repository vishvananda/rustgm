# PA4 fixtures

`manifest.json` declares outcomes independently of oracle generation. Every
source here is an expression fragment, including block expressions. The rustc
checker wraps it with `fn main() { let _ = ...; }`, or the explicit manifest
prefix/suffix needed for declarations, return types or inference.

The clusters cover precedence, postfix/aggregate forms, statements, control
flow, macro captures, rejection, and deliberately deferred semantic errors.
Any disagreement with whole-program rustc acceptance has a written reason;
these cases do not count as equivalent language acceptance. All fixtures are
original course examples. The owning grammar is the vendored Rust Reference.

Use `make test-pa4` to compare committed oracles and `make ref-test-pa4` only
for an explicitly reviewed contract change. `dev/tests/expressions.rs` contains
hand-checked structural and resource-bound assertions beyond text generation.
