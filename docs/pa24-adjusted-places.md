# Captured receiver adjustments

PA24 remains planned. Method and invocation capture collection now retains an
exact typed place after each selected implicit reference dereference. A sparse
`Environment.adjusted_accesses` entry is keyed by the consumer site and
`AccessKind::{Method, Invocation}`. Its path precedes capture precision and move
truncation; its count is the selected dereference prefix. The existing `accesses`
map still describes the unadjusted source place. Ordinary zero-dereference uses
allocate no adjusted entry.

The body consumer must construct this retained physical place directly when its
uncaptured source ancestor is unavailable. It must validate the recorded prefix
against the selected conversion and preserve ordinary borrow, unsizing, loan and
lifetime proofs. Merely obtaining an adjusted place grants no additional call or
mutation permission. Reference coercions and overloaded indexing remain separate
handoffs; this map does not claim to implement them.

`pa24/tests/adjusted-place-outcomes.json` declares eight positive and two negative
source controls. They cover shared and mutable method receivers, multiple
reference layers, move capture, a tuple field, function and closure invocation,
borrow conflict and shared-reference mutation. All ten match pinned rustc object
compilation in `obj/parallel/adjusted-places/rustc-v1/`. The source invariant
checks the selected capture mode and call kind and walks exactly the retained
prefix back to the original source path; execution of that invariant and native
qualification are pending combined runtime integration.

The combined producer, owner/root and scalar Image source passes strict release
all-target/all-feature Clippy, formatting, file audit, layout and upstream checks
in `obj/parallel/adjusted-places/static-v1/`. No assignment activation or text
reference generation is included in this producer checkpoint.
