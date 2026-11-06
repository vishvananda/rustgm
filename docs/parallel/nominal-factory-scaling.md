# Nested factory-context algorithm checkpoint

Repeated nominal factory calls exposed two growing suffix walks: constructor
unsizing hints descended through already Sized tail parameters, and partial
callable detection revisited the same completed inference subgraphs. The first
shortcut alone reduced work but failed the new doubled-depth invariant; its
149.31s failure is retained at fast1793966885696158978. Profiling then located
the callable-query traversal. The final query records the existing Completed
summary in its own postorder walk and reuses only immutable complete subgraphs.
Unknown inputs do not become cached absence, no canonical types are interned
early, and no Context/Term field or independent cache is added.

For nested depths8/16/32/64, ordinary inference work changes from
717/1647/4271/12587 to629/1215/2383/4715. Borrowed inputs change from
865/1931/4827/13687 to745/1427/2787/5503. Adding128 unrelated nominal owners
changes none of those counters. All16 frozen inputs independently pass Rust;
the retained depth128 predecessor fails the existing syntax nesting limit,
which is unchanged. The public regression enforces doubled-depth linear growth
and catalog independence. Two private controls cover completed reuse without
interning and a hole that subsequently becomes a higher-ranked callable; their
runtime execution remains due in full CI.

Twenty-four exact public controls across declaration/frontend/closure targets,
format, architecture and selected strict Clippy pass in125.31s. Both original
Coroutine native tests and the full PA18 text suite pass in38.79s. The unchanged
full from_yeet reference function and ordinary header/negative controls have also
been frozen independently for the next source producer; that feature is pending.

Immutable A12836562 and the qualified successor run48 A/A+ABBA observations on
identical depth64 inputs, retaining every sample and exact stdout/stderr equality.
The unoptimized CLI paired B/A wall ratios are0.787 ordinary and0.879 borrowed;
CPU ratios0.784 and0.876. A/A positional wall ratios are recorded separately in
the evidence JSON. These are small development-input measurements, not a claim
about full core, optimized builds or general crate throughput.

The main core frontier remains the last observed actual from_yeet declaration;
no original core/alloc/std artifact or whole assignment release is claimed.
