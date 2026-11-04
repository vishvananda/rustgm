# Required pinned and ordinary smart-pointer receivers

These eleven complete authored inputs freeze the next receiver contract under
PA43, with PA26 Deref hooks, PA20 proof, PA31 source authority and PA29 transport.
`contract.json` and `cases.tsv` list every input and its independently declared
Rust outcome. Pinned Rust 1.98.0 88d9e12ae178fab0fb5cc050a94da85685d449ea
confirms all eleven. This is required work, not a released or candidate-passing
assignment suite.

The foundation inherits the existing authored source Pin/constructor and
sizing/Copy/Clone/Destruct declarations; repository NOTICE and attribution
remain applicable. Actual primary rules are in
`compiler/rustc_hir_analysis/src/check/wfcheck.rs::receiver_is_valid` and
`library/core/src/pin.rs`. Without arbitrary-self features, the compiler follows
real Deref targets and requires LegacyReceiver for each nonterminal step.
References retain their built-in step. Exact Self equality includes generic
arguments and lifetime obligations; a matching nominal item alone is not proof.

Seven Rust-positive requirements are ordinary Self, shared Pin, unique Pin,
an ordinary similarly named wrapper with genuine Deref/LegacyReceiver evidence,
nested-reference and named-lifetime receivers and the actual constructor
parent Deref bound. The four negatives remove Deref,
remove LegacyReceiver, change Self's generic arguments or omit the constructor's
genuine Deref bound. In particular, the
Pin language item is neither sufficient nor necessary for the demonstrated
general smart-pointer receiver rule.

CLI4003bfd2 currently stops all eleven at the earlier unimplemented actual
`#[lang="deref"]` source contract (line79, byte3127 in the shared-self input).
The ordinary-wrapper input has the same role at byte3114. Therefore all seven
positive paths remain required; the four negative statuses are not semantic
qualification. Source Deref/Target and its actual operator/method consumers
must precede the receiver work. Do not bypass this by stripping a nominal
wrapper, recognizing its spelling or erasing Self arguments/regions.

Independent evidence is
`obj/parallel/pin-source-v1/receiver-reference-v2/receipt.json`, SHA256
bc5d5dcf13028d215cc47815142b016d3709fd68012e1f55bf5b635347052b9a.
Candidate frontier observations are
`obj/parallel/pin-source-v1/receiver-candidate-v2/receipt.json`, SHA256
1bc74454c60131c899456c436210285bbac8cf1996725af6ccb59efcdf4ab42a.
The [source prerequisite checkpoint](../../../../docs/pa31-pin-source-checkpoint.md)
records what is implemented and keeps original core's AsyncDrop frontier open.
