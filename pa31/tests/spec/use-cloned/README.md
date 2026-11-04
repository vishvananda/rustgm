# UseCloned source and operation contracts

PA31 owns the actual source role and header. PA20 owns ordinary trait proof,
PA26 the real Clone call, PA24 use/capture extensions, PA25 const effects and
PA29 authenticated transport. PA31 and the later whole assignments remain
planned; these fixtures do not release a whole assignment.

`contract.json` freezes 45 authored complete inputs, outcomes and hashes, plus
three auxiliary foundations/observers and an operation observer. `cases.tsv`
lists 27 source-contract controls. `required-cases.tsv` lists 18 operation and
raw-identifier controls. Every predicted outcome matches pinned Rust 1.98.0
88d9e12ae178fab0fb5cc050a94da85685d449ea. Three Rust object/C-observer programs
also execute: the two ordinary source companions print `9\n7\n`; the operation
companion prints `9\n7\n7\n9\n7\n9\n7\n7\n`.

The primary sources in the pinned read-only Rust checkout are
`compiler/rustc_hir/src/lang_items.rs` (UseCloned),
`compiler/rustc_hir_typeck/src/expr_use_visitor.rs` (consume_clone_or_copy),
`compiler/rustc_mir_build/src/builder/expr/into.rs` (ByUse),
`compiler/rustc_parse/src/parser/expr.rs` (capture clause), and
`compiler/rustc_hir_typeck/src/upvar.rs` (use captures). The foundation preserves
the inherited sizing attributes/comments from PA31's minimal provider; its
additional reference copying, Freeze, Destruct and drop_glue declarations follow
the existing qualified source providers. These are authored protocol tests,
not verbatim upstream test imports. Repository NOTICE/third-party attribution
still applies.

The language role requires a trait but imposes no additional header arity,
flags, members or Clone supertrait restriction. Rust's GenericRequirement::None
means unrestricted arity. Preserve real ordinary trait checks: actual impls,
supertraits, required members, provided-body types, unsafe impl rules, object
eligibility and the absence of Copy evidence. The source native companion calls
the actual Clone method through a generic UseCloned bound; the provided-method
companion calls the source default body. RustGM must prove original bodies and
ownership before lowering these controls. Type analysis alone is insufficient
for the duplicate-move opponent.

The 17 demanded operation cases remain implementation requirements. The raw
`r#use` field control is separate and already ordinary syntax. Copy precedes
cloning; a UseCloned proof selects the real Clone method; absence of both proofs
moves. An unbounded generic body retains its original move decision even when
later instantiated with a UseCloned type. Use closures apply the choice during
capture creation, while their body still determines Fn/FnMut/FnOnce. Preserve
real borrow/init/move checks, feature and macro keyword authority, exact capture
paths, one-time evaluation, original const effects and imported identity.
Current early syntax rejection of a Rust-negative operation does not qualify
its required semantic rejection. General transport remains explicitly fenced.

Oracle history is retained rather than relabeled: the first no-core fallback
fixtures omitted the role, and the borrowed-capture fixture omitted actual
reference Copy/Clone impls and crashed rustc. Those sources and the ICE report
remain under `obj/parallel/use-cloned-v1/reference-v1`. Later metadata acceptance
did not prove object readiness: the native foundation also needed actual Freeze
and Destruct/drop_glue roles. All outcome predictions remain fixed after those
foundation corrections. Final independent evidence is `reference-v4`;
candidate/native/core evidence is recorded separately in the
[checkpoint](../../../../docs/pa31-use-cloned-source-checkpoint.md).
