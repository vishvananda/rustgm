# Replacement qualification (source core, bring-up profile)

Five no_core source-core libraries assign through a reference in a const fn,
as pinned core's `fmt::FormattingOptions::width`, `slice::split_off_first`,
`ascii::Char::make_uppercase` and `net::SocketAddr::set_ip` do. They are
checked against rustc 1.98. The prefix is the `source-call-members` `owned`
prefix with core's `const_precise_live_drops` feature.

- `field` stores `u16` and `u8` fields through `&mut self`;
- `reference` replaces `*slot: &'a u8` with another `&'a u8`, as
  `split_off_first` replaces `*self` with the rest of the slice;
- `variant` replaces `*self` with a copied enum value;
- `generic` replaces `*slot: &'a T` for a type parameter `T`;
- `rejected` replaces a value whose type has a non-const destructor
  (rustc E0493).

Each store is a replacement cleanup candidate, and const qualification asks
whether the destination's exact place type may need a const destructor.
Behind a reference that type is no local's or temporary's, so it must be
asked for itself. The concrete libraries have a destructor, so their bodies
classify destruction.

Each accepted program exports `entry` and `probe`; `observe.c` prints `41 59`.
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are
recorded in `outcomes.json` (5/5 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/bringup_stubs.rs` checks every case and runs the accepted ones
natively.
