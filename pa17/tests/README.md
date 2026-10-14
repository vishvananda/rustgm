# PA17 fixture ownership

Every source is registered in `manifest.json` with declared frontend, rustc and,
for accepted programs, native outcomes. These are original RustGM tests;
`maintainer_origin` records promoted vectors. No upstream reference text is used.

- `identity/` and `headers/`: language-trait resolution, destructor receivers,
  lifetime bounds, Copy conflicts, aliases and invalid implementations.
- `field-moves/`, `loans/`, `replacement-loans/` and `temporaries/`: ownership,
  destructor-induced liveness and storage/temporary validity, including unused
  bodies that still require proof.
- `counters/`: observable cleanup counts and state assertions in unchanged source.
- `order/`: declared callback order, independently observed by rustc and by a
  test-only native LowIR observer after successful frontend compilation.
- `abort/`: explicitly declared panic-abort behavior.
- `boundaries/`: the type-generic and general-method capabilities owned later.

There are 199 cases: 132 accepted native programs (131 pass, one panic) and 67
rejections. Two documented boundary rejections are accepted by rustc. The order
fixtures' no-op observer stays unchanged in course text/native checks; the
separate maintainer native suite checks all 33 exact callback sequences. The
shared counter and order fixtures are read directly by maintainer tests.

Only explicit `ref-test` regenerates sidecars. Missing oracles, mismatched
outcomes, signals, timeouts and compiler panics fail; none counts as rejection.
