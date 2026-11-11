# Method properties after trait inference

Compiler `f93ffdb7fb7f413148406c0284408fe83f68f2d8` repairs the method ambiguity
found by full CI43009364083. When a candidate initially has an unknown `U`, its
`U: Copy` condition is only possible. Matching the caller's trait assumption can
then resolve `U` to a non-Copy type. Lookup must check that condition again before
counting the candidate. The unchanged PA19
`method-applicability/builtin-after-trait-inference.rs` owns this minimum.

Structural conditions now remain in their existing vector until trait equations
settle. A fresh property query reads that updated graph. Newly exposed projected
conditions transfer once to the existing semantic selector. Owned path queries
retain their private region frontier when retrying those goals. There is no new
normalizer, proof engine, dense record or permission. Selected calls still prove
their full bounds, regions, formation and effects. Field-only possibility remains
separate from actual Copy or Clone permission; infrastructure errors propagate.

The other CI failure was a public source test that still expected three old
normalization fences. Their independently valid outcomes were already preserved
in the source packet and private regression; the public test now agrees.

## Reductions and evidence

The [PA20 packet](../pa20/tests/spec/late-method-properties/README.md) adds four
concrete-projection cases: two Rust object accepts and two genuine E0034
ambiguities, paired between receiver and qualified lookup. All four full LowIR
outcomes pass. Two empty-main graphs and two ambiguity diagnostics were reviewed
before generating exactly four new text triples. All8,512 pre-existing PA19/PA20
oracle and receipt files remain unchanged, and the original PA19 course case
passes its existing text oracle.

Four generic projected-premise requirements remain in the packet's `pending.json`,
with complete source bytes, hashes and independent outcomes. The two valid unpaid
positives still report ambiguity before their property inputs settle. They are
required follow-on work, not passing course cases or language rejections.

Receipt1794397840938770713 passes both original CI failures and three existing
projected-property/scaling/query-state controls. That exploratory batch fails its
new generic projected-premise positive; it must not be reported as wholly passed.
Production identities are unchanged in the final receipts. Receipt1794398451632493330
passes the four finite concrete-projection outcomes. Receipt1794398656352842239
passes real Rust/CPPGM method priority41/41/23/23/41/31 and destructor execution1.
Selected strict Clippy, formatting and architecture pass. Upstream128/Reference126
and315 harness tests pass; registration remains468 roots/13 binaries/44 stages.
The [machine evidence](pa20-post-inference-properties-evidence.json) binds these
individual results and retains the failed exploratory evidence.

Full [43009364083](https://github.com/vishvananda/rustgm/actions/runs/43009364083)
on exact9fdf58f75, containing the preceding sourceb15, failed those two public
tests after all3,065 private tests passed. Later qualification stages were skipped.
Full successor [43019302026](https://github.com/vishvananda/rustgm/actions/runs/43019302026)
PASSED every stage on exact`bc401434ac3a525781359f6d194a65cf5811aa6f`, containing
this source repair, including native/debug/independent/PA25 qualification.

## Immutable observations and remaining work

CLI`f46b54f3248e174e3cafeb1d791811a5bdbbf96004bbdd4dc74a6f3df2c53bf6`
is pinned at B`obj/parallel/projected-ci-repair/candidate-dev-fast/rustgm`.
Its producer binds the source commit, empty tracked patch and3,902 Rust hashes.
Thirty exclusive A/A and paired ABBA observations compare it with the then-retained
b15 CLI94bd. Complete outputs agree for an ordinary import, the changed accepted
128-call property path and the actual method-priority LowIR graph.

Tiny unoptimized paired wall ratios are1.044/1.046/.930; CPU ratios are
1.045/1.046/.929, with A/A wall ratios.876/.993/1.004. The slower observations
remain recorded. There is no performance-neutrality, optimized or original-core
throughput claim. Repaired non-Copy inference positives have no accepting retained
baseline; the existing width/noise controls qualify bounded work instead.

After replacement publication, baseline94bd was explicitly pruned in receipt
1794399578264808714. Its executable is absent; producer and raw observations
remain. Only currentf46 and the older optimized21f baseline remain pinned. Retired
metadata is not an available executable for another run.

No new original-core observation was run for this CI repair. The last partial
observation remains b15's `LazyCell<T>::default` at cell/lazy.rs113:12567, passing
`T::default` to `LazyCell::new`. That WIP was restored and qualified as the
[callback checkpoint](pa25-callback-formation.md); its later partial observation
clears the reached initializer failure and first reports RefCell::replace. No original core/alloc/std artifact exists.
The two symbolic-index native artifacts, deep receiver inference, const-adapter
factories, bootstrap/imports, transport, tuple factories and optimized qualification
remain required. No compiler limit, source pin, backend or storage.py changed.
