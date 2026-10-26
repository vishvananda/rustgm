# Final trait functions: source consumer checkpoint

The qualified unchanged-core observer opens 72 of the 722 immutable pinned
inputs and stops at `library/core/src/clone.rs:389`, byte 14948, the actual
`Share::share` declaration. The source SHA-256 is
`fd3467602106e2a2ec416485d8d058b6d2b434ceac7689b4e2f4c792079e500f`.
The observed CLI SHA-256 is
`eb644b412597aa77a1d2d8accc539a802e6009e05a940d5ada6e0bfb87d8839a`.
The six focused source/body/dispatch controls and four paired native programs
are qualified, including the combined production/CLI integration. The next
unchanged-core observer remains a separate integrator observation.

## Pinned compiler contract

At revision `88d9e12ae178fab0fb5cc050a94da85685d449ea`, parser/item.rs
`parse_defaultness` retains `Defaultness::Final` and gates the actual keyword on
`final_associated_functions`. ast_validation.rs `check_defaultness` and
`check_final_has_body` admit final only on trait associated functions with a
body. check/check.rs `check_overriding_final_trait_item` rejects an actual impl
override. This differs from an impl method's effective specialization finality
and from an ordinary overridable trait-provided body.

traits/dyn_compatibility.rs exempts final methods from method compatibility
restrictions. traits/vtable.rs omits them from virtual slots. ty_utils/instance.rs
returns the actual trait item with its complete receiver arguments for an object
selection, so the real provided body executes statically.

## Existing producers and consumers

`Word::Final` already exists. One appended `gate_sites::Kind::FinalFunction`
uses the same sparse row, source incarnation, keyword/context authentication,
transfer, original replay and cumulative accounting as the existing source sites.
Feature bit 25 is `FinalAssociatedFunctions`; no LangItem or macro role is added.
The existing sparse unstable-item pass checks this gate even when cfg disables
the owner. No inactive-feature warning is invented.

The declaration signature path requires the actual trait owner and body; the
existing conformance path rejects an override against the actual trait member.
Report-aware dyn compatibility and vtable planning consume that same typed
source identity. No method flag, dispatcher source, table owner or cache is added:
object dispatch already retains the genuine FunctionItem and proof, and bypassing
virtual planning for a final member reaches the existing original/instance body
pipeline. Complete body, formation, lifetime and call obligations remain required.

The selected-header publication guard rejects a final method explicitly. Existing
packet bytes/profiles are unchanged. `NoDefaultModifier` means no written default
keyword; it cannot represent absence of a final trait override restriction. Public
Provided/default/final transport stays closed until an actual contract is encoded.
An imported missing record never becomes a completed final-policy absence.

## Independent evidence and focused qualification

The initial source contract is SHA-256
`c14041b6195dfc4465f4016d32e7a425fc97efacba0354687e07e9a80c8c638b`.
Eighteen source files produce 36 new pinned metadata observations in editions
2021/2024, with zero infrastructure failures. Binding SHA-256:
`403e85f777866aaaaa2b0e903a03bfa9a26b093188284ee2337e71780dd0bb79`.
Two initial predictions were wrong: a disabled final owner still produces hard
E0658 without the feature, rather than an unstable-syntax warning. Both raw
hypotheses and actual streams remain retained; no older oracle matrix was rerun.
Four native programs now match pinned rustc exactly: actual provided/final
dynamic bodies return 7; ordinary static/virtual overrides return 9. Their frozen
executable wrappers append only main plus a declared C observer to the immutable
metadata inputs. The common locked object backend is used without rebuilding CPPGM.

Six private groups cover actual provided and source Share bodies, override and
owner/body errors, exact gate tokens and disabled owners, real dynamic static vs
virtual calls, transport/origin opponents and sparse original replay/width/layout.
The PA5 grammar fixture is byte-identical to independently accepted provided-call;
its generated references remain pending official generation after source review.
No assignment is activated. The final source is the clean normal merge
`c6947a844a0ad5fe2bc0bb86558b047578cb4b3b` over public `0d6db3a43`.

The successful standard C12 gate is
`obj/qualification/fast/1793046023987938929`, receipt SHA-256
`a0c27bbf4a4f0bd404a18a519ff05562642a866ac78f221eb0030a0203174429`.
Metadata, formatting, architecture, strict private Clippy, fresh unoptimized
build and all six exact groups pass in 114.29 seconds with 2,782 fingerprinted
inputs. The retained earlier runs expose one conversion error, one Clippy arm
finding and a test observer that incorrectly queried uninstantiated function
prototypes. The corrected observer uses actual retained callables and real
vtable planning after full lowering. No oracle outcome was changed for it.

Actual layouts remain Item64, Function32, Syntax312, Unit688, Definition36,
Scope32, Report3040; the shared row24 and owner56 remain unchanged. The ordinary
no-modifier path retains no sparse owner. Actual sparse `(rows, capacity, steps,
prepaid bytes)` observations are `(8,8,266,824)`, `(32,32,1314,3128)` and
`(128,128,6274,12344)`. These are existing source-owner accounting observations,
not allocator counts. No allocation count, full native parity beyond these four
programs, public final transport or whole-core acceptance is claimed.

## Combined CLI and native execution

The integrator's combined source `62f54ec7c` passes 20 exact private controls and
one frontend control, production/CLI and private strict Clippy, format, architecture
and unoptimized builds in 266.90 seconds. Producer receipt SHA-256 is
`22e88784db4888e515332a5f9f88f2038497d411806a1113fd0f54c84be25f11`.
All 2,786 source fingerprints were verified at that integration boundary.

Actual immutable CLI SHA-256:
`192bb5369a2512de5f0945b4c0544d2d1524439ee6fccc4ebdd4c680633afe17`.
Producer binding SHA-256:
`85bbe12f2b602cf4e755565f37eef09cbb4d30e9618b99f843dbe3f1ef5a6cfa`.
The frozen native plan SHA-256 is
`b1dd6c7f454818ea5060a3591b806a083e58fc03ddc2f26b8980c6823d368aea`.
All four candidate/rustc comparisons pass under the unchanged C12 resource and
storage guards in 0.922 seconds, with exact stdout `7\n`, `9\n`, `7\n`, `9\n`,
status zero and empty stderr. Native binding SHA-256:
`72a618b3f2629746a17e5e12676f99534773246c6d355d75b6fb7cd2a172d219`.
The candidate hash stayed unchanged; disposable executables and objects were
removed after each case. All raw streams, commands, sources, producer fingerprints
and output hashes remain under
`obj/parallel/pa31/final-trait-methods-v1/native-observed-v1`.
The pending PA5 reference and public transport requirements remain independent.

