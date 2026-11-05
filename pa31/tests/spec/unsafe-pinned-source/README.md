# Actual UnsafePinned source storage

PA31 owns the actual nominal identity, PA12 physical/niche layout, PA16 normal
loans, PA17 field destruction, PA19/20 explicit and auto-trait proof, PA25/27
constant/raw storage and PA29 authenticated transport. This packet releases no
whole assignment and grants no future polling or Pin API permission.

Primary sources are Rust1.98.0 commit88d9e12ae178fab0fb5cc050a94da85685d449ea:
compiler/rustc_hir/src/lang_items.rs, compiler/rustc_ty_utils/src/layout.rs,
library/core/src/pin/unsafe_pinned.rs and library/core/src/marker.rs. The authored
foundation extends the attributed unsafe-cell-source packet; repository NOTICE
and upstream terms apply. Sources retain their original bytes after prediction.

predictions.json froze22 source outcomes and4 native observers before candidate
execution. Independent reference-v2 passes all22 outcomes and all4 observers in
1.5823s, SHA256dafa9ef351dae8ca57cf56afdb7847ad91c8b6929ad956c677bd9b8fb513743b.
Reference-v1 had the same correct statuses/native results; one diagnostic-cause
substring used different wording. Its original manifest and diagnostics remain
in scratch. No source or acceptance expectation was changed.

Native expected output is runtime23 23 23, layout8 16 16 8 (both genuine nested
UnsafeCell and direct-field language owners), and ordinary field-drop=31. The
runtime observer includes two aliased mutable UnsafePinned references. This is
allowed by the actual language role; normal shared-to-mutable source borrowing
still rejects. The current backend emits no Rust noalias/retag assumptions.

The actual source negative Unpin and UnsafeUnpin implementations remain the
proof; removing either restores the structural positive result. Freeze is
inherited from the actual UnsafeCell negative implementation; a direct-field
UnsafePinned does not invent a Freeze refutation. The actual explicit Sync
implementation overrides the inner cell's negative, while removing it rejects.
Explicit valid Copy, arbitrary struct generic arity and field counts remain
ordinary. Niche suppression must apply to the actual role itself, including
when its field does not contain UnsafeCell. A same-name ordinary struct has no
such direct policy.

Candidate qualification passes7 focused public tests, all125 PA12 text cases,
formatting, architecture and strict selected Clippy in103.79s. Both old/new
native test roots pass in34.58s (4 cell and4 pinned observers). Upstream and
full harness checks pass in13.29s. The joined Index/UnsafePinned checkpoint will
receive a new unchanged-core observation and performance check; neither has
been inferred from this source qualification. Full Pin,
projection, async/cancellation, shared-derived constant mutation, wider niches,
optimizer permissions and authenticated interior-storage metadata remain required.

Joined Index/storage qualification is now recorded in docs/parallel/source-index-pinned.md;
original core advances unchanged to actual VaArgSafe. This remains source admission,
not a whole checked library artifact or assignment release.
