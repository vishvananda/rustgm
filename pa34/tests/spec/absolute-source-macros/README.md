# Absolute paths to actual source macros

`outcomes.json` and `cases.tsv` declare fifteen outcomes before either compiler
executes this packet. The trigger is libc 0.2.185's unchanged `extern_ty!` output:
`#[::core::prelude::v1::derive(...)]` at `src/macros.rs` byte 7994, called for `DIR`
at `src/unix/mod.rs` byte 913.

The same-session host and guests provide actual macro declarations. Their
edition-aware typed path lookup, namespace/kind checks, privacy and crate
visibility determine the target. Source `use` and `pub use` aliases retain the
same canonical target. A source path cannot manufacture a builtin from its final
spelling. A true absolute path also cannot fall back to textual macro lookup.

Controls include the exact source-core derive reexport shape, guest/core bang
macros and aliases, nested exported calls, a misleading local `core` module,
missing/private/later targets, wrong macro kind, and shared source bytes whose
`::local!()` means the crate root in 2015 but the extern prelude in 2021. The
ordinary executable profile's external-macro fence remains an explicit profile
gap. Authenticated metadata-imported and synthetic targets retain their existing
fence; this packet does not qualify external macro transport.

The independent observer compiles each guest separately in graph order with its
declared edition and only earlier dependencies. RustGM uses source-mode bring-up
inspection with an empty skip list. No library artifact, native execution,
performance or assignment-completion claim follows from these source outcomes.

Two separately declared driver controls publish the authored scalar/macro provider with the existing ordinary metadata contract. Both consumers prepend the unchanged host input: relative lookup remains accepted; absolute lookup retains the external-metadata fence. These controls do not claim source library metadata publication. Synthetic ordinary bootstrap remains covered by the ordinary `::std::line!()` opponent.
