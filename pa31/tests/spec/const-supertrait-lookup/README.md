# Method lookup through `[const]` supertraits

Twenty-two no_core libraries, checked against rustc 1.98. They isolate census
22's record at core's `<*const VTable>::cmp(&self.vtable_ptr(),
&other.vtable_ptr())` (`ptr/metadata.rs:251`). That is a call of a method of
`const trait Ord: [const] Eq + [const] PartialOrd<Self>` through the non-const
`impl<T: PointeeSized> Ord for *const T`. RustGM found no applicable trait
method there.

Method lookup ignores constness. The method resolves through an ordinary impl
or an ordinary `T: Cmp` premise. A `[const]` supertrait of its trait is
required only when the call is in a const context, and there an ordinary impl
or premise does not satisfy it.

- **Accepted, in the type-qualified and method-call forms:**
  - record 1's shape;
  - the same call under an ordinary `T: Cmp` premise;
  - the non-const hierarchy;
  - a const trait without `[const]` supertraits;
  - const impls, from ordinary and const fns;
  - `const fn` callers under `T: [const] Cmp`;
  - four binder controls with `for<'a>` bounds on trait and inherent methods.
- **Rejected:**
  - the call in a const fn, in a const item and under an ordinary premise in a
    const fn (E0277);
  - the `[const]` hierarchy's `cmp` beside a plain trait's `cmp`, and the
    non-const twin of that pair (E0034).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of these bytes. Both are recorded in `outcomes.json` (22/22 agree).
Census batch 20's exploration probes
(`obj/parallel/census-batch20/probes/`) had other bytes.

The prefix is `spec/function-pointers/foundation.rs`. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/supertrait_lookups.rs` checks every case, and that a
`[const]` hierarchy's lookup pays exactly its plain twin's work.
See `docs/parallel/census-batch20.md`.
