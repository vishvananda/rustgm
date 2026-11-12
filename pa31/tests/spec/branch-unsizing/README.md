# Branch unsizing (source core)

Fourteen no_core source-core libraries reduce pinned core's `Split::as_slice`
7874 (`slice/iter.rs:427`, `if self.finished { &[] } else { &self.v }`) and
`SplitInclusive::next_back` 7900 (`slice/iter.rs:646`,
`let remainder = if self.finished { &[] } else { ... }`). They are checked
against rustc 1.98. Each case's first line names its prefix,
`pa31/tests/spec/source-index-places/unsized-output.rs` before its `Word`,
which declares `Unsize` and `CoerceUnsized` for references.

rustc's `CoerceMany` gives branches one type. With a known expectation, each
branch coerces to it. Without one, a later branch first coerces to the merged
type. If that fails, the earlier branches coerce to the later branch's type,
so an earlier `&[_; N]` unsizes to a later `&[T]`.

- With an expectation, `core-as-slice` (an empty array and a `&&[T]` deref
  coercion), `ret-expected` and `let-annotated` (two array lengths) accept.
- Without one, `core-next-back`, `lub-later`, `lub-earlier`, `match-lub`,
  `three-arms`, `else-if`, `noop-earlier` and `adjusted-earlier` accept. In
  `adjusted-earlier`, a `&mut [u8; 2]` arm has already become `&[u8; 2]`
  before a later `&[u8]` arm.
- Typing rejects `lub-arrays` (two array lengths with no expectation),
  `mut-slice-later` (`&[_; 0]` against `&mut [T]`) and `lub-deref` (`&&[T]`
  needs an expectation to deref), each with E0308.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each batch. 13 of the 14 declarations agree. `adjusted-earlier`
was declared rejected (E0308), and rustc accepts it; rustc is the oracle. Both
are recorded in `outcomes.json`. The batches are in
`obj/parallel/branch-unsize/core/declared.json`. The cases keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/branch_unsizing.rs` checks every case's typing through the CLI. It
runs prepared lowering for every case that types, and checks bounded work. Its
ignored native test runs four ordinary-mode programs against rustc. See
`docs/pa31-branch-unsizing.md`.
