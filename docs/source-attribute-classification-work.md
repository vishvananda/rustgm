# Source attribute classification work

LIB-042 exposed a resource-policy mismatch while expanding the unchanged core
integer methods. Immutable compiler `7ecaec04` opened 371 pinned source files,
then rejected work step 1,000,001 in `decorated_calls::macro_attribute`. Attempted
storage was 2,085,659 bytes and retained macro witnesses 53,959 bytes. The loop
classifies only the current configured owner's effects and identifier spelling;
it was charging this ordinary linear input work to the global panic-permission
budget. These are resource policies, not Rust legality limits.

Generic classification now pays the existing cold lint-policy Producer through
its cost-only `attribute_read` method. `ensure_attribute_policy` factors the same
lazy initialization already used by `lint_owner`. The change is SourceCore-only:
ordinary sources gain no classification debit or new policy allocation. The
existing producer's 10,000,000-step/64 MiB limits are unchanged. Its actual reads,
allocation prepayments and failed attempts remain counted.

Before classification, the source macro State is checked for sticky failure.
Actual macro header, allowance, frame, attribute path and storage consumers keep
their existing charges and 1,000,000-step/64 MiB policy. Classification creates
no feature permission or completed owner. No field, cache or map is added.

The actual generated-source controls use one macro invocation and verify its
real function inventory. For widths 8/32/128, general work was:

| Doc prefix | Width 8 | Width 32 | Width 128 |
| --- | ---: | ---: | ---: |
| 1 | 231 | 807 | 3111 |
| 8 | 623 | 2375 | 9383 |
| 32 | 1967 | 7751 | 30887 |

Permission work stayed constant across each generated width: 21, 56 and 176
respectively. The macro definition itself grows with the prefix and retains its
real authentication charges. A separate ordinary-attribute control preserves an
actual near-exhausted permission count of 999,999; the next real permission
debit fails, and classification cannot revive that failed State. Exhausting the
general payer separately blocks child expansion, retains failed attempts and
rejects repeated classification. Owner/capture, unknown-attribute, ordinary-path
and actual warm permission/frame exhaustion regressions also pass.

Source checkpoint `d0062fde36fbc845332cf6a37a82bba310f85926` passed seven exact
private controls and ten lexer checks. Metadata, formatting, global architecture
and both strict Clippy modes passed in the initial bounded gate; its private
binary build timed out before tests. The unchanged-source missing-only
continuation authenticated those actual static receipts, built the remaining
binary and executed the exact controls in 172.47 seconds. The original timeout
is retained; this is not a claim that its failed overall gate passed.

Evidence is under `obj/parallel/lib042-source-attribute-work-v1/`.
Static receipt: `obj/qualification/fast/1793091347460469414/receipt.json`, SHA-256
`7ba81b802ce414f4d63380fb7a690b356b909ea1f0cfcfb859df37c882d56d7c`.
Execution receipt: `obj/qualification/fast/1793091717189205451/receipt.json`, SHA-256
`521f524d72058e5c5cc20d235099739a247f9a8c3e696d2126a76fdfc76db3a5`.

Stacked derive attributes can separately replay an already-classified prefix
when the same owner resumes after each derive. That continuation and its
scaling contract remain deferred; this correction neither optimizes it nor
claims it caused the observed integer-macro rejection. No new core attempt,
release build, broad suite or native observation was run in this worker phase.
