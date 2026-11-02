# Actual source iteration protocol

This planned PA31 packet owns source language declarations; PA26 owns their
iteration/desugaring consumer. It is not a whole-assignment release.
`contract.json` declares ten independent Rust outcomes and a native observer.
`ownership-contract.json` adds two ownership rejections and actual destructor
execution. Every source/observer has a declared hash and outcome before its
reference run. The primary-source hashes bind the pinned compiler/library
contracts; all Rust fixtures are authored reducers, not original core artifacts.

The actual trait and method labels are deliberately renamed. The `iterator`
trait marker is optional for a for-loop, and the required method tagged `next`
may belong to another actual trait. `next` and `into_iter` must be required
trait methods. Missing method roles, wrong results and missing implementation
obligations reject. Some precedes None in the authored enum so the compiler
must consume real variant identities/ordinals rather than fixed tags.

The two C observers require `7\n` and `7 8 9 \n7 8 9 \n`. The latter verifies
item destruction before iterator destruction on normal exit and break. It
retains the real compiler-replaced DropGlue/Destruct contract from pinned
core. Moved iterators and escaping borrowed items reject through ordinary
ownership and region proof.

The metadata-only reference initially exposed two harness prerequisites:
a tagged required method cannot have a provided body, and native no_core
execution needs Freeze plus the real DropGlue/Destruct declarations for drop.
The contracts disclose those corrections; earlier failed inputs and diagnostics
remain retained in local evidence. Required native output was not weakened.

CLI `--emit-loans` intentionally accepts only concrete bodies, so it cannot
inspect the generic DropGlue helper. The declaration view and real exported-root
native compilation qualify that fixture; the failed full concrete-view probe is
retained explicitly. This packet does not claim complete generic inspection,
source-client metadata transport, the remaining iteration families, or an
original core artifact. See [the checkpoint](../../../../docs/pa26-source-iteration-checkpoint.md).
