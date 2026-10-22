# Nested reference growth: measured producer and required follow-up

The concrete-v2 probe uses the frozen compiler production sources at 1b93560f.
Its plan and completion receipt retain the library, probe and input SHA-256s.
The source contains a linear chain `let r0 = &f; let r1 = &r0; ...` and one
FnOnce caller. No instrumentation was added to the compiler for this probe.

| Reference depth | Caller statements | Regions | Constraint edges | Seed additions | Loan work |
| --- | --- | --- | --- | --- | --- |
| 8 | 60 | 54 | 174 | 272 | 2,431 |
| 32 | 180 | 594 | 2,238 | 3,392 | 23,567 |
| 128 | 660 | 8,514 | 33,534 | 50,432 | 397,436 |

Every measured caller has one block. The generic once template separately keeps
three regions, no edges and 219 loan steps at all three depths. The observed
caller counts fit regions = n*(n+5)/2+2, edges = 2*n*n+6*n-2, and seeds =
3*n*n+10*n. This establishes quadratic growth before downstream point-set
propagation; it does not establish that every compiler phase is quadratic.

Source inspection traces a producer to Checker::coerce_one: when the destination
head is still unknown, inferred_regions freshens all free lifetime positions of
the source term. An unannotated simple let uses precisely this path. The fresh
walk deliberately distinguishes positions and preserves bound slots, rigid
nominal inputs, closure identity and function-item coupling. Replacing its map
or bitset would leave the expanding region graph intact.

A follow-up must compare the same chain with a plain scalar referent and a direct
dereference value check, separating ordinary local inference from callable
adapter discovery. Any identity-sharing optimization needs independent positives
and negatives for later mutable assignment, covariant shortening, invariant
mutable referents, nested function binders, partial annotations, branch joins,
input/result escape and closure-captured lifetimes. The existing tests in
region_inference explicitly require independent directed sources for a mutable
inferred binding; blindly retaining its initializer type would be incorrect.

No production optimization or linearity claim accompanies this diagnosis.
