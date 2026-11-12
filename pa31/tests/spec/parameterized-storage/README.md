# Parameterized projected storage (source core)

Fifteen no_core source-core libraries reduce pinned core's `AtomicPtr::new`
6835 (`sync/atomic.rs:1478`) and `into_inner` 6842, checked against rustc 1.98.
Each case's first line names its prefix, which runs through the Fn traits of
`pa31/tests/spec/default-const-closures/count.rs`. Each case then adds a reduced
`AtomicPrimitive`, `Align8` and `#[repr(C)] Atomic<T>` whose field is
`UnsafeCell<T::Storage>`.

`transmute(p)` from `*mut T` to `Atomic<*mut T>` reads the field
`<*mut T as AtomicPrimitive>::Storage`. The implementation for `*mut T`
normalizes it to `Align8<*mut T>` without any caller premise, so both sides are
eight bytes.

- `ptr-new` and `ptr-into-inner` are core's forms, `ptr-runtime` is the same
  transmute outside a const fn, and `ptr-no-cell` drops the `UnsafeCell`;
- `nested-new` reads the field through a `#[repr(C)] Outer<T>`, and
  `marker-premise` adds a premise of a trait without associated types;
- `ptr-direct`, `usize-new` and `unshadowed-value` are controls;
- `param-storage` transmutes into `Atomic<T>` under `T: AtomicPrimitive`, whose
  `T::Storage` is dependently sized (E0512);
- `size-mismatch` normalizes `*const T` storage to sixteen bytes (E0512);
- `shadowed`, `sibling-shadowed` and `nested-shadowed` add
  `where *mut T: AtomicPrimitive`. rustc prefers that premise over the
  implementation, so the projection stays rigid (E0512). `sibling-shadowed` is
  a method of `new`'s own impl, after `new` has used the normal form;
- `shadowed-value` returns the same rigid projection as a value (E0308).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run; both are recorded in `outcomes.json` (15/15 agree). Two declared
probes on which RustGM keeps a conservative rejection are listed there and
excluded. It keeps Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`. `dev/tests/atomic_storage/parameterized_fields.rs`
checks every case. See `docs/pa31-parameterized-storage.md`.
