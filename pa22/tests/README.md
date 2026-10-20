# PA22 fixture ownership

Every source is registered in `manifest.json`. The 328 cases contain 236
acceptances and 92 diagnosed rejections, independently checked with Rust 1.98.0
object generation. LowIR text is the course oracle; rustc has no LowIR writer.

| Family | Cases | Responsibility |
| --- | ---: | --- |
| sizedness | 78 | Completed storage tails, scoped requirements, readiness and coherence |
| coercions | 47 | Contextual reference conversions, mutability and lifetime proofs |
| projections | 11 | Normalized tail fields and multiple changed marker arguments |
| pointers | 42 | Raw coercions, reborrows, comparisons and metadata forwarding |
| addresses | 27 | Raw place permissions, immediate conflicts and alignment |
| casts | 45 | Explicit metadata-preserving/discarding and integer address casts |
| constants | 49 | Persistent backing, selected lengths, promotion and CTFE restrictions |
| integration | 29 | Representation, reified DST bodies, UTF-8 lengths, bounds and destruction |

The 102 native executions have 97 clean successes and five declared panics.
Another 134 positive controls compile and link without asserting runtime behavior.
Several reify a function explicitly to force emission when safe source cannot
construct an unsized tuple or fixed packed tail argument. An empty `main` never
counts as exercising the declaration under test.

`maintainer_origin` identifies original RustGM test tables where applicable;
these are course-authored cases, not upstream copies. The integration programs
are independently declared controls. No fabricated pointer is dereferenced.
All expected outcomes precede oracle generation. Generated sidecars and the
reference receipt must be regenerated through the course runner, never edited.
