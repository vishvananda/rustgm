# Library stability and source feature inventory

This investigation identifies a required PA31/PA29 source-library contract.
It does not implement stability, activate PA31, or advance the whole-core build
past its recorded failure. The last whole-root observation stops at core's
`bstr_internals` request. That name is defined by actual library declarations;
adding it to the compiler's language-feature whitelist would not implement its
meaning.

The reference is Rust 1.98.0 revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, available in the shared immutable
source tree. The primary implementation seams are
[`rustc_passes::stability`](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_passes/src/stability.rs)
(`check_unused_or_stable_features`, `lookup_stability`, `lookup_const_stability`)
and
[`rustc_middle::middle::stability`](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_middle/src/middle/stability.rs)
(`eval_stability` and const-stability checks). The inventory must follow enabled,
expanded declarations and actual dependency identities, not a textual scan of
every attribute in an upstream directory.

## Independently established cases

The [declared contract](../pa31/tests/spec/library-stability/contract.json)
retains 33 byte-identical authored sources: two providers and 31 client/local
hypotheses. Both outcome sets were declared before pinned rustc execution.
All statuses matched: 41 process observations, including eight executable
clients with status zero and empty output. Local cases are metadata checks;
they are not object emission or runtime evidence. The profile is edition 2024,
x86_64-unknown-linux-gnu, panic=abort, overflow checks enabled, RUSTC_BOOTSTRAP=1,
and capped lints. Warning text is not asserted.

| Family | Observed contract |
| --- | --- |
| Feature inventory | An undefined root request rejects; a later or macro-generated definition supplies a known feature; a cfg-disabled definition does not. |
| Attribute validation | Missing public stability, missing staged_api, conflicting declarations, malformed stable versions and missing const stability reject. |
| Local ownership | Same-crate ordinary unstable calls work without opting into every local library feature; inherited unstable state is accepted in the tested module. |
| External API | Stable calls work; unstable calls require the exact feature; a different feature does not pay it; a known unused dependency feature is accepted. |
| Const API | A runtime-stable function can remain const-unstable. Its runtime client works without the const feature, while its const client needs it. |
| Stable const bodies | A stable const body cannot freely call a const-unstable callee; the tested explicit rustc_allow_const_fn_unstable allowance admits it. |
| Macro provenance | An allowed macro's own unstable call works; an unannotated macro's call rejects. A captured caller expression still needs its own feature. A nested allowed macro retains its allowance. |
| Root ownership | Corrected stable-module controls accept nested defined and unknown feature requests as inert; nested attributes do not enable crate features. |

The original `api/module-request-not-root.rs` rejection came from missing public
module stability. Its bytes and result are preserved, but it is not evidence for
the intended nested-request rule. The separate `macros/nested-*-request.rs`
controls remove that confounder and both accept. This distinction matters when
turning reducers into actual diagnostic assertions.

Raw declarations, commands, diagnostics and completion receipts remain in
`obj/parallel/pa31-library-stability/`. The original result digests are:

- API: `0d1259741ea7cb070ca0dbede30a3d0d385a24e2741684f4a92bbeee07012fd0`.
- Macros/root ownership: `8018c122b7b65a72abb51bf3e2d0a41c616c98fa452e74b196b981504882ff17`.

These observations use rustc-built providers only as an independent oracle.
No such artifact may enter a RustGM target graph. The sources currently have
no candidate text oracles or passing RustGM registration; PA31 stays planned.

## Implementation seams and order

1. Retain root requests separately from the compact language-capability flags
   in `resolve::bindings::policy`. Defer possible library requests until the
   configured/expanded declaration and dependency inventories are complete.
   An unrecognized request must never enable a language feature by spelling.
   Unknown requests still fail before completed publication or execution.
2. Consume retained attributes once, validate their shapes, versions, duplicates,
   allowed owners and staged-api requirements, and publish a cold inventory
   keyed by actual declarations. Compute inheritance through the existing owner
   graph once, preserving explicit overrides and regular, const and default-body
   policy as separate facts. Disabled subtrees and retired include templates
   contribute no definitions.
3. Finalize root requests against local and authenticated dependency feature
   inventories after generated declarations are available. Ordinary clients need
   this policy as well as source providers; own-provider-only mode is insufficient.
4. Check each actual resolved use against its declaration's completed policy.
   Same-crate ordinary permission, cross-crate API permission and stable const
   body restrictions have different rules. A library feature cannot establish
   const-callability, pay a conditional trait obligation or authorize execution.
   Coordinate this additional check with the existing reached const-call gate;
   preserve each check's original source and caller context.
5. Preserve exact macro expansion provenance for allow_internal_unstable.
   Transcribed definition tokens and captured invocation tokens retain their
   respective authorities. A whole-crate flag or the expansion's outer source
   location cannot represent this rule. PA29's imported macro protocol must carry
   authenticated defining provenance rather than granting all users an allowance.
6. Extend source-client metadata with regular/const/default-body stability,
   feature definitions and exact declaring profiles. Validate these rows and
   identities before transactional import publication. No paid use permission
   or const witness is serialized. The source-client header/body protocol must
   retain unsupported rows as errors until every owning consumer exists.

The first implementation slice should close local declaration inventory and
deferred request validation without claiming external-use support. Next close
authenticated provider/client API checks, const restrictions and macro origin
rules. Each slice needs real candidate positive/negative tests. Accepting
stability attributes without enforcing their effects is not an intermediate
compatibility result.

## Performance and remaining coverage

Reuse retained attributes, actual owner identities and source maps. Keep ordinary
non-staged source state absent. Index feature names and compute effective owner
policies once; never rescan all attributes or recursively walk a module's parents
at every use. Share immutable inventories across consumers while each use retains
its own feature/caller authority. Count declaration/edge work and retained bytes
under existing budgets, including rejected imports.

Before timing, freeze independent width, owner-depth, repeated-use and unrelated-
declaration axes. Include cold/warm lookups, invalidated dependency inventories,
cfg/expansion differences, same-shaped foreign identities and non-staged controls.
No performance claim is established by the current semantic probes.

Remaining semantic coverage includes precise owner applicability, reexports,
trait/impl and provided-default policies, unions/fields, stable feature versions,
implied_by relationships, deprecation and soft instability, force-unstable modes,
imported macro graphs and conditional const-stability interactions. The 31 cases
are an implementation seed, not a complete compatibility census. Whole original
core, alloc and std with native consumers remain the acceptance target.
