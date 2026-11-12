# Captured values inside try blocks

Pinned core's filter_try_fold reaches `try { acc }` inside a returned closure.
The failure140:3088 in iter/adapters/filter.rs:86 points at the try keyword.
Capture-free try closures already passed; copied, owned, mutable and actual
filter-helper captures reached the missing typed-capture visitor case.

The collector now shares existing value-use traversal for Loop and TryBlock
bodies. A try body constructs its result from the tail value, including when
an enclosing wildcard discards that result. Capture Copy permission, Fn/FnMut/
FnOnce classification, stored fields, feature authority, lexical try targets,
lifetime/ownership checking and actual source protocol calls retain their
existing consumers. There is no new engine, cache field, lowering shortcut,
backend/storage change or compiler-limit change.

## Declared contracts and evidence

Thirty-six independently frozen Rust1.98.0 outcomes declare26accept/10reject:
PA24ordinary14 serialized requirements, PA30ordinary-native1 and PA31source21.
All21 complete source outcomes qualify: fourteen capture/ownership/target cases,
six width/noise inputs and one actual native export. Fifteen ordinary contracts
remain required at their missing Try provider/import bindings; they are neither
qualified language rejections nor passing course cases. No assignment is activated.
The source foundations preserve the qualified source-protocol and exact Freeze,
Destruct/drop_glue declarations. The initial defining hypothesis lacked Freeze;
its failed independent setup and original bytes remain in raw evidence, followed
by revised inputs checked independently before candidate execution.

Initial focused receipt1794453757639414841 passes three exact tests in97.97s,
including existing iterator/capture and destructor/storage controls. Final public
receipt1794455612137610151 passes three exact tests across two targets in43.27s.
Full formatting, architecture and selected strict production/CLI/test Clippy pass.
Captured Copy values retain Fn, mutable captures retain FnMut and owned values
retain FnOnce, with one actual stored field. Repeated owned calls, wrong results,
missing gates and question marks escaping a nested closure all reject.

Type inspection is not complete ownership proof: repeat-once and repeat-wildcard
have --emit-types status0, then complete checking rejects the moved closure with
status1. Complete checking retains the required rejection. The production
change is the single missing visitor case; ownership proof is unchanged.

The new real source native program and old wrapped-fold control both execute.
New stdout `try-capture 7 43 23 123` matches independent ordinary/source Rust/C:
the captured input7 survives, the actual custom from_output returns43 instead
of input41, and the actual residual conversion returns23 instead of error9.
Three calls occur in order123. Both native groups pass with the already compiled
harness. This is actual execution, not an unused-body or empty-object claim.

Widths8/32/128 use inference5227/16411/61147, trait selections37/133/517 and
canonical steps1400/1640/2600. There is one stored capture field at every width.
Adding128 unrelated types changes no counter. The iterative visitor adds one
existing body edge per try block; these bounds do not claim optimized speed.

## Preceding CI text contracts

Full43145135573 on exact8c231/sourcecf75 passes all Rust regular tests, including
3065 private tests (zero failures, three ignored) and285 linking tests. It then
fails PA24 text626/642 on16 existing byte contracts, so later native/debug/
independent qualification stages are skipped. All16 differences already exist
in preceding6c; the try-capture compiler has byte-identical outputs to immutablee8
for them. Input bytes match the fully qualified5d source and acceptance/rejection
outcomes are unchanged.

The13 negative diagnostics now report an earlier concrete callable-output or
callable-kind failure. Three positive pointer returns use one fresh local slot,
store the returned pointer, immediately load it and return that same value.
Each slot is unexposed, with no other uses or intervening operations. Substituting
that store/load reproduces the old complete IR byte for byte: no call, ABI,
borrow target or lifetime change is hidden by the update.

The exact16 outcomes and new output contract were declared before oracle
generation. Independent pinned Rust checks all16 and RustGM/CPPGM actually
executes the three positives before generation. Normal scoped generation then
updates only the reviewed16 triples; all28,096 other reference hashes and the
base reference.json remain unchanged. No source outcome is inverted and no
reference is hand-edited. Upstream, maintainer harness and cleanup checks pass,
with28,145 current reference hashes preserved. Receipts and the new batch are
in the machine record.

Clean source `c388b27099c03f75e01adf830346fe3c2a04e3f6` freezes CLI67f146ab with3907 Rust
hashes and an empty tracked patch. Thirty exclusive A/A plus two ABBA accepting
observations retain complete output equality against preceding e8. Wall ratios
are1.272/.959/1.082; CPU1.124/1.043/1.095; A/A wall1.071/.952/.928.
These short controls retain slower/noisy observations without claiming optimized
speed, neutrality or core throughput. Raw measurements and frozen inputs remain.

The unchanged120s original observer ends at its deadline after121.087482s,
with78 partial records and14,320 item attempts. It reaches filter_try_fold4600
at140:3018 with no failure record, alongside all eleven earlier required bodies.
Two preceding failure records are absent when reached; no new failure appears
among previously passing reached e8 bodies. The first remaining failure is
filter_map_fold4634, body141:1230/error141:1253, filter_map.rs:39 at match.
The last attempted body is8892, str/pattern.rs239:68090,line1816. Partial counts
are neither a full census nor throughput evidence. Postcheck verifies all722
source hashes,375 opens, six configuration bindings,3907 current/immutable Rust
hashes and unchanged profile/limits; progressSHA5f7da4e31911e13155c609d3ad4069120d7e1deadab565920678ec1ef44ba52d.

No original core, alloc or std artifact exists. The next reduction owns match
inside closures, using actual typed pattern operations to distinguish discriminant
reads from payload moves. Full successor qualification remains required after
publication and a fresh no-active-full check. Retain all earlier requirements,
stable5d until full success and optimized21f. Preceding e8 remains the recorded
comparison baseline; retire its executable only after this replacement publishes.

The [machine record](pa24-try-captures-evidence.json) binds raw evidence under
Bobj/parallel/filter-try-capture and the preceding CI-reference-review directory.
