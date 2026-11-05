# Source async callable contract

The pinned original `library/core/src/ops/async_function.rs` is retained byte for
byte as `original.rs`; `third_party/sources.json` records attribution and its
SHA-256. The provider fixtures embed that source with a real `no_core` foundation.
Unused alternative declarations remain legal. Declaring a language role grants
neither an implementation nor an executable method body.

`cases.tsv` freezes 25 source and full body-proof outcomes. These cover the actual
AsyncFn/AsyncFnMut/AsyncFnOnce hierarchy, associated Output and future families,
ordinary implementations and reference forwarders, generic calls, wrong outputs,
wrong arguments, unsafe methods, borrowing and escaping references. Independent
Rust qualification and four C observers are bound in `predictions.json`.

`implicit-cases.tsv` adds 21 independently checked outcomes for automatic callable
implementations. Actual Rust functions, Rust function pointers and suitable
ordinary closures must produce a real Future with the requested Output. Capture
kind, tuple arguments, safety, ABI and quantified lifetime proof remain required.
C ABI functions and pointers are negative controls: pinned Rust rejects their
AsyncFn bounds with E0277. The initial incorrect positive predictions and original
diagnostics are retained in the independent receipt, as described in
`implicit-predictions.json`.

All five `implicit-method-*` positives require the actual source method to invoke
the selected function, pointer or closure. Their declared native observer result
is `23` followed by LF and exit zero. A recognized header, an associated-type
normalization or a successful metadata emission alone does not qualify execution.
The manual, generic, ordinary and loan observers likewise require their unchanged
source bodies. Async closure lending, generated async state/poll/drop, suspension,
capture cleanup and authenticated metadata remain required by PA42/43/29.

These are maintainer specification fixtures for planned assignments. They do not
activate an empty assignment or replace the cumulative course text oracles.

The [qualified checkpoint](../../../../docs/parallel/lending-callable-checkpoint.md)
now passes all46 callable source/body outcomes and all9 native observers above.
Private graph/header forgery controls and full successor CI remain pending; this
finite runtime profile does not close the broader async requirements.
