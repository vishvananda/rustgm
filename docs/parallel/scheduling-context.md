# Scheduling-context fence

Lane C's item after 5b-1, 2026-11-09, developed on origin/main `1aa288ad1`.
A read-only diagnosis, its notice, a pre-addendum and an addendum came before
any code. The integrator approved the design with amendments A1-A4 and
rulings F1, F2 and N1 (option γ).

## Before

A source client, built with `--source-client` and no `--source-core`,
rejected rustc-accepted programs with "implementation validation has no
source scheduling context". It did so whenever declaration preparation
const-evaluated a body whose Copy decision met an unvalidated imported impl:

1. A client signature such as `fn take(_: &[u8; COUNT])` demands
   `COUNT = base::four()` while declarations are prepared, before the eager
   `validate_trait_implementations` pass.
2. The constant executor prepares the reached body. `copy_decisions_waiting`
   decides the body's Copy goals with executable availability.
   `append_candidates` puts the unvalidated imported Copy impl in
   `unvalidated`, so the selection waits on `ImplementationValidated`.
3. The wait becomes `Work::ValidateImpl(owner, Begin)`. `advance_impl_validation`
   had only the source core's early scalar scheduler, which is never built
   without a source core, so `source_implementation_context` failed.

Only the eager walk, after declaration preparation, proved imported impls.
The fence was an ordering gap, not a missing proof. rustc runs the upstream
crate's MIR and trusts upstream impls. RustGM's counterpart of that trust is
the walk's imported conformance, which can run earlier.

Core declares Clone and Copy for every primitive, so the fence blocked alloc
and std clients that const-evaluate anything imported from core.

## Change

- **One per-owner function.** `imported_conformance` (`validation.rs`) proves
  one source-free header. The eager walk calls it, and so does a client's
  executable demand. In selected mode it calls the selected dependency's
  `require_imported_implementation`, as the walk did. Otherwise it skips an
  owner the registry already holds as validated. That skips only the owner's
  conformance: the walk still checks every overlap pair before any owner (A3).
- **The demand arm.** `advance_impl_validation` (`scalar_readiness.rs`) calls
  `demanded_conformance` when the package has no source core and is not in
  selected mode. Selected mode has no constant executor and completes imported
  conformance only through its selected dependency, so it keeps the base path
  and text (ruling N1, option γ).
- **Source headers.** A client header with a source node keeps a named
  fence: "client implementation validation before the eager walk requires a
  source scheduling context" (A4).
- **Specialization gate (F2).** `require_unordered_import`
  (`candidate_order.rs`) stops an imported owner with "imported implementation
  validation before its specialization conformance order" when the client's
  specialization graph relates it. `Graph::relates` (`registry_graph.rs`)
  matches a source vertex, an attempt's entries or a relation's child or
  parent while the graph is building, and a forest vertex once it is
  conforming or sealed. No real input reaches the gate:
  - `compare` authenticates both entries before inserting an attempt;
  - witness capture fails for any definition without a source node;
  - `settle` runs only after `local()`;
  - `drain` re-authenticates every attempt.

  A private test forges each endpoint.

## Traces

The probes ran on the base pin `23030f23` (built from `1aa288ad1`) and on
rustc 1.98.0 before the fixtures were declared. They are kept under
`obj/parallel/auto-traits/constants/probe-tr`.

| Trace | Result |
| --- | --- |
| (i) bucket width | An imported body's usize goal waits on a Copy impl for u8 or bool alone, but not for a unit struct. A local body's identical goal waits only on usize's |
| (ii) local exemption | Local bodies are not exempt: a local place copy reaches the fence (H3). A literal body `{ 4 }` has no Copy goal |
| `min_specialization` | alloc's and std's shape reaches the same fence |
| selected mode | Selected clients cannot const-evaluate at all ("expression requires the general constant evaluation assignment") |
| overlap | The base pin stops at the fence before the walk's pair check |

The efficiency item is recorded with its hypothesis, which is untraced. An
imported body's goal reaches `candidates_for` without a closed shape and reads
the coarse `heads` bucket (`implementations.rs:237-242`). A local body's
resolved goal reads the exact `closed` bucket (227-235). The change stands as
it is because every impl that demand pays for is one the eager walk validates
anyway.

## Results

`pa29/tests/scheduling-context-outcomes.json` froze the outcomes before any
run of its fixtures. One amendment is recorded there, before the run it
governs.

| Evidence | Result |
| --- | --- |
| fixture cases | 35 cases over 8 providers and 12 clients; the rustc oracle matches every declaration |
| flips | 14 clients rejected at the fence on the base pin are accepted, and each exits 42 natively with empty stdout |
| unchanged | 17 accepted clients stay accepted |
| overlap (A3) | Rejected before and after, in a different order. The base pin reports the fence; the candidate reports the overlap text, the same diagnostic line as `overlap-twin` |
| client-local Copy (A4) | `local-copy` reports the source-backed text, a known rejects-valid |
| `generic-usize` | A failed prediction, rejected by an existing, independent cleanup fence (amendment 1) |
| scaling | 14 demanded owners for `four` (D=1) and for `eight` (D=8), and 1 for `local-placed`, each owner once |
| gate (F2) | A forged forest vertex, relation and comparison each stop demand by name; the unforged `min_specialization` client relates no imported impl |
| named-constants `const-call` | Accepted, and exits 42 natively (its amendment 9) |
| flip survey | 1,401 runs over 39 packets against the base pin: only named-constants `const-call` flips, and all 48 exported packets are byte-identical |

- **Amendment 1.** The first focused run rejected `generic-usize` with
  "defining cleanup requires its actual language declaration" at the
  provider's `id` body. That fence is existing and independent of this item.
  On the base pin, `four` over a provider with only `const fn id<T: Copy>`,
  and `body` over a provider with usize's impls and `id`, are rejected with
  the same text. Defining cleanup of the Copy-bounded generic body requires
  the Destruct language item. The branch provider's 57 lines do not declare
  it, and core does. The blanket provider had mixed two controls, so it was
  split: `blanket-provider.rs` drops `id`, so `overlap` reaches the pair
  check, and the new `generic-provider.rs` holds usize's impls and `id`.
  `generic-usize` and `local-copy` moved to it. No other case changed.
- **`u8clone` is excluded.** Its provider is invalid: `impl Clone for u8 {
  fn clone(&self) -> Self { *self } }` moves out of a reference without a
  Copy impl, so rustc and RustGM both reject the provider itself.
- **Focused evidence.** Fastcheck passed eight exact tests:
  - the `scheduling_context`, `named_constants` and `reified_items`
    integration controls;
  - the gate and scaling invariants;
  - the PA31 generic Copy wait, the source core's early primitive constants
    and a selected-mode reload, all unchanged.

  Receipt: `obj/qualification/fast/1794219797616483762`. The ignored native
  tests for this packet and for named constants passed in
  `obj/qualification/fast/1794219860834649831`. The upstream fixture
  verifier, the maintainer suite registry and the assignment layout check
  pass.

## Recorded

- **Selected-mode completion.** Completing imported conformance through
  `selected.validate_implementation` on demand becomes reachable only if
  selected mode gains a constant executor.
- **Known rejects-valid (A4).** A client-local Copy impl reached through an
  imported generic const fn during declaration preparation keeps the
  source-backed text. Its follow-up is an early scheduler for client source
  headers outside a source core.
- **Cleanup fence.** A client const evaluation of an imported Copy-bounded
  generic const fn stops at "defining cleanup requires its actual language
  declaration" under a provider without the Destruct language item. Core
  declares it.
- **The efficiency item** above, with its hypothesis.
- **Next.** Lane C's next item is R2, with `value_rows.rs`'s `emit()`
  factored first.
