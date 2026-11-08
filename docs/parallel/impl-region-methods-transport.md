# Methods of inherent impls that capture lifetimes

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/impl-region-methods-20261107`, based on main `0311a54ea`. It is a
transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment.

It is the first task after the lifetime-nominal checkpoint. Core 1.98's
`Location<'a>`, `Arguments<'a>` and `PanicInfo<'a>` have methods in
`impl<'a> X<'a>`: `Location::caller`, `Arguments::from_str` and
`PanicInfo::new`. Before this slice, such a method stopped at "source function
early/captured callable frame transport is not implemented". Version 13 now
carries methods of an inherent impl that captures lifetimes. Provider bodies
and fresh consumers call them, and each consumer re-proves the declared
captured and late results.

## Wire rule (version 13)

There is no new tag, column or row shape. Existing rows take new values, and
the rule is in spec.md's version-13 captured-method paragraph.

- **Impl frame.** The impl's header frame binds its lifetimes as its region
  slots (`region_captures`, the DeclarationHeader binder count). Its Self
  recipe names them as `Bound` regions of that frame. Impl region slots have
  no definition, and an impl type slot's ordinal and index count only the
  impl's type slots.
- **Method frames.** Each method declares `regions.captures` equal to its
  impl's `region_captures`. Its original and public binders both have
  `total` slots, the captured slots `0..captures` first and its own late
  slots after them.
- **Receiver.** The receiver base is exactly the impl's Self, each impl
  frame region replaced by the same captured slot of the method's original
  binder.
- **Calls.** A Call opening binds only the method's own late slots (public
  count minus captures). The function application's leading region
  arguments supply the captures in slot order. An inherent call names no
  method request.
- **Coerced reborrows.** A Borrow whose referent differs from its place's
  type is admitted only where the original records a plain one-dereference
  reborrow coercion from that reference to a reference of exactly that
  referent. Without this, `Info::new(held, flag) -> Self { Info { held, flag
  } }` and the free `wrap` failed provider validation with "source Borrow
  changed its actual referent or writable place". That failure is on main
  too, for free functions.

No region relation, outlives requirement or result proof travels. No new v13
tag or shape was allocated.

## Producer, validation and consumer

- **Export.** `authored_functions::function` admits captures under
  `body_nodes()` when the function's parent is an inherent impl, and
  `callable_binder` opens `count - captures` slots. `scoped_parameters`
  records an inherent impl's type slots with type-only ordinals and indices
  (`Leading::Inherent`). `self_requests` writes no method request for an
  inherent target. Older stages keep main's paths and diagnostics.
- **Validation.** `frame_captures::method` admits the method family under
  `body_nodes()`. It checks:
  - the impl is inherent and unmodified, with no region bounds;
  - the method is ordinary (not const), has the impl's captures, no early
    slots and no region bounds;
  - both scopes' region counts;
  - the receiver against the impl's Self.

  A method that claims captures under an impl that declares none is a
  forged frame. `late_slots` admits late slots after the captures.
  `request_ports` opens only the own late slots and admits an inherent
  target with no request. `node_rows` and `applied_debts` accept the leading
  region arguments. `applied_uses` relocates only own late slots and maps a
  captured public slot to the application's region argument.
  `value_requirements` and `recorded_inputs` admit the coerced reborrow
  described above.
- **Consumer.** `formal_headers`, `scheme_ports` and the readiness check
  accept the inherent captures. `template_ports::registration` rebuilds the
  method's template from its original signature and region scheme:
  `callable_mapping` maps the captures to leading early parameters and the
  own late slots to the public frame, as a local method template does.
  `ordinary_recordings::registered_callable` checks that registration again
  when the recording is replayed. The consumer's own region check proves
  each body against its declared result.

## Forgeries

`forged_captured_method_rows_reject_in_fresh_consumers` uses a generated
minimal provider: the lifetime-nominal provider, `Hold<'a> { t: &'a u8 }` and
`impl<'a> Hold<'a> { pub fn choose<'b>(&self, a: &'a u8, b: &'b u8) -> &'b u8
{ b } }`. The variant `-> &'a u8 { a }` supplies the claimed rows.

| Mutation | Fresh consumer |
| --- | --- |
| original and public results replaced by the variant's | "required lifetime does not outlive its target" |
| only the public result replaced | "source signature reference mutability or exact lifetime differs" |
| region scheme claims no captures | "source captured method changed its impl's exact lifetime frame" |
| region scheme claims two captures | same |
| impl header claims no captured lifetime | same |
| first own late slot claims the captured index 0 | "source late opening has foreign, repeated or nonregional formals" |
| relabelled version 12 | "unknown source declaration kind" |
| every strict prefix | fails to decode |

The first row is the semantic control. The body returns `b`, but the claimed
result is `&'a u8`, and the consumer's own proof rejects it.

The three capture-count rows were declared with later diagnostics: "source
public binder omitted actual late original region formals", "source
early/captured region schemes require a complete wider profile" and "source
header binder has foreign captures or duplicate ownership". The method-family
check runs first and rejects each by name. The impl-captures row first
reached the generic "source provided associated bodies require a wider
explicit profile". The family check now names a capture disagreement as a
forged frame, and that change was observed again. These amendments follow
integrator decision 3 and are recorded in the outcomes file.

The old `abcfea354` late-region probes, replayed by
`obj/parallel/impl-regions/tools/forge.sh`, stay rejected. The baseline and
the candidate give byte-identical outputs
(`obj/parallel/impl-regions/older-forge{,-baseline}`).

## Evidence

The fixtures are in `pa29/tests/spec/impl-region-methods/`. Their outcomes
were declared at 2026-11-07T23:15:02Z, before the oracle and the candidate
ran. They are in `pa29/tests/impl-region-method-outcomes.json`. The
integration tests are in `crate_linking::lifetime_methods`.

- **Oracle.** In the single rustc 1.98.0 round, the provider compiles and all
  thirteen accepted clients exit 42. text-escape and cell-shrink fail with
  "lifetime may not live long enough", info-local with E0515 and pick-borrow
  with E0597. All as declared.
- **Candidate.** All seventeen outcomes are as declared. The three lifetime
  opponents fail with "required lifetime does not outlive its target", and
  pick-borrow with "access conflicts with a live reference loan".
- **Native.** All thirteen accepted clients exit 42 natively, as rustc's
  do.
- **Older profiles.** `Held<'a>` with `get` on the source-nominals provider
  is rejected under versions 12 and 10 with main's "source lifetime binding
  has no actual original slot". It exports under version 13, and a client
  that binds `get`'s result as `&'static str` is accepted. The lifetime-
  nominal packet's gate control `impl<'a> Held<'a>` now exports, so its gate
  became the trait-impl form with the same diagnostic (amended in
  `lifetime-nominal-outcomes.json`).
- **Fences.** A trait impl that captures a lifetime, a method whose
  `where 'a: 'b` makes `'b` early, and a const inherent method stay rejected
  by name under version 13.
- **Scaling.** Packets are 45,102, 66,414 and 151,662 bytes for 6, 24 and 96
  methods in one `impl<'a> Wide<'a>`. That is 1,184 bytes per method, exactly
  linear. The client LowIR is 1,053 bytes at every width.
- **Byte identity.** The baseline is main `0311a54ea` (pin sha `0f5f95ee`).
  The candidate is sha `bbadb875`. They were compared over every pa29 spec
  provider under versions 10, 12 and 13, each with every fresh consumer in
  its directory. That covers 3,185 metadata, LowIR, status and diagnostic
  files, including all 21 earlier version-13 packets. The 3,131 files outside
  the new packet's version-13 run are identical. The 45 that differ are all
  in that run: the baseline rejects the new provider with "source
  const/lifetime generic argument is not a type slot", and the candidate
  exports it. Under versions 10 and 12 both binaries reject it with the same
  bytes.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `lifetime_methods` integration (4), `lifetime_transport` (4) and the neighbouring transport tests (borrowed, branch, cold, constructor, direct calls, fn roles, guard, imported records and regions, inherent, instantiable, match, node, normalized results, records, source nominals, spelled roles, statics, str-never) | 65/65 | `1794094618032091595` |
| native `--ignored`: this packet, lifetime nominals, inherent methods, source nominals, constructors, enum match, str-never, normalized results | 8/8 | `1794095126186397894` |
| native `--ignored`: body nodes, cold, guards, branches, fn roles, spelled roles, direct calls, instantiable | 8/8 | `1794095174029089480` |
| private selection (39): the hot-record and derive-owner layout invariants, the authored-function effect storage, conditional and required-member tests, header reservations and origins, nominal record forgeries, scope slots, the input plan, anonymous-function recipes, collected formals, consumer domain demand and selected templates | 39/39 | `1794094823478994252` |
| PA1-PA12 | pass | `1794095392663765011` |
| PA13-PA19, PA21-PA24 | pass | `1794095470884246546` |
| PA20 | 1,451/1,451 | `1794095571657773357` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/impl-regions/test-upstream.log` |
| `make test-harness` | 304 script tests; 464 roots in 13 binaries, 44 stages | `obj/parallel/impl-regions/test-harness.log` |

All rows ran on the final code. `Coverage` (port-plan scratch) gains one set
of reborrow keys, charged through `retained_bytes`; no layout invariant
covers it. No hot record changes size.

The rustc 1.98.0 receipt is `obj/parallel/impl-regions/oracle1.log`, and the
first candidate round is `candidate1.log` beside it. The identity receipt is
`obj/parallel/impl-regions/identity/receipt.txt`. The pinned binaries are
`obj/parallel/impl-regions/{baseline,candidate}/`.

## Remaining gaps

- Const inherent methods of capturing impls: core's
  `Arguments::from_str` is `pub const fn` in `impl<'a> Arguments<'a>`.
- Method region bounds and early method lifetimes (`where 'a: 'b`).
- Trait impls that capture lifetimes.
- Writes through a `&mut` field of an immutable by-value binding (gap 6).
  The `Cell::set` fixture takes `mut self` to stay clear of it. The
  [unique-write checkpoint](unique-writes-transport.md) fixes it.
