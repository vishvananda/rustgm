# Adjusted capture receiver consumers

Method and callable-trait receivers request the retained exact adjusted path
before visiting an unavailable source ancestor. The builder checks the path's
reference-only suffix, source identity, capability and type coverage, then skips
only that prefix in the existing receiver conversion. Each actual call retains
ordinary borrowing, unsizing, ownership and lifetime checks. Validation is cached
per source path, adjusted path and count; ordinary calls keep their previous work
continuation when no sparse capture record exists.

The first source/native run accepted six adjusted positive cases and rejected
both negative controls. The other two Rust-valid cases, `method-deep` and
`call-mutable`, reached ordinary lifetime proof and exposed missing formation
bounds for discarded source ancestry. Their declared accepted outcomes remain
unchanged. The exact child/parent input-formation interface is a separate shared
proof dependency; this consumer source handoff is not final endpoint qualification.
No ancestry assumptions or equalities were added in the consumer.

All five existing typed capture value-check programs execute successfully. Two
additional value controls independently pass Rust 1.98 default-lint qualification:
repeated reads through a deep shared reference and two mutable closure calls whose
final counter must be nine. Their frontend/native runs await the formation proof
dependency. A malformed selected receiver count smaller than the retained capture
prefix is independently rejected by body construction.

Evidence is retained under `obj/parallel/pa24/adjusted-execution`: `first.json`,
`remaining.json`, `probe.json`, `finish-first.json`, and `helper-second.json`.
The two exact-root helper invariants, strict all-target/all-feature Clippy and
the architecture audit passed at `66e9adcf`. The helper selects actual capture and
mention roots with exact child/parent types and bounded shared-ancestry work.

Reference coercions, opaque receiver storage views, stronger callable adapters
and generic captured bodies remain separate boundaries. No assignment activation
or reference regeneration is included.
