# Owned boxes (source core)

Thirteen no_core source-core programs exercise `#[lang = "owned_box"]`, which
rustc 1.98.0 makes the built-in owning pointer. Their shared prefix is a mini
core and a mini alloc written for this packet: its `Box<T: ?Sized, A = Global>`
keeps alloc's `(Unique<T>, A)` field shape, `Unique` holds a `NonNull` around
the content pointer, and `Box::new` and `Box`'s `Drop` allocate and free
through `host_alloc` and `host_free`. `Noisy` reports its drop through
`host_drop`. `observe.c` prints `alloc SIZE`, `free` and `drop VALUE` for each
call, then `value VALUE` for the result of the exported `run`.

`*b` is a place of its own behind the box's pointer. A move out of it leaves
the box to free its allocation without dropping the content, the content can
be assigned again, a field access steps through the box without a `Deref`
call, and dropping a box drops the remaining content, then runs the box's own
`Drop`.

- `move-out` moves `*b` out and drops it before the box frees;
- `drop-whole` drops a box whose content was only read;
- `partial-move` moves one field out of a boxed pair; the other field is
  dropped with the box;
- `reinitialize` moves `*b` out and assigns it again;
- `nested` drops a boxed list that owns a box of its own type;
- `method-move` moves a field out of `self: Box<Self>`;
- `slice-content` drops an unsized content through its length;
- `borrowed-content` writes through `&mut *b`;
- `use-after-move` (E0382), `move-from-borrowed-box` (E0507),
  `use-whole-after-partial` (E0382), `move-from-drop-content` (E0509) and
  `immutable-box` (E0594) are rejected controls.

Outcomes were declared before either compiler ran and checked against rustc
1.98.0, whose status, errors and native output are recorded in
`outcomes.json`: all 13 statuses and error codes agree, and `nested`'s declared
allocation size was wrong (this `NonNull` has no niche, so `List` is 24 bytes).
`dev/tests/box_contents.rs` checks every case and runs the accepted ones
natively.
