# Upstream library bootstrap and real-crate implementation order

This is the implementation plan adopted on 2026-10-22. It preserves PA1–PA30's
owning contracts and extends the goal through all assignment requirements needed
to build the original upstream core, alloc and std crates and execute their
consumers. It replaces the old optional library follow-up with required
assignments before async.
Every new or reordered assignment remains planned until its actual release
contract passes. A draft checkpoint is not an assignment release.
The [pinned source inventory](library-bootstrap-inventory.md) records the concrete
compiler contracts and target constraints behind this order.
The [live blocker tracker](library-build-blockers.md) maps concrete failures to
their owning assignments, tests and workers. The next implementation priority is
the observed unchanged-core blocker; independent later prerequisites can proceed
in parallel. A passing internal checkpoint is not a crate-build milestone.

The [2026-10-26 execution plan](library-bootstrap-acceleration.md) is the current
operational priority and reviewed imported-body preparation policy. It supersedes
historical immediate-task descriptions below while preserving this document's
full upstream acceptance ladder and assignment ownership.

## What we compile and what the compiler supplies

Compile pinned upstream `core`, `alloc` and `std` from their original sources.
Ordinary types, methods, trait implementations, iterator adapters, containers,
UTF-8 algorithms and collection algorithms come from those libraries. Missing
`Result::map` or `Vec` is a bootstrap requirement, not an instruction to add
another compiler implementation of the API.

The compiler supplies language-item identity, built-in macro/derive expansion,
intrinsic semantics, primitive/type-system rules, layout, calling conventions,
constant evaluation and the internal language features used by the pinned
sources. Register source-declared identities through validated attributes and
ordinary crate identities; a familiar path or spelling grants no privilege.
Check duplicate/missing items, signatures, feature gates and exact owning crates.
Current canonical library declarations and checked body factories are explicit
bootstrap scaffolding. A real-source sysroot must replace their library
definitions without creating a second set of Option/Iterator/Clone identities.
Do not silently fall back to synthetic definitions when source compilation fails.

The pinned Rust 1.98.0 [core root](https://github.com/rust-lang/rust/blob/1.98.0/library/core/src/lib.rs)
uses `no_core`, internal attributes and unstable features, including const-trait
and specialization-related machinery. Inventory active declarations and bodies,
not just the stable public API. Classify each attribute as semantic, diagnostic,
optimization-only or inactive in the selected configuration. Ignoring a semantic
attribute or accepting an unimplemented feature is not compatibility.

The pinned core root directly imports stdarch and portable-simd modules. Their
active parsing, typing, layout and target-feature requirements belong to the
core bootstrap, even when a public API is unstable. This is substantial compiler
compatibility work, not only supplying a few missing functions. Track the source
feature inventory as separate checked milestones within PA31/PA32.

Use CPPGM's hosted progression as the model: its PA29 establishes real compiler
builtins and hosted-source compatibility; PA30 compiles heavy STL headers; PA31
checks linked execution. Its builtin registry supplies capability answers from
the same descriptors consumed by semantic analysis and lowering. RustGM should
likewise have one typed source of truth, followed by real library and native
consumer tests. Rust separate compilation adds an earlier requirement: generic
bodies and their contracts must cross crate boundaries.

## Revised assignments after PA30

| PA | Responsibility | Required evidence |
| --- | --- | --- |
| 31 | Compiler/library bindings | Source-declared lang items and intrinsic/builtin declarations, internal-feature inventory, explicit sysroot ownership and panic-abort boundary |
| 32 | Pinned core bootstrap | Build original upstream core and execute independent consumers using its real generic implementations |
| 33 | Allocator and alloc bootstrap | Allocator/runtime contracts plus original alloc; execute allocation, reallocation, destruction and collection consumers |
| 34 | Edition and crate build integration | Rust 2021 alongside 2024, dependency/features/target configuration, build scripts and generated inputs with reproducible provenance |
| 35 | Upstream crate acceptance | Original pinned serde_core without defaults, then serde_json with alloc and its complete selected dependency graph |
| 36 | Std and platform bootstrap | Original upstream std for the pinned platform and abort profile, followed by default-std serde_json consumers |
| 37 | Procedural macro host | Macro crate builds, host execution and token/span protocol |
| 38 | Derive and attribute macros | Original serde_derive and applications exercising Serialize/Deserialize derives |
| 39 | Panic and unwinding | Full former PA31 contract: unwind cleanup, double panic, personalities and C-unwind |
| 40 | Edition and attribute closure | Remaining former PA36 contract, including 2015/2018 and complete edition/attribute edge cases |
| 41 | Target-specific closure | Complete target/assembly contract and explicit backend capability coverage |
| 42 | Async lowering | Future state machines, suspension and resume behavior |
| 43 | Async ownership | Pinning, suspension loans, cancellation and advanced async forms |
| 44 | Frontend acceptance | Full language/library census, cross-feature corpus, debug locations, reproducibility and scaling |

Reproducible self-hosting remains the optional PA45 extension, outside the
required upstream library bootstrap goal.

Panic-abort support needed by core/alloc is required in PA31–PA33; full unwinding
is preserved at PA39. Likewise PA34 owns the Rust 2021 behavior used by the pinned
crate graph and the Rust 2015/2018 behavior required by original std's
rustc-demangle/cfg-if dependencies. Every internal or target feature required by an earlier
workload must be implemented and tested before that workload passes; PA40/PA41
cannot be used to waive a failing active source path. Broad closure follows the
first usable library profile rather than delaying it.

## Implementation priority and worker handoffs

Assignment numbers describe cumulative release ownership. Implementation work
follows dependencies and can proceed ahead of release without declaring a
prerequisite complete. Start source bootstrap while remaining PA25–PA30 corner
cases are tracked; do not wait for every assignment release to investigate real
library builds. Their actual blockers determine the order of remaining work.

1. Integrate the already qualified PA25–PA30 changes and finish in-flight proof,
   iterator and layout checkpoints. Retain their original valid-but-unsupported
   cases and the full assignment requirements.
2. Make PA29's generic crate transport and PA31's source-library binding path the
   critical path. Transport external type/const/lifetime identities, traits and
   impl conditions, associated items, macros, checked generic/const bodies,
   retained proof contracts and drop/closure dependencies. No local arena index,
   source-text reparse or fake body may stand in for those contracts.
3. Inventory the pinned core/alloc source and dependency graph while transport
   is developed. Record each failure at its actual owning phase, with an original
   source reproducer and independent outcome. Use that evidence to order PA25
   constant-evaluation, PA26 language-hook and PA27/PA28 validity/intrinsic work.
   Start the binding path with source intrinsic declarations and
   `size_of`/`align_of`/`needs_drop` through `SizedTypeProperties`. Register real
   Tuple/FnOnce/Output identities, rust-call and const-callability obligations
   before qualifying `const_eval_select`; callback shapes alone are insufficient.
   Follow with pointer/memory bodies and generic atomic declarations. The
   [callable-selection probes](library-bootstrap-inventory.md#callable-selection-followup)
   retain accepted runtime pointers and compiler ICEs separately from source
   rejections; do not encode an unqualified function-items-only restriction.
4. Compile core end to end. Start the PA34 build/edition work in parallel so it
   does not become a serial prerequisite after the libraries work. As soon as
   core and the required build path work, build and execute PA35's original itoa
   and no-default-features serde_core graph. Do not wait for alloc or std to
   exercise these core-only targets. Their failures feed back into the earliest
   owning compiler assignment.
5. Compile alloc and execute its consumers, then build and run the complete
   selected serde_json alloc graph. Preserve the earlier core-only crate checks
   as regression gates.
6. Build PA36's std graph and default-feature serde_json, then procedural derives.
   Async and full unwinding do not gate the initial abort-profile target.

These are implementation checkpoints, not changes to cumulative release order.
PA35 owns upstream crate acceptance even when its core-only checks are developed
before PA33's alloc release. PA34's build/edition contract can likewise be
implemented against small separate crates before the library assignments pass.
The first real-crate result must identify the complete successful graph and
feature profile; it cannot imply that alloc, std or the rest of PA35 is complete.

Within PA31/PA32, keep four observable boundaries: validated source bindings;
configured whole-source loading/expansion; complete semantic and exported-body
checking; and object emission plus native consumers. For each boundary, record
the exact first failure, reached source set and owning compiler contract. A
requested internal feature gate alone does not prove that its constructs are
active on the selected target. Classify gates and attributes using source/config
evidence, then diagnose unsupported active constructs at their actual phase.
Never count root parsing or a reduced source package as a whole-library pass.

With three GPT-6.1-Sol/high workers, keep one lane on const proof closure followed
by generic metadata; one on genuine language hooks followed by source-declared
library bindings/core inventory; and one on memory/layout/intrinsics followed by
the selected core/alloc capability gaps. The integrator owns combined checks,
crate-build orchestration, roadmap consistency and publication. Handoffs follow
coherent committed checkpoints, not simultaneous edits to shared enums.

Do not expand bootstrap factories into ordinary API coverage while the same
effort can establish the source-library path. This priority change does not
remove PA26's required desugaring, derives, formatting or iteration semantics.
Those semantics must continue to work against actual upstream identities and
implementations.

## Current integrated checkpoint

The [local stability and source callable checkpoint](library-stability-callable-checkpoint.md)
qualifies local regular API inventory, closed source const callbacks, runtime
selector lowering and source macro import resolution. The subsequent
[constant selector checkpoint](source-ctfe-selector-checkpoint.md) executes the
actual selected const callback and preserves proved static callable targets
through owned Code publication. It passes 3,488 maintainer cases, 8,382 text cases
and a 51-input performance matrix. These checkpoints preserve explicit
boundaries for dependency stability, const/default-body APIs and imported source
roles. No original whole core, alloc or std build passes.

The [live tracker](library-build-blockers.md#observable-crate-milestones) owns
the current original-source frontier and worker queue. Later observations have
advanced beyond the historical macro/const checkpoints above to foreign VTable
loading, with alias/Pointee, actual source panic and imported provided bodies
still being joined. No whole upstream library artifact is qualified. Use the
[execution review](library-bootstrap-acceleration.md#execution-review-and-next-decisions)
for current scheduling and the tracker's separate assignment closure queue;
historical checkpoint summaries do not define additional immediate work.

## Acceptance ladder

Each milestone records its source revision, complete transitive dependency lock,
edition, target facts, features, panic/overflow mode, runtime and backend profile.
The initial library target is Rust 1.98.0, x86_64-unknown-linux-gnu, panic=abort.
The matching Rust source revision is
`88d9e12ae178fab0fb5cc050a94da85685d449ea`; freeze its source tree, selected dependencies and file
checksums before first execution. The locally installed 1.95 rust-src tree is
reference material, not that pin. In the pinned sources, `SizedTypeProperties`
supplies layout constants and atomic aliases use `Atomic<T: AtomicPrimitive>`.
These are concrete compiler protocols to implement, not reasons to add more
hardcoded type names.

Freeze compiler-private cfg facts as well as public ones. The pinned x86-64
profile exposes atomic widths 8/16/32/64/ptr; 128-bit atomics are target-conditional.
Internal `target_has_atomic_load_store` facts cannot be inferred from the shorter
stable `rustc --print cfg` output. Allocator shims use Rust ABI and typed
Alignment/NonNull parameters plus generated symbols and an OOM handler; directly
aliasing them to C malloc does not establish compatibility. Even an abort profile
requires correct panic language-item routing and const-evaluation/UB-check hooks.

| Milestone | Workload | What proves it |
| --- | --- | --- |
| Bindings | Small no_core declaration packages and ordinary clients | Correct identities/obligations; malformed, duplicate and forged declarations reject |
| Core | Entire selected upstream core crate plus original consumers | RustGM metadata/LowIR/object output; executable methods, conversions, formatting, slices, iterators and const consumers |
| Core crates | itoa 1.0.18 and serde_core 1.0.229 without default features | Separate target-crate builds and native formatting/visitor consumers, including generated inputs |
| Alloc | Entire selected upstream alloc crate plus consumers | Real Vec/String/Box/collection behavior, allocation failure and exact destruction observations |
| JSON with alloc | serde_json 1.0.151 with default features disabled and alloc enabled | Complete serde_core/itoa/memchr/zmij graph; parsing/serialization, escaping, invalid input and round-trip native checks |
| Std | Upstream std and default-feature serde_json | Platform/runtime link closure and actual std-facing I/O/error consumers |
| Derives | serde_derive and a Serialize/Deserialize application | Host macro execution followed by RustGM compilation and native round trips |

The crate versions are the inspected initial targets, not a claim that they
compile today. Freeze package checksums and all transitive versions before the
first build. Serde's build scripts produce private modules/configuration;
serde_json selects arithmetic configuration. Preserve their real outputs and
inputs. The base libraries do not require Serde procedural derives; their own
tri macro also means question-mark is not their first observed blocker.

Use an explicitly selected host toolchain for build-time tools when bootstrap
requires it, recording that dependency separately. All target Rust library and
crate code must go through RustGM. A rustc-built target sysroot, opaque rustc
metadata, prebuilt target dependency or frontend fallback cannot satisfy these
milestones. A fully RustGM-hosted build-tool chain belongs to self-hosting.

## Backend and configuration gates

Keep CPPGM as the explicit LowIR-to-native boundary. Establish each required
operation with an independent backend probe; existing scalar volatile/U32 atomic
evidence does not prove every width, SIMD operation or assembly constraint.
The inspected memchr/zmij sources include target-specific paths. Maintain honest
target cfg values and implement the selected paths before claiming their crates.
Do not set miri, falsify baseline CPU features, patch out code, or silently select
an older dependency to evade an unmet contract.

When a required operation cannot be represented by the frozen backend, record
the exact source, typed operation and failed capability probe. A separate reviewed
backend extension may be necessary. No change to CPPGM, its optimizer or its
code generator is authorized merely by this roadmap. The milestone stays
incomplete until the selected dependency graph works with an established backend
contract. A documented upstream-supported alternate configuration can be a
separate milestone; it does not replace the original declared target.

## Tests, performance and resource limits

Follow CPPGM's division between focused compiler-contract tests and real-library
workloads. Small cases have exact text oracles in the earliest owning assignment,
declared before generation and independently checked with pinned rustc. Full
library/crate tests retain deterministic source/configuration/build summaries,
artifact identity checks and native output oracles; they do not substitute an
empty success file or require RustGM machine code to equal rustc machine code.
Test repeated clean builds, dependency-order changes, rejected metadata and
missing hooks. Freeze each new runner's text format before activating its stage.

Inspect original upstream modules directly; preserve attribution and checksums.
Extracted reducers accompany, rather than replace, complete upstream acceptance.
One successful parse, one monomorphization, one cfg configuration or a header-only
test does not prove an entire crate or all its optional features.

Parse source once, retain typed facts, cache completed proof/instance work and
import shared graphs with explicit indexed remapping. Measure dependency count,
generic instances, shared projections, array extents and unrelated source size
separately. Use immutable A/B binaries, A/A calibration, paired ABBA observations,
RSS/instruction measurements and output verification; qualify changed output
contracts independently. Full library compile time and peak memory become
release gates, with budgets declared from a measured baseline rather than guessed.

Keep the existing three-job controller, two Cargo/test threads per job, 8-CPU and
12-GiB per-job limits, 150-GiB global reserve and 50-GiB worker allowance. Benchmarks
remain exclusive. Share immutable upstream sources instead of copying build trees
per worker. Use guarded scoped cleanup; preserve inputs, diagnostics and receipts,
and retire superseded binaries/caches. Full suites run at coherent publication
checkpoints; focused source/native and scaling checks drive development.

## Number migration

Only unreleased assignments are reorganized. Old PA31 maps to PA39; PA32/PA33 to
PA42/PA43; PA34/PA35 to PA37/PA38; PA36 to PA40; PA37 to PA41; PA38 to PA44.
The former optional PA39/PA40/PA41 become required PA32/PA33/PA34–PA36. The optional
self-hosting idea moves from PA42 to PA45. Historical checkpoint records and the
vendored CPPGM LowIR specification retain their original numbers; current
handouts, assignments.json and ROADMAP.md use the new ownership.
