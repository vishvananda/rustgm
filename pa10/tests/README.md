# PA10 fixture ownership

The manifest declares 48 body-view fixtures and every auxiliary source.
`execution/` covers integer CFG behavior/evaluation order; `values/` typed
values; `divergence/` paths without continuations; `expansion/` and `packages/`
combine CFGs with earlier features. `reject/` checks no partial stdout on error.

The 33 execution inputs have declared results and are exercised by the test-only
CFG machine versus pinned rustc. Other fixtures have independent acceptance
checks. No ownership, region, constant-evaluation or native-lowering proof is
inferred from a PA10 pass. LowIR execution belongs to PA11.

Only `make ref-test-pa10` generates sidecars. Never hand-edit their bytes.
