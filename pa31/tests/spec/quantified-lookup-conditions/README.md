# Method lookup through callable sugar

Twenty-two no_core libraries, checked against rustc 1.98. They isolate census
23's record at core's `drain::Drain::new(&mut me, &mut f)` in
`<[T; N]>::try_map` (`array/mod.rs:643`). `Drain::new` is a `const unsafe fn`
of `impl<'l, 'f, T, U, F: FnMut(T) -> U> Drain<'l, 'f, T, F>`, beside the
trait method `ZipImpl::new`. RustGM reported "quantified clause requires a
scoped consumer" there.

Callable sugar opens a binder even when it names no lifetime. A binder that
quantifies nothing and captures nothing outside it is an ordinary lookup
condition, and it is the same condition as its flat twin
`F: FnMut<(T,), Output = U>`.

- **Accepted:**
  - record 1's shape;
  - the same call from a non-const caller, with a non-const `new`, with a safe
    `new` and with by-value arguments;
  - the method-call form;
  - core's `const impl Drop for Drain`;
  - a default method of a trait with a sugar supertrait;
  - an inherent `new` that does not apply, falling back to a trait `new`;
  - three controls: a unique inherent candidate, an ordinary `Call<T>`
    bound and a bound-region sugar bound with no trait `new`.
- **Accepted by rustc, still rejected by RustGM (known rejects-valid):** six
  bound-region shapes, `for<'a> Tr<'a>` and `FnMut(&T)`, beside a trait method
  or in the method-call form. Among them is core's `slice/iter.rs` `Split::new`
  shape. They keep "quantified clause requires a scoped consumer" until lookup
  opens a quantified lifetime.
- **Rejected:** an inferred callable type that does not implement the sugar
  bound (E0277), and an explicit non-callable `F = u8`, with and without a
  trait `new`, including under `FnMut(&T)` (E0599).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of these bytes. Both are recorded in `outcomes.json` (22/22 agree).
Each file is byte-identical to one of census batch 21's declared exploration
probes (`obj/parallel/census-batch21/probes/`).

The prefix is `spec/function-pointers/foundation.rs`. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/empty_binder_lookups.rs` checks every case. It also
checks that a sugar bound's lookup has exactly its flat twin's conditions,
in the same order, and the same method selection work.
See `docs/parallel/census-batch21.md`.
