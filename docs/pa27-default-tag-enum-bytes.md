# Ordinary multi-variant enum byte storage

This checkpoint consumes the existing completed direct-tag ABI for ordinary
Rust enums. It does not change enum layout, introduce a public storage shape or
claim that Rust's implementation-defined layout equals rustc's layout.
The pinned Reference `src/type-layout.md:166-187` (submodule commit
86635e30bf861a038dc197d7e16fd09e7e514e7a) specifies the limited guarantees of
`repr(Rust)`. Canonical Option's separately guaranteed nullable layout remains
separate and keeps its existing Niche precedence.

## Frozen contract

The unchanged required source is
`pa27/tests/spec/default-enum-bytes/default-layout-still-required.rs`, SHA-256
f2922eaa1095252fe92781b9fc71b52bf45c1769f564389a4267e995cfc431b9.
It and the original default enum sources retain their original bytes.
`obj/parallel/pa27/default-tag-enum-bytes/declared.json` and
`first/qualification.json` preserve twenty independently declared outcomes.
All match pinned Rust; fifteen native positives pass. The baseline compiler
bc90b163d1e8dbd8c90cca281aea0768f9bec09a0e2e25426165e4997754d980
rejects the original required case and the new byte-storage positives, while
ordinary whole-array typed copying already passes and is not new coverage.

The original `active-invalid-bool` expected-Rust-rejection hypothesis was
previously disproved and is not rewritten. Rust's chosen bool niche maps byte2
to Empty. This compiler retains a separate I64 tag, so writing2 to its selected
bool payload leaves A with an invalid bool. This is a target-layout observation,
not a portable Rust rejection or native-validity contract. It lives in the
separate layout_observations list and separate maintainer test.

## Typed consumer boundary

`Reader::enumeration_shape` admits ordinary Rust enums only with the exact
retained Aggregate tag of I64. Explicit tags and C tags retain their original
checks. Unique tagless layouts and canonical nullable layouts keep their
existing paths. The existing full-TypeId tag index maps signed/sparse physical
bits to canonical VariantIds; the iterative codec visits only the selected
payload. Tag caches never establish initialization, scalar validity, provenance,
liveness or access permission. No Value/View/Origin/Pointer/Frame layout changes.

A paired ordinary baseline control is important: default E<closure> fields
already work, while an explicit-tag counterpart hits the existing byte-family
boundary. Physical-offset knowledge must not force whole-parent byte conversion
for those ordinary reads. The private const-specialized address-place entry
therefore shares the existing iterative place walk but does not reuse or fill
the logical projection cache. Borrow/Address use it; ordinary values retain the
existing typed path and cache. Default fields select physical byte offsets only
for an actual address/raw/converted-storage consumer. Unsupported byte families
retain ordinary typed references using prepared type/layout capability facts;
they are not discovered by a failed conversion or granted new byte permissions.
A converted root remains exclusively byte-backed.

Field addresses retain exact canonical owner/variant/ordinal and completed
physical offsets, independently of later tag changes. Incoming pointer write
restriction and logical/physical bounds remain separate from the family cache.
The zero-backing ordinary path avoids an ancestry walk. Address replay charges
its actual work; it does not widen Frame caches or retain a second proof source.

Immutable Image paths and static storage are classified from the actual retained
root and checked field path. An Image elsewhere in the evaluator does not force
an unrelated closure-bearing root into byte storage. The readonly Image-path
helper is shared with the existing write consumer; no whole Cell freeze or new
layout traversal is introduced to make that decision.

## Qualification

The final frozen production snapshot passed strict release Clippy and the Rust
audit (1,729 files, zero findings). Thirty-three focused enum invariants passed,
including sparse negative tags, malformed layout facts, current initialization
and provenance on warm caches, selected-reference expiry, tiny-budget rollback,
repeated field access and address replay. The earlier release invariant run and
the latest debug invariant run are retained independently.

The final source group independently checks 23 cases against pinned Rust and all
three compiler views: 19 valid positives and four validity negatives. Nineteen
literal native comparisons pass against pinned Rust and the frozen CPPGM tools.
The separate bool-layout observation preserves Rust acceptance and this target's
invalid-bool rejection without claiming portable representation parity. Tagless
enum and nullable Option source regressions also pass.

The first final source batch stopped because the null-reference negative failed
earlier than its expected diagnostic: the actual rejection is `symbolic pointer
read requires one complete provenance word`. The expected substring was corrected
without changing the compiler or the source. The failure and successful followup
remain in `functional-first/` and `functional-second/`. Earlier failed naming-audit
and immutable-Image field-routing attempts remain in `foundation-first/` and
`foundation-second/`; the latter motivated the shared Image-path classification.

Address replay varies depth 1/8/32, repetitions 8/32/128, and unrelated type count
0/1,024. At repetition counts 8/32/128 the step totals are respectively:

| Depth | 8 | 32 | 128 |
| --- | ---: | ---: | ---: |
| 1 | 1,711 | 6,127 | 23,791 |
| 8 | 2,964 | 10,740 | 41,844 |
| 32 | 7,260 | 26,556 | 103,740 |

The marginal iteration cost is exactly `164 + 20 * depth`; unrelated types change
neither work nor the checked result. All 18 generated enum sources independently
object-compile with pinned Rust and pass the three candidate views. Three distinct
ordinary-record counterparts also pass Rust and have byte-identical baseline and
candidate outputs in each view. They are separately declared inputs, not rewritten
qualifications of the enum sources. Existing width/repetition controls show one
backing conversion with warm selected-field work independent of payload width.
These are logical-work invariants, not wall-time or RSS neutrality claims.

The closing tail passes 98 Python harness tests, layout checking, and verification
of 24 upstream fixtures plus 126 pinned Reference files. Evidence, exact commands,
source/tool hashes and the immutable candidate pin are recorded under
`obj/parallel/pa27/default-tag-enum-bytes/`; `handoff.json` indexes the receipts.
The integrator owns combined active-text and release/performance qualification.

Default/niche optimizations beyond this direct-tag family, unsupported payload
backing, broader pointer validity, source-library bindings and remaining PA28
families stay required work. Ordinary typed closure fields are supported controls;
their general byte backing remains separate. This checkpoint does not activate
assignments or change generated references.
