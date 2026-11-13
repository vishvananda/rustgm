# Derived field path bounds (source core)

Sixteen no_core source-core libraries check the bounds of derived impls whose
field types name a type parameter's associated type, against rustc 1.98.
Pinned core derives `Clone` and `Debug` for `iter/adapters/peekable.rs:13`'s
`Peekable<I: Iterator>` (field `Option<Option<I::Item>>`) and
`array_chunks.rs:16`'s `ArrayChunks` (field
`Option<array::IntoIter<I::Item, N>>`). It derives `Debug` for
`intersperse.rs:8`'s `Intersperse`, which also writes `where I::Item: Clone`.
`peekable-clone`, `peekable-debug`, `chunks-form` and `intersperse-form`
reduce them. Each case's prefix is the first 372 lines of
`derive-field-privacy/debug-tuple-submodule.rs`: the first 356 lines of
`const-parameter-derive/actual-source.rs` and that packet's authored formatter
support. More authored support follows: a
`clone` module re-exporting `Clone`, a `default` module, `Debug` for `bool`,
and the `debug_struct_field2_finish` and `debug_struct_field4_finish` helpers,
all with `loop {}` bodies and no runtime claim. The other cases are authored.

rustc's derive (`find_type_parameters`) bounds every field type path whose
first segment names one of the item's type parameters, unless the path is that
parameter alone. The predicates follow the written ones, one per occurrence,
with the derived trait. They are skipped where the trait is not added to type
parameters: `StructuralPartialEq` and an enum's `Default`. A qualified path
`<I as Iter>::Item` gets no predicate. rustc rejects a type macro in a field
of an item with type parameters.

- Positives:
  - `peekable-clone`, `peekable-debug`, `chunks-form` and `intersperse-form`
    reduce core's forms;
  - `reference-field`, `enum-variant-field`, `copy-projection`,
    `partial-eq-projection` and `two-parameters` vary the field's position,
    the item and the trait;
  - `default-enum-skip` relies on the skipped bound;
  - `sole-parameter` and `type-macro-plain` are controls.
- Negatives:
  - `qualified-unbounded` (E0277): the derived `clone` needs
    `<I as Iter>::Item: Clone`, which no predicate provides;
  - `item-bound-required` and `unused-projection-bound` (E0277): a use whose
    `Item` lacks the derived trait. The second derived body does not need the
    bound, so only the predicate rejects the use;
  - `type-macro-generic`: "`derive` cannot be used on items with type macros".

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All sixteen
declarations agree. `obj/parallel/derive-projections/core/declared.json`
records three batches. Batch 1 had two malformed probes, which batch 2
corrected. Batch 3 holds ungraded lowering diagnostics. The cases keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/derived_field_bounds.rs` checks every case's typing through the
CLI. It runs prepared lowering for every accepted case, and checks each
rejection's message and site. It also checks bounded work. `reference-field`
stops in prepared lowering at a recorded gap, which a manual impl shares. See
`docs/pa31-derive-projection-bounds.md`.
