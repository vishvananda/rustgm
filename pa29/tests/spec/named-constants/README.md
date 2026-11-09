# Named-constant packet

`named-constant-outcomes.json` freezes four providers and sixteen clients:
their hashes, edition, target, independent outcomes, the local twins, the
generated opponents, the forged rows and the controls. They were declared in
scratch before the rustc oracle, the base-pin probe and any candidate run.
Each amendment is recorded in that file before the run it governs.

Every provider is the first 57 lines of `spec/source-branches/provider.rs`
(unchanged) followed by authored lines. The inherited lines keep their
attribution, and the repository NOTICE applies to them. The authored lines are
new. The Clone and Copy impls let rustc's match lowering read the matched
primitives under `no_core`.

- `provider.rs` declares eight public constants of primitive and unit types,
  a private `SECRET`, three item-local consts all named `INNER`, and bodies
  that read them, including the const fn `lane_count`.
- `unnamed-provider.rs` declares `const _: u8 = 7;` before the named `AFTER`
  and its reader. `unnamed-twin.rs` is the same file without the `const _`
  line, and the two packets are byte-identical.
- `generic-local-provider.rs` declares the generic `generic_inner<T>`, whose
  body reads its own item-local const.

rustc accepts thirteen clients, and each returns 42:

- `read`: runtime reads of `LANES`, `FLAG`, `LETTER` and `NEG` in one match;
- `local-const`: a client const initialized from `base::LANES`;
- `length`: `[u8; base::LANES]` in a client signature;
- `pattern`: `base::LANES` as a match pattern. Its `eq` and `structural_peq`
  items are oracle-only, under `cfg(rustc_oracle)`, which RustGM never sets;
- `body`, `secret` and `nested`: imported provider bodies that read a public,
  a private and an item-local constant;
- `float` and `wide`: f32 and f64 bits, u128's sixteen bytes and the unit
  constant;
- `const-call`: the imported const fn `lane_count()` as a client array length.
  RustGM rejects it at an existing client fence, "implementation validation
  has no source scheduling context" (amendment 5);
- `inner-pair`: the three `INNER`s, read through their three bodies;
- `unnamed-after`: the constant declared after the unnamed one, and its body;
- `generic-local`: `generic_inner::<u8>`, instantiated by the client.

Three clients are rejected, each as its local twin is: `wrong-length` and
`wrong-type` with "incompatible types", and `private` with "private value name
`SECRET`".

The generated opponents and the older-profile controls are listed in the
outcomes file and built by `crate_linking::named_constants`. rustc accepts
each. RustGM's version-13 export rejects each of these by name:

- tuple, array and record constants;
- reference constants, including `&str`;
- raw-pointer constants;
- inherent and trait associated constants;
- statics.
