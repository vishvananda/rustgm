# Static str and never transport packet

`static-str-never-outcomes.json` freezes one provider, eleven clients, their
hashes, edition, target, independent outcomes and the wire controls. They were
declared before any oracle or candidate run of these files. Later amendments
record what was observed and why.

The provider reuses `spec/inherent-methods/provider.rs` (lines 1-126)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The static-str-never
section after them is authored. It has the shape of core's
`panicking::panic` and `panicking::assert_failed`, without their attributes
and formatting:

- `AssertKind` and a `panicking` module with `panic(&'static str) -> !` and a
  generic `assert_failed<T: ?Sized, U: ?Sized>(AssertKind, &T, &U) -> !`;
- `check` and `exact`, which call `panic` from an `if` branch and a match arm;
- `expect`, which calls `assert_failed` with two reborrowed arguments;
- `name`, `pass` and `pick`, which return and forward `&'static str`
  literals, `pick` through a merge;
- `spin() -> !`.

The eight accepted clients each return 42: `check`, `untaken` (the client's
own unreached `panic` call with its own literal), `arm` (a diverging client
match arm), `name`, `literal` (a client literal through `pass`), `spin`,
`expect` and `pick`. `name`, `literal` and `pick` read the first byte of the
returned string.

Three opponents are rejected:

- `name-type` binds `name()` as `u8` (E0308).
- `panic-arity` calls `panic` with no argument (E0061).
- `str-method` calls `len` on the returned `&str`, which this provider does
  not define (E0599).

The `crate_linking::str_never_transport` integration controls generate the
outcome file's other controls in their output directory: older-profile `str`
and `!` gates on `spec/source-nominals/provider.rs`, a minimal forged-wire
provider and a 6/24/96 scaling provider.

The provider carries rustc-only shims under `--cfg rustc_oracle`. RustGM never
configures that cfg. The oracle links the provider object (`--emit=link,obj`).
