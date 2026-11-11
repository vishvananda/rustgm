# Callback formation and const invocation

Source `5798bd7f161f4420117079ac02306458d9a8b621` distinguishes creating a
function value from invoking it. `LazyCell<T>::default` stores `T::default` as
an initializer; creating that value pays ordinary bounds. An actual const call
must additionally prove its conditional effects. Existing member-use clauses,
instantiation, typed argument bindings and trait selection perform both checks.
Method/operator/implicit calls keep their actual invocation requirements.
Required members of const traits receive the same actual-call inspection as
explicit const functions. There is no new proof engine, permission or dense state.

The canonical FunctionItem argument reader reuses the existing known_arguments
API. Locals and references preserve typed callable identity during semantic
checking. The separate checked-body literal-callee limitation is still required
work for two independently valid paid calls. No backend, storage.py, source pin,
compiler limit or observer window changes.

## Reductions and validation

The earliest owning packets contain46 independently declared Rust1.98 object
outcomes: PA19=3, PA24=2, PA25=37 and PA31=4;37accept/9reject. The four source-role
counterparts preserve the exact attributed foundation prefix. All46 declared
type-phase outcomes and44 complete LowIR outcomes agree. PA25 alias-paid.rs and
reference-paid.rs remain REQUIRED full LowIR positives: types pass, but the body
checker rejects their actual local/reference callee reads. Their manifests retain
required exit0 and observed failure1. Unpaid alias/reference opponents reject
with actual trait obligations; the erased-pointer call rejects in checked-body
const checking. No positive outcome was inverted or hidden by an unrelated fence.

Final public receipt1794402541806766803 passes six exact tests across three targets
in206.71s: the46 contrasts, twelve creation/invocation width/noise inputs, existing
const eligibility,15 source-adapter contracts, unused children and21 projected
method controls. Strict selected production/CLI/test Clippy, full format and
architecture pass. Creation and paid invocation widths8/32/128 keep bounded
inference/trait/method work;128 unrelated types leave the counters unchanged.

Final native receipt1794402928426840181 passes three exact tests in75.08s.
Three new real Rust/CPPGM programs check stored initializer23, const-created
pointer invoked later23, and actual paid CTFE41. Existing method priority checks
41/41/23/23/41/31 and one actual destructor execution also pass. Only the independent
unstable Rust child receives its explicit bootstrap/debug-assertion/opt0 profile;
existing ordinary native checks and production compiler profiles stay unchanged.

Exactly five new PA19/24 text triples were independently reviewed, generated and
rechecked: four empty-main graphs and one genuine missing-bound diagnostic.
All5,461 old PA19/24 oracle/receipt files remain byte-identical. No planned PA25/31
course oracles or assignment release are claimed. Upstream128/Reference126,
315 harness tests and468 roots/13 binaries/44 stages pass. The
[machine record](pa25-callback-formation-evidence.json) binds the raw identities.

## Frozen comparison and original core

Immutable CLI05cc3029540dc069cf360821f180521d74278c5f5986a8e8e69255dc7955def5
at Bobj/parallel/lazy-default/candidate-dev-fast/rustgm binds source5798, an empty
tracked patch and3,903 Rust source hashes. Thirty exclusive matched dev-fast
A/A+twoABBA observations preserve complete output/status equality against f46
for ordinary import,128 already accepted paid calls and the method-priority
LowIR graph. Paired wall ratios.965/1.018/.994 and CPU.990/1.026/.961 retain the
slower paid-call observation. A/A wall1.026/1.019/.693 includes noisy calibration.
There is no neutrality, optimized or whole-core throughput claim. Newly accepted
formation has no accepting retained baseline; the twelve bounded-work inputs
qualify changed-path scaling instead.

The original observer retains its120s deadline and fails at121.668676s, with118
partial records through function6558,range/iter.rsfile195:3846,node310. It reaches
LazyCell's default function3647 at113:12543 with no file113 failure record.
The reached initializer failure is cleared. First now is **RefCell::replace**,
cell.rsfile112:35607,line1026: incompatible types at
`mem::replace(&mut self.borrow_mut(), t)`. The next records are clone, clone_from,
partial_cmp and cmp in the same file. Reduce the mutable-borrow coercion before
claiming those later failures share a cause. Earlier shorthand Cell::replace
identified only the module and was imprecise.

Progress SHA972dee75d1fc8bcc9cfd77fc38928e798f07ba823cc9f20587fb53b12975d177.
Postcheck1794405273338140972 verifies all722 original source hashes,375 opened
sources,six bindings,immutable CLI/producer/current3,903 Rust hashes and unchanged
profile/limits. The deadline remains an infrastructure failure. Partial record
and reached-function counts are not comparable throughput or complete censuses.
No original core/alloc/std artifact exists.

Prior full [43019302026](https://github.com/vishvananda/rustgm/actions/runs/43019302026)
on exactbc401434ac3a525781359f6d194a65cf5811aa6f, containing sourcef93, passed
EVERY fast/release/course/native/debug/independent/PA25 stage. A full successor
for this callback source is due after publication and an active-run check.
Do not restart that successor for documentation/input descendants.

The two paid-alias artifacts, two generic projected-premise inference positives,
two symbolic-index native artifacts,128-depth inference, source factory executions,
bootstrap/imports, transport, tuple factories and optimized qualification remain
required as listed in handoff.md. Whole PA25–PA44 remain planned.
