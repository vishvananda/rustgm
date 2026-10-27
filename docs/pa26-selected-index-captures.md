# Selected Index and IndexMut capture join

The unchanged pa26/tests/spec/index-mut/captured.rs now reaches checked LowIR and
executes successfully through the pinned CPPGM backend. Its SHA-256 remains
c9065b4f217c4eeaa8f1e8acb8465d8a58f273258fcf69738132d1a5ce9f5415. Preserve
NOTICE and the pinned Rust source attribution. This checkpoint does not activate
PA26 or publish the proposed operator-kernel assignment unit.

Capture-use collection consumes the existing selected Index/IndexMut method's
actual reference borrow, independently of the result's leaf Copy capability.
Index shares its receiver; IndexMut mutably borrows it. The existing capture
place graph chooses precise fields, shared-reference truncation, unique immutable
captures and move storage. Index results remain invocation-time places; they
cannot become captured aliases. The index expression retains its own ordinary
uses, including dynamically selected or separately captured values.

An Index variant in the existing one-byte AccessKind identifies the consumer in
its existing sparse adjusted_accesses. Actual automatic reference dereferences
are selected at the Index site, while the selected language method's separate
receiver adjustment remains zero. The builder reuses validated path/type prefixes
and subtracts consumed dereferences once. Built-in array/slice indexing retains
its original whole-container and leaf Copy rules. No new owner, map, cache, body
field, declaration role or metadata/backend profile is introduced.

The ordinary CFG still evaluates a receiver before its index argument and uses
an immediate Index/IndexMut receiver borrow. Primitive compound assignment keeps
its existing RHS-first evaluation. Reads, moves, reference mutability, conflicting
loans and lifetime requirements continue through ordinary body proof. Capturing a
mutable reference does not require mutable binding permission for that reference;
replacing it retains the independent binding rule. Nested captures import the
existing paths instead of inventing a second projection representation.

## Frozen observations and executable exit

Prediction commit a6a84e8e1 precedes 22 independent metadata observations. Followup
commit a34eac451 precedes five more. All 27 match pinned Rust 1.98.0, with no
infrastructure failures or prediction disagreements. The original unsupported
candidate error and outcome remain historical evidence; they are not a language
rejection. The other seven original operator-kernel positive gaps remain required
and unchanged.

The first source gate passes six exact tests in 112.41 seconds at
fast/1793134724907478023, including strict production/CLI Clippy, full format and
architecture checks, actual capture modes/prefixes and existing closure controls.
Native fast/1793134943860428685 passes in 47.29 seconds. It independently executes
21 rustc/CPPGM pairs: 18 valid captured profiles, original store/compound order and
an original scalar dynamic-index closure. Every execution exits 0 with empty
stdout/stderr. Each successful native case retains source/LowIR and executable
hashes, then retires the generated executables through the existing harness.

Only after actual source/native success does the integration test move captured
from REJECTED to ACCEPTED and remove its Rust-only acceptance exception. Its draft
outcome becomes exit 0/native execute-pass. No official references are edited or
generated, and publication remains the integrator's separate responsibility.

## Bounded state and remaining scope

Repeated adjusted index sites use the existing graph and sparse access rows.
Final fast/1793135085759819391 passes four exact tests in 50.25 seconds, including
all 44 inherited mutable-index outcomes with the original captured case accepted,
24 checked source opponents, selected mode/prefix controls and scaling. Its strict
production/CLI Clippy, format and architecture checks pass. The same qualified
suite binary emits actual counters and negative diagnostics; upstream verification
passes all 24 pinned fixtures and 126 Reference files. Native and final compiler
fingerprints differ only in the integration test mutation.rs: the legacy candidate
predicate was changed after native qualification, with identical compiler bytes.

| Index sites | Inference/body work | Interned paths | Captures | Adjusted rows | Map capacity | Capacity payload bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 974 | 2 | 1 | 1 | 3 | 60 |
| 8 | 2,912 | 2 | 1 | 8 | 14 | 280 |
| 32 | 9,416 | 2 | 1 | 32 | 56 | 1,120 |

An ordinary direct-array closure retains one path and zero adjusted rows/capacity.
AccessKind is one byte; AdjustedAccess eight; Environment 328, Body 456 and
Operation 16. Their existing fields/widths are unchanged. Map payload estimates
count retained key/value slots and exclude allocator/hash-table overhead.
Negatives retain actual immutable binding, shared-storage, live-loan, absent
indexing hook and non-Copy reference-move rejections. Payload-capacity observations are
logical Rust storage estimates, not allocator bucket or wall-time speedup claims.

This closes the declared finite captured indexing profile, including shared,
mutable, deep-reference, moving, nested, unsized slice, non-Copy output borrow,
precise field and dynamic-index controls. Further closure grammar, user Deref
hooks, imported capture transport and the remaining seven operator-kernel gaps
remain independently owned work; this supplies no new language/backend shortcut.
