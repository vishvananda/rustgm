# PA28 runtime-memory component release

This releases the finite **runtime-memory** work unit. PA28 itself remains
planned. The graded contract of this unit is its published manifest plus the
deferred list below.

## Released contract

The canonical manifest is [runtime-memory.json](../pa28/tests/units/runtime-memory.json):
323 declared outcomes, 228 accept/95 reject, all `--emit-lowir`, under Rust
1.98.0, edition 2024, x86-64 Linux, checked overflow and aborting panics. 82
accepted programs execute and pass, and 146 are code-generation-only. 265
cases use file input; the 58 programs of the four `*-memory-native` families
use standard input behind the declared check prelude.

| Family (`tests/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `spec/owned-pointer-memory` | 49 | 30 | 19 | 0 |
| `spec/unaligned-memory` | 32 | 16 | 16 | 0 |
| `spec/counted-memory` | 36 | 22 | 14 | 0 |
| `spec/overlapping-memory` | 46 | 32 | 14 | 0 |
| `spec/byte-fill` | 39 | 25 | 14 | 0 |
| `byte-fill-native` | 15 | 15 | 0 | 15 |
| `owned-memory-native` | 14 | 14 | 0 | 14 |
| `unaligned-memory-native` | 10 | 10 | 0 | 10 |
| `counted-memory-native` | 11 | 11 | 0 | 11 |
| `overlapping-memory-native` | 23 | 23 | 0 | 23 |
| `spec/volatile/source` | 36 | 18 | 18 | 0 |
| `spec/volatile/effects` | 3 | 3 | 0 | 0 |
| `spec/volatile/native` | 9 | 9 | 0 | 9 |

Outcomes come from the committed draft tables for requirements 28.1 and 28.2:
the `volatile` rows of `manifest.draft.json`, `owned-memory-outcomes.json` and
the `unaligned-memory`, `counted-memory`, `overlapping-memory` and `byte-fill`
contracts. No fixture was authored or edited. Each contract outcome is the
table's pinned-Rust outcome. Four schema conversions apply:

- The byte-fill contract names a separate standalone runtime observer for 15
  of its sources. Each observer under `byte-fill-native/` is published as an
  executed program; its `spec/byte-fill` source stays code-generation-only.
- `volatile/source/denied-unsafe-function.rs` is rejected by a lint level and
  declares `default_lints`.
- `volatile/source/implicit-unsafe-function.rs` is accepted with the reviewed
  `unsafe_op_in_unsafe_fn` warning, which pinned rustc reports too.
- The 58 native observers of the owned, unaligned, counted and overlapping
  tables call the maintainer native harness's `check`, which that harness
  supplies as a fixed prelude. Each case declares
  `prelude: tests/prelude/check.rs`, a byte copy of that prelude, and both
  compilers read the prelude followed by the fixture as one translation unit.

## Deferred list

| Fixtures (`tests/`) | Rust | Observed with the candidate |
| --- | --- | --- |
| `spec/counted-memory/`: numeric-zero, zst-numeric, zst-null, null-source-zero-allow, null-destination-zero-allow; `spec/overlapping-memory/`: the same five names; `spec/byte-fill/`: zst-null, null-zero-allow, numeric-zst, numeric-misaligned-zero, numeric-zst-literal (15) | accept | `ERROR: constant counted numeric pointer access is not implemented` |
| `spec/byte-fill/`: packed-field-zero, packed-field-one (2) | accept | `ERROR: constant counted pointer is unaligned` |
| `spec/volatile/unsupported/`: aggregate-read, aggregate-write, zero-sized-read, aggregate-pointer, wide-pointer-read (5) | accept | `ERROR: aggregate and zero-sized volatile operations are not implemented` (wide-pointer-read: `aggregate and wide-pointer volatile operations are not implemented`) |
| `spec/owned-pointer-memory/static-mutable-read.rs` (1) | accept | `ERROR: constant execution cannot read mutable static storage` |
| `spec/counted-memory/`: null-source-zero, null-destination-zero, misaligned-zero, zst-misaligned; `spec/overlapping-memory/`: the same four names; `spec/byte-fill/null-zero-default.rs` (9) | reject | Rejected for the wrong reason: `ERROR: constant counted numeric pointer access is not implemented`. An implementation-gap message is not published as an expected diagnostic |

The 23 accepted programs in the first four rows are Rust-valid and are not
published as expected rejections.

Not owned by this unit and unchanged: the `source-transmute` rows of the draft
manifest (28.6); the two PA25 `inline-owner-context` controls listed in the
owned-memory table; `byte-fill-native/invalid-char-transport.rs`, which the
byte-fill contract records only as the counterfactual of its published
`-literal` variant and gives no outcome; the atomic families (atomic-u32); and
the intrinsic-declaration, reference-transmute and remaining PA28 packets.

## Qualification

The candidate is the release CLI built from clean `80229d28d`, SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. Checks ran on 2026-11-15 through the resource and
storage guards:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit runtime-memory` | 323 cases valid |
| `check_syntax.py pa28 --unit runtime-memory` | 323 independent rustc checks agree (7.1 s) |
| `course.py pa28 --unit runtime-memory --bless` | 323/323 declared outcomes, then references written (2.8 s) |
| `check_layout.py --unit runtime-memory` | complete oracle provenance |
| `course.py pa28 --unit runtime-memory` | 323/323 against the committed text (1.9 s) |
| `check_native.py pa28 --unit runtime-memory` | 323 checks; 82 executable and 146 code-generation comparisons (10.6 s) |

The first release published 265 cases and deferred 90, of which 58 needed the
check prelude. The prelude input was added to the course runner and the
checkers the same day and the unit was probed, declared and checked again:
all 355 candidates ran, the 58 passed, and no declaration or sidecar of the
265 earlier cases changed. A review of the rejection diagnostics flagged one
implementation-gap wording, whose nine cases are deferred above. Receipts are
under `obj/publication/runtime-memory/`.

## Requirements

Requirement 28.2 and the runtime-memory share of requirement 28.1 are
implemented components covered by this release. The const-memory share of 28.1
and requirements 28.3 to 28.7 are unchanged. No performance claim is made.
