# Const inherent-method transport packet

`const-method-outcomes.json` freezes one provider, fourteen clients, their
hashes, edition, target, independent outcomes and the generated, wire and
rustc controls. They were declared before any oracle or candidate run of these
files. A later amendment records what was observed first, what changed and
when the change was declared. One declared client, `static-held`, was
withdrawn; the outcome file records why, and its bytes are kept only in the
generated output.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The const-method section
after them is authored:

- `Held<'a>`, whose `impl<'a> Held<'a>` captures its lifetime and declares the
  const methods `new`, `from_str`, `count`, `text` and `relay` (which calls
  `Self::new` and reads a field of its result) and the ordinary `runtime`;
- `Plain`, whose impl captures no lifetime, with the const `new` and `get`;
- the const free function `make`, which calls the const method `Held::new`.

The ten accepted clients each return 42:

- `new`, `from-str`, `relay`, `make` and `plain` call the methods at runtime;
- `const-count`, `const-relay`, `const-plain` and `const-held` evaluate them
  in `const` items;
- `const-fn` calls them from the client's own const function.

Four opponents are rejected:

- `runtime-in-const` calls the ordinary `runtime` in a const (E0015);
- `runtime-argument` reads a runtime argument in a const (E0435); RustGM
  reports the unresolved value name;
- `wrong-payload` passes `1u16` (E0308);
- `text-leak` returns `h.text()` for `Held<'x>` as `&'static str`.

The `crate_linking::evaluated_methods` integration controls generate the
outcome file's other controls in their output directory:

- older-profile, fenced-shape and temporary-field gates on
  `spec/source-nominals/provider.rs`;
- a minimal forged-wire provider, `Gate<'a>`, and its runtime-call variant;
- a 6/24/96 scaling provider.

The provider's rustc-only shims (in the shared lines) are configured under
`--cfg rustc_oracle`, which RustGM never sets. The oracle links the provider
object (`--emit=link,obj`).
