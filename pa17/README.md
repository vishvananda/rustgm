# RustGM Programming Assignment 17: Destruction

Status: **implemented-draft**. This is a bounded course language profile.
Prerequisite: PA16. Profile: Rust 1.98.0, edition 2024, Linux x86-64, panic abort.

## Required implementation

Extend checked Rust-to-LowIR lowering with implicit destruction. An initialized
value is destroyed when its drop scope ends or before an assignment overwrites
it. Moves transfer ownership without destroying the source. A partial move
leaves cleanup only for initialized remaining fields. A value that was never
initialized must not be destroyed. Check every typed body, including unused
functions, before allowing execution.

Recognize the language `Drop` trait by its resolved identity, including prelude,
qualified, imported, reexported and macro-generated paths. A user trait merely
named `Drop` is a different trait. Support implementations for concrete local
structs and enums, including lifetime parameters, aliases and explicit lifetime
bounds already supported by PA15. Require exactly the applicable `drop` method,
a mutable reference receiver, unit result and the Rust calling convention.
Validate receiver lifetimes, implied bounds, duplicate implementations and the
incompatibility of `Copy` with custom destruction. An implementation cannot
specialize lifetimes or require stronger bounds than its target type.
Direct calls to or extraction of the destructor method are rejected. General
trait/inherent methods and type/const-generic instances remain PA18–PA20 work.

A custom destructor runs before automatic destruction of its fields. Struct
fields and tuple fields are destroyed in declaration order; enum cleanup uses
only the active variant's fields. Array elements are destroyed in increasing
index order, including zero-sized elements. A reference does not own or destroy
its referent. A zero-length array owns no elements, but an evaluated nonconstant
repeat operand still has its own cleanup obligation. An enclosing custom
`Drop` implementation forbids moving non-Copy fields out of that object;
whole-object moves, Copy reads, borrows and valid field replacement remain valid.

## Scopes, evaluation and loans

Follow the pinned Reference's [destruction rules](../doc/rust-reference/src/destructors.md).
Block locals drop in reverse declaration order, including declarations initialized
later. Bindings within a pattern use their declaration order; or-patterns take
that order from the first written alternative. Parameter bindings and remaining
parameter fields keep their specified ordering. Nested scopes exit from inside
out on normal completion, return, break and continue. Values transferred by a
return or break remain owned by their destination.

Retain temporary creation scopes and edition-2024 lifetime extension through
borrow, aggregate, constructor and block expressions. An ordinary call does not
acquire constructor-style lifetime extension. Do not promote values that need
destruction to permanent storage. Expression statements, unused temporaries,
partially consumed aggregates, match scrutinees, let-else failures, condition
chains and failed or retried guards must all use their actual cleanup scopes.
Temporary lifetime extension and cleanup ordering must agree: extending backing
storage while destroying its value early is incorrect.

Assignment evaluates its incoming value, destroys the initialized old destination
and then stores the replacement. Preserve projection evaluation and one-time
side effects. Reinitialization, conditional initialization and partial moves
require path-sensitive cleanup; loops must refresh the appropriate state.
Real successor edges can carry cleanup. Imaginary pattern proof edges must not
execute cleanup or become runtime branches.

Run initialization, storage, region and loan checking against the ordered cleanup
plan. Destructor uses can keep a loan live beyond the last ordinary read. Keep
ordinary use requirements distinct from destructor requirements. Whole moves
and defining replacement stores end the old owning value's obligation; partial
field moves preserve the containing value's lifetime requirements. Permit access
after a completed replacement only when no other live holder retains the old
loan. Reject early referent access, references to expired storage, incompatible
reborrows and invalid destructor field moves. An empty destructor body still
imposes its declared lifetime requirements.

## LowIR contract

`dev/rustgm --emit-lowir SOURCE` emits the existing typed LowIR dialect only
after the complete destruction proof succeeds. `--count-lowir` performs the same
proof and lowering and reports the ordinary LowIR counts. Both accept `-o OUTPUT`.
Language rejection returns 1, emits a diagnostic and leaves stdout empty.
Signals, timeouts, panics and exit 86 are implementation/infrastructure failures.
No invocation of rustc or CPPGM implements a Rust frontend phase.

Consume typed statement, cleanup and flag events; never reparse rendered proof
text. Shared destructor glue is emitted on demand as `rg.drop.N`, with one
pointer parameter and no result. `N` identifies the selected recipe in deterministic
construction order. Custom destructor bodies use the ordinary `rg.N` function
identity and receiver ABI. Block, slot, temporary, signature and global numbering
follow the existing deterministic LowIR writer. The checked program retains the
selected plans; an unproved candidate list cannot authorize lowering.

Flags start false and change at the selected initialization/consumption events.
Guard a whole contiguous array interval outside its counted element loop when
one condition covers the interval. Tag checks guard variant-specific field
cleanup. Reuse layout facts and field identities. Give zero-sized receivers
stable aligned backing. Implicit cleanup of a packed field copies it into aligned
storage before calling its glue; an ordinary unaligned source borrow stays invalid.

Earlier type and body inspection modes may expose pending destructor facts.
The PA14–PA16 ownership, region and loan inspection modes retain their explicit
pending-destruction boundary; they do not inspect the completed PA17 execution
plan. PA17's checked LowIR path provides that combined proof. Panic remains abort:
there is no unwinding cleanup, landing pad or exception ABI until PA31. Standard
library destruction helpers, custom smart pointers, closures, unsized owned
values and async cleanup retain their later assignment owners.

## Efficiency and validation

Parse once and consume retained typed facts. Use compact IDs, flat arenas,
sparse move paths/flags and iterative worklists. Share nominal recipes across
lifetime instantiations; do not confuse a layout representative with lifetime
arguments needed by the loan proof. Never enumerate array lengths during
analysis or emit one instruction sequence per element. Preserve zero-sized
runtime call counts even though their byte stride is zero.

The reference caps recipe query work at 1,000,000 steps, selected planning and
cleanup emission at 32,000,000 work units, and selected plan/recipe storage at
64 MB under their respective accounting. The ordered timeline caps at 4,000,000
points; PA16's storage/loan limits still apply. Release per-body proof scratch
promptly. Bodies with no runtime cleanup retain no execution event tape.

The suite declares 199 cases: 132 accepted programs and 67 rejections. One
accepted program declares runtime panic; the other 131 require clean execution.
Two rejections are documented later-stage boundaries accepted by rustc. All
other acceptance outcomes agree. Required families cover identity, headers,
field moves, destructor loans, replacement, temporaries, counters and ordering.
The 33 `order/` sources define a no-op `trace(u8)` observer. Their course oracles
remain uninstrumented LowIR. The explicit maintainer native suite independently
checks actual callback order: rustc uses a printing observer, while the completed
LowIR program receives a test-only observer checking each callback and the final
count. Cleanup generation is never replaced or supplied by that observer. The
22 accepted counter programs execute unchanged source through both compilers.

Run `make test-pa17`, `make test-rustc`, `make test-native-pa17`,
`make test-maintainer-native`, and root lint/audit/harness checks. Only explicit
`make ref-test-pa17` regenerates the text, diagnostic and status sidecars. Review
those outputs together with independent acceptance, execution and performance
evidence before accepting a reference update.
