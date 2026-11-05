# AsyncDrop source and nominal checkpoint

This finite checkpoint authenticates actual local AsyncDrop trait declarations
and nominal implementations. It does not complete async runtime, PA31/42/43,
original core, alloc or std. Baseline is investigation b44f41f58. Root's helper
integration is deliberately separate: b197353e0,07f2e2c8a,1c93d9524,af08592e3
are shared qualification predecessors, not this lane's implementation.

The trait producer uses the actual source catalog, canonical trait header and
existing completion ledger. Unused empty, ordinary, renamed and static-receiver
headers stay ordinary declarations. Only demanded cleanup authenticates the
actual first member, safe Rust signature, exact single-binder Pin<&mut Self>
receiver and genuine opaque Future/Output=() relationship. No spelling-based
member selection, fabricated future or helper body is supplied.

Destructors owns one optional cold async map. Each entry retains only its real
impl Self definition, nominal item and Coverage. Existing associated schemas
own target/captures/bounds/source, and the ordinary implementation registry owns
member selection/conformance. Async entries never enter synchronous method or
Self indexes; explicit async calls remain valid. A shared nominal proof kernel
and borrowed lifetime contract preserve type/const/lifetime coverage, field
assumptions and the existing single coverage-region clone. Registration seals
after exhaustive preparation; later validation still completes ordinary trait
conformance and required body checking.

The real async_drop request appends bit 67 to the existing checked feature mask
and remains a Library request. The exact language-request exception preserves
library API feature identity without inventing a feature definition. Nominal
applicability remains checked without this request; only the required synchronous
companion is conditional on it. Existing synchronous Drop owns Copy exclusion.
Ordinary and const source Drop preserve their actual constant flag; membership
grants no const execution or Destruct payment.

Scheduler joins are prepare_async_destructors after prepare_trait_implementations
and validate_async_destructors after validate_destructors. Root's existing
compiler-owned helper query consumes actual async_item entries; its state/CFG
boundary remains closed. Source/imported async transport profiles remain fenced.

## Frozen independent and candidate evidence

The PA31 async-drop-source and PA43 async-destructors manifests contain35 cases:
16 acceptance and 19 rejection predictions. Twenty-six exact no_core sources
and 9 pinned-standard-core companions match Rust 1.98.0. Each companion preserves
the authored body tail byte-for-byte; the missing ResumeTy producer is never
counted as body rejection parity. causes.tsv additionally freezes 14 specific
semantic negative causes, all checked independently and by the candidate.

Initial empty/default-only/renamed impl probes produced upstream101/ICE; they
remain raw evidence, not parity. A specialized-without-feature0 prediction was
explicitly disproved by E0366 and corrected to1. Early derive-dependent Copy
negatives and obsolete impl-const syntax failed for fixture setup reasons;
manual Clone/Copy and real const-impl opponents replace them. Final Copy+Drop
requires E0184, applicability requires E0366/E0367, and the ordinary const-impl
opponent requires the non-const-trait cause. Original observations remain under
obj/parallel/async-drop/source-v1, including initial-packet, feature-timing-*,
copy-macro-* and obsolete-const-impl.*.

Local terminal checks:

- fastcheck1793850819880208269: 26 exact tests across frontend/ownership,90.63s;
  formatting, architecture and strict Clippy pass. Covers actual async/Future,
  receiver, destructor conformance/regions and deferred feature availability.
- fastcheck1793851180116048590: 4 affected public tests,35.11s; final repaired
  syntax and semantic diagnostic assertions pass with the same strict checks.
- oracle-receipt.json: 35 statuses and 14 semantic causes pass,1.97s.
- native/receipt.json: ordinary source Drop constructor/borrowed mutation/field
  destruction matches an exact no_core Rust object and the locked b7c6b8e72
  backend,0.22s. Stdout is record5,scalar7,record5,borrowed9,record5,record3,owned8.
  This is synchronous runtime evidence, not async execution evidence.
- support-receipt.json: registration 410 roots/13 binaries, assignment layout 44,
  upstream 28 fixtures and 126 Reference files pass. No private harness link,
  release sweep, broad native matrix or new benchmark was run.

Early failed gates retain naming/format/doc/slice fixes and actual feature/Drop
header producer failures. They are not successful semantic evidence. Root owns
combined helper/private/original-core qualification and the cold-owner size
expectation update (7112 to7120); this lane changed no dense body/node record.

## Fingerprints and integration

Hashes use SHA-256. Aggregates hash sorted path,NUL,content-hash,newline entries.
The final fastcheck receipt binds 3474 code/tool/manifest source entries; the
fixture aggregate separately binds all 35 sources,9 companions and native observer.
Fastcheck receipts are obj/qualification/fast/<number>/receipt.json; the others
are obj/parallel/async-drop/source-v1/<name>. All raw evidence stays in obj.

| Artifact | SHA-256 |
| --- | --- |
| Qualified source aggregate | edc9ec4d4bf30581f435eb3b32529ad361fdef42b9b4f2e242926f187df5ede9 |
| Fixture/companion/observer aggregate (45 files) | 55fc1cf73708265b24915a7ce3fbfbb20e37dd83275b3be45bdd818620014b97 |
| handoff-evidence.json | 3038e1b1a853a5fcf503638d040fa360b0790f975fd283633f62a88f1a8e8743 |
| contract.json | 1c9806a28a78bb5ea9c964b03c799daf6765155fa302c128518ef9aa2d3370ae |
| Full focused receipt | be1b537b60e19a0bb527ac168a1d3ab41565460aa4edae1444e30da1663146c3 |
| Final affected receipt | 6786f1554de2a0ea19d0b841b052c8512f9a946e8e34e6a79a59938d69e0c913 |
| Independent receipt | d3fc2524ad9cfaa209f8ede1fc2957b11540cf4d00443cd223968264b821d266 |
| Native receipt | d95eea2fcd4d0f9b0875f32ca4e15398aee0fb610fa4cfaafdbd3647f9f2b8cb |
| Support receipt | ac8117ebc5a343c0670c1e686d377baf80d2a92276b2f28a9d0a79f8449fbc7b |
| Native LowIR | 16594a584accd144622f0edba387dfa7a8a7e38fe01197dffe990d77b1f103a1 |
| backend.lock.json | f60c0760f39f1f554486f5a1eb990d2c0fef663fbd1bff944cd1eef8a014a581 |

handoff-evidence.json records the fixture aggregate and six exact pinned upstream
file hashes at revision 88d9e12ae178fab0fb5cc050a94da85685d449ea. No backend pin,
source metadata profile, spec, tracker or original upstream source was changed.

## Required remaining work

Generated future/state construction, real poll protocol and execution, suspension
regions/loans, completion, cancellation, recursive field/array destruction, sync
fallback and actual selected operation obligations still require implementation.
The written async_drop_in_place placeholder must remain nonexecutable. Existing
projected nominal lifetime-contract fences also remain required general coverage
work. Imported async contracts need authenticated transport. Root's private
static-receiver demanded-operation opponent is authored but not linked locally.
No whole-library source/artifact success follows from this checkpoint.
