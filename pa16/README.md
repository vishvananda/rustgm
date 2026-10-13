# RustGM Programming Assignment 16: Borrow checking

Status: **implemented-draft**. This is a bounded course language profile.
Prerequisite: PA15. Profile: Rust 1.98.0, edition 2024, Linux x86-64.

## Required implementation

Combine the PA15 lifetime contracts with definite storage availability, PA14
moves/initialization, non-lexical use liveness and loan conflicts. Preserve
source place identity before erasing references into LowIR. Check every typed
body, including functions that executable reachability later omits. Imaginary
pattern successors remain proof edges.

Infer reference-use regions over CFG points. Last use can end a loan before its
lexical scope ends; a reference that survives a branch or loop must remain
valid on every relevant path. Reborrow relationships retain their directed
ancestry. Accessing a mutable borrow's referent differs from overwriting the
pointer variable that contains it. Check temporary, local and promoted storage
ends, including edge-specific ends and diverging control flow.

Shared loans permit shared reads and exclude conflicting writes. Mutable loans
exclude conflicting reads/writes while permitting access through their own
checked reborrow chain. Check shallow pointer/tag/metadata access separately
from deep referent access. Moves, writes, storage ends and guarded captures
participate in the same proof. Disjoint fields and selected pattern slices may
be borrowed separately; arbitrary runtime indices remain conservative. Empty
and zero-sized selections still require valid storage and permissions.

Implicit mutable call-argument reborrows can reserve a loan before evaluating
later arguments and activate it at the consuming call. Explicit `&mut` borrows
are immediately exclusive. A reservation permits intervening shared reads but
not writes or a conflicting live loan at activation. Do not extend a temporary
reborrow through a block tail merely because it appeared as a call argument.

Match decisions protect tested places and read-only guard bindings, retry
written alternatives in order, and delay owning captures until success. Slice
patterns test length before element selection. Their typed minimum lengths and
excluded prefix/suffix counts justify disjoint captures, including nested rest
slices. String patterns compare exact bytes after a length check. These checks
must remain valid through guard retries and imaginary proof paths.

## Reference execution

Only the combined checked program can cross into LowIR. Sized references use
thin pointers. Slice and `str` references use the course's 16-byte, eight-aligned
pointer/length representation; length counts elements or UTF-8 bytes. This is a
course ABI, not a stable Rust ABI claim. Dynamic indexing checks bounds before
address calculation. Metadata and reference coercions preserve their typed
proofs, including zero-sized elements and empty slices.

Borrowed values need stable storage, including aligned nonnull backing for
zero-sized values. Preserve snapshots when arguments, returns or assignments
copy aggregates containing references. Persistent constants and readonly statics
retain typed data identity and relocations. Static addresses do not read static
initializer values. Packed fields retain their declared offsets; taking an
unaligned reference is rejected even in an unused function. Eligible promotions
share backing while preserving source effects and checks.

PA17 owns user destruction and destructor liveness. General trait methods,
unsized user types, const functions/mutable constant memory, unsafe accesses and
unwinding remain their later assignments. Successful borrow checking cannot
silently authorize those capabilities.

## Text contract

`dev/rustgm --emit-loans SOURCE` emits `loans-crate 1` only after contracts,
storage, ownership, loan conflicts and alignment pass. `--count-loans` runs the
same inspection and emits its count line. Both accept `-o OUTPUT`. Language
failure returns 1 with empty stdout and a diagnostic; infrastructure failure
never becomes a language rejection.

Bodies follow typed-program order, beginning `body N source=FILE:NODE`. Bodies
with no borrow operations have no loan records. Other bodies contain:

- `  place pN root=lN|vN|static:N`, optionally ` parent=pN`, then a projection
  and ` shared=true|false`. Projection spellings are `dereference mutable=BOOL`,
  `tuple-field=N`, `nominal-field=N`, `index=N`, `range=START..END`,
  `from-end=N minimum=N`, `subslice prefix=N suffix=N` and `dynamic=vN`.
- `  loan qN borrow=vN place=pN region=gN kind=shared|mutable|two-phase`.
- `  point tN block=bN at=sN|end reachable=true|false`.
- `  live-region gN word=N bits=HHHHHHHHHHHHHHHH`.
- `  state tN word=N live=HHHHHHHHHHHHHHHH activated=HHHHHHHHHHHHHHHH`.

Places and loans retain deterministic construction order. Place IDs belong to
the canonical overlap index, not the typed-body projection arena. Local/value
roots refer to BodyIR identities; `shared` records a shared dereference in the
path. Guard-view aliases may expand one borrow value into several loans. The
region IDs refer to that body's PA15 graph.

Points enumerate each block's statements and terminator position in block order,
including unreachable blocks. `sN` is the global BodyIR statement index. A live
region word contains 64 point bits, least significant first; a loan-state word
contains 64 loan bits. Words are lowercase, zero-padded hexadecimal. Omitted
words are zero. Live-region records sort by region then word; state records
sort by point then word and are present only for reached, nonempty states.

State is recorded after pruning by use liveness and before the point's access.
At calls it includes two-phase activation; a newly generated loan first appears
at a following point if still live. `activated` marks activated two-phase loans;
immediately mutable loans are exclusive by their `kind` even with a zero
activation bit. Joins retain may-live/may-activated loans, not an execution trace.
External lifetimes cover all CFG points; the contract-only local sentinel is
never a runtime seed. Do not interpret a live-region bit alone as a generated
loan: forward loan reachability is separate.

Each body ends `end-body`. The final line is
`eof bodies=N loans=N places=N points=N region_words=N state_words=N`.
Count mode omits `eof `. `--emit-lowir` is the supplemental execution mode;
inspection sources are never counted as native runs.

## Efficiency and validation

Use sparse canonical place recipes, compact IDs, iterative fixed points and
bounded bitsets. Retain only observed references and projection relationships;
never enumerate an array merely to analyze a pattern range. Loan block state
caps at 4,000,000 words (64 MB) and charges 32,000,000 work units per body.
Runtime regions cap retained sparse words/pending deltas at 500,000. Optional
inspection snapshots cap at 2,000,000 nonzero words (48 MB). Executable
verification runs the same proof without allocating inspection traces.

The release suite declares 292 cases, including 112 LowIR executions. It covers
last-use and conflicting access, reborrows/two-phase activation, disjoint fields
and patterns, loops, guards, temporary extension, promotion, thin/wide reference
execution and persistent backing. Outcomes are declared before oracle generation
and checked independently with pinned rustc. Native cases also validate and run
through the pinned CPPGM backend, checking clean success or declared panic.

Run `make test-pa16`, `make test-rustc`, `make test-native-pa16`,
`make test-maintainer-native`, and root lint/audit/harness checks. Only explicit
`make ref-test-pa16` regenerates sidecars. Handout, manifests, independent checks
and reviewed text oracles must be coherent before activation.
