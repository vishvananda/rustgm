# Shared imported environment and trait-input kernels

The imported conformance wrapper previously required every body Environment Arc
to equal its declaration Environment Arc. Genuine Contract originals have a
distinct body environment. The wrapper now delegates that relationship to the
same authenticated origin validator used by canonical declaration readiness.
Required methods and Declaration originals still require equality; Contract
originals require distinct Arcs. Unsupported origins remain rejected.

`Linkage::require_source_method` receives the actual Declarations and retains
its existing catalog/header/full lifetime-scheme checks. The shared environment
entry authenticates the actual Store arena, imported declaration, provider,
function and original body before applying the existing origin rule. It does
not replace a body environment with its declaration environment or establish
original-body proof.

The ordinary and private trait-input paths now use one
`predicate_remap::translation::trait_inputs` producer. Its sealed Mapping port
supplies the actual imported trait declaration, and the existing translator
derives the complete canonical prefix, full input suffix and Self membership.
The ordinary port uses the correspondence already authenticated by its Context;
its producer order and allocation accounting remain unchanged. The private port
uses the genuine reserved RecipeView and current Catalog.

Private ScopeInputs retains that move-only Prepared token for traits. Other
declarations keep their ordinary input Arc. A warm trait-scope lookup checks the
token against the same actual imported declaration before reuse. No predicate
or subject-vector clone is added to the production path, and no invented
declaration, source node, role, environment or generic formal is accepted.

## Focused qualification

Compiler source `eef24e84c629e02f806813842d39fd311d7c7d35` passes hosted run
[40273325629](https://github.com/vishvananda/rustgm/actions/runs/40273325629).
All 2,719 source fingerprints match the retained artifact and local source.
The exact six tests pass in 0.42 seconds; the helper finishes in 122.07 seconds.
Fresh formatting, full architecture audit, private strict Clippy and the
unoptimized harness build pass. Metadata and production strict Clippy are
retained from their actual successful steps in run 40272252699 after exact
production-source/test-only-delta authentication and byte equality of the
mandatory newly built CLI. The original run remains failed.

The six groups cover:

* A real fresh private consumer, actual Holder Self/V preparation and moving its
  token into HeaderInputs::imported; foreign owner/catalog and same-shape foreign
  token replacement on an existing warm scope reject.
* Actual codec/fresh ordinary imports with a required clone method, generic
  Declaration original and nongeneric Contract original; opposite Arc
  relationships, missing declaration environments and a foreign arena reject.
* Actual inherited Self/V/U and nonzero Declaration body controls, selected
  scope/universal widths, failed prepayment and repeated use.
* The existing zero-domain environment controls.
* Actual selected-template registration in the same fresh consumer.
* The old first-encoding v0/v1 and fresh-client sentinel: all seven retained
  provider, artifact, row and configuration files remain byte exact.

Two earlier attempts ran no tests. Run 40272252699 found the new test's missing
mutable owner and a filename-prefix audit finding; run 40272789874 found only
the new test's function-length lint. Their raw diagnostics and receipts remain
retained. The source corrections affect only private tests and their cfg(test)
module path. No failed test outcome or reference was replaced.

## Costs and remaining proof

Measured x86-64 Scope grows from 112 to 120 bytes: eight additional bytes for
each prepared cold boxed scope. ScopeInputs and Prepared are both 40 bytes.
Existing accounting prepays and reconciles the actual Scope size; there is no
new heap table. Environment Owner80, Body280, State232, dormant Session10352 and
fresh ProofSession10160 retain their measured enclosing sizes. The dense scope
cells remain optional Box pointers. No Store or body field changes.

At unrelated counts 1/8/32, inherited warm1000 remains exactly 65,000 charged
steps; widths 1/8/32 have the same warm cost. Neutral bodies retain 113,000 steps
for warm1000 at signature-universal counts 1/8/32. These are bounded-work
controls, not a timing or zero-cost claim. Full inherited formal vectors retain
their actual storage costs.

The source/receipt handoff is retained under
`obj/parallel/pa29-source-client-staging/lib011/shared-input-kernels-v1/`.
The qualified source map SHA-256 is
`3e8e2767cf405333346f32b3779d60e1776f6a0d443a70bf2009c7f8a3465f6a`;
the actual CLI SHA-256 is
`00e53315f576f7971c34b22349822f99180a99280249c25326edaa61557d863f`.

This checkpoint grants no private checked-header, role, implementation,
original-body, fresh-instance or public Provided readiness. Complete retained
trait/schema/member append must use the same actual consumer and demand memo.
Const/default and parented/nonzero Contract environment producers remain
required. Carry original plus a fresh instance, the complete source family,
borrow, artifact/codec, advisory/default transport and native result 7 remain
the execution milestone. Present-ledger BODY qualification is still pending.
