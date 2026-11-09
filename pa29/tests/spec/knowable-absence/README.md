# Knowable coherence absence packet

`knowable-absence-outcomes.json` freezes one provider and 13 clients: their
hashes, edition, target, independent outcomes, diagnostic locations and the
generated controls. They were declared at 2026-11-08T20:27:56Z, after the
rustc oracle and baseline probes and before any candidate run of these files.
Amendments are recorded beside the entries they amend.

The provider is `spec/unsafe-negative-impls/provider.rs` (unchanged) followed
by one authored line, `impl !Error for &str {}`. Beside that provider's
`impl<T: Error + ?Sized> Error for &T`, it is core 1.98.0's same-head pair in
`core/src/error.rs`. The inherited lines keep their attribution, and the
repository NOTICE applies to them. The core-derived shape follows the Rust
project under its MIT/Apache-2.0 licence.

A consumer re-proves the pair from this packet's perspective, where `Error`
is local, and each client's own pairs from the client's. Six clients return
42:

- `answer`: `&Fault` and `&&Fault` are Error;
- `answer-local-impl`: an unrelated local `impl base::Error for Local {}`
  beside the pair, which must still prove disjoint;
- `local-error-ref`: `impl base::Error for &Local {}` beside the blanket
  `Error for &T`, disjoint because `Local: Error` is knowable for the client;
- `local-mut-ref`: `impl<T: base::Error> Mine for T` beside
  `impl Mine for &mut Local`;
- `local-copy`: `impl<T: base::Copy> Mine for T` beside `impl Mine for Local`;
- `trait-argument`: `impl<T: base::Add<Local>> Mine for T` beside
  `impl Mine for u8`, knowable through a local trait argument.

Seven clients are rejected, each at its own impl:

- E0119, because another crate could add the missing impl: a foreign Self
  (`foreign-self`, `base::Counter`), a reference to an uncovered parameter
  (`uncovered`), a tuple (`tuple-self`), a plain foreign wrapper
  (`nonfundamental-wrapper`, the provider's `Slot<Local>`, which is not
  `#[fundamental]`) and a local trait over `&T` (`local-uncovered`);
- the orphan rule: `impl<T> base::Error for T {}` (`orphan-blanket`, E0210)
  and `impl base::Error for str {}` (`orphan-str`, E0117).

rustc 1.98.0 gave every declared outcome, and its six accepted clients exit
42 natively.

`crate_linking::absence_transport` consumes the clients and builds its
generated controls from this provider: a client whose imported condition
holds, and a seal provider whose `?Sized` conditions are read under negative
coherence. `overlap::disjointness`'s scaling control builds its generated
pairs from it too.
