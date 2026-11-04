# Local default-body stability and original-core source progress

The frontend captures actual `rustc_default_body_unstable` declarations as a
third policy axis, separate from regular API and const stability. Local provided
methods and explicit overrides retain their ordinary checked source bodies.
This clears the LIB-128 source-attribute frontier. Full Drop/Pin semantics,
external inheritance policy transport and original core/alloc/std artifacts
remain required; PA31 stays planned.

The [frozen packet](../pa31/tests/spec/default-body-stability/README.md) qualifies
48 independently predicted outcomes against pinned Rust 1.98.0. Five fresh
reference clients use an actual provider rlib: an external impl inheriting the
unstable body requires its feature even when unused; explicit overrides and
calls through an existing provider-side impl retain stable API permission.
These five RustGM imported consumers remain required. The other 43 local/source
provider programs run through both actual candidate CLI phases. Broad attribute
targets, bodyless declarations, invalid bodies, missing required methods,
duplicate policies, renamed features and const-call boundaries retain their
real outcomes. No body or permission is synthesized from an annotation.

Capture uses one `(NodeKey, PolicyKind)` map instead of separate regular/const
maps and another default-body map. The existing cold policy vector retains the
actual source origin. The temporary regular-import projection allocates its
prepaid capacity once; filtering cannot cause unaccounted geometric growth.
Work scales through 16/64/256 declarations. Library152, Inventory112, Policy56,
syntax Node36 and Body456 remain unchanged. No per-call policy scan or ordinary
AST/body field is added. No wall-time speedup is claimed.

Old metadata profiles explicitly reject default-body policy they cannot encode.
The provided-body family and the availability family must be composed with real
owner/body identities, explicit absence and external impl-creation checks before
the imported reference cases can qualify. Existing wire bytes remain intact.
The full two-method source Drop/Pin guard also remains intact.

Final compiler/test snapshot evidence:

- Twelve exact private tests, format, architecture and strict Clippy, 135.46s:
  `obj/qualification/fast/1793752028343538223/receipt.json`, SHA-256
  `ace39d28c84fac3df7d33a8dcb743911c63c610d88687a7f28602845231ec80f`.
- Production smoke, feature requests and availability: 27 exact tests across
  two targets, 224.22s. One availability native test retains its ignored marker
  for full native CI. Receipt
  `obj/qualification/fast/1793752166515243330/receipt.json`, SHA-256
  `4ac3b58e5ba971644164481f8dc8117d160a1e18da8eecf1564d94aa68d22f69`.
- Independent Rust: 48 outcomes, including five real imported consumers,
  1.43s. `obj/parallel/default-bodies-v1/oracle-v3/receipt.json`, SHA-256
  `edfa4271f8e945c15fae6e6ff7f7cda624022c0817a7879b0a50214f143c6b02`.
- Joined candidate qualification, 38.59s: 86 CLI phase observations, previous
  candidate rejection, actual source-lowered native `7 9` through the locked
  CPPGM object backend, upstream/harness checks and unchanged original core.
  `obj/parallel/default-bodies-v1/qualification-v1/receipt.json`, SHA-256
  `ab1106c09f5afb8c1d344a65f09b452c8db9cc16b2094c740c1657d75fc9664f`.

Immutable candidate:
`obj/parallel/default-bodies-v1/cli/6bc0e5209fc5fe18f4dc9e0452aedf6be7672298e927da7f50408014e20891b6/rustgm`.

Original core now reaches LIB-129: `#[lang = stringify!($lang)]` in the genuine
panic-function macro, `library/core/src/panicking.rs:172`, file188 byte7851.
The error is `lang requires one string literal`. Observation takes18.7710s,
375 loading opens, all722 source hashes and six configuration hashes unchanged,
with the same target, capped lint profile and finite budgets. Result SHA-256:
`eafce59a5e485ae1770db33a8cdd28996d6ea190c867884a04455578db67f047`.
Next expand real macro-valued attributes before reserving language-item identity;
do not recognize the source spelling as authority or replace the real panic body.

Full predecessor [CI41742788707](https://github.com/vishvananda/rustgm/actions/runs/41742788707)
failed one private promotion expectation; its fast job passed. The retained
[owner regression](../pa31/tests/spec/source-promotable/owner-checkpoint.md)
keeps the original conflicting feature fixture and adds a distinct-feature
negative case for the intended direct-const-owner boundary. Both pinned Rust
rejections and the repaired exact test pass. This does not relabel that CI run
as passing. Full successor qualification remains separate from these focused
checks. Earlier failed oracle setups, lint findings and the mistaken root-node
target check remain preserved with their diagnostics.
