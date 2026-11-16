# Forwarded calls (source core)

Six no_core source-core programs call a generic callable parameter whose
actual type is a reference to a nominal callable, as pinned core's
`Filter::next` calls `find(&mut self.predicate)` with core's `IsNotEmpty`
(`str::split_whitespace`) and `Map::next` calls `&mut LinesMap`
(`str::lines`), both `impl_fn_for_zst!` types. The bring-up skip list stubbed
both `next` bodies for this cause. They are checked against rustc 1.98. The
prefix is the `source-call-members` prefix through its `&mut F` call impls;
`shared` adds core's three `&F` impls (`ops/function.rs`).

Like rustc's core `&F` and `&mut F` impls, a reference forwards its call to
its referent: `&mut F` calls `F: FnMut` and `&F` calls `F: Fn`, so `Output`
is the referent's `FnOnce::Output` and the call executes the referent's own
impl member through the references. A reference with an impl of its own, such
as `source-call-members`' `&mut Relay`, keeps it.

- `mutable` calls `G: FnMut(u8) -> u8` with `G = &mut Count`, and the call
  mutates the original `Count`;
- `once` calls `G: FnOnce(u8) -> u8` with `G = &mut Count`;
- `nested` passes `&mut &mut Count`;
- `shared` passes `&NonZero` and `&&NonZero`, a zero-sized `Fn` over
  `&'a u8` written as `impl_fn_for_zst!` expands;
- `filter` is core's `Filter::next` shape, `find(.., &mut self.predicate)`;
- `mutable-shared` calls `G: Fn(u8) -> u8` with `&mut Count`, which no
  `&mut F` implements (E0277).

Each accepted program exports `entry` and `probe`; `observe.c` prints
`41 59`. Outcomes were declared before the frozen rustc 1.98.0 oracle. Both are
recorded in `outcomes.json` (6/6 agree). The packet keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/explicit_call_members.rs` checks every case and runs the accepted
ones natively.
