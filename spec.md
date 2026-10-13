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

Built-in semantic traits retain their own definition identities; a user item
with the same spelling cannot acquire language behavior. Resolve aliases and
qualified paths through ordinary typed namespace facts. Demand external metadata
only when used, keep it separate from the crate root, and retain configured
prelude policy per module so lookup does not rescan attributes.

Resolve a destructor's impl target and receiver once. Retain the nominal item,
method identity, whole-reference receiver binding and captured lifetime slots.
Anonymous impl lifetimes are real slots, not fabricated declaration IDs. A
destructor must cover every instantiation of its nominal type: target lifetimes
are distinct universals; unused impl lifetimes are existential during coverage
checking. Prove declared requirements against nominal well-formedness, including
field-implied bounds, using the shared directed outlives graph. A receiver has an
independent late lifetime; reference mutability and binding mutability are
separate facts. A typed destructor is not an ordinary callable function value
and is not evidence that cleanup has been proved or emitted.
Index declared lifetime membership separately from first-use slot assignment;
capturing a wide impl binder must not scan all its declarations per parameter.

Analyze generic bodies without reparsing syntax. A monomorphization key must
include all type/const arguments and semantic context affecting its result.
Trait solver caches include parameter environment, binder universe and mode;
negative results need equally complete keys. Each demanded fact is computed
once; cycle detection and explicit solver budgets are part of the contract.

Record selected methods, adjustments, casts, layouts and obligations where
later phases consume them. Lowering cannot repeat lookup or inference. Missing
facts are invariant failures, never a reason to fall back to text or rustc.

Constant patterns constrain types during ordinary body inference. Retain pending
value/range/coverage work in flat arenas with the originating file, module and
body; finish it after initializer typing in one shared query session. Reuse
resolved dependencies, selected operations and already computed discriminants.
Cache demanded constant-definition values across these phases in the current
monomorphic profile; leave ordinary node facts compact and allocate no query
storage for bodies without constant-backed patterns. Do not instantiate a
whole-crate checker per use, repeat initializer typing, or interpret pattern
views. Deferred checks must retain the matching module's visibility context.

Demanded tuple/array constant values use flat shared arenas. Repeats retain one
element and their length; field/index queries select from that representation
without materializing the repeated array. Translate constants to the existing
selected product/scalar patterns with use-site provenance. Share repeated
product subtrees within a use; retain each projection's distinct place identity
for decisions and execution. Check type-level PartialEq obligations even for
empty products, then validate actual scalar leaves (including NaN rejection).
Reserve expansion work before allocating siblings so pending work is bounded.
Neither value storage nor pattern translation grows dense ordinary node facts.

Nominal constant products retain selected constructor and field identities from
typing; updates fill only missing fields from the shared base value. Validate
constructor callees and update bases even for zero-sized values or completely
overridden fields. Equality obligations are a separate cached type query from
actual structural-value eligibility. Check every enum field for the former and
only the active variant's values for the latter. Derived reference cycles use
an iterative greatest fixed point with reverse dependency edges: publish all
positive and negative results only after propagation completes. Count actual
nodes, dependencies and propagation work under an explicit query budget.

Selected string and byte-string literals retain decoded bytes in one flat,
interned byte arena, addressed by sparse syntax-node facts. Constant evaluation
and body construction reuse those identities without decoding again.
Equality compares exact bytes; identifier Unicode
normalization must never affect text values. Share repeated text subpatterns
within a constant use while preserving each selected place. Byte-string patterns
translate to ordinary fixed-array or exact-slice decisions under the existing
preallocation work budget. String coverage partitions only the current query's
observed values plus one class for every other string. Do not scan unrelated
bodies or enumerate a string domain. Matching a value constant compares type
structure without equating its storage lifetime to the input lifetime; ordinary
expression constraints remain explicit. These pattern facts do not discharge
the region and loan proofs required for reference execution.

Demanded shared constant references use flat place records containing an optional value,
canonical type, access provenance and packed alignment. Separate place queries
from value reads so borrowing a non-Copy static or selecting its Copy field
does not move its container. Check Copy reads before consulting a definition's
cached value. Field/index queries consume selected identities and sparse
autodereference facts; they neither repeat lookup nor materialize repeated
arrays. Preserve provenance across projections and use the actual referent's
alignment after dereference. Borrowing requires alignment, while an owned field
read need not form a reference. Reference chains use bounded worklists for
evaluation, pattern translation and destruction. Keep ordinary node facts
compact; reference operations still require region/loan proofs before LowIR.
Intern static allocation identities before evaluating their initializers. Forming
an address or fixed projection uses typed storage facts; it must not introduce
a dependency on the stored value. Actual reads demand and cache location
contents through the bounded query worklist, after checking access and Copy
requirements. Complete reachable static backing with a growing identity queue
before lowering. Address cycles are valid storage graphs; initializer value
cycles remain errors. Constant patterns demand finite referent expansion and
diagnose cycles independently of executable pointer graphs.
Coverage derives root validity from the selected input place and retains it
through deferred binding checks. Explicit and automatic dereferences cannot
reuse owned-value inhabitation to prune variants with potentially invalid fields.

Floating pattern domains use constant-time bit/rank conversions. Numeric ranks
collapse signed zero, while a separate unordered class keeps NaN visible to
exhaustiveness. Reuse scalar interval indexes and composite coverage decisions;
do not enumerate floating values or add per-value domain storage. Execution
compares the original typed value and captures its original bits. Even a range
covering every numeric rank must test its input because it excludes NaN.

Dynamic-slice coverage retains exact/minimum lengths and typed start/end/rest
projections. Lazily collect tested positions from the current query's selected
roots, keyed by canonical slice type. Specialize only lengths at which a tested
start and end position can alias; keep the remaining length intervals shared.
Canonicalize aliases to one place identity, including nested patterns. Wildcards
and unconstrained captures add no alias points. Charge collection, position pairs,
specialization and graph work to the same bounded query; sort the completed alias
set once rather than using ordered node containers for hot-path membership.

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

Compute destruction requirements from canonical types with a bounded iterative
query, memoizing shared shapes and errors. References do not own their referents;
zero-length arrays do not require element destruction. Array length must not
expand query work. Programs without custom destructors need no pending-drop
metadata in the current closed type profile.

Pending body facts retain destructor identities, type requirements and candidate
store/storage exits. Reuse persistent live-stack ranges and existing exit IDs;
do not copy every local at every exit. These candidates can overlap and are not
an execution plan: initialization, moves, owned rvalue scopes, ordering and
destructor-induced liveness still need proof. Complete ownership, region and
loan proofs and executable lowering reject pending elaboration. Raw body views
may expose it explicitly. A value needing destruction cannot be promoted to
permanent storage, even while its cleanup plan remains pending.

Retain destructible rvalue creation scopes while building the body, including
discarded values that never become borrowed places. Forwarding a block result
forwards its temporary scope without inventing a second object. Pattern captures
use their retained binding scope. Mark returned values as transferred before
local cleanup, preserving order relative to storage exits at the same point.

Initialization has a distinct may-be-initialized plane; immutable-binding write
history cannot substitute for it after moves. Share the existing transfer and
fixed-point implementation, with additional state retained only on demand.
Expose bounded point queries over retained entry states for cleanup planning.
Preserve sparse array element/gap partitions from move-path numbering when that
analysis requests them; never reconstruct these mappings from output or expand
unselected array elements. A whole-path mixed state can include partial moves
and does not authorize dropping the entire object behind one flag.

Internal match-guard views retain selected place identity and delay owning
captures until success. Protect tested places without runtime loads or address
formation for inactive variants. Track guard protection liveness across control
flow; a diverging path does not imply a later guard use. Internal view proofs
are distinct from the pending loans and regions of user reference values.
Build guarded pattern retries with shared first-match and resume continuations,
not an expanded Cartesian product. Preserve written alternative/field order and
end attempt-local storage on failure. Static path offsets may be cached as typed
recipes, but addresses must be formed where they dominate their actual uses.

After region and loan proofs, sized references lower to thin pointers and their
lifetime binders have no runtime representation. Collect address-observable
places sparsely. Materialize borrowed scalar temporaries once at their defining
evaluation, preserve object snapshots, and use persistent backing for promoted
roots without deleting their original effects or runtime checks. Borrowed
zero-sized values require aligned, nonnull backing; their addressable fields
retain canonical offsets even though value loads and parameters remain erased.
Reference pattern paths share persistent dereference prefixes alongside ordinary
offset recipes. Load each pointer in the current use block, restoring referent
alignment after dereference. Representation-preserving lifetime conversions may
reuse values; thin-to-fat coercions must construct metadata explicitly.

The course x86-64 representation for references and raw pointers to slices or
`str` is a 16-byte, eight-aligned object: a data pointer at offset zero and an
unsigned 64-bit length at offset eight. This is an explicit course ABI, not a
claim of a stable Rust ABI. Slice length counts elements, including zero-sized
elements; string length counts UTF-8 bytes. Unsized referents themselves still
have no fixed storage layout. Use the existing object calling convention for
wide values and retain metadata sparsely beside unsized place projections.
Array-to-slice conversions use the checked source length and compatible element
layout. Dynamic indexing checks that length before address calculation, even
when the selected value is zero-sized. Reborrows retain the source metadata.

Literal backing is immutable and shared by canonical bytes. Empty strings and
byte strings receive nonnull backing with zero logical length. Persistent wide
values serialize both the address and length at their canonical offsets;
packed fields use the existing byte serializer and pointer startup fixups.
Metadata and literal identities must not enlarge ordinary scalar place facts.

Dynamic slice patterns test exact or minimum length before selecting any element.
Suffix projections retain the tested minimum length as a typed fact; rest
projections retain excluded prefix/suffix counts. Loan overlap uses those facts
to distinguish disjoint pattern captures. An arbitrary runtime index does not
receive the same disjointness proof. Normalize nested rest projections without
enumerating elements. Pattern metadata reads are distinct from element reads
and moves, and retain initialization, storage and liveness obligations.

Ordinary and guarded decisions consume the same selected constructor conditions.
Guard retry recipes rebuild both addresses and lengths in their actual use
blocks; cached recipes must not cache branch-local runtime values. String tests
compare exact interned bytes after checking length. Emit a bounded-size comparison
loop instead of one instruction sequence per literal byte, and never load bytes
on a failed length test. Constant text patterns obey the same execution path.

Borrow checking runs before references and moves lose their source semantics
in LowIR. Region/dataflow analysis uses typed worklists and bounded lattices,
with per-body storage and complete dependency edges. Pattern usefulness and
trait solving must expose work counters and explicit complexity limits.
PA15's region inspection exposes the retained directed contract graph separately
from PA16's checked loan inspection. Render deterministic IDs, direct edges and
sparse point/loan words; never use inspection text as proof input. Optional loan
snapshots observe the final checked transfer pass, after pruning and call
activation. Executable verification must not allocate those snapshots or retain
the per-body inspection arenas. Counts describe the same facts as the full view.

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

Persistent data consumes completed constant values through a read-only typed
interface. Reserve global identity before following address dependencies;
statics retain their own storage and projections retain their original root.
Serialize canonical field offsets and exact scalar bits with bounded flat work.
Memoize zero-filled products and keep repeated zero data compressed. By-value
constant aggregates still produce independent runtime snapshots. When the
external backend cannot relocate an unaligned pointer field, retain its packed
offset and install the address before Rust entry code; never silently realign it.

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
