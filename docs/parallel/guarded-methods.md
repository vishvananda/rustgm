# Unsafe associated functions

Lane C's associated-unsafe item, 2026-11-09, on
`implement/guarded-methods-20261109` from `a6be80084`. The integrator approved
the notice (`obj/parallel/associated-unsafe/notice.md`, sha256 `83df7582`),
its source footprint and the movers M1, M2 with M2a, M3 and M4 by name. The
item lets a version-13 module-body export carry `unsafe` inherent methods,
trait members and impl members, and the unsafe calls inside or to them, with
no wire change. It is the fmt chain's next blocker after HR/C's
prerequisites: core's `Argument::fmt`, `NonNull::add` and `as_ref`,
`unwrap_unchecked` and `SliceIndex::get_unchecked` are unsafe associated
functions.

## Before

- `ordinary_profile::functions` admitted `unsafe` only on a free function
  (`free_effects`). An associated function's declared or public `unsafe`
  answered "source nonordinary callable effect/safety/convention requires a
  wider profile" at export self-validation, in every profile.
- `request_ports::call_safety` answered "source unsafe associated Calls
  require their wider actual profile" for any unsafe direct or pointer call
  inside or to an associated function, in every profile.
- F withdrew its unsafe inherent `Lane::get` (amendment 2), and R2 and (ii)
  recorded their associated opponents as known rejects-valid.

## Rule (no wire change)

- **Gate A, `ordinary_profile::functions`.** Under version 13 an associated
  function's original and public `unsafe` no longer need a wider profile.
  Its constness, convention and variadic terms keep `free_effects`, so an
  unsafe `extern "C"` method keeps the nonordinary text, and a trait default
  body keeps "source provided associated bodies require a wider explicit
  profile".
- **Agreement.** Under version 13 an associated function's original and
  public safety must agree: "source associated callable changed its actual
  original/public safety". A free function keeps its own check.
- **Member safety, `member_safety`.** Under version 13, for each
  implementation member linked to a trait member, the two public safeties
  must be equal, as rustc's E0053 requires: "source implementation member
  changed its trait member's safety". The ledger already requires trait
  members to be local.
- **Gate B, `request_ports::call_safety`.** The associated-call text stays
  below version 13. At version 13 direct and pointer calls inside or to an
  associated function follow direct Call's matrix: an unsafe callee needs
  an unsafe block or an unsafe caller, and a safe callee claims no
  obligation. Each consumer re-proves the matrix at validation and again over
  the rebuilt body.
- Versions 12 and 10 keep both texts, and so does an older reader over a
  version-13 packet. The source footprint is `ordinary_profile.rs` and
  `request_ports.rs` only, with no body or dispatch edit.

## Fixture and evidence

[`pa29/tests/spec/guarded-methods`](../../pa29/tests/spec/guarded-methods/README.md)
holds the callable-fields provider plus authored lines and 17 clients. Every
outcome was declared at 2026-11-09T18:11:59Z, before the rustc oracle, the
base-pin baseline and any candidate run, and frozen before the oracle
(`pa29/tests/guarded-method-outcomes.json`).

| Control | Result |
| --- | --- |
| 16 fresh clients | accepted; each exits 42 natively, as rustc's do |
| `missing` | rejected: "call to unsafe function requires an unsafe context"; rustc: E0133 |
| `safe-impl`, `unsafe-impl` | rejected by RustGM's frontend ("incompatible types"), no packet; rustc: E0053 |
| `extern-c-method`, `trait-default` | the nonordinary and provided-body texts at export; rustc accepts both |
| movers M1 to M4 | each exports under version 13; the M2a, M3 and M4 clients exit 42 natively and M1's is accepted |
| older controls, versions 12 and 10 | the trait member's nonordinary text, the record struct's conformance-record text and the associated-call text; version 13 exports all three |
| old reader, base pin `5f9aa57d` | the fixture's, M1's, M2's and the older trait and inherent packets: the nonordinary text; M3's, M4's and the older call's: the associated-call text |
| baseline, base pin | the frozen provider rejected with the nonordinary text |
| 10 forged rows | rejected by name: agreement twice, caller twice, callable, lexical twice, claim, member twice |
| versions 10 and 12 over the older-call twin's forged rows | the nonordinary text twice and the associated-call text; version 13 gives agreement and member |
| N = 8, 16, 32 | packet bytes equal to the safe twins'; step excess within the declared linear bound |
| self-demand probe, k = 1, 2, 4 | retained bytes, observed bytes and visits equal the twins'; step excess within the bound |
| flip survey against the base pin, 1,929 runs | among existing inputs only M1 to M4 flip, under version 13; 78 compared packets are byte-identical |

- **Clients.** `Lane::get` by method syntax and by path, `Lane::make` with
  no receiver, a provider safe fn and a provider safe method calling unsafe
  functions in unsafe blocks, `Lane::twice` calling `get` bare under
  `#[allow(unsafe_op_in_unsafe_fn)]`, RawWakerVTable's `Table::invoke`, a
  generic impl at `u8`, a late region, a lifetime-capturing impl, a const
  unsafe method in a client const, the unsafe trait member `Get::fetch` on
  the concrete type, through a bound and implemented by the client, a
  reified `Lane::scale`, and `Argument::fmt`'s shape.
- **Opponents.** A trait's unsafe method implemented safely, and the
  reverse, are rejected by rustc (E0053) and by RustGM's own frontend
  ("incompatible types"), with no packet. An unsafe `extern "C"` method and
  an unsafe trait default body are known rejects-valid.
- **Movers.** M1: the const-methods `pub const unsafe fn get` control
  exports and its const client is accepted. M2 and M2a: F's
  `export_controls.inherent` (`Lane::get` with `enable = "sse"`) exports
  with bit 3 and `pa41`'s `inherent.rs` returns as its client. M3: R2's
  `H::call` exports and `base::H::call(base::raw)` exits 42. M4: (ii)'s
  `Table::go` exports and `t.go(p)` exits 42. Each is recorded beside the
  declaration it amends: const-methods' superseded control, F's amendment
  6, pointer-call amendment 3 and callable-field amendment 3.

## Recorded

- **Fixture amendment 1.** rustc 1.98.0 ICEs (`error performing operation:
  fully_perform`, status 101) on M1's declared client at `match N { 42 =>
  42, _ => 7 }`
  (`obj/parallel/associated-unsafe/oracle/movers/m1-const/client.stderr`).
  It is the class of callable-field amendment 1: a match over a provider that
  declares `Copy` but implements it for no primitive, here the
  source-nominals provider. The integrator approved the amendment. The
  client now returns `N as i32`, its prediction unchanged, and the one
  bounded retry exits 42 with no warnings. The client loses its in-client
  match, but the exit code still pins `N == 42`.
- **Harness corrections.** The first fastcheck stopped at strict Clippy's
  doc-markdown on an amended comment. The second failed the forged-row test
  before any row ran: the harness looked up a method by its nominal, but a
  member's header parent is its impl's declaration. The harness now resolves
  the inherent impl by its self type and `Key::fetch` through its member
  link. No declared outcome changed.
- **Notice departures, declared.** The scaling and probe twins are called in
  the same unsafe block, so only flag bytes differ; rustc would warn
  `unused_unsafe` there, but the twins run only in RustGM's in-process tests,
  whose frontend has no such lint. The older-profile controls use the
  source-nominals provider: versions 10 and 12 stop a unit struct at "source
  declaration kind is outside the initial source profile", so the trait
  controls implement for `u8` and the inherent control uses a record struct,
  which stops at its conformance record.
- **Limits.** Trait default bodies, dyn calls, `extern "C"` and variadic
  associated functions and safe target-feature methods keep their fences.
  Primitive-impl unsafe methods such as `u8::unchecked_add` follow from gate
  A but are untested and not claimed. `SliceIndex` (`slice/index.rs:149`) is
  a const unsafe trait and also needs const trait impls. Clearing both unsafe
  bits on a method whose body has no unsafe operation and that no provider
  body calls is undetectable, as for a free unsafe fn.
