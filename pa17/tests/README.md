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
- `boundaries/`: the general inherent-method capability owned by PA19.
  The type-generic destructor source has moved unchanged to PA18.

There are 206 cases: 135 accepted native programs (133 pass, two panics) and 71
rejections. One documented boundary rejection is accepted by rustc. The order
fixtures' no-op observer stays unchanged in course text/native checks; the
separate maintainer native suite checks all 33 exact callback sequences. The
shared counter and order fixtures are read directly by maintainer tests.

Only explicit `ref-test` regenerates sidecars. Missing oracles, mismatched
outcomes, signals, timeouts and compiler panics fail; none counts as rejection.
