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
Resolver::declare_scoped(Context,NodeId,Item,Access) returns the actual declaration
Context and validated Transparency. It consumes the exact enabled item,
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
