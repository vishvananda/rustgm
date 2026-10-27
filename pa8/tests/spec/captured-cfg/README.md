# Captured cfg strings and atomic target gates

These unchanged inputs were observed with pinned Rust 1.98 before implementation
in both editions 2021 and 2024. Literal, raw, escaped, literal-root expression,
metadata and forwarded captures retain their actual typed string value.
Parenthesized/arbitrary expressions, byte strings, integers, booleans and
suffixed strings do not become cfg strings.

Every demanded operand is validated, including operands after a known true/false
result and a false `cfg_attr` predicate. An unselected nested attribute does not
evaluate its own cfg predicate. A preceding false `cfg` removes later attributes;
an earlier gated predicate still requires permission.

The explicit unstable probes use the case's `RUSTC_BOOTSTRAP=1` environment only
to observe pinned Rust's actual gate and target behavior. RustGM reads no such
environment in its frontend: the real source feature request or authenticated
macro allowance owns permission. Stable rustc's rejection of the feature request
itself remains in the independent evidence, separate from these language cases.

The fixed target has bare `target_has_atomic_load_store` and widths
8/16/32/64/ptr, but not 128. The paired PA3 cases compute target facts before PA8
feature authorization. Reference generation uses the reviewed course writer.
