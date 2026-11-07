# Const closure creation prerequisite

The exact existing `inherited-overflow/const-closure-v3.rs` now passes
`analyze_source` followed by `build_roots_prepared` with no exports. The ordinary
child owns the closure identity, captures and checked original. A same-file
wrapper `NodeId` authenticates the real parsed const gate and child; a wrapper
does not turn an ordinary nested closure into a const context.

The shared CFG constructor checks const eligibility for unused originals too.
Creation outside a const context, creation inside an ordinary child, and an
unused const closure calling a runtime function reject. The overflow marker,
feature/target checks and arithmetic checks retain their separate contracts.
Conditional defining inputs still require a scoped continuation. A const state
without its actual published semantic body cannot supply const trait evidence.

The final focused gate passes 20 exact public tests in 90.31 seconds, including
ordinary closure ownership, unused-body proofs and source-size work controls.
Formatting, architecture and selected strict Clippy pass. Earlier audit,
format, gate-identity and scope failures remain bound in
[`const-closure-creation.json`](const-closure-creation.json).

Actual retained-library probes measured the semantic closure row at 72 bytes
before and 76 after. Its marginal four bytes apply to ordinary `None` rows too.
Capacity growth is prepaid through the existing caller payer, with attempted
and publication work charged even with spare capacity. Source-free bodies use
the existing inference work budget for marginal bytes plus work; this is not a
new retained-byte ledger or a historical allocation audit. Declaration stays
one byte; Kind24, IR Body456, Owner20, Value24 and syntax Node36 remain exact.

Actual const calls, scoped defining obligations, receiver adapters, CTFE capture
storage/ownership/cleanup, native consumers and authenticated transport remain
required. The independent reference packet is `a29a3b9ec`; its installed-library
native observations are not RustGM execution evidence. Its no-core object
failure requiring real panic infrastructure remains preserved.
