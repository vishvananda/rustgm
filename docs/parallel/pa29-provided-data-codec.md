# Provided body data codec checkpoint

`Source2024Provided` is a separate unpaid data profile: envelope tag 11,
row version 0 and defining Rust2024 byte 0. `Source2024Functions` retains tag 10
and its existing byte layout and admission rules. Both use the existing source
column codec and the shared structural staging/freeze producer.

The wider profile retains the complete neutral body columns: function and call
recipes, conditional requests, projections, borrowed values, allocation inputs,
normalization-owned intervals and one numbering unit per original. It preserves
the distinction between absent and present empty alias ledgers. Structural
validation grants no defining-environment, region, loan or execution permission.
The constructor is `Payload::from_source_provided(owner, rows, base, work)`;
ordinary functions keep `Payload::from_source`.

The genuine supplied default has a `ConstTraitDefault` body environment and a
`ConstDefinition` body context while its declaration remains ordinary. The
ordinary `u8` Clone implementation retains its runtime context. These records
are distinct; implementing a const trait does not make an ordinary method
const-callable.

All four canonical source-client admission boundaries remain Functions-only.
No frontend Provided publisher, imported numbering allocator, imported default
preparation or semantic attribute/UB-policy transport is activated. Actual
catalog/arena/body/signature binding, required alias presence, original
Contract/Required identity, region/loan checks and fresh instance proof remain
required before a future imported Provided body can execute. Missing old policy
or attribute rows do not establish checked absence or permission.

Focused development qualification completed in 223.17 seconds through
`make fastcheck`, using the unchanged 16 GiB guarded cap. Full formatting,
the content-cached architecture audit, strict private-harness Clippy and five
exact private controls passed. The controls cover the original complete family,
byte roundtrip and import denial, every truncated column prefix, version and
extra-byte rejection, malformed domain/environment/context opponents, aggregate
budgets, old-profile early denial and the unchanged Functions column suffix.
No release, full private inventory or native sweep was run for this checkpoint.

Evidence is under `obj/qualification/fast/1792903982333879597`; receipt SHA-256
`0c984f0f0e69b1cafacdff2f86278f9474f11e2266b6d607de484acfeb7dc02d`.
Earlier audit and Clippy failures and the separate resource refusal remain
retained under their original fastcheck roots. The audit failure was a test
variable parsed as a raw-reference token; the Clippy failure was one missing
test semicolon. Neither was suppressed or treated as a compiler result.

The merged codec and inverse-allocation helper are qualified together at source
`03190e3c`. Fourteen exact private controls passed in 157.01 seconds, including
formatting, the architecture audit and strict private-harness Clippy; all 198
Python harness tests and the upstream/audit-cache contracts also passed in
[focused CI](https://github.com/vishvananda/rustgm/actions/runs/40171134133).
The inverse helper charges failed queries and allocation before mutation, joins
actual original/alias domains and has fixed warm work with unrelated originals.
The canonical Functions publisher and all four import boundaries remain closed
to Provided data. The next checkpoint reserves actual declarations and typed
inputs in a distinct unpublished session; it does not intern or execute bodies.
