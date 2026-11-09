# Scheduling-context packet

`scheduling-context-outcomes.json` freezes eight providers and twelve clients
in 35 cases: their hashes, edition, target, independent outcomes, the base
pin's outcomes, the controls and the scaling counts. They were declared in
scratch before the rustc oracle and any candidate run. Each amendment is
recorded in that file before the run it governs.

Every provider is the first 57 lines of `spec/source-branches/provider.rs`
(unchanged) followed by authored lines. The inherited lines keep their
attribution, and the repository NOTICE applies to them. The authored lines are
new. Each provider exports under `module-bodies-v13`.

- `none-provider.rs` declares no Clone or Copy impl.
- `usize-provider.rs`, `u8-provider.rs` and `bool-provider.rs` declare Clone
  and Copy for that one primitive.
- `nominal-provider.rs` declares them for a unit struct only.
- `primitives-provider.rs` declares them for fourteen primitives: every
  integer, `bool` and `char`. It is the scaling provider.
- `blanket-provider.rs` declares usize's impls and `impl<T> Marker for T`.
- `generic-provider.rs` declares usize's impls and `const fn id<T: Copy>`.

The providers' const fns return usize: `four()` returns a literal,
`suffixed()` returns `4usize` and `placed()` copies a usize place. The
nominal provider omits `suffixed()`, the primitives and blanket providers
declare only `four()`, and the generic provider declares only `id`.

The clients read a const in a client array length unless noted:

- `four`, `suffixed` and `placed`: the matching imported const fn;
- `body`: `four()` as a client const read only inside a body;
- `local-placed` and `local-literal`: a client-local const fn that copies a
  usize place or returns a literal;
- `minspec`: `four()` in a `min_specialization` client, alloc's and std's
  shape;
- `eight`: eight client consts, each `four()` as its own array length;
- `generic-usize`: the imported generic `id(4usize)`;
- `overlap-twin`: a client `impl base::Marker for Local`, which overlaps the
  blanket impl, without a constant;
- `overlap`: that impl plus `four()`;
- `local-copy`: client-local Clone and Copy impls for `Local(usize)`, reached
  through `id(Local(4))` and read as `[u8; L.0]`.

rustc accepts every client except the two overlap clients, which it rejects
with E0119. Each accepted client returns 42.

On the base pin, 14 cases stop at "implementation validation has no source
scheduling context" and are accepted afterwards. Each exits 42 natively with
empty stdout. 17 accepted cases stay accepted. Four are rejected:

- `overlap-twin` and `overlap` report "overlapping trait implementation
  headers require coherence proof", with the same diagnostic line. The base
  pin rejects `overlap` at the fence instead.
- `local-copy` reports "client implementation validation before the eager
  walk requires a source scheduling context", a known rejects-valid.
- `generic-usize` reports "defining cleanup requires its actual language
  declaration", an existing fence independent of this packet: the inherited
  lines declare no Destruct language item. It is a failed prediction
  (amendment 1).

The `crate_linking::scheduling_context` integration control and its ignored
native twin consume the packet. The `executable_demands` invariant analyzes
clients of the primitives and nominal providers, and `unordered_imports`
analyzes `minspec` over the usize provider. `u8clone`, a probe whose provider rustc and RustGM
both reject, is excluded.
