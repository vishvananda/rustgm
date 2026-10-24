# Scoped declarations without scoped execution

This successor starts from integrator checkpoint07985b2c and preserves the
qualified allowance source222b637a/docs1ba69022 separately. It deliberately
excludes field checkpointb937. The new corpus is declared before production
changes: pa31/tests/spec/declarative-macros/declaration-contract.json contains
31 immutable hypotheses, predicting102 independent observations and20 native
answers. All are unrun; every disagreement must remain recorded.

The subset validates and publishes local scoped declarations. It does not
admit scoped invocation, foreign macro export, scoped builtin stubs or completed
macro stability. The first actual core frontier is macros/mod.rs:170; removing
that declaration gate cannot itself load all of core. The next known scoped
builtin is cfg_select at line235, with real uses elsewhere. Its claimed role and
execution require a separate genuine builtin contract.

The exact original assert_matches declaration is retained byte-for-byte inside
a new attributed fixture. Its pinned file/slice offsets, hashes and Rust source
attribution are in the contract. Expansion declaration retention is a candidate
positive; final local macro-stability remains explicitly unsupported. Ordinary
unused concise/braced declarations have independent metadata/object/native
expectations. Source-positive invocation cases deliberately remain candidate
unsupported. No original macro, field or allowance hypothesis is rewritten.

## Typed producer and consumers

The shared cold producer belongs to declaration resolution. A proposed
Resolver::declare_scoped(Context,NodeId,Item,Access,Option<NodeId>) returns the
actual declaration Context and an owned cold ScopedDeclaration: transparency,
defining edition and existing optional Rc header. The optional owner comes from
the actual attributed traversal, without inferred owners or a source scan. It consumes the exact enabled item,
configured attributed owner, defining edition and retained parameter/body
forests. It uses compile::declarative_in_edition once to validate matcher,
fragment/follow-set and repetition syntax, even for unused declarations.
Uninstantiated template identifiers are not resolved or executed.

Root decl_macro authorizes syntax. A retained rustc_macro_transparency attribute
requires actual rustc_attrs and a macro owner; it accepts precisely the pinned
transparent/semiopaque/opaque strings, rejecting malformed input and duplicates.
Absent override means opaque for scoped declarations. These facts are declaration
data, not invocation permission. The existing owner traversal must reject wrong
attribute targets without borrowing another owner's state. Legacy overrides
cannot be silently interpreted as the currently hardcoded semiopaque execution;
keep unsupported overrides explicit while preserving ordinary legacy behavior.

Generic Resolver::declare_item delegates its scoped case to this same producer.
The expansion-facing Paths::scoped_item shares ordinary publication finalization
(introductions, exact binding_key, scope/namespace/access and import wakeups).
Engine must not validate the same forest again. It retains an explicit
Body::ScopedDeclaration(Transparency) marker, not an empty executable or a
legacy Compiled body. Measure Body and Definition; no hot AST/type/body/frame
fields are planned.

A scoped item has Namespace::Macro and DefKind::Macro and its actual item context
and visibility. It does not enter Scopes::define, macro_use textual propagation
or legacy macro_export root insertion. Ordinary forward lookup and imports use
the real item binding. Publishing a duplicate or wrong owner must not replace
an existing binding or issue false wakeups.

Engine::attempt rejects a selected scoped marker before input parsing, matching,
capture adoption, new marks or Invocation publication; transcribe has a defensive
explicit guard too. Any enclosing legacy/eager expansion has separate work.
The scoped child's zero-work assertion is relative to its own selected boundary,
not a false claim that its valid parent did no work. No builtin/textual fallback
may execute a selected scoped shadow.

Resolver::bodies skips an already-validated uninstantiated scoped template.
Stability attributes remain owned by the actual existing collector; final
unsupported macro-stability must reject instead of dropping annotations.
Existing public-name export and authored-declaration export reject Macro. Keep
those exact foreign-export frontiers and opponents; no metadata remapping is
fabricated. Source builtin claims on scoped declarations remain unsupported;
known cfg_select is not accepted as an empty template, and unknown roles do not
become inert positive builtin declarations.

## Required qualification

First run the frozen independent corpus under the normal resource/storage
wrapper. Classify Rust language outcomes separately from candidate unsupported
frontiers and harness failures. Preserve all original bytes and predictions.
Then fresh format/audit/all-target strict, one debug build, full nonignored
private inventory plus exact affected tests, and focused source/native controls.
No scoped execution is inferred from matcher unit tests or declaration success.

Counters must show matcher compilation once per declaration in direct and
expanded modes, no second compile on repeated import queries, and no scoped
capture/mark/invocation work on denied calls. Exercise width8/32/128, ordinary
legacy controls, exact contextual identities, visibility/alias/forward lookup,
changed configured owners, duplicates and rejected publication. Measure cold
Body/Definition and retained storage. Re-run the unchanged original core with
exact provenance/configuration and report its actual next diagnostic; an
advanced frontier is not crate acceptance.

## First independent observation

The immutable31-source corpus ran102 observations, including20 native programs.
Exactly100 rows matched the predeclared predictions. The unused unknown template
metavariable was accepted by pinned rustc in both metadata and object modes,
contrary to its predicted rejection. Both original mismatch rows, empty
stderr, source bytes and contract remain retained. This is accepted unused
declaration data; no invocation permission follows. The candidate failed-matcher
invariant instead uses the already declared wrong-separator grammar. No compiler
matcher semantics were changed to force the falsified hypothesis.

Evidence is obj/parallel/pa31/scoped-declarations-independent-v1. Runner5dec88c9,
planfa68afe8 and contractc6e7de62 bind the run. Results SHA-256 is
c8eceeaad3d3eebcb0a69e0619deb98c658bc92a4ac642cb57fad61d217f0e28.
Resource1cb47966 finished1/errornull; scoped cleanup1792853452947711888
removed62 disposable outputs while preserving every observation. This is oracle
evidence, not candidate qualification. The original20 native programs all passed.

The unchanged unknown-substitution source separately passed its missing native
continuation: native-build, link and execution returned0, and actual stdout was
7 followed by a newline. Results SHA-256 is
6feb51115230d6cf5a810f69195bf7c1399da1fba0acd7115d811ee854bc6f8f.
The new3 rows supplement the original102 rather than replacing the two original
mismatches. All31 source programs now have62 compile observations;21 accepted
programs have actual native evidence. Nine sources retain language rejection
observations, while the unknown builtin rejection remains a separate claim-role
frontier. Candidate-only scoped invocation and stability restrictions remain.

Before candidate validation, source dependency821fcb7b retains integrator4653's
attribute_claim extraction and environment-to-reproof module rename. Dependency
3fa5ac2b retains qualifiedfa14's callable marker projection-normalization repair;
the known earlier projection regressions are not reinterpreted as scoped results.
Both are separate from this declaration-only feature. Cold Body and Definition
have no added fields; their actual sizes and the temporary descriptor size are
reported by the private width control when executed.

## Body consumer review before candidate recovery

The complete production variant inventory includes the ordinary legacy/builtin
constructors in expansion.rs and the sole scoped marker constructor in
expansion/unavailable.rs. The selected-call guard in Engine::attempt runs before
cached completion and eager dispatch; Engine::invoke repeats it before input or
context construction. transcription.rs repeats the guard before the existing
Declarative-only body match. The Stringify/eager predicates cannot select this
marker; eager.rs's final wildcard returns an invalid-builtin error. The source
panic dispatcher requires the exact Builtin role. permission_frames.rs rejects
both the marker and the actual typed scoped AST item, including a substituted
body tag; immutable session warm checks cannot mutate a report behind a borrowed
session. No definition is promoted into an executed permission frame.

render.rs has the only other exhaustive body match. CandidateV1 exposed its
missing arm at strictE0004, after fresh format and architecture audit passed;
no tests executed. The recovery adds an explicit scoped-declaration mode and
execution=unsupported diagnostic, without executable rules. The source-view
control checks that marker and excludes a rules count on those lines.
count_expanded counts real declarations without describing them as invocations.
Metadata rejection remains at the existing public-name/authored-export owners.

## Provisional core discovery and actual provider correction

CandidateV3 at a48b40e7 passed fresh static gates and all1437 nonignored private
cases, including36 required names. Body=96, Definition=136 and the temporary
ScopedDeclaration=16 bytes were measured. Width8/32/128 had80/320/1280 input
forest trees, exactly one compile per declaration,8 aliases per declaration,
and no selected scoped execution. Actual denied-call snapshots were unchanged
across the guard, including nested legacy/eager parent work counted separately.
The frontend batch passed10 of12; native was not run and the overall gate
remains failed101. One diagnostic expectation used private macro name instead
of the actual intended private import or widened visibility rejection. The
ordinary original-header fixture hits the existing actual-source-provider
permission boundary before the predicted final stability frontier.

The original source and candidate disagreement remain retained. A distinct
predeclared declaration-source-assert-matches.rs adds the actual no_core profile
and genuine sizing-role declarations; the original attributed assert_matches
slice remains byte-exact. declaration-provider-contract.json declares metadata,
object and native7 expectations before any observation. These new4 observations
are unrun. A real Mode::Source configuration is required for its candidate
expansion; ordinary source must keep rejecting borrowed provider permission.
No production authorization or stability rule changes are part of this recovery.

At root's request, the exact failedV3 CLI was pinned for a separately declared
provisional unchanged-core discovery. It is not a qualified-component claim.
The original root advances beyond the assert_matches declaration at line170
and stops at macros/mod.rs line234 byte8704: rustc_builtin_macro on cfg_select
whose pub macro declaration is line235. The diagnostic is scoped builtin macro
declarations are not implemented. The same4 original files were opened; all
core/source/tool provenance remained unchanged. The compiler exited1/noninfra;
wrapper c09b7d2d finished0/errornull and cleanup1792854985778441011 removed0files.
Evidence: obj/parallel/pa31/scoped-declarations-provisional-core-v1; result SHA
 a672235c3e492411f263cfbda96ea6755f6315c71b98c7c5822f68b632c1f9f8.
CLI SHA1c2298f1ac0ac64e30e3ae6db4ecf5432e86219a69adb64c0ede50747ad2a29f.
Old success-required probe proposals remain unrun. This discovery does not
close declaration qualification, scoped invocation, stability, export or cfg_select.
