# Implemented trait scope — PA19

Compiler 17baa8cb9 makes an impl's implemented trait eligible for method lookup
throughout that impl's lexical scope. The original-core discovery was a string
literal's `fmt(f)` call inside `impl fmt::Display for TryFromIntError`. An ordinary
module-qualified user trait and primitive or str receiver reproduce the failure,
so PA19 owns the reduction. No original core, alloc or std artifact exists.

The existing visibility index now visits associated owners once and consumes each
source impl's resolved trait declaration. It adds that identity at the actual
impl Self definition's scope. Group traversal and owner visits use the existing
visibility budget. Discovery continues to share inherited candidate lists and
respect macro contexts. No hot record, new cache, quota or backend contract changes.

The contribution is lexical eligibility. It supplies no named import, parameter
bound priority, fixed trait arguments or additional supertrait visibility.
Receiver applicability, conformance, method requirements and ambiguity still
require their ordinary proofs. Another application of the same trait can infer
its own arguments. Inherent and actual parameter-bound candidates retain priority.

## Assignment reductions and independent checks

[The PA19 packet](../pa19/tests/spec/impl-scope/README.md) registers all 34 inputs
in the assignment manifest: 25 accepts and nine rejects. Their final source hashes
and expected outcomes were independently checked with Rust 1.98 before the
compiler edit; all 25 positives also executed successfully. The packet covers
primitive/str and reference receivers, explicit import/qualification, module
aliases and reexports, nested scopes, expression macros and generated methods,
impls and nested functions. Other trait arguments, inherent/bound priority,
outside/different-impl invisibility, missing impl/method proofs and three genuine
ambiguities preserve the selection boundary.

Independent Rust rejected the initial concrete-Self supertrait prediction with
E0599. That source remains a negative. A separate generic-receiver control uses
the actual bounded field; its initial wrapper-receiver spelling also received
E0599 and was corrected before implementation. Initial packets, errors and source
revisions remain in local evidence. Neither failed prediction generated an oracle.

At widths 8, 32 and 128, six work counters remain at most 4x+1024 per fourfold
call growth. Adding 128 unrelated unit structs leaves inference, canonical work,
trait selections, method steps and probes unchanged. Visibility adds 384 steps
for their scopes and type/value bindings, independent of call width. The initial
test omitted scope accounting; the reviewed final bound includes all three
components and also checks that the metadata cost does not grow with call count.

## Qualification

Evidence is in B's `obj/parallel/impl-trait-scope/`:

- `public2.log`, receipt 1794338767901514056: all 34 complete LowIR outcomes and
  four existing import/macro/bound/glob controls pass. The aggregate failed only
  the initial two-component noise expectation. `scaling-final2.log`, receipt
  1794339031157546122, passes the strengthened final invariant in 41.79 seconds.
- `native.log`, receipt 1794339103528845675: all 25 new positive programs pass
  Rust/CPPGM execution in 65.26 seconds. Selected production, CLI and test Clippy,
  formatting and architecture checks pass.
- `final/steps.json`: exactly 34 new reference triples were generated; all 1,233
  PA19 text cases pass. Every existing PA19 reference hash is unchanged. Before
  generation, all three ambiguity errors were checked as multiple applicable
  methods, matching independent E0034 even with discriminating arguments/results.
  Reference provenance is `pa19/reference-batches/1794339229354772468-41b9950d1c80.json`.
- The harness passes 315 tests, 468 roots in 13 binaries and 44 assignment stages;
  all 128 pinned upstream fixtures and 126 Reference files verify.
- `measure/passed.json`: 30 immutable same-profile A/A and two paired ABBA
  observations preserve complete import/qualified-control LowIR and all 74 frozen
  survey outcomes. Candidate/baseline ratios are 1.069/0.994/1.004; A/A ratios are
  0.885/0.991/1.016. These are output-equality controls, with no speed claim.

The first attempt stopped before compilation on the repeated filename-stem audit;
the test is now `enclosing_trait.rs`. A later test edit missed a formatted
assertion and failed compilation before execution. Both failed receipts remain.
No lint allowance, full local private sweep or release build was added.

Immutable dev-fast CLI
`9beaaac99f97adebc999cc42342c9bace29730ed14cbfb9600281e6408929a2d`
binds commit 17baa8cb943d5386cd056c0d89c5be2b05baf5d5, an empty patch and
3,885 compiler/CLI/test Rust hashes. The prior full run 42882896962 on 8feedb05a
is still active after lint, architecture, harness and host graph checks. This
semantic successor's full qualification and the earlier optimized nested-proof
controls remain pending. Do not cancel an active full run by redispatching.

## Original core

The unchanged 120-second observation fails its infrastructure deadline at
121.2436 seconds. Its partial stream retains 253 failures and 5,122 item attempts,
ending at function 10219, file 279 byte 9949 node 427, in x86/ssse3.rs. All 722
source hashes, six configuration bindings, profile and quotas remain fixed.
Progress SHA-256 is
`769a0935d408714c3614bc02c95c9eba52287755bf855d467ccaa0926c5d9308`.

The six num/error.rs item attempts have no retained failure record; both previous
literal/match-result `fmt` lookup fences are absent. This is progress through the
recorded prefix, not a complete body census or an original artifact. Prefixes end
at different items, so their raw counts do not establish whole-core speed.

The first retained failure is Alignment::try_from at mem/alignment.rs, file 55
byte 11798: `align.get().try_into()` returns `Result<Alignment, Self::Error>` from
the actual const TryFrom<NonZero<usize>> impl. Its diagnostic is associated type
projection requires normalization. First reduce the ordinary blanket conversion
and return-type inference in the earliest owning PA; retain the actual const/source
contract separately if the ordinary reduction passes. The next failure is
Discriminant<T>::fmt's actual Debug obligation, not another implicit trait import.
No complete census, checked-body count or original core/alloc/std artifact exists.
