# Source cfg predicate roots

The source builtin removes at most one trailing root comma and preserves commas
inside nested predicate groups and forwarded metadata. The frozen ordinary
profile retains its prior nested-comma gap. Source syntax/feature/owner checks
still apply to every predicate, including false branches.

All19 authored cases were declared before independent execution:12 accepts and
7 grammar rejections. Accepted cases use compile-time array lengths to check the
predicted boolean. The original allocator predicate is copied exactly from the
pinned std source; outcomes.json preserves its full-source/invocation hashes and
the independent/baseline authoring history. No native or library artifact claim.

Inputs: foundation.rs, setup.rs, ordinary.rs, atom.rs, outer-comma.rs,
inner-comma.rs, nested.rs, both-commas.rs, false.rs, negated.rs, empty-all.rs,
empty-any.rs, forwarded.rs, forwarded-outer.rs, original-allocator.rs, empty.rs,
multiple.rs, double-outer.rs, empty-inner.rs, not-arity.rs, missing-group.rs,
double-inner.rs, cases.tsv and outcomes.json.

The candidate passes all19 cases, the ordinary gap, bounded8/32/128-predicate
controls and three existing regression groups. Strict Clippy, formatting,
architecture, metadata, harness and pinned-upstream checks pass. The source
fix does not alter the frozen ordinary macro path. Actual std replay is pending.
