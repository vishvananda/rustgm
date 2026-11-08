# Unsafe trait and non-auto negative impl transport packet

`unsafe-negative-impl-outcomes.json` freezes one provider and 16 clients:
their hashes, edition, target, independent outcomes, and the generated and
wire controls. They were declared at 2026-11-08T19:16:33Z, after the rustc
oracle and baseline probes and before any candidate run of these files.
Amendments are recorded beside the entries they amend.

The provider is `#![feature(negative_impls)]`, then
`spec/lifetime-nominals/provider.rs` (unchanged), then a tail section. The
shared lines carry the attribution inherited from the earlier packets, and the
repository NOTICE applies to them. The tail holds authored impls shaped as
core 1.98.0's:

- `!Clone for &mut T` (`core/src/clone.rs`), `!DerefMutLike for &T` beside
  `DerefMutLike for u8` (`core/src/ops/deref.rs`'s `!DerefMut for &T`) and
  `!Iterator for [T]` (`core/src/slice/iter.rs`): negative impls of traits
  with members, which map none of them;
- an empty `Error` with `Error for &T` where `T: Error + ?Sized` and
  `Error for Fault` (`core/src/error.rs`);
- an empty unsafe `TrustedFused` with `unsafe impl TrustedFused for Counter`
  and the conditional `unsafe impl<I: TrustedFused> TrustedFused for Fuse<I>`
  (`core/src/iter/traits/marker.rs`, `core/src/iter/adapters/fuse.rs`);
- an unsafe `Searcher` with a receiver method `haystack` and a receiver-less
  `make`, implemented unsafely for `u8` and `Counter`, and a safe
  `impl !Searcher for Fault` (`core/src/str/pattern.rs`);
- a safe empty `Plain` with `Plain for Counter`.

The core-derived shapes follow the Rust project under its MIT/Apache-2.0
licence, and the repository NOTICE applies to them. Their bodies are
authored: the traits are reduced to the members each control needs.

Five clients prove or use an imported trait and return 42:

- `trusted`: `Counter` and `Fuse<Counter>` are TrustedFused, and both
  Searcher impls answer `haystack` and `make`;
- `local-trusted`: a local type with its own unsafe TrustedFused and Searcher
  impls;
- `exclusive`: `DerefMutLike::exclusive` on `u8`;
- `error-fault`: `Fault`, `&Fault` and `&&Fault` are Error;
- `local-negative`: a local type with its own negative Error, Searcher and
  DerefMutLike impls.

Eleven clients are rejected:

- E0277: `str: Error` (`error-str`), `[u8]: Iterator` (`iterator-slice`),
  `&u8: DerefMutLike` (`deref-shared`), `&mut u8: Clone` (`clone-mut`),
  `u8: TrustedFused` (`fuse-plain`) and `Fault: Searcher` (`searcher-fault`);
- `safe-searcher` (E0200), `unsafe-plain` (E0199) and `unsafe-negative`
  (E0198);
- `deref-positive`: `DerefMutLike for &Local` beside the provider's negative
  impl (E0751);
- `negative-coherence`, which rustc accepts (exit 42): a local `Pick` with
  `impl<T: DerefMutLike> Pick for T` and `impl<T: ?Sized> Pick for &T`,
  disjoint through the provider's `!DerefMutLike for &T`. It is a recorded
  frontend gap: negative coherence does not pay a `?Sized` negative impl's
  MetaSized premise, with every item local as with this imported one.

rustc 1.98.0 gave every declared outcome, and its six accepted clients exit
42 natively.

The generated and wire controls are built from this provider by
`crate_linking::modifier_transport`; the outcomes file lists each with its
rustc result and expected text.
