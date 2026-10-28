# LIB-060 concrete dispatch feature scope

Concrete body dispatch now authenticates its actual source body/contract and
retains only that caller's feature membership. The existing immutable index
shares its arena-bound feature set into a paid feature-only view. Dispatcher
caller storage changes no fields; its ordinary environment remains absent and
the view grants no symbolic trait, type or lifetime premise. Feature-free bodies
keep the existing default dispatcher and cache behavior. Scoped selections cannot
enter the global closed-target cache.

Actual function/closure/initializer/anonymous source identities supply the scope
through existing completed environments. Feature IDs have no substitution
payload, so authenticated source instances share the original feature snapshot.
Missing imported/adapter evidence supplies no feature permission; explicit
metadata transport remains required. No source scan, string/set clone, callee
marker or ambient enabling is introduced.

Private gate `1793225144567741758` passes five exact controls in 235.90s, including
strict private Clippy, formatting and architecture checks. Both unchanged
`matching-caller.rs` and `generic-forward-guarded.rs` reach the public frontend,
roots builder and LowIR writer. Opponents cover missing/unrelated callers after a
successful selection, wrong source, original hard source failures, foreign arena
and failed payment. Feature-only views cannot prove an accompanying trait;
warm work/bytes stay constant across 8/32/128 features. The actual handler control
and existing default dispatch reuse also pass. Failed payer work reaches body
accounting through the retained constructor Result, including failures.

The initial strict Clippy style failure is retained. Production/CLI and joined
native execution remain pending; Root owns those checks. This checkpoint grants
no metadata, original-core or release completion claim. Source-linked evidence
is under `obj/parallel/concrete-feature-dispatch-v1/`.
