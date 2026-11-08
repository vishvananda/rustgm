# Tuple and unit struct constructors

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/struct-constructors-20261108`, based on local main `4a9a3441f`
(rebased from `0987b5f00` onto Lane B's variant re-export batch `8b18cbcfc`
and then onto its architecture-audit fix). It is a
transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment.

It is the first task of the second Lane C batch. Core 1.98 builds tuple and
unit structs by constructor: `Wrapping(x)`, `Saturating(x)`, `NonZero(x)`,
`PhantomData`, `Alignment(..)` and many private helpers. Before this slice,
every tuple or unit struct stopped export at "source declaration kind is
outside the initial source profile", because its constructor definition had
no wire form. Version 13 now carries such constructors. Provider bodies and
fresh consumers call them, read and match them, and every consumer re-derives
their shape, payload, name and visibility.

## Wire rule (version 13)

There is no new tag, column or row shape. The integrator decided that the
constructor reuses the existing variant declaration (kind 11). The rule is in
spec.md's version-13 struct-constructor paragraph.

- **Declaration.** A tuple or unit struct's constructor is a kind-11
  declaration parented by its struct, carrying the struct's name, with the
  constructor's own `public` flag. No enum column describes it, so a packet
  without enums still has no enum columns.
- **Binding.** The constructor is bound in the struct's module value
  namespace, directly after the struct's type binding (bindings sort by
  module, name and namespace). Its visibility is computed as rustc computes
  it: the narrowest of the struct's and its fields' visibilities.
- **Fields.** A tuple struct's fields are named by their ordinals in
  canonical decimal (`0`, `1`, ...). A record field cannot spell such a name,
  so the field rows distinguish the two forms. A unit struct has no fields.
- **Uses.** Calls, paths, patterns and unit values use the existing function
  application, aggregate and constructor-test rows, as for a tuple variant. A
  unit constructor travels as a struct aggregate, as before.

Nothing else travels. Under the older profiles, export keeps main's paths:
the constructor is still "source declaration kind is outside the initial
source profile", and a positional field name is still "source nominal tuple
fields require their wider profile" at export and "invalid canonical metadata
path component" at decode.

No new v13 tag or shape was allocated. In memory, the validated header index
gains `HeaderRef::Constructor(nominal)`. It names the constructor's struct
header and has no wire form.

## Producer, validation and consumer

- **Export.** `exported_kind` publishes a value-namespace
  `DefKind::Constructor` as a variant declaration under `body_nodes()`, and
  the existing declaration loop parents it by its field group's struct.
  `record_headers::struct_constructor` requires that struct to have exactly
  that one field group. It fences `struct T();` (the empty tuple form, which
  the unit form's rows would misdescribe) and `#[non_exhaustive]` forms (no
  row carries the crate-wide narrowing) by name. `nominal_fields` names tuple
  fields by their ordinals under `body_nodes()`. `transported_source` leaves
  every struct constructor to the consumer, as it does enum constructors.
- **Validation.** `sum_contracts::headers` binds each struct-parented
  variant declaration as `HeaderRef::Constructor` after the enum tables. It
  checks:
  - the parent is a struct with its nominal header;
  - every field of the struct is positional, so a record struct has no
    constructor;
  - the struct has no second constructor;
  - the name is the struct's.

  A struct with positional fields and no constructor is rejected.
  `record_contracts` admits positional names only in version 13.
  `Constructor::of` and the new `ConstructorPayload` give the body-port
  checks one payload for both variant and struct constructors.
  `implementation_domains` binds a struct-parented variant in the value
  namespace of its struct's module. This extends Lane B's version-13
  variant-binding dispatch, which re-derives a binding's namespace from the
  constructor's installed parent: an enum parent binds a re-exported variant,
  and a struct parent, which Lane B rejected by name, now binds its
  constructor. Any other parent is "variant module binding parent is not an
  enum or struct". `constructor_visibilities` requires each
  direct constructor binding to follow its struct's direct type binding and
  to have exactly the narrowest of that binding's and the fields'
  visibilities.
- **Consumer.** `mapped_records` installs a struct with a constructor as one
  external variant (unit or tuple) with its positional fields, and
  `mapped_members` accepts positional fields. `definition_cells` types a
  unit constructor as the struct and a tuple constructor as a function item
  over the struct's formals and regions. `template_ports` registers each
  tuple constructor's callable through the same `Constructor::register` as
  tuple variants. Constructor bodies are synthesized from the
  re-authenticated struct header.

The consumer's own resolver then gives `base::Wrap(..)`, `use base::Wrap as
W`, braced `Wrap { 0: x }` literals and patterns, constants and `Self(..)` in
provider methods their local meaning, privacy and borrow checks.

## Forgeries

`forged_struct_constructor_rows_reject_in_fresh_consumers` uses a generated
minimal provider: `spec/source-nominals/provider.rs` plus `Tup(pub u8)`,
`Two(pub u8)`, `Rec { pub x: u8 }`, `enum Sum { One(u8) }` and
`m::n::Sec(pub(super) u8)`. A client destructuring `base::Tup(42)` is
accepted on the honest packet.

| Mutation | Fresh consumer |
| --- | --- |
| `Tup`'s constructor re-parented to the record struct `Rec` | "source record struct has no constructor" |
| `Two`'s constructor re-parented to `Tup` | "source lexical child has a foreign kind, gap or duplicate ordinal" |
| the same, with the next lexical ordinal 1 | "source struct has a second constructor" |
| the enum variant `Sum::One` re-parented to `Tup` | "source lexical child has a foreign kind, gap or duplicate ordinal" |
| the same, with the next lexical ordinal 1 | "source variant changed its actual enum owner, ordinal or fields" |
| `Tup`'s constructor renamed `Tuq` | "source struct constructor changed its struct's name" |
| `Sec`'s constructor binding widened from `m` to the crate root | "source struct constructor visibility differs from its struct and fields" |
| `Sec`'s constructor made public and its binding widened to public | same |
| `Rec`'s field `x` renamed `0` | "source tuple struct fields require its constructor" |
| relabelled version 12 | "unknown source declaration kind" |
| every strict prefix | fails to decode |

The two re-parenting rows were declared with the constructor and variant
diagnostics. The moved declaration keeps its lexical ordinal 0, which
duplicates `Tup`'s own constructor, so the declaration graph rejects it
first. Each row now has a second form with the next ordinal, which reaches
the declared diagnostic. These amendments follow integrator decision 3 and
are recorded in the outcomes file. The public-widening row (a splice that
drops the binding's four-byte module) was added after the first candidate
run of the fixtures and declared before its own run.

A forged public flag alone is already rejected by main's direct-binding check
("direct module binding changed its actual declaration/owner/access"). A
forged constructor on an empty braced struct `struct R {}` would be admitted
as a unit constructor, because the wire cannot tell the two empty forms apart.
No control covers this; by construction it grants nothing `R {}` does not,
because its visibility is still the struct's.

## Evidence

The fixtures are in `pa29/tests/spec/struct-constructors/`. Their outcomes
were declared at 2026-11-08T00:56:25Z, before the oracle and the candidate
ran. They are in `pa29/tests/struct-constructor-outcomes.json`. The
integration tests are in `crate_linking::struct_constructors`.

- **Oracle.** In the single rustc 1.98.0 round, the provider compiles and all
  eleven accepted clients exit 42. private-constructor, private-pattern and
  crate-field fail with E0603, private-field with E0616, private-braced with
  E0451, wrong-arity with E0061, wrong-payload with E0308, unit-call with
  E0618 and held-escape with E0597. All as declared.
- **Candidate.** All twenty outcomes are as declared. The narrowed
  constructors fail with "private value name", the private fields with
  "aggregate field is private" and held-escape with "access conflicts with a
  live reference loan".
- **Native.** All eleven accepted clients exit 42 natively, as rustc's do.
- **Older profiles.** The full provider is rejected under versions 12 and 10
  with "named, associated or generic alias transport requires its wider
  profile". On `spec/source-nominals/provider.rs`, `Wrap(pub u8)` and `Unit`
  are rejected under both with main's "source declaration kind is outside the
  initial source profile". `Wrap` exports under version 13, and clients of
  `base::wrap(42).0` and `base::Wrap(42).0` are accepted.
- **Fences.** `struct Empty();`, `#[non_exhaustive] struct Open(pub u8)` and
  the `phantom_data` role stay rejected by name under version 13.
- **Scaling.** Packets are 43,793, 62,441 and 137,033 bytes for 6, 24 and 96
  tuple structs, each with a constructor function. That is 1,036 bytes per
  struct, exactly linear. The client LowIR is 1,598 bytes at every width.
- **Byte identity.** The baseline is main `4a9a3441f` (pin sha `f120c641`).
  The candidate is this checkpoint (sha `22fe9718`). They were compared over
  every pa29 spec provider under versions 10, 12 and 13, each with every fresh
  consumer in its directory. That covers 3,485 metadata, LowIR, status and
  diagnostic files, including all 24 earlier version-13 packets (Lane B's
  variant re-exports among them). The 44 files that differ are
  all in the new packet's version-13 run: the baseline rejects its provider
  with "source declaration kind is outside the initial source profile", and
  the candidate exports it. Under versions 10 and 12 both binaries reject it
  with the same bytes.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `struct_constructors` integration (4), `unique_writes` (4), `lifetime_methods` (4), `lifetime_transport` (4) and the neighbouring transport tests (borrowed, branch, cold, constructor, direct calls, fn roles, guard, imported records and regions, inherent, instantiable, match, node, normalized results, records, source nominals, spelled roles, statics, str-never) | 74/74 | `1794102004562301785` |
| native `--ignored`: this packet, captured methods, lifetime nominals, inherent methods, source nominals, constructors, enum match, str-never, unique writes | 9/9 | `1794102187452577707` |
| native `--ignored`: body nodes, cold, guards, branches, fn roles, spelled roles, direct calls, instantiable, normalized results | 9/9 | `1794102256352988726` |
| private selection (29): the hot-record and derive-owner layout invariants, nominal record forgeries, Lane B's variant re-export forgeries, module-contract budgets and layouts, transport domains, numbered units, publication attributes, the input plan, consumer domain demand, selected templates, header reservations and origins | 28/29 | `1794102387738968343` |
| PA1-PA12 | pass | `1794102616049008584` |
| PA13-PA19, PA21-PA24 | pass | `1794102666643626703` |
| PA20 | 1,451/1,451 | `1794102737248924093` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/struct-ctors/test-upstream2.log` |
| `make test-harness` | 304 script tests; 464 roots in 13 binaries, 44 stages | `obj/parallel/struct-ctors/test-harness2.log` |

All rows ran on the final rebased code; the integration row adds main's
frozen macro-metadata outcome test. The one private failure is a stale assertion
on main, not this change. `domain_budgets.rs:42` still asserts
`size_of::<SourceRows>() == 784`, but main's `18232e371` made it 792. The
passing hot-record invariant in the same run (`policy_transport.rs:257`)
asserts 792. `verbatim_links.rs` and `versioned_predicates.rs` carry the same
stale 784. This change adds no field to any retained row. `Option<HeaderRef>`
keeps 8 bytes with its eighth variant.

The rustc 1.98.0 receipt is `obj/parallel/struct-ctors/oracle1.log`. The
first candidate round is in `obj/parallel/struct-ctors/cand1/`, and the forgery
observations are in `obj/parallel/struct-ctors/forge-observe/`. The identity
receipt is `obj/parallel/struct-ctors/identity2/receipt.txt`. The pinned
binaries are `obj/parallel/struct-ctors/{baseline2,candidate2}/`.

## Remaining gaps

- `PhantomData`: the `phantom_data` role is still fenced as part of the
  Pointee contract ("source Pointee contract transport is not implemented"),
  and a consumer still rejects a role on a nominal declaration. Core's
  `PhantomData<T: PointeeSized>` and `Location`'s `_filename` field need it.
- `#[repr(transparent)]` structs ("source nominal representation requires
  its wider complete contract"): core's `Wrapping`, `Saturating`, `NonZero`
  and `NonNull` are transparent.
- `#[non_exhaustive]` tuple and unit structs are fenced. A
  `#[non_exhaustive]` record struct exports without the flag on main, and a
  consumer then accepts a struct literal that rustc rejects (E0639). That
  predates this checkpoint.
- `struct T();`, whose callable constructor takes no arguments.
- `use base::*` of an extern crate root ("glob target is not a module or
  enum", also on main) belongs to the shim-glob work.
