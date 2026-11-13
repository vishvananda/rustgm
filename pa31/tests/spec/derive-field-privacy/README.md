# Derived tuple field privacy (source core)

Seventeen no_core source-core libraries check field privacy in derived
`PartialEq` and `Debug` impls against rustc 1.98. Pinned core derives both on
tuple structs whose fields are private to their own module, such as
`array/mod.rs:182`'s `TryFromSliceError(())`. Each case's prefix is the first
356 lines of `const-parameter-derive/actual-source.rs`. The six `debug-*`
cases then add authored formatter support with `loop {}` bodies and no runtime
claim. That support covers shared reference unsizing, `Debug` for `&T`, `u8`
and `()`, and the derive's `debug_tuple_field1_finish`,
`debug_tuple_field2_finish` and `debug_struct_field1_finish` helpers.

rustc spans a derived tuple index with its field
(`Ident::from_str_and_span(index, field.span)`), so `self.0` in the derived
impl is checked from the module that wrote the field.

- Positives:
  - `tuple-submodule`, `tuple-two-fields`, `tuple-nested`,
    `tuple-mixed-visibility`, `tuple-generic`, `tuple-macro-defined` and
    `tuple-eleven-fields` derive `PartialEq` on private tuple fields in a
    module. The forms are one or two fields, a nested module, mixed
    visibility, a generic field, a `macro_rules` item and eleven fields;
  - `debug-tuple-submodule`, `debug-unit-field` (core's form),
    `debug-two-fields` and `debug-eq-both` derive `Debug`, the last with
    `PartialEq`;
  - `record-submodule`, `debug-record-submodule` and `tuple-root` are
    controls: named fields and a root-module item.
- Negatives:
  - `outside-access`, `manual-impl-outside` and `debug-outside-access` read
    `.0` from outside the module, by a function and by manual `PartialEq` and
    `Debug` impls (E0616).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 17
declarations agree. `obj/parallel/derive-privacy/core/declared.json` records
four batches, with one malformed batch and two malformed diagnostics. The
cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/tuple_index_privacy.rs` checks every case's typing through the CLI.
It runs prepared lowering for every accepted case, and checks bounded work.
See `docs/pa31-derive-field-privacy.md`.
