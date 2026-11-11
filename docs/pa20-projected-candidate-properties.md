# Projected method properties and actual borrowed places

Compiler `b15d29240f5f1a00894a2149bac1ddc46d18ed20` removes the reached
`try_from_fn_erased` normalization diagnostic at `guard.array_mut.len()` in the
unchanged original core. The next first diagnostic is `LazyCell<T>::default`
in `cell/lazy.rs`, where `T::default` is passed to `LazyCell::new` as an initializer.
The 120-second observation remains partial and fails its deadline; no original
core, alloc or std artifact exists.

Immutable dev-fast CLI
`94bd08b42caf5680d4ccce6efbeef449e6eb159473ae3668b1a21a94a746f808` is retained at
B's `obj/parallel/projected-properties/candidate-dev-fast/rustgm`. Its producer
binds the source commit, an empty whole tracked diff and 3,902 compiler/CLI/test
Rust hashes. The [machine evidence](pa20-projected-candidate-properties-evidence.json)
binds the inputs, focused receipts, generated references, measurements and
original observation.

## Reuse semantic selection for projected conditions

The ordinary minimum is `Wrap<R::Out>.get()` under `R: Family`; a plain wrapper
already reproduces the failure. Source slice methods reproduce core's actual
receiver filtering, while ordinary builtin slice `len` controls bypass it.
Conditional Copy/Clone, concrete outputs, explicit calls and paid/unpaid caller
bounds separate the relevant cases.

The structural property query now returns a typed request for semantic work.
Ordinary and unresolved candidate possibilities keep their existing path. A
resolved projection or hidden nominal size condition sends the original builtin
bound through the existing non-refining trait selector. It owns projection
formation, caller assumptions, proof scopes and pending declarations. Owned path
queries retain their existing trial and region frontier across those waits.
The selected callable still proves every actual bound, lifetime and effect.

No normalization error becomes an absent candidate or an inferred permission.
Budget and malformed-graph failures propagate. The coarse field-only declaration
possibility remains separate from actual Copy/Clone permission. A raw shape's
deferred size request is explicitly refused before another property root uses
that walker; completed siblings remain reusable. The public regression alternates
an unpaid nominal size request with a byte property query and observes both
requests without losing the independent query.

## Prepare the referent place actually constructed

Full LowIR preflight exposed a separate existing failure in five explicit-call
controls. Baseline `00d205dc` also typechecks those inputs and then fails with
`generic aggregate requires instantiated fields`. A retained debugger stack
locates the demand in destruction classification of a reborrow place.

The ordinary `Wrap<T>::get(w)` form needs only PA18. The previous pointer-specific
root producer now supplies all actual selected reference adjustments to the two
existing field-preparation consumers: published body storage and value-type
publication. Source order, raw field identities, declaration waits and concrete
normalization evidence stay with those producers. No inference-arena scan or
expansion of unrelated reference types is added.

This is structural field preparation. It grants no move, Copy, lifetime or source
call permission. The negative borrowed move still rejects; the real non-Copy
referent's destructor runs exactly once at its owning scope's end. No dense
record, global cache, compiler quota, backend contract or `storage.py` changed.

## Assignment reductions and qualification

- [PA18](../pa18/tests/spec/borrowed-place-fields/README.md): five new programs,
  four independent Rust object accepts and one genuine move rejection. All full
  LowIR outcomes pass; three unused-body controls declare compilation only and
  the destructor program has actual Rust/CPPGM execution. Four existing shared
  call/reborrow inputs remain byte-identical and keep their full ownership checks.
- [PA20](../pa20/tests/spec/projected-method-inputs/README.md): 27 new programs,
  24 accepts and three rejections. Conditional method priority executes the six
  independent values 41/41/23/23/41/31. The other 23 positives declare unused-body
  compilation contracts. Width 8/32/128 and 128 unrelated types check bounded
  inference, trait selections, method steps and probes.
- [PA31](../pa31/tests/spec/projected-method-inputs/README.md): seven exact source
  counterparts independently emit Rust objects and pass source analysis. The
  attributed foundation bytes are preserved. The existing 13 source lookup cases
  reuse their frozen Rust language outcomes; the three known valid normalization
  failures now accept, and both genuine rejections remain.

Final production receipt `1794391956383811769` passes eight exact tests across
two targets in 115.16 seconds, including full ordinary LowIR, source contracts,
scaling, all 14 prior symbolic-index language contracts and real CTFE47.
The query-state receipt `1794392290746405887` passes its exact public regression.
Final native receipt `1794392342902064203` passes both Rust/CPPGM programs in
37.54 seconds. Strict selected production/CLI/test Clippy, full formatting and
the syntax-aware audit pass. The existing bounded nested-constructor field
regression separately passes in `1794391641399698727`.

Exactly 32 new text triples were independently reviewed, generated through the
scoped course runner and checked again. The review checks 26 empty-main graphs,
four genuine diagnostics and the two executed call/cleanup graphs. Private main
IDs match each definition to its entry call. Name-mode dumps use a different
binding configuration and provide no numbering authority. Every pre-existing
PA18/PA20 oracle and receipt remains byte-identical. The upstream 128 fixtures,
126 Reference files and 315 harness tests pass; registration remains 468 roots
in 13 binaries and 44 assignment stages.

Focused public checks supply local development evidence. Private affine, budget,
fields-only and pending-header controls remain in full CI. The preceding full
run [42988281942](https://github.com/vishvananda/rustgm/actions/runs/42988281942)
on exact `31be0c24f` passed every stage. Successor
[43009364083](https://github.com/vishvananda/rustgm/actions/runs/43009364083) runs
on exact `9fdf58f7588a35c69b85601fe3b70e242a937247`, containing source `b15d29240`.
Fast, qualification lint/architecture/harness and the independent host graph
passed; release/course was running at the recorded observation. Documentation
descendants do not restart this run.

## Measurements and original observation

Thirty exclusive immutable observations use A/A and two ABBA cycles on three
frozen controls: ordinary imported-trait LowIR, an existing unused generic-reborrow
library contract with empty LowIR and an unresolved-projection source analysis. All
complete stdout/stderr/status identities agree. New rigid-projection acceptance
has no successful baseline; its changed-path growth is checked by the independent
width/noise cases and bounded work counters.

The tiny unoptimized controls retain paired wall ratios 1.150, 1.167 and 1.035;
CPU ratios are 1.153, 1.172 and 1.035. A/A ratios are 0.690, 0.632 and 0.952.
The slower observations and calibration variability remain recorded. They prove
neither performance neutrality nor an optimized or whole-core speed result.
Optimized performance and the older nested-proof/stack qualification remain
required; no smoothing rerun or quota increase is used.
The old `00d205dc` executable was explicitly pruned after comparison and replacement
publication in receipt `1794395534452651774`. Its producer and raw observations
remain; current `94bd08b4` and the older optimized `21f` baseline remain pinned.

The original observer fails its unchanged 120-second deadline after
121.279946 seconds, retaining 159 partial records through function10718,
file283:70779/node3113. Progress SHA-256 is
`486344dc4ca80525c50eb9badff41e64679392a3959f1712c0c377bd1cb63133`.
The reached array function3385 at file98:30975 has no `try_from_fn_erased`
record. First is now `default`, file113:12567, with an unsatisfied trait
obligation; its mapped source is **`cell/lazy.rs`**. File112's later `replace`,
`clone`, `clone_from`, `partial_cmp` and `cmp` records map to **`cell.rs`**.
Earlier attribution of these records to Option was incorrect.

Postcheck `1794393775288810739` verifies all 722 input hashes, 375 opened files,
six configuration bindings, immutable binaries, compiler hashes, profile and
limits unchanged. There is no complete census, comparable body count, throughput
result or original artifact. Next reduce creation of the `T::default` initializer
under the actual `LazyCell::new` contract, reusing the earlier conditional
function-item and const-adapter requirements. Keep Cell replacement/cloning
separate until evidence establishes a shared cause.
