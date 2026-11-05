# Required pinned and ordinary smart-pointer receivers

These12 complete authored inputs freeze a receiver contract under PA43, with
PA26 Deref hooks, PA20 proof, PA31 source authority and PA29 transport.
`contract.json` and `cases.tsv` list every input and independently declared Rust
outcome. Pinned Rust1.98.0 88d9e12ae confirms all12. The original11 fixture hashes
remain unchanged; the additional native companion observes actual methods.
This packet is qualified locally, not a released whole assignment.

The foundation inherits the existing authored source Pin/constructor and
sizing/Copy/Clone/Destruct declarations; repository NOTICE and attribution remain
applicable. Primary source rules are in the pinned compiler's
`compiler/rustc_hir_analysis/src/check/wfcheck.rs::receiver_is_valid` and
`library/core/src/pin.rs`. Without arbitrary-self features the compiler follows
real Deref targets and requires LegacyReceiver for each nonterminal step.
References retain their built-in step. Exact Self equality includes generic
arguments and lifetime obligations; a matching nominal item alone is not proof.

Eight Rust-positive requirements cover ordinary Self, shared/unique Pin, an
ordinary similarly named wrapper with genuine Deref/LegacyReceiver, nested
references, named lifetimes, the actual constructor parent bound and the native
read/write consumer. Four negatives remove Deref, remove LegacyReceiver, change
Self arguments or omit the genuine constructor bound. Pin identity is neither
sufficient nor necessary for the demonstrated general receiver rule.

Published predecessor CLI4003bfd2 stops the original11 at the earlier unsupported
actual source Deref contract. Those statuses remain historical prerequisite
evidence, not receiver qualification. Current CLI0cc88c10 passes all12 through
actual source traits, associated Target normalization and full Self/lifetime
validation. Its native methods print `record=5`, `record=4`, `result=9`, exactly
matching pinned Rust through the same C observer. Source API bodies stay intact.

The [checkpoint](../../../../docs/pa31-deref-receiver-checkpoint.md) records receipt
hashes and the captured-impl substitution repair. `qualification.json` binds
current evidence; old prerequisite observations remain in the prior checkpoint.
Unpin/projection, async state/cancellation/destruction, PA29 transport and all
other assignment requirements remain required. Original core still reaches
AsyncDrop at the same location; no library artifact is claimed.
