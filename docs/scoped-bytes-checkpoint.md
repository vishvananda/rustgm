# Scoped execution, privacy and source byte concatenation

Compiler snapshot `ca54609a` combines actual scoped macro invocation, physical
semantic type privacy and source-declared `concat_bytes`. Three ordinary privacy
programs are published in their earliest owning PA12 lane. PA31 retains its
source-provider contracts. PA1–PA24 remain implemented drafts; PA25–PA44 remain
planned. There is still no complete original upstream crate artifact.

## Functional qualification

The complete release inventory has 3,791 passes. Recovery executes 1,868 fresh
private/frontend cases, including their ignored native tests, and authenticates
1,923 passes from unchanged release targets. This is complete qualification
with explicit retained provenance, rather than a claim of a fresh full rerun.
All 8,385 active text cases pass, with no preexisting oracle changed.

The first full release found a real macro-layout regression: inline compiled
scoped matchers widened Body to 104 bytes and Definition to 144. The repair
keeps that uncommon compiled state behind a Box, charging its 96-byte payload
before allocation. Recovered layouts are Body 96, Definition 136, Builtin 1,
SyntaxKind 28, Node 36, SourceCore 176 and Package 256 bytes. Ordinary declarative
matchers remain inline. The failed run and its exact input/output hashes remain.

The compiler checks privacy using solved types, selected calls and adjustments.
It shares an iterative visited type graph per body and physical module ancestry;
primitive-only checks allocate no graph. Actual opaque public interfaces retain
their hidden representations privately. Source byte concatenation authenticates
the actual declaration and preflights typed operands, work and payload before
allocation. Neither feature reparses rendered output or manufactures permission.

Formatting, architecture/file audit, all-target strict Clippy, harness, upstream
provenance and backend checks pass. Affected independent source PA7/8/12/19/20/21,
native PA12/19/20/21 and transitive metadata checks pass. All three release
adapters are bound to the same qualified library. The inventory authenticates
1,832 required exact test names; these names are a coverage subset, not extra
passes added to the inventory total.

Evidence is under `rustgm-validation-work/obj/parallel/scoped-bytes-release-v1/`
and `scoped-bytes-recovery-v3/`.

| Artifact | SHA-256 |
| --- | --- |
| CLI | `5d84d4ee880aed7bba63fcb51e7bb6816b67f336326407cdab7439b294cdc87f` |
| Functional receipt | `487ce8baf728fc4b9f8e637bff76483824d939a3923428a60bd2c6ae1452e991` |
| Checks | `0509e0aafcf75760d1907a9bc958b6db64023681eb04e9aa11993563cc74565a` |
| Inventory | `027ef219be6850bce4f4dbe4d07724347001d612ebee5c08d4413168ed68e3e5` |

## Performance qualification

The immutable comparison against qualified public `65f20e8b` retains all 110
preceding workloads and adds fifteen byte/scoped/privacy controls. All 125 output
preflights pass: 116 successful byte-equal pairs and nine candidate-only inputs.
All 100 native preflights and 68 growth controls pass. No instruction increase
crosses the unchanged 5% review threshold. The comparable range is -0.157% to
+3.367%, median +0.353%; these are per-input instruction changes, not a whole-core
compile-time claim. A/A calibration, ABBA wall observations and every RSS sample
remain in `obj/parallel/scoped-bytes-perf-v3/`. Unsupported baseline cases are
never reported as speedups.

The largest new scoped-function growth is 3.793x for 32→128 functions. Bounded
privacy depth grows 2.270x and repeated uses grow 3.030x over the same factor-four
dimension. The separate typed DAG invariant retains exact linear visitation.

Seven original RSS increases cross the unchanged +512 KiB/+5% review threshold.
A fixed follow-up takes exactly one Massif heap/stack profile and one GDB exit
mapping per A/B/input pair, fourteen observations total. Exact original compiler
outputs and fresh side outputs are verified. No original timing/RSS observation
is replaced or rerun to obtain a favorable comparison.

| Input | Original RSS delta (KiB) | Peak heap delta (bytes) | Peak total delta (bytes) | Exit compiler mapping delta (KiB) |
| --- | ---: | ---: | ---: | ---: |
| library-owner-width-128 | 534 | +0 | +128 | -136 |
| history-introduced-contexts-8 | 728 | -1768 | +144 | +256 |
| history-introduced-contexts-32 | 642 | +256 | +240 | +244 |
| history-historical-predecessor-128 | 746 | +1024 | +1088 | +268 |
| history-latest-shadow-8 | 720 | +4852 | +96 | +164 |
| privacy-linear-depth-32 | 608 | -286 | +48 | +188 |
| privacy-linear-depth-128 | 646 | +0 | +5320 | -28 |

These instrumented mappings do not numerically reproduce the original RSS
samples. Together with the peak-heap/total measurements, they distinguish the
retained allocation costs from executable residency/layout and allocator/stack
variation. Accept these bounded costs for this feature checkpoint, retaining
all seven original triggers. Do not claim zero memory regression or treat the
instrumented run as a replacement RSS pass threshold.

Results SHA-256: `ded0b024ddfd808cb21129cc692203cc7478f9af19b2c185551f8237049caba4`.
Analysis SHA-256: `3d539a377a25bea31ca8f805f9a8c50739bc13ff811ea515077d4c5907e41387`.
Memory results SHA-256: `1140e1e0bd36cc55d89ad5fd058b6108bc4b72cd202410f124f285e17ab13b13`.

The independent runtime-control preparation retained a failed stress hypothesis.
Binary aliases repeating the complete prior tuple at every level expand into
an exponential type tree: rustc hit the 12 GiB kernel memory limit at depth 32.
This is an infrastructure failure, never a language rejection. Original inputs,
predictions, partial observations and the kernel OOM journal remain unchanged.
A second run retained six successful scoped/linear-depth cases, then was stopped
when source review found the repeated-use controls retained that same binary
depth-32 shape. Fresh bounded repeated-use controls complete the independent
18 metadata/object and nine native checks, with six unchanged cases retained.
The new linear-depth graph has one recursive branch and one shared Leaf edge
per level. The separate typed privacy invariant still covers shared DAG depths
8/32/128 and repeated uses with exact linear work counts. No failed stress
result is used as a speedup or silently replaced by a successful observation.

Evidence: root worker `obj/parallel/scoped-privacy-perf-oracle-v1/` through `v3/`;
complete independent results SHA-256
`3369a74a61fdbc5f35abe75e7261b661dd7b6e365c7fdd7d83905103d9641dde`.

## Original core and remaining requirements

The same immutable CLI opens four unchanged original core files and reaches
`autodiff_forward`, `core/src/macros/mod.rs:1553`, byte 57236. The previous
published compiler stopped at `concat_bytes`, line 1126. The probe fixes target,
edition and source identity; its exit 1 records an expansion blocker. It does
not establish complete parsing, semantic checking, body export or an artifact.
Whole-core receipt: `obj/parallel/scoped-bytes-whole-core-v1/result.json`, SHA-256
`cadf73eb8ec9dcb507dc90a2b87153ecb781ada0519ea086adca3c3bd8d58370`.

The stricter attribute-consumer worker separately exposes an earlier semantic
requirement, `rustc_coherence_is_core` at `core/src/lib.rs:65`. Its implementation
must support actual primitive inherent declarations and selection, rather than
ignore that active attribute to retain a later expansion frontier. Other open
work includes generic artifact/instance proof, reference-valued anonymous const
callers, attribute consumers, nonempty formatting and foreign macro transport.
See the [blocker tracker](library-build-blockers.md). Ordinary core, alloc and
std algorithms continue to come from the original source.
