# Proof-only caller alias frontier

`Inputs::aliases` still proves every source alias equation under the actual caller
header before exposing its equality folder. Its selected body debts, exact source
ownership, full instance/epoch/availability, original and instance proof, paid
premise and escape checks are unchanged. It now consumes `Constraints::check_aliases`,
which returns only charged work, never a Solution, proof token or observed bounds.
The existing projection verification runs first. A graph containing membership
or verification state falls back to the unchanged full solver.

For a plain graph, only rigid nodes with incoming requirement edges seed
propagation. An isolated self-pair has no edge to visit and no observer on this
path. Every required edge, declared lifetime slot, assumption and parameter is
retained. Propagation, universe promotion and assumption entailment are unchanged.
The complete frontier scan is charged; failure remains conservatively charged by
the existing caller budget. Ordinary solve/prove, conditional output/outer-condition
elimination, membership observers and verification paths keep full seeding.

This addresses the integrator's measured repeated alias-solve cost without
skipping caller validation or caching positive proofs. Dense binder construction,
marks and scanning remain width-dependent; no instruction or timing improvement
is claimed before the integrator's immutable comparison.

The initial private matrix compares 4,096 plain graph cases with the full solver,
including cycles, assumptions, static/local bounds and younger universes. Rich
member/verification fallbacks preserve exact result and work. Foreign identity
and budget failures remain explicit. Widths 8/32/128 retain two sparse seeds with
HashSet capacity3 versus 10/34/130 dense seeds. Charged work 19/43/139 per context
includes the real width scan, and contexts 1/8/32/128 scale exactly linearly.
Evidence: `obj/parallel/pa25-alias-demand/first/`; the original component matrix and
raw paid-selection debt invariant also pass. This is not PA25 activation or a
resolution of the remaining conditional instance/output or generic callee work.

The closing `obj/parallel/pa25-alias-demand/close/` receipt passes strict
all-target/all-feature release Clippy, audit 1,826/zero, 28 graph tests, 9 paid/debt
tests, the catalog/layout check, 17 selected-initializer source tests, two explicit
native groups (six paid and five rigid programs), and both formerly legacy
independent-selection/lookup suites. All active PA1–24 references from public
11200c52203e729e8354cd29ef9de9ca16fff7b3 are byte-exact, with no output delta.
No full Cargo or worker performance run was added.

Functional pin: `close/pins/rustgm`, SHA-256
`f5e6eb26ff9655b9d937c390a8fde6492509532ed7792c6836f7e32440c25c63`;
1,043-file production source-map SHA-256
`ec982ae2b94aba5ceaa83fe5148fec1b9ea36b6e1aa5c65c79245f87fb038d21`.
Caller 64, Catalog 216, witness 8, Target 28, Request 56 and Code 336 byte layouts match
the receiving baseline; this patch adds no data fields. The pin uses a debug
functional profile and is not performance evidence.
