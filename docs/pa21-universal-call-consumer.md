# Closed universal call proofs

The retained f01 program has only `Tr<'static> for u8`, while an inherent
constructor requires `for<'r> Tr<'r>`. Baseline bf168bc7 accepts its generic
semantics, including the original no_core bytes af54492f. Its ordinary complete
LowIR path rejects later with a lifetime error. This checkpoint closes that
generic-semantic admission defect; it does not claim the baseline emitted an
invalid native program.

Before publishing a completed closed universal call selection, consume its own
scoped lifetime obligations in a bounded graph. Preserve every original proof
block for later consumers. Goals with outer parameters, free caller regions or
opaque inputs retain their existing deferred proof path. Low-level
Selection::Proven still carries lifetime debt; no global solver contract changes.

A self-contained proof needs no caller graph. A proof using another actual caller
lifetime reuses OriginalInputs, with declared input well-formedness and outlives
premises, and the body's existing universal aliases. The cold cache belongs to
one body and adds no dense CallTypes bytes (still728). Foreign body witnesses
are validated and registered as variables; selected debts never become caller
assumptions. Construction, graph queries and failed bounded checking are charged.

PA21 owns nine ordinary higher-ranked reductions. Their statuses are declared
before reference generation and independently checked by pinned Rust1.98 object
compilation. They distinguish static-only/global-universal implementations,
forwarding, actual caller universal evidence, captured witnesses, and the
positive/negative pair differing only in `'x: 'static`. The PA31 source-language
packet remains separate. A scoped nine-case reference overlay is generated and
independently reviewed with the implementation and fixtures; no prior oracle
bytes change.

Production1794298347935194902 passes the original seven outcomes and the existing
higher-ranked LowIR group. Private1794298494239587809 passes ten exact solver,
call-inference, captured-input and body-proof controls. Final production
1794298710594212971 passes all nine cases. Formatting, architecture and strict
selected/private Clippy pass. The immutable final batch repeats all nine
independent object outcomes, the exact original/rooted f01 and universal twins,
all nine course references,313 harness tests/467 roots/44 stages and pinned
upstream128/Reference126 checks.

Immutable dev-fast CLI77ddb7332c7299097a579adcf8b49afbf97ecd13e388509e905e9bf09dae7410
binds compiler ffa610c52, all compiler/test Rust hashes and an empty patch. Fifty
A/A and two ABBA observations preserve complete outputs for8/32 closed calls,
8/32 captured-caller calls and74 ordinary controls. Candidate/base ratios are
1.198/1.218 for closed calls,1.107/1.163 for captured calls and0.977 for ordinary
controls. These short dev-fast observations record the additional proof cost;
they support no optimized throughput claim.

The unchanged722-input original-core census opens375 sources and checks42310
declarations, none failed/blocked, bodies0, in70.49s. The diagnostic table remains
byte-identical (9a585c80afd2f737e5359956b85c9be14097442b91abd2bc667dbde5896645e2):
local_annotations at slice/mod.rs:2251, byte82861, Split::new requires a scoped
consumer. f01 is closed before extending that lookup. No original library
artifact exists. Full successor CI remains pending; multiplication predecessor
42773069633 and combined-query predecessor42765929708 passed every stage.

Raw attempts, declared predictions, independent objects, reference receipts,
immutable binaries and paired observations remain under
pa26-work/obj/parallel/scoped-inherent.
