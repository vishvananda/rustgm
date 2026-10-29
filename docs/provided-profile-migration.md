# Unpaid Provided profile migration

The current unpaid source producer uses `Source2024ProvidedCurrent`, encoded
with source-envelope tag 11, version 2 and the existing Provided columns. This
appended profile preserves earlier enum variants and envelope encodings. It is
data ownership, not permission to import or execute a provided/default body.

`Source2024Provided` now identifies historical tag-11/version-0 data. Its decoder
and encoder retain the original bytes, including the frozen `provided-v0.hex`
capture under `pa31/tests/spec/provided-contracts-v1/`. The historical producer
included an implicit `Self: MetaSized` requirement on auto traits. Current source
semantics correctly omit that requirement. The fixture's Freeze header therefore
has zero canonical inputs and one declared input, its implicit own-trait premise.
Ordinary Clone and Copy requirements remain intact.

The old capture is a historical byte contract, not a current-producer oracle.
The encoding baseline still compares actual first-producer bytes for the four
unaffected function/module profiles. For current Provided data it checks the new
envelope, actual source rows and roundtrip equality without generating a golden.
The historical Provided capture separately retains exact decode/encode equality.

Canonical translation rejects historical version-0 data. Before either ordinary
graph creation or private unpaid signature mapping, one paid validation pass also
rejects auto-trait canonical Self requirements targeting the actual MetaSized
role. It compares typed declaration addresses and application subjects; changing
a source-site number or optional Self classification cannot bypass the check.
The same check covers the unchanged version-1 ProvidedContracts envelope. It
preserves ordinary authored requirements and does not erase feature or lifetime
clauses.

Foreign auto-trait canonical Self trait requirements remain explicitly unsupported
until dependency-role provenance is available. Absence of a local role binding
cannot turn such a foreign requirement into a trusted premise. No dependency
graph replay, new role map, dense record field or synthetic source bound is added.

The focused migration controls cover the three original CI failures, historical
bytes, current rows, changed provenance, foreign-role absence, codec and combined
budgets, and actual unpaid signature readiness/failure. Broader release CI
remains a separate gate; subsequent joined qualification is recorded below.

Root's joined integration passes20 private controls in150.67s and3 production/
CLI controls in109.42s, including all three original CI failures. Both strict
Clippy modes, formatting, architecture,272 harness controls and pinned upstream
verification pass. The current source-bound core attempt retains the unrelated
union Clone recipe blocker. Full release/native/independent CI remains pending;
no original library artifact or whole assignment release is implied.
