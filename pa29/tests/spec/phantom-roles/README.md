# PhantomData role transport packet

`phantom-role-outcomes.json` freezes one provider, nine clients, their
hashes, edition, target, independent outcomes and the generated and wire
controls. They were declared at 2026-11-08T05:59:47Z, before any oracle or
candidate run of these files. The integrity controls of the integrator's
option (b) were declared at 06:03:14Z, also before any run.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The section after them
is authored. It declares core's
`#[lang = "phantom_data"] pub struct PhantomData<T: PointeeSized>;`, the
function `marker<T>()` returning one, and
`Marked<T> { pub v: u8, pub marker: PhantomData<T> }` with `marked` and
`marked_value`. Core's `Clone` and `Copy` impls for `PhantomData<T>` are left
out: generic trait impls on provider nominals have no transport evidence yet.

The six accepted clients each return 42:

- `value` and `marker` build a `PhantomData` with its unit constructor and
  through `marker`;
- `marked` and `marked-call` build a `Marked` with a record literal and
  through `marked`;
- `shrink` shortens `PhantomData<&'b u8>` to `PhantomData<&'a u8>`;
- `local` declares its own `struct Mark<T>(base::PhantomData<T>);`.

Three opponents are rejected:

- `grow` lengthens `PhantomData<&'a u8>` to `PhantomData<&'b u8>` ("lifetime
  may not live long enough");
- `marked-grow` does the same through `Marked<&'a u8>`;
- `mismatch` assigns a `PhantomData<u16>` to a `PhantomData<u8>` (E0308).

Both `grow` clients are rejected only because the formal is covariant. A
plain unit struct's formal would be bivariant and would accept both.

rustc 1.98.0 gave all nine declared outcomes in its first run.
