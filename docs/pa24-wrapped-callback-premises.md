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

Current compiler SHAe8ca58dbc0bc0baab5dbf5c883dffd389d85a213f768445b6ff77d8e5747820f
has the local evidence above. A new immutable original-core observation has not
yet run. The last original6c observation still ends at its unchanged120s deadline;
its first failure is ByRefSized::fold134:1322. No original core, alloc or std
artifact exists. Freeze this checkpoint, compare accepting immutable controls,
then repeat the unchanged observer and verify all original/configuration bindings.
Keep stable5d until a full successor qualifies, current6c for comparison and
optimized21f for its retained qualification. Intermediate81 is absent.

The [machine record](pa24-wrapped-callback-premises-evidence.json) binds raw
observations under Bobj/parallel/by-ref-fold. All earlier pending requirements
in handoff.md remain required.
