# Source-owned prelude selector checkpoint

This PA31 fixture lane exercises the actual internal `prelude_import` request
and source-owned `use` selector. PA12 retains namespace, constructor and privacy
rules; PA34 owns automatic crate/edition-specific injection. This checkpoint
adds no automatic standard-library substitute or metadata transport.

`contract.json`, `contract-v2.json` and `contract-v3.json` preserve predictions
frozen before their corresponding independent observations. `qualification.json`
joins the exact sources to retained raw receipts: 22 metadata observations,
18 qualified inputs and five native programs. Four predecessor setup mismatches
remain excluded: an opaque generated module/member could not be named from the
caller, a root non-glob import did not enter a child scope, and the no-core bool
constant needed genuine Sized declarations. Successor bytes are separate files.
The first two complete oracle batches remain disagreements; only their
individually successful rows are used as evidence.

The native constructor/method, explicit-path and captured opaque cases return
`7`; local and ordinary-glob shadow cases return `11`. All exit zero with empty
runtime stderr. These are pinned Rustc staticlib/C observations. Candidate tests
reach checked local LowIR; Root's fresh native and original-core joins remain
pending at this handoff.

The candidate retains one authenticated cold selector in the existing lexical
container owner. It reads the selected module's ordinary member/glob table after
lexical lookup, preserves binding lineage and dependency keys, and reuses the
trait-member scope index. It suppresses bootstrap library-name/method fallback
when a real selector exists. Actual builtin identity separately governs the
`no_implicit_prelude` macro exception. No dense resolution, scope, node, body or
code field grows. Cold allocation and failed attempts use the existing request
owner's payer.

The exact eight private controls live under `resolve::prelude_scope::tests`,
plus the existing `container_scopes` layout and `methods::visible::prelude_tests`
foreign-trait opponent. They cover real lowering, shadowing, disabled implicit
lookup, privacy, missing gate/forms/owners/targets, source/effect/tree/module
seals, rejected-repeat preservation, warm reads and failed cold allocation.

`non-glob-root` and `multiple` are independently accepted Rust inputs and explicit
unsupported candidate positives. `forward-selector` is independently rejected
by Rust's early module resolution. Automatic injection, 2021/other edition
closure, selector artifact transport and unsupported imported request-owner
profiles remain required; this lane does not activate PA31 or PA34.
