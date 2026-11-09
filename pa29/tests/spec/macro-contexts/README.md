# Macro-context transport packet

`macro-context-outcomes.json` freezes one provider, five clients, their
hashes, edition, target, independent outcomes and the generated export
controls. They were declared at 2026-11-09T00:57:03Z, before any oracle or
candidate run of these files.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The macro-context section
after them is authored:

- three private `macro_rules!` macros, `two`, `sum` and `forty`, where `forty`
  invokes the other two;
- `forty_two` and `plus_forty`, whose bodies expand them. `plus_forty`'s
  captured operand `x` is root syntax;
- the exported `answer!`, whose `$crate::forty_two()` names the provider, and
  `relay`, which invokes it locally.

The four accepted clients each return 42:

- `forty-two`, `plus-forty` and `relay` call the provider's functions;
- `answer` expands the imported `answer!` itself, so its `$crate` path must
  resolve to the provider.

One opponent is rejected: `mismatch` binds `relay()`'s `u8` to a `bool`
(E0308).

The generated controls are written under `obj/parallel/macro-contexts` from
lines 1-147 and one declared control each. rustc 1.98.0 accepts every one.
Five keep the provenance fence: a local, a label and a parameter bound by
marked tokens, a macro 2.0 expansion and a macro 2.0 item. A macro-generated
`#[macro_export]` macro keeps Lane B's existing rejection, located at the
generated definition. An exported `$crate` macro that the provider never
invokes exports before and after the slice.
