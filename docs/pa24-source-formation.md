# Shared-reference truncation and checked source formation

This development slice retains the exact original PA24
`captures/shared-reference-truncation.rs` bytes. Its original checked executable
failure was `nested output requires its checked parent contract`. The parent
returns an opaque callable but has no ordinary type parameters, so the old
partition put the parent in the template family and its child in the ordinary
family. Template inclusion and concrete exclusion now use the same generic-or-
opaque owner predicate, including nested children. The plain concrete builder
still retains the whole original family.

The corrected partition exposed an independent instance debt. The child stores
only the static field reference, but its retained source root is `&'b0 S<'b1>`.
Both original source well-formedness and later instance formation require
`'b1: 'b0`. The parent proves this from its parameter; the selected closure
instance had neither that premise nor a caller obligation representing it.
Removing either well-formedness obligation independently still rejects. Both
remain in the implementation.

After the entire immutable source family passes region, ownership and loan
verification, its sealed publisher extracts only the assumption-independent,
conjunctive scalar well-formedness requirements of each retained input root.
Static and representable depth-zero formal pairs join the existing output
condition catalog. Parameter, projection, private-variable and other duties stay
in the original contracts. Extraction does not use a solved graph, ambient
assumptions, an alternative disjunct or guessed lifetime images.

The canonical invocation selector requires each condition's exact actual image
in the caller contract. Only the already checked caller path can specialize the
callee with the declared catalog as premises. Public/manual specialization
still requires those relations. Missing seals, swapped images and unpaid
callers must reject. Joint source verification precedes publication; a physical
instance or table cache grants no source permission.

The collector creates its no-assumption query lazily and reuses it only during
one immutable checked publication pass. Ordinary bodies allocate no additional
query or pair collections. A bounded membership index deduplicates new pairs
while preserving deterministic source/query order, and an unchanged condition
list retains the existing Arc. Root, query, pair, lookup and clone work is
charged. Prospective map/vector/Arc storage is checked before allocation.

`pa24/tests/formation-truncation-hypotheses.json` records the independently
predeclared source outcomes without activating the assignment or generating
text oracles. Historical evidence is retained under
`obj/parallel/pa24/formation-truncation/`. The qualified candidate fixes the
original, after-scope, generic-owner and nested checked views. All twelve declared source controls match their independently
qualified Rust object outcome and explicitly declared three-view status. Four
native comparisons execute the original, after-scope, generic-owner and nested
cases with the pinned Rust compiler and unchanged CPPGM backend.

The seven new private tests verify missing-source-seal rejection, Required/manual
rejection, exact paid versus swapped caller images, unchanged original and added
formation duties, immutable Arc reuse, lazy ordinary behavior, formal-slot
filtering and deterministic work/storage exhaustion. Ten adapter proof and
fifteen entry proof regressions pass, as do eight captured source/native groups
and twenty-one specialization/opaque regressions. Two preexisting optional opaque
native tests were not selected in that regression batch. Strict all-target,
all-feature Clippy passes; the syntax audit reports 1,469 files and zero findings.
Upstream checks verify 24 fixtures and 126 Reference files; layout reports 38
stages without active orphan fixtures.

Actual capture-root widths 8/32/128 retain one scalar pair and five structural
query steps. Total collector work after 8/32/128 additional publications is:

| Roots | 8 repeats | 32 repeats | 128 repeats |
| --- | --- | --- | --- |
| 8 | 230 | 830 | 3,230 |
| 32 | 878 | 3,206 | 12,518 |
| 128 | 3,470 | 12,710 | 49,670 |

The query cache remains unchanged across repeated publication and each repeated
publication has constant work for a fixed root width. These are logical work
measurements, not wall-time neutrality claims. All seven independently frozen
scaling inputs (root width and invocation count 8/32/128, plus an ordinary reference-call control) pass Rust object and frontend
LowIR checks. All active PA1–23 text comparisons are exact; no references were
regenerated. Exact hashes and evidence paths are in the closing receipt.

The original Rust-positive `nested-truncation` now passes all three checked views
and native execution. Its earlier partition/formation failures remain in the
historical receipts. `reference-result` remains a required valid-language gap:
Rust object qualification succeeds, types succeeds, and LowIR/constants reject
at file 0 offset 162 with `required lifetime does not outlive its target`.
Its exact original bytes are retained. This is not an independent semantic
negative or assignment completion; the next checkpoint investigates that
reference-result proof separately.

The later [input-only bridge checkpoint](pa24-input-only-bridges.md) supersedes the
reference-result boundary above with unchanged fixture bytes. The receipt here
remains historical evidence for the original catalog checkpoint.
