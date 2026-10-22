# Canonical iteration method visibility

The iterator implementation originally supplied selected calls for `for` and
qualified calls but omitted canonical iteration declarations from ordinary
method lookup. Independently Rust-positive `i.next()` and `i.into_iter()`
therefore failed with `no applicable method for this receiver`, including
explicit imports and generic Iterator bounds. The unchanged generic-reference
prototype is retained as an additional positive; callable storage readiness is
the separate prerequisite `cf81a22f`.

The existing visibility index now includes methods from the exact prepared
Iterator/IntoIterator records. Explicit, aliased and anonymous imports enter
its existing lexical sets; implicit-prelude modules use its existing prelude
candidate path. The tiny canonical `owns_trait(DefId)` query is also used by
applicability so inherited Self parameters receive the same ordinary
instantiation as other registered language traits. Nothing recognizes method
spelling as selection evidence. The selected complete method Instance,
associated-type obligations, receiver coercion, loans and source-body proof
continue through the existing machinery.

No records, schemas or persistent caches were added or widened. Index creation
visits each registered member once, and subsequent calls reuse its existing
scope/name cache and work counters. No iteration registry or visibility work
is introduced for the scalar-only control.

Fourteen source outcomes were independently settled with Rust 1.98.0/2024.
They cover prelude, explicit/aliased/anonymous imports, explicit no-prelude
imports and missing-import rejection, generic bounds, unchanged generic
reference Items, source lookalikes, inherent priority, shared-receiver
rejection and two user/canonical ambiguity cases. Ten positives execute using
their literal fixture bytes in both Rust and CPPGM native paths.

One initial hypothesis was wrong: introducing a same-named local Iterator
trait does not remove canonical prelude method candidates. Rust rejects that
source with E0034 when both implementations provide `next`. The old frontend
accepted it. The original source/hash, expected-positive hypothesis and actual
Rust diagnostic remain under `obj/parallel/pa26/iteration-visibility/declared-v1`;
`declared-v2` records the corrected negative without changing its bytes. The
other ambiguity control likewise exposes an older false acceptance. Missing
no-prelude imports remain actual lookup rejection, not an implementation
boundary. Collection iteration and the other PA26 families remain required.

Early test wiring observations are retained: the grouped child module needed
an explicit path, and the failure message must format the error rather than
LowIR Program, which has no Debug implementation. Neither changed the
production implementation or declared source outcomes.

## Closing evidence

`probe-v3` passes all 14 source outcomes and ten literal-source native pairs.
Its private invariant records visibility work 100/172/460 for 8/32/128 calls,
and 103/175/463 for one call plus 8/32/128 unrelated functions: both increase
by exactly three units per added input. The scalar-only control has no iteration
registry and zero visibility work.

`close-v1` passes the existing selected-dot and comparison outcome groups,
strict release all-target/all-feature Clippy, architecture audit (1,716 files,
zero findings), layout, discovery and upstream checks. `review-v1` confirms
both ambiguity failures specifically report multiple applicable methods, shared
receiver rejection reaches ordinary mutable-borrow authority, and inherent and
source-lookalike LowIR outputs/status/stderr remain byte-identical to the
callable-storage baseline. The candidate is
`e30f843440d3779f7c7b99ad2ace466bd5d085c250e324449b156e3e4b2dba9f`;
the baseline is the immutable `cf81a22f` production pin documented above.
No references or activation state were changed. Full release and active-text
qualification remain centralized at integration.
