# Actual UnsafeCell source storage

PA31 owns source identity and completed nominal policy; PA12 owns physical
layout, PA25/27 the constant/raw storage rules, PA16 ordinary loans, PA17
destruction and PA29 authenticated transport. This packet releases no whole PA.

The primary sources are Rust 1.98.0 commit
88d9e12ae178fab0fb5cc050a94da85685d449ea:
compiler/rustc_hir/src/lang_items.rs, library/core/src/cell.rs and
library/core/src/marker.rs. The authored provider foundation inherits the
existing source sizing, Clone/Copy, Freeze/Sync and synchronous Destruct/drop
protocols and their attribution; repository NOTICE and third-party terms apply.

`predictions.json` freezes each Rust input and expected independent outcome before
its candidate observation. `cases.tsv` selects15 semantic outcomes. Four real
Rust object/C observers independently print:

```text
runtime: 23 23
layout: 8 16 8
field-drop: drop=31
nullable-shapes: 23 0 23 0 16
```

The16th source, unused-empty.rs, triggers an upstream variance ICE (exit101).
It is explicitly unqualified, absent from the semantic parity manifest and not
a language rejection. Its original prediction, source bytes and diagnostics
are retained. Initial missing freeze_impls/Destruct/Copy/drop_glue setup and a
nullary size_of call outside const also retain their failed observations.
Reference-v6 qualifies the15 semantic cases and four native observers; its
receipt SHA256 is8827adc5ca128b84e5afa4f814b5afececf18ceff4da14ebb627db18ed11a572.

The actual role accepts a struct without an extra field-count, representation
or generic-arity rule. Its completed shape publishes interior storage before
any physical/promotion/static query. This does not suppress field destruction,
grant Copy, or grant Sync/Freeze: explicit source implementations remain real
proofs. In particular the packet independently accepts an explicit Copy
implementation and a Freeze use after removing its actual negative impl.
Shared references still cannot produce an ordinary mutable field borrow.

The original layout observer exposed absent ordinary source enum niche
selection: RustGM printed8 16 16. The input and expected Rust output remain
unchanged. The source producer now recognizes actual empty/single-field enum
shapes through the existing nullable layout kernel, including reversed variant
order. Actual transparent fields may carry a reference niche; interior storage
blocks it. Explicit representations keep their ordinary tags. Shape grants
no Option role, variant identity or API permission.

Shared-derived constant writes, broader scalar/multifield enum niches, unsized
API closure, optimization permissions and complete nominal/policy metadata
remain requirements. Existing metadata profiles must reject unencoded interior
storage instead of publishing it as an ordinary struct. Local/native source
qualification is distinct from original core checking and crate artifacts.

Candidate qualification now passes seven focused public controls, all125 PA12
text cases, strict selected Clippy, formatting and architecture checks, plus the
four native observers on the current pinned backend. The machine-readable
[checkpoint evidence](../../../../docs/pa31-unsafe-cell-evidence.json) binds every
packet file, both candidate receipts and the independent reference receipt.
