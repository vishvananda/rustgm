# PA25 retained storage and query reuse checkpoint

Required expression sessions now share one report-owned value-query scratch
cache. Completed roots return the cleared cache instead of discarding its
append-only definition/initializer indexes. `constant_work.value_preparation`
counts the actual new index/source-budget preparation visits; repeated-root
controls at 8/32/128 roots require preparation bounded by the report's indexed
entries, with exactly one CFG for the const function and one for `main`.
These counters do not include the existing required-initializer table lookup;
replacing that lookup with an append-time index is the next small patch.

Suspended interpreter states retain their frames and symbolic cells, while their
query scratch moves back to the common cache. The provider charges all parked
states together and subtracts their retained slots from the next active
machine's allowance. A nested initializer-annotation query cannot obtain a fresh
full allowance while another query is parked. Frame pushes include existing
symbolic cells before allocating frame arrays. The nested source control passes
with a larger allowance and rejects at the independently sufficient isolated
query bound.

The owned-storage contract is a saturating logical payload charge, not a heap
measurement. `Body::retained_bytes` includes the body object, every owned vector's
capacity, storage scopes/exits, owned-value proof vectors/index, promotion
vectors/index and resolved-callable map, including separately boxed objects.
`Template::retained_bytes` additionally includes destruction requirements and
the template object. Shared immutable `Arc` static maps and template environments
are charged to their original owner; clones exclude those pointees. Allocator
headers and hash control bytes are excluded. Existing maps charge entry
capacity; newly constructed promotion maps reserve a conservative four entry
slots per element before allocation under the pinned standard library.

Execution snapshots charge their own copied CFG fields, promotion map and
callable-target map before cloning. They omit proof-only body data they do not
retain. Full early generic-template copies use the full template charge before
specialization/handoff; the existing generic-instantiation budget also uses the
same full-body accounting. The code and template budgets remain distinct
64 MiB logical budgets. Interpreter slots bound logical frame/value/cell entries,
not resident bytes or collection spare capacity.

The nested control exposed a separate readiness omission: demanded functions
could encounter an ordinary forward value with no declaration type, and a
requested initializer never advanced from prerequisite readiness to body
checking. The body checker now yields the exact unresolved const/static/function
header or constructor owner/fields, retains the current expression work item,
and resumes it. Initializer execution requests use the same resumable body
checker and pending owner states as function requests, after memoized declaration
prerequisites. Header readiness alone never requests an initializer's value.

Validation passed under the shared resource controller:

- Cargo formatting/check and strict all-target/all-feature Clippy.
- 13 storage-filter library tests, including retained proof/callable-map charging,
  nested aggregate session limits, all eight forward/reordered header controls,
  exact CFG construction counts and true dependency-cycle rejection.
- Two CFG execution tests covering 24 independently asserted values and limits.
- Four required-value tests, including 18 independently qualified/native positive
  cases, seven independently object-qualified rejections, early/late CFG reuse,
  and repeated-root preparation scaling.
- Syntax-aware architecture/file audit: 1,197 files, zero findings.

Evidence: `obj/pa25-storage-checkpoint/{0..5}.log` and
`source-sha256.json` (SHA256
`1f9699046e300e3add3adcf28441cbdf7de2a4cb1ff6c219f5139a74ad98b51e`).
The earlier failed readiness/threshold/style runs remain in named failure logs.
Final resource receipt:
`obj/parallel/resources/9e3ceefb5c7749f88a973395ed234daf.json` (return code 0).
The source/native suite passed before the final helper-import/style-only cleanup;
affected unit tests, formatting, strict lint and audit were rerun afterward.
The root's accepted `4f6c7414` temporary-allocation/scheduler refactors are included
verbatim so this checkpoint remains lint-clean on its older merge base.

PA25 remains incomplete. Pending work includes indexed initializer lookups,
general associated initializer execution, slice/text projections and validity,
raw/union/intrinsic const validity with their owning workers, deterministic
`--emit-constants`, owning assignment fixtures/manifest, and enum discriminants
whose match patterns name constants. The latter still reports the existing
explicit general-evaluation boundary; its independent positive rustc outcome
must remain recorded. The prior 594 KiB scalar-control RSS increase remains
unresolved in `pa25-core-performance.md`; this logical accounting is not evidence
that resident memory decreased. The approved small immutable follow-up measures
repeated roots and an ordinary control, with actual preparation counts.
