# Tuple and unit struct constructor transport packet

`struct-constructor-outcomes.json` freezes one provider, twenty clients, their
hashes, edition, target, independent outcomes and the wire controls. They were
declared before any oracle or candidate run of these files. A later addition
records when it was declared.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The struct-constructor
section after them is authored:

- `Wrap(pub u8)`, whose provider functions `wrap`, `unwrap` and `open` call
  its constructor, read its field and destructure it, and whose inherent
  `new` builds it with `Self(x)`;
- the generic `Pair<T>(pub T, pub u8)`, with `pair` and `second`;
- the unit struct `Unit`, with `unit` and `accept`;
- `Secret(u8)`, whose private field makes its constructor crate-private, with
  `secret` and `reveal`;
- `Held<'a>(pub &'a u8)`, whose `impl<'a> Held<'a>` captures its lifetime;
- `inner::Narrow(pub(crate) u8)`, whose crate-visible field narrows the
  constructor of a public struct, with `inner::narrow`.

The eleven accepted clients each return 42:

- `called` goes through the provider's own constructor call and field read;
- `direct` calls `base::Wrap(42)`;
- `pattern` destructures with `let` and matches tuple-struct patterns;
- `method` calls `Wrap::new`;
- `generic` calls `base::Pair` with a struct payload and reads `pair`'s field;
- `unit` passes and matches `base::Unit`;
- `imported` uses `Wrap as W`, `Unit` and a constructor function value;
- `braced` writes `base::Wrap { 0: 42 }` and the braced pattern;
- `constant` builds a `const` with the constructor;
- `held` builds `base::Held(&x)` and reads the captured result;
- `secret` uses the private-field structs through their provider functions.

Nine opponents are rejected:

- `private-constructor`, `private-pattern` and `crate-field` name a narrowed
  constructor (E0603);
- `private-field` reads a private field (E0616);
- `private-braced` writes a private field in a braced literal (E0451);
- `wrong-arity` passes two arguments (E0061);
- `wrong-payload` passes `1u16` (E0308);
- `unit-call` calls the unit struct (E0618);
- `held-escape` keeps a borrow of a dropped local (E0597).

The `crate_linking::struct_constructors` integration controls generate the
outcome file's other controls in their output directory:

- older-profile gates on `spec/source-nominals/provider.rs`;
- fenced shapes on the same provider;
- a minimal forged-wire provider;
- a 6/24/96 scaling provider.

The provider's rustc-only shims (in the shared lines) are configured under
`--cfg rustc_oracle`, which RustGM never sets. The oracle links the provider
object (`--emit=link,obj`).
