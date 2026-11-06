# Actual raw-autoref policy

The existing lexical lint ledger now handles dangerous_implicit_autorefs for an
actual inserted borrow through a raw-pointer place. Field/Index projections
check that borrow; method syntax independently consults the real
rustc_no_implicit_autorefs declaration contract. Direct raw fields, explicit
references and proved builtin raw indexing do not acquire a diagnostic merely
from containing a pointer. Lint levels/caps grant no unsafe or call permission.

The new identity appends slot16, retaining all old indices. Actual compiler
layout guards establish Policy160 versus the previous152 and cold Expectation40
unchanged; the expectation mask becomes32 bits. Existing source-owned rows,
expectation progress and sizeof-based accounting remain the sole owners.

The5 existing independent Index raw outcomes are1/0/0/0/0. Their historical
candidate source fences remain recorded. The two allowed/capped active controls
now require their genuine Rust-positive result, and the denied control requires
the actual lint cause. A private marked-method expectation is likewise updated
from the old profile fence to the real diagnostic, preserving its unchanged
unmarked positive.

Qualification belongs to the final dependent union with genuine Field/stream
consumers:12 exact public tests pass214.01s; all13 C observers pass; strict
all-target/all-feature metadata, upstream32/Reference126 and harness282 pass.
The final evidence ledger records exact fingerprints and pending private
execution. This commit does not claim an independently linked private run,
complete diagnostic inventory, generated-coroutine runtime or library artifacts.
