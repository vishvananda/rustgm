# Nominal CoerceUnsized declaration checkpoint

This PA31 declaration prerequisite removes the explicit nominal-implementation
fence using the existing canonical CoerceUnsized identity and implementation
proof. Source and target must have the same actual struct owner. Both complete
applications use the existing semantic-field substitution kernel, with one paid
Bound mapping per endpoint. After releasing those iterators, the same impl Proof
normalizes the fields, compares full types including lifetimes, skips only actual
checked PhantomData, and requires exactly one changed non-phantom field. That
field must prove its actual recursive CoerceUnsized relation. Existing pointer
implementations still prove Unsize and their ordinary directed region relation.

CoerceUnsized retains its distinct legality: unchanged scalar fields, repr(C)
and packed structs are valid. DispatchFromDyn's representation/ZST restrictions
are not applied. Ordinary trait conformance, formation, source header authority
and the existing retained trait-region proof remain required.

`SemanticFields::field_paid` shares the original field/Bound kernel and prepays
actual external-source work/storage before steps and growth. Both endpoint folds,
the simultaneous pair-vector capacity and temporary handles are paid. No fields
were added to SemanticFields, Rules, Body, Owner, Kind or Declarations; this
checkpoint makes no throughput or measured-layout claim.

The sixteen exact source predictions were frozen before implementation and
candidate observation. All match pinned Rust 1.98.0. The existing independently
positive `unsizing-roles/nominal-coerce-required.rs` now passes as a real trait
goal consumer. Focused gate `1793966366564702209` passes formatting, architecture,
strict selected Clippy and three exact public test groups in 205.30 seconds.
The first gate's formatting-only failure `1793966287267469768` remains retained.
The independent reference and subsequent upstream/harness receipts are bound by
the adjacent evidence JSON. Private execution and broader release checks remain
pending integration/CI; no private cold harness was linked.

This does not implement wrapper value conversion. The unchanged full original
Coroutine result-coercion source remains SHA256
`16b1d23d193074c832a23a5b672d7089b09564cb5520ec0b6c50d27374c8b09f`.
Its prior exact nominal-fence test will need an intentional next-phase update
when the actual value consumer is qualified. Required continuation is the paid
typed wrapper plan, selected caller CoerceUnsized and region proof, exact terminal
metadata, whole-input ownership transfer, one target cleanup, nested/side-effect/
Drop native observers, CTFE and authenticated transport. Declaration validation
does not supply those permissions or complete PA31 or any library artifact.
