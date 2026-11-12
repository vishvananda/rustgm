# Wrapped callback premises before fold result access

Pinned core's `ByRefSized::fold` calls
`I::try_fold(self.0, init, NeverShortCircuit::wrap_mut_2(f)).0`.
The wrapper factory left its second input unknown even though the caller's
complete `FnMut(B, I::Item) -> B` premise determined that input. Explicitly
supplying only `I::Item` made the reduction pass. Free helpers reproduced the
same failure; direct closures were accepting controls.

Canonical premise comparison now uses the existing binder-aware worklist to
bind a hole to a complete canonical projection. Comparisons between projection
results remain blocked and cannot infer their subjects or other inputs. The
shared worklist collects those blocked pairs for exact identity or the existing
certified-normal-premise relation. Callable binder, capture and region handling
is shared. Generic deferred equations are unchanged: self-dependent projection
equations still wait for normalization. No new proof engine, cache field,
backend, storage policy or compiler limit is introduced.

## Contracts and qualification

The packets freeze68 independent Rust1.98.0 outcomes,60accept/8reject:
PA24ordinary29, PA25conditional-const4, PA30native1 and PA31source34.
All55 supported complete prepared-root outcomes qualify: ordinary26 and
source29, including one actual exported native program. Four further source
const-factory inputs qualify typing while retaining required full artifacts.

Thirteen accepting artifacts remain required: three ordinary cases (two trait
overrides and one multiple-input premise), four ordinary const imports, one
ordinary native, one source multiple-input premise and four source const-call
owner lowering cases. These are not language rejections. The initial proposed
multiple-input negative was accepted by independent Rust before candidate
execution. Its bytes and original observation remain frozen; acceptance is now
required. Ordinary pending cases are serialized losslessly outside the active
course inventory. No whole assignment is activated.

Focused receipt1794449501835039798 passes the new direct premise invariant,
projection-cycle controls and old callback regressions. Final public receipt
1794450414769391285 passes five exact tests across three targets in36.68s,
including complete source exports and the explicitly type-only const checks.
Formatting, architecture and selected strict production/CLI/test Clippy pass.
The earlier public run1794450255456626962 has six passing controls but fails
four const full-artifact expectations; those exact remaining requirements are
recorded above. It is not counted as a passing batch.

The same compiled harness passes five native groups/eleven actual programs,
including the new source fold. It prints `by-ref-fold 23 1234`, matching both
ordinary and source independent Rust/C observations: one wrapper creation,
one iterator invocation and one callback, in order. No extra Rust build was
needed for native qualification. Ordinary native acceptance remains required.

New widths8/32/128 use ordinary inference2613/8065/29825 with three trait
selections, and source inference4216/14156/53868 with25/73/265 trait selections.
There is no method work. Adding128 unrelated types changes no counter. Both
preceding constructor and multi-input counter families remain exactly unchanged.
These are bounded-work observations, not optimized performance or core throughput.

The26 new PA24 text triples were independently declared, preverified, generated
and reviewed against complete CLI outputs. All28,065 prior reference hashes,
35 preceding callback text cases and11 old diagnostic controls remain unchanged.
Independent selection checks all26 cases, including22 explicitly empty objects
and no runtime comparisons. PA24 now has642 cases:459accept/183reject, with396
runtime, five code-generation and58 object classifications. Original upstream
source bytes and checksums remain unchanged. Upstream, maintainer harness and
cleanup checks pass; their receipts remain under upstream-review/.

## CI and original core

Full43138039057 on exact eff52eb9d/source d1980b325 passed fast, all-target lint,
architecture/harness and the independent host build graph. Its release regular
run has3064 private passes, one diagnostic failure and three ignored tests;
linking has283 passes and two diagnostic failures. Later qualification stages
were skipped. Both previously pending signature/refinement invariants passed.
The first private mismatch hid a second case in the same test. All22 declaration
controls were checked through the current CLI; Rust independently rejects the
two affected cases with E0277/E0599. The two PA29 negative clients and provider
hashes retain their original independently declared rejection outcomes.

Four exact diagnostic expectations now follow the earlier associated-result or
concrete type mismatch. No rejection became an acceptance, and no diagnostic
alternatives were added. Public linking receipt1794451117884602258 passes both
complete fresh-process transport tests in59.61s, including strict selected lint.
The private harness rerun and full successor remain for CI; its earlier fixed
251s compilation timeout is not retried locally or converted into a pass.

Immutable CLIe8ca58dbc0bc0baab5dbf5c883dffd389d85a213f768445b6ff77d8e5747820f
binds clean semantic source cf75c3f61e718dfcb0878f71a69577ecd7dbb0b5,3907 Rust
hashes and an empty tracked patch. Thirty exclusive A/A+twoABBA accepting-control
observations retain complete output equality versus6c. No accepting baseline
exists for the newly accepted implicit forms; the width/noise controls measure
that changed path. Raw noisy ratios remain in the machine record, without an
optimized, neutrality or original-core speed claim.

The unchanged120s original observation ends121.563414s with61 partial failure
records and13,261 item attempts, last function6964,file203:1097,line44 in
fmt/builders.rs. It reaches ByRefSized::fold4429/body134:1245 and
rfold4435/body134:2178 without their failure records. The five previous ArrayChunks
bodies and four earlier regressions are also reached without those records.
Forty-four prior failure records are absent among reached bodies; no new failure
occurs among previously passing reached6c bodies. These are incomplete observations,
not a completed census or throughput measurement.

First remaining is `filter_try_fold`, iter/adapters/filter.rs140:3088,line86,
"this typed capture use is not implemented". Its attempted body is function4600,
file140:3018. Postcheck1794451995494437060 verifies722 original hashes,375 opens,
six configuration bindings, all3907 current/immutable Rust hashes and the unchanged
profile/limits. ProgressSHA8bbdcf63f52c70ff87547e5a4b0c664d3319d71ab5658df44185134e2add06dc.
The deadline remains an infrastructure failure. No original core, alloc or std
artifact exists. Published8c231518e/sourcecf75. Full43145135573 is running on that exact head
after a fresh no-active-full check; do not restart it for documentation descendants.
Reduce the actual typed capture failure next. Keep
stable5d until full qualification,6c for accepting comparisons and optimized21f
for retained optimized work. Intermediate81 is absent. Duplicate6c fixed-dev-fast was explicitly retired
after publication; the identical test-fix-dev-fast baseline remains available.

The [machine record](pa24-wrapped-callback-premises-evidence.json) binds raw
observations under Bobj/parallel/by-ref-fold. All earlier pending requirements
in handoff.md remain required.
