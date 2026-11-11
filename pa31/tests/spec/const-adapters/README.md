# Actual source callable adapter contracts

This draft packet owns 31 authored source-role reductions: 24 independently
accepted Rust programs and seven language rejections. Its ordinary language
counterparts remain in the [PA25 packet](../../../../pa25/tests/spec/const-adapters/README.md).
The actual FnOnce/FnMut/Fn and Destruct declarations preserve the existing
PA25 support prefix. No original core input or assignment activation changes.

The defining-contract checkpoint checks all 28 cases outside the three concrete
factory execution requirements. Conditional self, binary and forwarding results
now prove their instantiated const defining conditions. Ordinary owner bounds
remain checked by their original producers; paid/unpaid Marker controls oppose
an accidental omission. Local FnMut-to-FnOnce and borrowed-lifetime adapters
retain the actual source body and every parent proof. Widths 8/32/128, both
separate functions and many local closures in one parent, have linear work and
identical counters with 128 unrelated nominal headers.

The direct ordinary-call opponent has two phase expectations: its type view
accepts ordinary call typing, while complete const body checking must reject
the non-const call. The exact maintainer regression checks that real diagnostic
through the shared typed pipeline. The CLI's library artifact request instead
encounters an earlier unsupported source declaration metadata profile; that
status alone is not evidence of const legality checking. Unpaid conditional
calls, ordinary closures, wrong results and runtime const creation retain their
independent rejection contracts.

`runtime-self.rs`, `runtime-forward.rs` and `ctfe-self.rs` remain required
positives. All three compile to Rust objects, link with `observer.c` and print
`adapter 23\n` with status zero. Current RustGM runtime cases pass type analysis
but stop at actual const call receiver ownership during LowIR construction.
The CTFE case stops at completed callable formal-order preparation. These
execution gaps are declared separately and remain open; they are not converted
to rejection tests or represented as candidate-native qualification.

`manifest.json` enumerates every input and phase expectation.
`independent-review.json` is a generated observation receipt binding final hashes,
independent diagnostics, current compiler and pending executions. There are no
course text oracles or whole-PA release claims. The initial twenty PA25/PA31
reductions preceded implementation; expanded outcomes were declared before their
independent checks. Disproved exploratory predictions remain in the raw evidence.

Independent commands use pinned Rust 1.98.0/bootstrap1/edition2024, panic abort,
overflow checks enabled, debug assertions disabled and optimization zero, with
`--crate-type=lib --emit=obj`. Candidate type analysis uses
`--source-core minicore@adapters --crate-name minicore --crate-disambiguator adapters`.
The checkpoint and raw receipt locations are in
[the implementation review](../../../../docs/pa25-const-adapter-reductions.md).
