# Source callable-role transport packet

`callable-role-outcomes.json` freezes one provider, twelve clients, their
hashes, edition, target, independent outcomes and the wire controls before
candidate execution. The provider reuses lines 2-57 of
`spec/source-nominals/provider.rs`: the authored sizing, receiver, Clone, Copy
and Freeze declarations of PA31's exact-source-linkage and Pin foundations.
Their inherited attribution and the repository NOTICE apply. The `Tuple`
marker, the `FnOnce`/`FnMut`/`Fn` roles, the `u8` Clone/Copy impls and the
three generic appliers are authored controls.

`provider.rs` declares the `tuple_trait` marker, `FnOnce` with its
`fn_once_output` result and `extern "rust-call" fn call_once`, `FnMut:
FnOnce` and `Fn: [const] FnMut`, all `const trait`s with
`#[rustc_paren_sugar]`. It exports `apply` (`F: FnOnce(u8) -> u8`),
`apply_mut` (`FnMut`) and `apply_ref` (`Fn`), each `{ f(x) }`. For each kind,
the `-item`, `-closure` and `-capture` clients pass a plain function item, the
consumer's own non-capturing closure and its own capturing closure; all nine
return 42. `wrong-output` passes a function item returning `i32` (E0271),
`ref-mutating` passes a closure that assigns its capture to `apply_ref`
(E0594) and `mut-consuming` passes a closure that moves its capture to
`apply_mut` (E0507).

The provider carries a rustc-only shim under `--cfg rustc_oracle`: the
`drop_glue` item that rustc's drop elaboration of the owned `FnMut`/`Fn`
argument requires. RustGM never configures that cfg. The oracle links the
provider object (`--emit=link,obj`).

Pinned Rust 1.98.0 (`RUSTC_BOOTSTRAP=1`, `-Cpanic=abort`) compiles the
provider; the C observer returns 42 for all nine accepted clients and the
three opponents fail with E0271, E0594 and E0507. Exploratory candidate probes
of similar scratch sources preceded the freeze; no frozen file was run by the
candidate before it.

The candidate profile is `module-bodies-v13`. The wire controls (forged call
traits and receiver borrows, a relabelled role spelling, a version relabel and
every prefix) are mutations of the provider export; their expectations are in
the outcome file.
