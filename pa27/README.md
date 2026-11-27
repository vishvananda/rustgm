# RustGM Programming Assignment 27 — Unsafe language

Extend the ownership checks from [PA14](../pa14/README.md), loan checks
from [PA16](../pa16/README.md), and closure captures from
[PA24](../pa24/README.md) with unsafe operations. Use Rust 1.98.0, edition
2024, on Linux x86-64 with checked arithmetic and aborting panics.

[The PA27 manifest](tests/manifest.json) covers unsafe permissions, raw and
packed places, static storage, and runtime unions. PA45 carries the broader
constant pointer and byte provenance, interior mutation, and wide-value
validity requirements.

## Unsafe contexts and diagnostics

- Track unsafe contexts lexically while checking typed bodies. An unsafe block
  supplies permission to descendant expressions and closures. A nested item
  starts its own context. A call to an unsafe function requires permission,
  including selected methods and calls through function pointers.
- Apply the edition's `unsafe_op_in_unsafe_fn` policy to operations in unsafe
  functions. Lexical lint levels for this policy and `static_mut_refs` can
  reach nested items independently of unsafe permission. An explicit unsafe
  block satisfies the unsafe-operation lint. Descendant `allow` or `warn`
  cannot weaken `forbid`.
- Emit warnings with their original source or expansion location. Preserve
  warnings even if a later error rejects the program. Report a use only once.
  Where allowed by a lint, continue normal type, move, initialization, loan,
  layout, and constant eligibility checks.

The [raw dereference in an unsafe function](tests/spec/contexts/raw-fn-body.rs)
and its [expected diagnostic](tests/spec/contexts/raw-fn-body.rs.ref.stderr)
illustrate a passing program with a warning.

## Raw and packed places

Use different typed operations for raw-pointer and reference dereferences.
Check the pointer value before accessing its referent. Unsafe permission is
required for raw loads, stores, and field projections. Enforce pointer
mutability, Copy requirements on by-value reads, and alignment for references.
A plain raw address creates no reference loan or reference alignment promise.
The direct `&raw const *p` and `&raw mut *p` forms can preserve the pointer
without reading its referent.

Access to mutable statics requires unsafe permission. Under edition 2024's
default lint settings, forming an explicit or implicit reference to one is
denied. A raw address is a distinct operation. When an assignment overwrites
an initialized mutable static, run the applicable destruction before storing
the replacement.

Keep packed by-value operations separate from creating references to packed
fields. Form a reference only where its required alignment is guaranteed.
Capture paths stop before a packed field, at a union, or before a raw-pointer
dereference. Retain the full path for checking the actual access. Writing a
raw pointee can capture the pointer by read. Writing the pointer's own slot
instead requires a mutable capture.

## Unions

Lay out each union field at offset zero. Initialize exactly the selected field
and share the backing storage across field views. Reading or borrowing a field
requires unsafe permission. Assigning a field follows normal assignment rules.
Different fields overlap for loans, while disjoint subfields of one field can
retain their separate places. A move from a non-Copy field consumes the
containing union. Use the canonical `ManuallyDrop` rules for eligible fields,
and run an explicit union destructor without automatically destroying the
overlapping fields.

## Run the tests

Use the PA11 `rustgm --emit-lowir` interface. Run the PA27 test target from
the repository root.

```sh
make -C pa27 test
```

Use the [PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
for executable cases. The manifest declares input preludes where needed, so use
the test target for exact results on those cases.
