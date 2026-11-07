# Inherent methods and late-region originals

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/inherent-methods-20261107`, based on main `abcfea354`. It is a
transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment. It covers four things:

- the decision-4 cost fix;
- the `Columns` hot-record repair;
- inherent impls on provider nominals;
- late-region reference parameters in version-13 free originals.

Guards in provider bodies remain open.

## Cost fix (decision 4)

A rebuilt imported body scans its values to record which imported
constructors it references. That scan is now canonical body work: one
`body_work.canonical_steps` step per value, charged before the scan.

## Cold columns (hot-record repair)

Version 13 added `associated_definitions` (24 bytes) and the enum tables
(72 bytes) inline. That grew the frozen `Columns` from 768 to 864 bytes. Both
groups, and the sparse original-body index, now live in one out-of-line
`ColdColumns` box. The box is absent unless a packet has an associated
definition, an enum or compiler operations. Writers allocate it through a paid
accessor: the exporter's `cold()` and the decoder's `Input::cold`. Storage
preflight counts the box and its vectors.

Two consequences:

- A packet with compiler operations but no cold group allocates the box at
  freeze. `freeze_budget` prepays it.
- Readers go through `enums()` and `associated_definitions()`, which return
  empty columns when the box is absent. Every late read charges exactly as
  before.

`Columns` is 768 bytes again. Raw `SourceRows` is 792. It has no earlier
optional pointer to absorb the box, so exactly 784 is unreachable for any
out-of-line form. The assertion is amended from 784 to 792: one pointer in
place of 96 inline bytes. The same private invariant then fails at `Imported`
(216, asserted 160). That growth comes from Lane B's macro rows (`macros` and
`macro_roots`, `cbef4d469`), not from this lane.

## Inherent impls (version 13)

There is no new wire tag. An `ImplHeader` row already encodes an optional
trait application, and a `MemberLink` an optional trait member. An inherent
impl is the absent case of both: a 48-byte header row with option byte 0, and
member links with `trait_member: None`. Only `body_nodes()` profiles admit it.

- **Export.** `implementation_headers::inherent_implementations` exports each
  local `impl` with no trait whose Self is a struct or enum of this crate.
  Inherent associated constants and types, and `unsafe`, `const` or negative
  modifiers, are rejected by name.
- **Descriptor.** `inherent_contracts::require_profile` rejects an inherent row
  under an older profile ("inherent source impl requires its wider member
  profile"). It also rejects a Self that is not a nominal of this provider,
  and it charges one step per implementation. A relabelled packet never
  reaches this gate, because the version-twelve reader already rejects the
  version-thirteen stream. A private control validates the exported rows
  under both older profiles instead.
- **Consumer.**
  - The staged header (`impl_headers.rs`) carries `predicate: None`.
  - `owner_packets::receiver_owner` installs a new
    `OwnerKind::ImportedInherent` owner. That owner is distinct from
    `ImportedImpl`, so every trait-only path skips it.
  - `Graph::inherent` re-reads the Self from the consumer's own installed
    type and requires a nominal whose coordinate is this provider.
  - `checked_rows` rejects an owner/row kind mismatch.
- **Lookup.** `reserve_inherent_headers` publishes each imported method in
  its nominal's inherent bucket through the local `publish_inherent`. The local
  method lookup, auto-ref and auto-deref, and path calls then select it
  unchanged. The bucket growth is observed as canonical work, because the
  member storage was prepaid at owner install.
- **Calls.** The selected-call index (`versioned_calls::Index`) skips
  inherent impls. A call to an inherent method is an ordinary function
  application.
- **Formals.** A callable registration lists formals outermost first. A body
  contract lists them own-scope first. `ordinary_recordings::scope_order`
  regroups by owner, paying per formal.

## Late-region originals (decision 2)

- **Gate.** Under version 13 only, a free provided original with no captures
  and no early regions may bind late region formals. Every older profile keeps
  "source original region formals require their complete wider body profile".
- **Named lifetimes.** Named lifetimes now reach imported body catalogs.
  `body_origin::publication` admits `DefKind::Lifetime` declarations.
  `ingested_readiness` gives version 13 a `RegionBinding` readiness for a
  region formal of a provided function. Older profiles keep their rejection.
- **Re-proof.** The wire admits the header's original result against the
  body's result by shape only. Before this change no consumer proved their
  regional relation. A packet claiming `-> &'b u8` over a body returning `a`
  was accepted. `rebuilt_inputs` now adds a declared-result `Coercion`
  obligation to each imported original's contract whenever the two types
  differ, and validates it as part of the complete expected set. The forged
  packet is rejected with "required lifetime does not outlive its target",
  the local frontend's diagnostic for that source.

## Decision 1

An imported constructor's synthesized body is anchored at the consumer root.
Its emission identity is its `Instance`, the variant item plus its instance
arguments. The anchor is never part of that identity.
`inherent_methods_survive_wire_and_fresh_consumers` checks the `map-call`
LowIR: `Opt::Some::<u8>` is defined once and called from both `entry` and the
`map` instance.

## Evidence

The fixtures and outcomes are in `pa29/tests/spec/inherent-methods/` and
`pa29/tests/inherent-methods-outcomes.json`. The integration controls are in
`crate_linking::inherent_transport`, and the descriptor gate's private control
is `support::crates::generics::source::inherent_contracts::tests`.

- **Native.** The six accepted clients and three generated clients (the
  source-nominals inherent client, the late-region gate client and the
  `first` client) exit 42 natively, as rustc's do.
- **Opponents.** The four opponents are rejected, as rustc rejects them
  (E0599, E0308, E0382, E0599).
- **Forgeries.** Every forged packet is rejected by name.
- **Scaling.** Packets are 23,858, 49,850 and 153,818 bytes for 6, 24 and 96
  methods, which is exactly linear. The client LowIR is 2,372 bytes at every
  width.
- **Byte identity.** Checked against the main `abcfea354` baseline over the
  older v10, v12 and v13 fixtures: 448 metadata, LowIR, status and diagnostic
  files, with zero differences.
- **Text oracles.** PA1-PA19 and PA21-PA24 pass in full. PA20 passes
  1,439/1,443; the four failures are the `assoc-bounds` cases that main fixes
  after this base.
- **Private invariants.** The hot-record invariant passes its `Columns` (768)
  and `SourceRows` (792) assertions and fails only at `Imported`. The
  `fn_allowances` transport case `consumer-grant-not-request` also fails on the
  `abcfea354` baseline.

## Remaining gaps toward `core::option::Option::map`

- const callable bounds (`[const] FnOnce + [const] Destruct`);
- lang and diagnostic items, stability attributes, derives and `must_use`;
- closures in provider bodies;
- callable bounds written by the consumer;
- imported tuple-struct constructors;
- guards in provider bodies;
- a match on a single-variant enum in a provider body. The provider exports
  it, but every fresh consumer rejects the body with "source downcast field
  use is not dominated by its variant test". Main `abcfea354` behaves the
  same way, so the scaling control uses a two-variant enum.
