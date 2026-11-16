# Tagged Option constants (source core)

Three no_core source-core programs read `Option<u8>` and `Option<u32>` from
bytes in constants. Neither payload has a niche, so each option keeps a tag,
which rustc 1.98.0 makes one byte at offset 0 (`None` 0, `Some` 1) with the
payload after it at its own alignment. Their prefix is a mini core with core's
`Option` declaration and `transmute`.

- `transmuted` transmutes integers to both options and `Some(5)` to a `u16`;
  `observe.c` prints the five values;
- `invalid-tag` transmutes tag byte 2 to `Option<u8>` and is a rejected
  control (E0080);
- `zeroed` fills a local `Option<u8>` with `write_bytes`, as core's
  `MaybeUninit::zeroed` does in alloc's `vec/is_zero.rs`, and reads it with a
  match; `zeroed-observe.c` prints both values. Only the bring-up profile
  executes `write_bytes` in constants; the strict profile keeps it fenced.

Outcomes were declared before either compiler ran and checked against rustc
1.98.0, whose status, error and native output are recorded in `outcomes.json`:
all 3 statuses, the error code and both outputs agree.
`dev/tests/tagged_option_constants.rs` checks every case and runs the accepted
ones natively.
