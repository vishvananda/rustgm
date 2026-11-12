# Specialized trait inputs (source core)

Nine no_core source-core libraries reduce pinned core's `Zip` `fold`
(`iter/adapters/zip.rs:275`), checked against rustc 1.98. They use the
`source-copy-captures` prefix with `min_specialization` added. A private
`ZImpl<A, B>` has two or three `impl<A, B> ZImpl<A, B> for Z<A, B>` headers
over `It` and the `#[rustc_specialization_trait]` markers `Fast` and
`Faster: Fast`.

`ZImpl::size(..)` leaves the trait inputs to inference. Every applicable impl
resolves them to the same `ZImpl<A, B>`, and rustc's winnowing drops the
specialized impl, so the call selects the most specialized one.

- `trait-path` (core's by-value form), `trait-path-ref`, `method` and
  `three-level` call from inside a specializing impl;
- `qualified` names the inputs, and `default-body` calls from the default impl,
  where only it applies; both are controls;
- `caller-concrete` calls on `Z<S, S>` with `S: Fast`, and `call-concrete`
  lowers that call from a root `main`;
- `distinct-inputs` has two unrelated `impl P<u8>`/`impl P<u16>` (E0283).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json` (9/9 agree). It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.
`dev/tests/associated_declarations/specialized_inputs.rs` checks every case.
See `docs/pa20-specialized-trait-inputs.md`.
