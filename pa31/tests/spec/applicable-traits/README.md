# Ordinary always-applicable trait policy

PA31 owns source authority and the actual attribute owner; PA20 owns minimal
specialization and implementation applicability. PA29 owns imported policy and
generic-body execution. Every source input is bound in contract.json.

The primary rules are pinned Rust1.98 revision
88d9e12ae178fab0fb5cc050a94da85685d449ea, particularly compiler/
rustc_hir_analysis/src/impl_wf_check/min_specialization.rs and coherence/mod.rs.
The attribute belongs to an actual trait, not only a compiler language role.
Implementing it requires specialization/min_specialization at the actual impl.
Root min_specialization enables the always-applicable checks even when Full is
also enabled. Full without Min retains its distinct behavior.

All44 independent outcomes match rustc. cases.tsv selects43 locally matched
outcomes. required-cases.tsv retains the valid quantified predicate that still
needs PA20 elaboration; it is not a language rejection or an empty-suite success.
The native contract declares7/9 from actual marked-supertrait/ordinary dispatch.
The C observer and Rust input bytes are pinned in the same manifest.

Minimal no_core sizing traits are ordinary source declarations until their real
specialization annotations are supplied. The original incorrect generic/Sized
predictions are retained in failed observation evidence; unmarked rejections and
genuine marked-sizing positive counterparts both remain in this packet.
Duplicate specialization-trait attributes are hard errors in this revision.

The implementation reuses the existing cold source-marker owner, canonical
trait facts, worklist and budget. It never recognizes BytewiseEq or any other
ordinary trait by name. General trait policy remains explicitly untransported;
existing exact sizing-role metadata remains supported. Other trait/type policy,
quantified and projection-constrained checks, full assignments and original
core/alloc/std artifacts remain required.
