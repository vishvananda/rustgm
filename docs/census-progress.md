# Interrupted core analysis retains diagnostic progress

The 2026-11-11 extension emits `item\timplementation` before each source
trait implementation's conformance check. It uses the existing typed observer,
definition and AST location; it supplies no semantic facts or permission. This
covers declaration work that precedes body events. Focused receipt
1794413794013879880 passes three exact tests, including an actual implementation
location before body progress and byte-identical final tables with and without
the observer. Strict selected Clippy, full formatting and architecture pass.
Item locations now use the same provenance projection as ordinary diagnostics;
the AST key remains available separately to typed observers. This also corrects
function/initializer events for macro-generated nodes. Final receipt
1794414341537683152 passes four exact tests in42.85s, including a generated impl
whose source position matches the ordinary diagnostic and its macro trace. The
canonical final table stays byte-identical. Earlier invalid selection, formatting
and test-helper failures remain in obj/parallel/refcell-coercion; none qualifies
as a passing batch.

A bounded90s diagnostic using the first implementation-event compiler53e2509c1
reached bodies (function3335,file98:11654), after11,356 item attempts. Its stack
was in method applicability during ordinary body checking. Earlier90s samples
placed source7c7 in implementation signature normalization, prior5798 in bodies,
and a non-equivalent diagnostic hint-revert in local annotations. These phase
snapshots do not establish a repeatable performance regression or its cause.
The diagnostic hint-revert is never a qualifying compiler. Original-core
qualification still requires the normal observer and unchanged source bindings.

Compiler9482507e3 adds an optional typed census observer and the CLI option
`--census-progress PATH`. A fresh, flushed stream records the source map,
item attempts and reached failure records. Its header explicitly says
`partial-analysis-only` and `not-a-qualification`; it has no completion summary.
The existing canonical census table and its acceptance rules remain unchanged.
Observer errors are sticky and prevent publication of a final table. All I/O
stays in the CLI. Ordinary compilation has no observer.

The original-core observer accepts this stream only with census mode and
preserves its hash and partial summary on failure. A timeout still returns
failure. No input, source-contract quota or observation ceiling is increased.
The crate argument branches move unchanged into their existing CLI owner.
The original manifest explicitly rebinds dev/bin/rustgm.rs to363d4efa and
dev/bin/crates.rs to4a76d76f; the other four code pins remain unchanged.
All722 original source hashes and the semantic profile remain fixed.

Private1794313627645635120 passes the sticky-error observer check. Production
1794314038699228153 passes all14 census-mode tests, including byte-identical
ordinary final tables, partial stream records, item events, destination/usage
errors and refusal to overwrite progress. Selected/private strict Clippy,
formatting and architecture pass. Harness315,467 roots/13 binaries/44 stages
and upstream128/Reference126 pass. No fixture or text oracle changes.

Immutable dev-fast CLI
792f38b97b91be192abdb05f08be85250414e63087aec3c7ea4d42867e8bc6cc
binds9482507e3, an empty patch and all compiler/test Rust source hashes.
Thirty A/A+twoABBA observations preserve complete ordinary static/correlated
fallback outputs and74 ordinary outcomes. Ratios1.063/1.133/0.985,
A/A0.579/1.010/0.965; no speed claim. Evidence is under
obj/parallel/census-progress, including failed development attempts.

The120-second original-core run failed at120.99s with an observer deadline,
retaining345 failure records and6469 item-attempt events. The last item is
function11792, file296 byte710139, _mm512_mask_cvtepi32lo_pd in stdarch's
x86/avx512f.rs. The stream SHA256 is
2e0483425b2d8436e731cd9c8eb8e5fed1f78760961f33c7fef6d4956c3bfacc.
This is an incomplete prefix, not a completed census or a count of checked
bodies. It demonstrates forward progress through bodies. No original artifact
exists. The old dyn-upcast fence is absent from this prefix; later body failures
can still occur.37 reached inclusive-range call-plan failures form the next
shared semantic reduction group.

Full predecessor42813128062 on2a75c680b passed all fast, release/course,
native, debug and independent stages. The diagnostic successor is due after
publication. Pending optimized nested-proof qualification remains pending.
