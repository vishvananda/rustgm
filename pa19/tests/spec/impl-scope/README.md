# Implemented trait scope (PA19)

The original core string-literal `fmt(f)` inside a module-qualified Display impl
reduces to an ordinary user trait, primitive/str receiver and module-qualified impl.
PA19 owns every feature here. Expected object, native and complete LowIR outcomes
are declared in contract.json before implementation and generated references.

Positive controls retain explicit import/qualification, alias/reexport headers,
nested functions/blocks, expression macros, other trait arguments, inherent and
parameter-bound priority, references and type-qualified calls. Negative controls
retain outside/different-impl invisibility, missing proofs and real ambiguities.
Supertrait scope is checked separately for Self and an unrelated concrete receiver.
Width8/32/128 with128 unrelated types checks bounded shared lexical indexing.

The initial concrete-Self supertrait prediction was rejected by independent Rust
E0599 before implementation. It is retained as a negative; a generic receiver with
the actual supertrait bound is a separate positive. No oracle was generated.

The generic-receiver control calls `self.0.parent()` on the bounded parameter;
its initial `self.parent()` spelling instead called the concrete wrapper and was
independently rejected before being corrected. Both original predictions and
Rust errors remain in the local evidence. All34 final inputs were independently
checked before implementation:25 accept/native pass,9 reject.

The visibility index charges each unrelated unit struct once for its scope and
two bindings (384 steps for128 structs), independent of call width. Inference,
canonical, trait selection, method work and probes remain unchanged by that noise.
The initial two-binding-only test omitted scope accounting; its failed receipt is
retained and this three-component bound was reviewed before the final rerun.
