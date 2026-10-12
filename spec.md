# Compile-time-first Rust frontend architecture

This is the normative architecture of the maintainer reference frontend.
It adapts the principles of cppgm-extended's `spec.md`, pinned in
[docs/cppgm-spec.md](docs/cppgm-spec.md). Preserve its attribution in
[shared/CPPGM-NOTICE](shared/CPPGM-NOTICE). The C++ implementation is a guide to
efficient representations and algorithms, not an implementation of Rust rules.

## Scope and pipeline

Target Rust 1.98.0, first edition 2024, on Linux x86-64. Freeze language release,
Unicode tables, edition, target, panic mode and overflow-check mode in each
relevant test manifest. Older editions arrive explicitly at PA36. Library,
Cargo and rustc-private binary/metadata compatibility are separate objectives.

```text
source image -> Rust lexer -> token-tree/parser/expansion/resolution worklist
             -> syntax with canonical semantic facts
             -> typed Rust body CFG -> initialization/ownership/region checks
             -> drops, demanded instances, closure/async transformations
             -> typed LowIR -> explicit text backend adapter -> CPPGM native
```

The external LowIR serialization is an authorized temporary backend boundary.
No Rust frontend phase may communicate by printing and reparsing token, syntax,
type or diagnostic text. After an in-process backend becomes available, pass
typed LowIR directly. All backend-visible facts must have a LowIR spelling.

## Source, lexing and parsing

Read source in bulk; retain immutable source images and compact file/offset
identities. An ordinary LF source requires no normalization copy. A CRLF file
may keep one normalized view beside its original for physical diagnostics.
Do not build per-character records. Source line/column lookup is cold and lazy.
Identifiers use pinned Unicode XID and NFC rules when interned; raw spelling
and semantic identity are different facts.

The lexer yields typed tokens with borrowed source ranges on demand. Ordinary
syntax consumes a bounded cursor. Retained token trees are allowed for macro
definitions, invocations, procedural-macro interfaces and source provenance;
they are not an excuse for successive owning token streams of the whole crate.
Raw strings and nested comments must scan linearly in input size, with bounded
delimiter lookahead. No token spelling allocation on the ordinary path.

Parse each source or newly produced expansion fragment once, apart from bounded
grammar checkpoints and macro matcher alternatives. Macro fragment matchers
call the ordinary Rust parser. Retain captured opaque fragments and hygiene
identities; never stringify them to recover syntax. Memoize repeated matching
states and diagnose resource limits rather than retrying exponentially.

Macro expansion and resolution form a worklist with explicit unresolved,
in-progress, complete and failed states. Do not repeatedly scan the whole crate
after each new item. Spans carry compact source and expansion context IDs.

## Semantic identity and demand

Intern names, types, declarations, trait goals, instances and layouts with
compact typed IDs. Canonical equality is O(1). Dense per-node facts belong on
nodes or dense side arrays; sparse composite query keys may use hash tables.
Rendered names/types are never semantic keys. Deterministic output must not
force ordered node-based containers onto the lookup hot path.

Analyze generic bodies without reparsing syntax. A monomorphization key must
include all type/const arguments and semantic context affecting its result.
Trait solver caches include parameter environment, binder universe and mode;
negative results need equally complete keys. Each demanded fact is computed
once; cycle detection and explicit solver budgets are part of the contract.

Record selected methods, adjustments, casts, layouts and obligations where
later phases consume them. Lowering cannot repeat lookup or inference. Missing
facts are invariant failures, never a reason to fall back to text or rustc.

## Rust body representation

A typed control-flow IR retains places, projections, moves, borrows, storage
lifetimes, initialization and abstract drop obligations. It is an executable
body representation, not a second copy of the complete syntax tree. Do not call
it MIR without qualification: CPPGM already uses MIR for machine IR.

Share live storage tails persistently. An exit records the suffix it ends,
without copying the surrounding live-local list or repeating every storage-end
statement at every failure branch. Preserve reverse lifetime order and skip
already-closed nested scopes. Storage compaction is distinct from ownership
proof complexity: charge the paths/words actually visited and retain explicit
dataflow memory and work budgets.

Internal match-guard views retain selected place identity and delay owning
captures until success. Protect tested places without runtime loads or address
formation for inactive variants. Track guard protection liveness across control
flow; a diverging path does not imply a later guard use. Internal view proofs
are distinct from the pending loans and regions of user reference values.

Borrow checking runs before references and moves lose their source semantics
in LowIR. Region/dataflow analysis uses typed worklists and bounded lattices,
with per-body storage and complete dependency edges. Pattern usefulness and
trait solving must expose work counters and explicit complexity limits.
Destruction consumes initialization/move facts to elaborate conditional drops.
Async transformation accounts for borrows, pinning and cancellation cleanup.

## Lowering and backend facts

Lower only checked, typed facts. Give every emitted instance, drop shim, vtable,
closure and coroutine a stable emission identity and emit it once. Rust signed
overflow, saturating casts, zero-sized types, fat pointers and panic behavior
must not inherit C++ assumptions. In particular, do not blindly emit C++
`nowrap`, `typed` or alias facts. State only what Rust analysis proves.

Use the shared LowIR model contract; serialized text is an output view. The
temporary backend adapter accepts LowIR only, validates it with CPPGM and then
builds native code. No oracle object or reference function may replace a failed
Rust compilation. General assembly and target intrinsics require an explicit
capability review before PA37; the initial backend is not assumed complete.

## Ownership, allocation and complexity

Use owned arenas/slabs/vectors and compact IDs instead of Rc/Arc pointer graphs,
per-node boxes or recursively owned trees on hot paths. Prefer slice/range
children and reuse per-query/per-body scratch capacity. Bulk destruction must
follow phase ownership. Avoid an allocation per token, edge or solver step.

Each function owns mutable body/LowIR records. Editing one function must not
rebuild another. Release per-function scratch promptly and discard frontend
ownership before whole-program LowIR optimization. A future O0 in-process
backend should consume and release functions incrementally.

Lexing should be O(source bytes + produced tokens); parsing near-linear except
explicit language-required matching. Resolution visits relevant scopes and
candidates, not unrelated declarations. Lowering should be linear in emitted
IR; fixed points must make monotonic progress in a bounded lattice/worklist.
Any broader bound needs a documented semantic reason and adversarial tests.

Telemetry observes work already being done: bytes/tokens, retained token bytes,
allocations or arena high water, solver goals, retries, region edges, demanded
instances and phase time. Disabled instrumentation must not change semantics.
Track throughput and peak RSS at several input scales. Compare frozen binaries
with identical inputs, A/A calibration and paired ABBA blocks; retain all raw
samples and verify output equality. Wall-time noise alone is not a regression.

## Safe Rust and audits

Forbid authored unsafe compiler code. The input language still includes unsafe
Rust; fixtures are not compiler implementation. Keep OS/process/environment
access at explicit driver/build boundaries, outside `dev/src/` environment
reads. Audit dependencies and feature sets before use.

Run rustfmt and Clippy all/pedantic with warnings denied. The file audit counts
code lines: 3000 per implementation module, 2400 for registered interfaces,
1800 for internal models; functions 240, or 360 for cohesive dispatch with at
least eight arms. Nesting warns past six and fails past eight. Do not hide logic
in macros, compressed lines, generated files or mechanically split modules.
