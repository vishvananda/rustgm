# Lifetime-parameterized structs and enums

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/lifetime-nominals-20261107`, based on main `2a13ab4a9`. It is a
transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment.

It is the first task of the core panic path after the panic-attributes
checkpoint. Core 1.98's `Arguments<'a>`, `Location<'a>` and `PanicInfo<'a>`
are structs with lifetimes. Before this slice, every provider that declared
one stopped at "source lifetime binding has no actual original slot".
Version 13 now carries a struct's or enum's lifetimes, and fields, variant
payloads, signatures and constructor calls may use them. A fresh consumer runs
such a provider natively.

## Wire rule (version 13)

There is no new column or tag. Existing rows take a new shape, and the rule
is in spec.md's version-13 nominal lifetime paragraph.

- **Slots.** Each lifetime of a struct or enum is a region slot (kind tag 1)
  of the nominal. Its region slots come before its type slots. A region
  slot's index is its lifetime's declaration position. A type slot's index
  counts only the nominal's type slots, so a lifetime-free nominal keeps its
  exact slot bytes.
- **Fields.** A field or variant payload names a lifetime as region tag 2,
  `Parameter { slot, depth: 0 }`, in the nominal's own header frame.
- **Arguments.** The nominal's Self recipe and every application of it carry
  the region arguments first (argument tag 1), in slot order.
- **Constructors.** A tuple variant's constructor binds its enum's lifetimes
  late. A called constructor's application carries the enum's full
  arguments, regions first.

No variance, implied outlives requirement or well-formedness claim travels.

## Producer, validation and consumer

- **Export.** `scoped_parameters::nominal_region_slots` records the region
  slots before `type_slots` under `body_nodes()` only. Older stages keep
  main's path and its "source lifetime binding has no actual original slot".
  `canonical_nodes::nominal_frame` marks a nominal header frame, and
  `regional_roots` writes its bound regions as slot parameters. A declared
  lifetime bound (`'b: 'a`) is rejected with "source nominal lifetime bounds
  require their wider profile", because no row carries it and a consumer
  would lose it.
- **Validation.** `slot_domains::OwnSlots` admits leading nominal region
  slots at their exact indices. `record_contracts` rejects a nominal region
  slot under older profiles by name and checks the Self recipe's region
  arguments. `node_rows` admits `Parameter { depth: 0 }` only for a nominal's
  own slot, checks each nominal argument's kind against its formal, and
  admits a constructor application's leading region arguments. The port plan
  (`rewritten_nodes`, `rewritten_roots`, `construction_values`,
  `signature_matches`, `applied_uses`, `applied_debts`) rewrites and compares
  nominal region arguments. A provider signature that applies another
  artifact's nominal with region arguments stays rejected.
- **Consumer.** `slot_bindings::nominal_region` maps a nominal's region slot
  `i` to the template lifetime `bound(0, i)`, as a local nominal's field
  types name it. Only type slots get parameter identities. The constructor
  template binds the enum's lifetimes late. A lifetime-generic import sets
  `has_nominal_requirements`, so bodies prepare storage as they do for a
  local generic nominal. Variance and the implied `T: 'a` come from the
  consumer's own `lifetime_variances` and nominal outlives passes over the
  imported fields.

One more change is in lowering transported originals. Version 13 now
requests a formation fact for a reborrow of a reference to a type parameter
(`value_rows::parameter_reborrow`). Without it, the provider's own validation
rejected `get`'s `x.r` reborrow with "source operation omitted a required
nontrivial formation input". The consumer proves the fact again.

## Forgeries

`forged_lifetime_nominal_rows_reject_in_fresh_consumers` uses a generated
minimal provider: lines 1-126, `Zinv<'a> { slot: &'a mut &'a u8 }` with
`store`, which writes through the slot, and `Zarg<'a> { text: &'a str }` with
`take`.

| Mutation | Fresh consumer |
| --- | --- |
| `Zinv`'s region row given depth 1 | "source nominal Self changed its exact formal identity or argument order" |
| `Zinv`'s region slot index set to 1 | "source nominal region slot changed its leading declared index" |
| `Zarg`'s region row renamed to `Zinv`'s slot | "source nominal Self changed its exact formal identity or argument order" |
| `Zarg`'s field type made to name `Zinv`'s region | "source consuming type root captures another parameter owner" |
| `take`'s region argument relabelled as a type argument | "source nominal argument changed its formal kind" |
| `Zinv`'s outer field reference made shared, `store` client | "source projected field type differs after actual nominal substitution" |
| relabelled version 12 | "unknown source declaration kind" |
| every strict prefix | fails to decode |

Region rows are interned. A nominal's Self recipe names the same row as its
fields, so the two row forgeries are caught by the Self identity check. The
field-retarget control, declared after that observation and before it ran,
reaches the field's own frame check.

## Evidence

The fixtures are in `pa29/tests/spec/lifetime-nominals/`. Their outcomes,
declared before the oracle and the candidate ran, are in
`pa29/tests/lifetime-nominal-outcomes.json`. The integration tests are in
`crate_linking::lifetime_transport`.

- **Oracle.** The first rustc 1.98.0 round rejected `fixed` ("struct literals
  are not allowed here"). It was parenthesized before any candidate run, and
  no outcome changed. In the second round the provider compiles and all nine
  accepted clients exit 42. inv-shrink, args-grow and text-grow fail with
  "lifetime may not live long enough", escape with E0515 and outlives with
  E0309.
- **Native.** All nine accepted clients exit 42 natively, as rustc's do:
  args, shrink, first, field, fixed, invariant, reference, text and word.
- **Opponents.** All five are rejected with "required lifetime does not
  outlive its target". This is the consumer's own region check over the
  variances and outlives requirements it derived itself. Covariant shrinking
  (shrink) is accepted, so the variance is not just invariant.
- **Older profiles.** `Gate<'a>` on the source-nominals provider is rejected
  under versions 12 and 10 with main's own diagnostic, byte for byte the
  baseline's, and exports under version 13. `Bounded<'a, 'b: 'a>` and a
  method in `impl<'a> Held<'a>` are rejected by name under version 13. The
  full provider stops at the shared inherent-methods gate.
- **Scaling.** Packets are 33,662, 54,056 and 135,632 bytes for 6, 24 and 96
  lifetime structs, each with one function. That is 1,133 bytes per struct
  and function, exactly linear. The client LowIR is 856 bytes at every width.
- **Byte identity.** The baseline is main `2a13ab4a9` (pin sha `8351d7df`).
  The candidate is sha `477f4c3a`. They were compared over every pa29 spec
  provider under versions 10, 12 and 13, each with every fresh consumer in
  its directory. That covers 2,940 metadata, LowIR, status and diagnostic
  files, including all 19 earlier version-13 packets. The 2,895 files outside
  the new packet are identical. The 34 that differ are all the new packet's
  version-13 run, which the baseline rejects with "source lifetime binding
  has no actual original slot".

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `lifetime_transport` integration (4) and neighbouring transport tests (borrowed, branch, cold, constructor, direct calls, fn roles, guard, imported records and regions, inherent, instantiable, match, node, normalized results, records, source nominals, spelled roles, statics, str-never) | 61/61 | `1794088534144126476` |
| native `--ignored` (this packet, enum match, constructors, source nominals, inherent methods) | 5/5 | `1794088288309720074` |
| private selection (32): the hot-record and derive-owner layout invariants, nominal record forgeries, scope slots, the input plan and anonymous-function recipes, collected formals, consumer domain demand and selected templates | 32/32 | `1794088685647607306` |
| PA1-PA12 | pass | `1794088911157003807` |
| PA13-PA19, PA21-PA24 | pass | `1794089040789877108` |
| PA20 | 1,451/1,451 | `1794089110999128627` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/lifetime-nominals/test-upstream.log` |
| `make test-harness` | 464 roots in 13 binaries, 44 stages | `obj/parallel/lifetime-nominals/test-harness.log` |

All rows ran on the final code. The export `Frame` record gains one flag
(16 to 20 bytes). It is export scratch, charged through `size_of`, and no
layout invariant covers it. The port-plan `Node` keeps its size, because
`NominalRegion` is no larger than `Reference`.

The rustc 1.98.0 receipts are
`obj/parallel/lifetime-nominals/oracle/round1.txt` and `round2.txt`. The
identity receipt is `obj/parallel/lifetime-nominals/identity/receipt.txt`.
The pinned binaries are `obj/parallel/lifetime-nominals/{baseline,candidate}/`.

## Amended controls

Each amendment is recorded in the outcomes file with its observation.

- **`fixed`.** Parenthesized after oracle round 1, before any candidate run.
- **`Gate` under older profiles.** The declaration named "source
  const/lifetime generic argument is not a type slot". The older stages stop
  earlier, at main's unchanged "source lifetime binding has no actual original
  slot".
- **The wire provider's `store`.** It binds `mut z`. Written with an
  immutable `z`, the candidate's provider export rejected it ("source Store
  writes through an immutable dereference"). The port plan requires the
  by-value binding to be mutable to write through its `&mut` field, and rustc
  does not. This is a gap.
- **Three wire forgeries.** The depth and foreign-slot row forgeries are
  caught by the Self identity check (interned rows). The shared-field forgery
  is caught by the field substitution check before the Store check. All three
  are rejected by name. One new control, the field retarget, was added before
  it ran.

## Edits outside crate transport, for owner review

- `lower/transported_source/cfg_rows/value_rows.rs`: `parameter_reborrow`
  adds a formation request under version 13 only.

Everything else is in `sema/crates/` and `support/crates/`.

## Remaining gaps toward core's real `panic`, `assert_failed` and `Option::map`

In dependency order:

1. **Methods of `impl<'a> X<'a>`.** Core's `Arguments::new_const`,
   `Location::caller`, `Location::file` and `PanicInfo::new` are such
   methods. Export rejects them with "source function early/captured callable
   frame transport is not implemented" (`authored_functions.rs`). The late
   slots and ordinary and conversion profiles reject captured regions, and
   the consumer's `header_port` blocks captures.
2. **Declared lifetime bounds** on nominals (`'b: 'a`, `T: 'a`). No row
   carries them. An explicit `T: 'a` clause exports but the consumer rejects
   it ("source assumption is outside the ordinary actual trait profile").
3. **Region arguments of another provider's nominal** in a signature (the
   port plan's `Regions::Rejected`). Transitive core users need these.
4. **Tuple structs** ("source declaration kind is outside the initial source
   profile"). Core's `NonNull<T>` and `PhantomData`-carrying newtypes are
   tuple or unit structs.
5. **Fields of function-pointer type with a region** ("source anonymous
   callable requires its supported closed Rust frame").
6. **Writes through a `&mut` field of an immutable by-value binding.** The
   transport port plan rejects them, though rustc and local checking accept
   them.
7. Then, as before: `Location` (`NonNull<str>`, `PhantomData`), the caller
   ABI, `panic_fmt`'s body (`extern "Rust"` `panic_impl`, `PanicInfo::new`,
   `&[&str; 1]` to `&[&str]` unsizing), trait impls with methods, and
   `Option::map`'s closure transport.
