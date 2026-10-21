# Captured callable objects with reference Self

The accepted `1857ac39` baseline already supports direct captured closure objects
and physical nested reference receiver adapters. It accepts the ten positive
reference-Self inputs in type and constant inspection, but rejects their LowIR
with `captured callable object requires a checked conversion adapter`.
All fourteen original independently declared inputs are copied byte for byte,
with their SHA256 values verified. The four opposing Rust rejections retain their
original permission, loan and local-escape outcomes.

The shared neutral helper `types::closure::reference_leaf` follows only checked
reference storage heads, returning a closure leaf and a bounded visit count. It
rejects invalid IDs and stops at raw pointers or other nonreference types. The
existing one-million-reference bound remains. The helper grants no call
permission and never replaces the full concrete Self. Body candidate and
conversion selection charge visits; table planning charges one classification
for each newly created callable table and skips that walk for ordinary tables.

Conversion selection reads the captured leaf's signature while keeping the
complete reference Self in its Conversion and ordinary adapter identity. Existing
source matching, binder opening, lifetime conditions, reference mutability and
blanket call permissions remain mandatory per source conversion. Physical table
cache hits do not cache or supply those source proofs. Existing nested adapters
perform the physical reference loads; no new lowering or backend operation is
needed.

Before captured targets are published, every adapter descriptor is validated and
its instance/requested permission must match the selected entry. Checked Self is
the adapter's full receiver when present, otherwise the direct closure instance.
It must equal the conversion's concrete Self. Invalid descriptors publish no
partial targets. New callable tables distinguish captured leaves from ordinary
function targets once, including unused FnOnce views whose consuming slots stay
empty. No owned dynamic FnOnce invocation is added.

The fourteen frozen controls now pass pinned Rust object qualification and all
three frontend views. Ten independent native comparisons pass: shared and weaker
views, mutable and nested references, mixed mutability, generic and quantified
calls, an outer reference result and an unused FnOnce view. The four opposing
controls still reject; loan/local-escape errors remain deferred from type
inspection to checked views. These source bytes and expectations are unchanged
from the integrator's original declaration.

Eight table invariants and fifteen execution/proof invariants pass. The new
checks bind full Self, leaf instance, requested permission and physical execution
ABI; reject wrong Self, omitted adapters and inconsistent requested permissions
without target-cache growth; retain empty FnOnce slots; and count exactly the
touched reference chain while stopping at raw pointers. Existing captured,
quantified, nested receiver, owned adapter and reference-conversion source/native
suites pass. Strict all-target/all-feature Clippy passes, the syntax/architecture
file audit reports 1,465 files and zero findings, and upstream24+126 pins verify.

At 8/32/128 repeated conversions, one table and two captured targets remain
shared. Planning work is 198/678/2598 and canonical work is 504/1968/7824. At
reference depths 8/32/128 with one conversion, planning work is 79/151/439 and
canonical work is 98/170/458. The neutral traversal itself visits depth+1 nodes.
These are bounded structural/work controls, not wall-time or RSS measurements.

Development evidence retains two pre-execution build mistakes (helper visibility
and a test erasure constructor), the unchanged opposing diagnostic that corrected
a test substring, and audit failures for fixture embeds and crowded filename
prefixes. The repairs use normal fixture reads and specific module filenames;
no audit exception or source-permission relaxation was added. These setup/test
failures are not language rejections or backend failures. Raw attempts and the
passing final batch remain under `obj/parallel/pa24/reference-objects/`.

Source fingerprint (sorted dev/**/*.rs path, NUL, bytes, NUL):
`550d235b939cd8fb6044a099b1c2e7ac685fda5d88e801075e9787a1e896cc87`.
Compiler SHA256: `fee48477fdc7aeecdd5de03e06581e0ebf6c6cf22e8436cd68104bc383ebc68c`.
The six generated depth/repetition inputs independently pass Rust and LowIR
with an explicitly declared `recursion_limit = "512"`. The initial default-budget
depth-128 hypothesis failed with pinned rustc E0275; its exact original source,
declaration and diagnostic remain under `close/`, while the corrected declared
profile is separate under `close-recursion512/`. No compiler limit changed.
All active PA1–PA23 text comparisons pass with zero differences, and assignment
layout passes for all 38 stages. The accompanying `reference-objects-outcomes.json`
binds the frozen source bytes, source/native output hashes, all final gates,
scaling profiles and retained development failures.

Empty-closure and ordinary function-reference objects remain separate explicit
boundaries. This change adds no raw-reference bypass, callable auto-trait
permission, general owned trait-object call or assignment activation. Full
combined release and immutable performance qualification belong to integration.
No backend, reference sidecar or active manifest changes are included.
