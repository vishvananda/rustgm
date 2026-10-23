# Explicit source Drop calls and function values

This checkpoint follows the header prerequisite on merged base
`d76f58de22764d5de7efae5819becc96e6504843`. It rejects explicit uses of the actual
Drop member during semantic function-item checking, before Copy preparation,
destructor-registration readiness or executable publication. It adds no fields,
IR variants, synthetic declarations or ordinary library bodies.

The existing destructor registry still rejects collected implementation methods.
The new `compiler_items::explicit_drop_member` facade additionally recognizes the
real validated source Drop declaration and its checked single-member profile.
Concrete source overrides require the canonical typed implementation header,
its actual owner/application and exact declaration-to-override pair. Trait and
type aliases retain those same identities. Neither the trait's source name nor
the method spelling decides permission. Unrelated traits, inherent methods and
free functions named `drop` remain ordinary functions.

A missing canonical header for a possible source Drop owner is a readiness error,
not a cached negative or an alternative execution witness. The exact retained
source trait reference can establish that an owner is unrelated; it cannot
replace a missing matching implementation header. Worker C's ordinary ImplHeader
continuation remains responsible for publishing actual Drop's normal selection
and conformance facts alongside the separate destructor registry. Full Pin's
`pin_drop` and its default-body calls remain a distinct required contract; this
single-member profile does not generalize to all future members of a larger
Drop owner.

## Independent and candidate controls

The frozen corpus is `obj/parallel/pa31/source-drop-uses-v1/plan.json`, SHA-256
`06d1ecae28e3866502d82ef9ea2c3a500aad167f714f2cbe1cf7446cb4be9862`.
It declares 22 exact sources before candidate qualification: thirteen explicit
Drop-use rejections and nine accepted controls. Pinned Rust 1.98 object checks
settle every outcome. All thirteen negatives reach Rust's E0040 explicit-drop
rule, including generic/concrete UFCS, dot calls, bare function items, reified
pointers, aliases, a differently named actual trait owner and a warmed unrelated
same-spelling member. Four accepted sources also execute natively: same-spelling
trait-method and free-function pointers return 7 and 11, both with and without an
actual Drop declaration. `independent-warm/summary.json` records the union with
hash-bound reuse of unchanged prior observations, SHA-256
`fa3012c90072a90e6c017961933d21392d7ec8555c5224b30fffa1ca2f3aa296`.
`independent-explicit-rule.json` binds each negative to its actual E0040 stderr.

The candidate's two focused frontend tests cover all thirteen rejections and
seven actual-Drop-present typing positives. Every negative explicitly reaches
`explicit destructor calls or function values are forbidden`; registration is
not its rejection reason. The private identity control alternates real trait,
real override and unrelated member queries 32 times, then withholds/restores
canonical headers and validated roles. Matching pending states reject, known
unrelated identities remain unrelated, and restoration cannot reuse a stale
negative. The existing `drop_headers` independent-language root also passes.

Candidate native execution covers seven programs: ordinary same-spelling method
and free-function pointers return 7 and 11 without an actual Drop role; the five
unchanged Copy/Clone/FnOnce/ordinary-Destruct/runtime-conditional programs retain
their declared answers. The two original actual-Drop-present native positives
remain runtime-gated until C's exact completed-registration marker is available;
their candidate typing success and independent native success are distinct
observations. No readiness gate was bypassed to claim execution.

## Qualification and retained limits

`development-v2` completed formatting and all-target/all-feature strict Clippy
before its resource scope was interrupted with SIGTERM 143 during the test build.
No test outcome was inferred from that interrupted build. `closing-v1` rebuilt
successfully but correctly rejected an imprecise module-only selector; it ran no
tests. `closing-v2` uses exact test selectors and passes four focused test
functions, the seven-program native group and the 1,973-file architecture audit
with zero findings. Source remained unchanged across those retries. Its final
source inventory has SHA-256
`1c6b1602e7f412d36b2b1b81bfdf8b42aec3cf0d270fdf6299c06f28bfac938c`.
The earlier test-only lint failures, interrupted retry and missing-selection
failure remain preserved. Literal-delimiter corrections retain identical source
bytes, checked against the frozen matrix in the equivalence receipts.

The facade performs only bounded identity/header lookups and inspects the checked
one-member method profile. The ordinary compilation path returns immediately
when no source catalog exists. There is no new persistent cache, body flag,
record growth, source traversal or cross-caller permission. Root owns combined
active-text/full-release qualification; this worker did not regenerate oracles
or repeat those broad gates.

Actual Freeze and drop_glue are the next concrete dependencies for C's retained
cleanup-execution source. Full source Drop/Pin/Deref, conditional const payment,
source Freeze structural semantics, generic Copy/FnOnce inspectors and the
remaining PA26/PA30/source-library requirements remain required. This checkpoint
claims neither whole-core acceptance nor public-trait metadata transport.
