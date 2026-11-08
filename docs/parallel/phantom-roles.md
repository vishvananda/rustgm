# PhantomData roles

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/static-sync-20261108`, based on pushed main `e07b0c511`. It is a
transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment.

It is the third prerequisite of the caller-location chain. Core declares
`#[lang = "phantom_data"] pub struct PhantomData<T: PointeeSized>;`, and
`Location`, `NonNull` and many alloc types hold one. Before this slice,
version 13 stopped export of the role with "source Pointee contract transport
is not implemented", and a consumer rejected every role on a nominal
declaration.

The slice also fixes a false rejection it found on main (see below) and
amends three existing controls whose subjects changed.

## Wire rule (version 13)

The integrator approved this shape before it was used, and chose option (b)
for the integrity anchor.

- **No new byte.** The role row is version 13's existing spelled form: tag
  255, the spelling `phantom_data` and the declaration address. Older readers
  already reject tag 255.
- **Exporter.** `PhantomData` alone leaves the Pointee fence, and only under
  version 13 (`authored_traits`). `PointeeTrait`, `MetadataType` and
  `DynMetadata` stay fenced in every profile. The role form admits a struct
  for this role only, and the nominal header (`record_headers`) admits the
  struct's PhantomData aggregate under version 13. The header is an ordinary
  Rust-representation unit struct header, so no kind, variance, ownership or
  layout fact travels. Versions 10 and 12 never reach the fence: unit structs
  are version-13-only, so they reject the declaration first ("source
  declaration kind is outside the initial source profile").
- **Reader.** `ordinary_profile::role_header` admits a nominal header only for
  this role and only under the language profile. Every other nominal role
  keeps "ordinary source roles cannot claim a nominal declaration".
- **Consumer.** `mapped_records` stages the struct named by the packet's own
  role row as `aggregate::Kind::PhantomData`. The consumer's existing code
  keyed on that kind then gives it a covariant formal, logical ownership,
  structural auto-trait dispatch and zero-sized layout. The role re-proof
  (`phantom::imported`, called from `admitted_headers::trait_headers`) checks
  the provider's own conditions over the consumer's facts:
  - the declaration is module-owned, a unit struct with no fields, and
    staged as PhantomData;
  - it has exactly one type formal, with no default;
  - it declares `T: PointeeSized`, naming the imported `PointeeSized` role.

  The source and imported checks share the formal and bound helpers. The bound
  is read through the declaration inputs, which carry an import's clauses.

## Integrity anchor: option (b)

A deleted role row would leave the struct a plain unit struct whose formal is
bivariant. A lifetime could then be extended through an imported type, so the
gap cannot be recorded as a known limit.

The consumer already infers every aggregate's variance from its own field
layouts in `lifetime_variances` (`dev/src/sema/declarations/lifetimes.rs`),
imported structs included, and rejects a bivariant formal. Const formals are
always invariant and so exempt, as in rustc. No new pass is needed. The one
change is the message for a source-free nominal: "imported source nominal has
a bivariant formal: its packet is forged or corrupted". Every exporter rejects
a bivariant formal itself ("nominal lifetime parameter is unused or only used
recursively"), so only a forged or corrupted import reaches it. Local
nominals keep their message. The only exemption is the kind that the
re-proved role gives its own struct. The cost is the existing solver's,
linear in each header's field types.

The moved-row forgery targets the formal-free `pub struct Unit;`, not a
second one-formal unit struct. An honest packet cannot carry one: a type or
lifetime formal would be bivariant, which the provider rejects, and const
arguments do not transport.

## Found false rejection: imported generic unit values

A fresh consumer of a provider with no lifetime-generic nominal could not
lower a local that holds a unit value of an imported type-generic nominal. It
failed with "generic aggregate requires instantiated fields". rustc accepts
both witnesses, and both exit 42 natively
(`obj/parallel/phantom/oracle3/oracle3.log`):

- `let _m: base::PhantomData<u8> = base::PhantomData;` against the gate
  provider, `spec/source-nominals/provider.rs` plus `PhantomData`;
- `let _o: base::Opt<u8> = base::Opt::None;` against the same provider plus
  `pub enum Opt<T> { None, Some(T) }`. This one already failed on the
  joined-main CLI (sha `93651a06`).

The cause is in `definition_cells.rs`. `regional_nominals` set the consumer's
nominal-requirement flag only for an imported nominal with a lifetime, while a
local nominal with any formal or own clause sets it
(`aggregates/headers.rs`). Without the flag no body prepares storage for a
value's type. A call or a record literal prepares it on another path, but a
unit value has neither, so lowering's layout query found no instance. The
fixtures built on `lifetime-nominals` always carried a lifetime nominal, and
so does real core, which hid the gap.

`generic_nominals` replaces it, and `commit_ports` uses it. An imported
nominal with a lifetime or type formal, or an own clause, sets the flag,
exactly as a local one does. As the integrator required, consumers of
type-generic-only providers now also run the nominal bound proofs. The
controls and their outcomes were declared in `phantom-role-outcomes.json`
(`found_false_rejection`) before the rustc run and before any candidate run of
the fix.

## Fixtures and outcomes

`pa29/tests/spec/phantom-roles/` holds one provider and nine clients. The
outcomes were declared in `pa29/tests/phantom-role-outcomes.json` at
2026-11-08T05:59:47Z, before any rustc or RustGM run of these files. The
integrator's option (b) was declared at 06:03:14Z, also before any run.

The provider is the shared lines of the earlier packets plus core's
`PhantomData`, `marker<T>()` and `Marked<T> { v: u8, marker: PhantomData<T> }`
with `marked` and `marked_value`. Core's `Clone` and `Copy` impls for
`PhantomData<T>` are left out: generic trait impls on provider nominals have
no transport evidence yet.

Six clients are accepted and exit 42:

- `value` and `marker` build a `PhantomData` with its unit constructor and
  through `marker`;
- `marked` and `marked-call` build a `Marked` with a record literal and
  through `marked`;
- `shrink` shortens `PhantomData<&'b u8>` to `PhantomData<&'a u8>`;
- `local` declares its own `struct Mark<T>(base::PhantomData<T>);`.

Three opponents are rejected as rustc rejects them:

- `grow` lengthens `PhantomData<&'a u8>` to `PhantomData<&'b u8>` ("lifetime
  may not live long enough"): "required lifetime does not outlive its target";
- `marked-grow` does the same through `Marked<&'a u8>`;
- `mismatch` assigns a `PhantomData<u16>` to a `PhantomData<u8>` (E0308):
  "incompatible types".

Both `grow` clients are rejected only because the consumer interns the
covariant formal. A plain unit struct's formal is bivariant and would accept
both.

## Amended controls

- **This slice's older-profile gate**
  (`older_profiles_fence_phantom_roles_by_name`). It was declared as the
  Pointee fence under versions 10 and 12. The first candidate run showed that
  both reject the provider earlier with "source declaration kind is outside
  the initial source profile", and a plain `pub struct Unit;` does the same.
  Unit structs transport only in version 13, and the provider's own check
  requires `PhantomData` to be one, so the fence is unreachable there. It
  stays as defence in depth. The amendment is recorded in the outcome file.
- **The struct-constructor PhantomData gate**
  (`struct_constructors::older_profiles_and_unsupported_struct_shapes_reject_by_name`).
  It expected main's "source Pointee contract transport is not implemented".
  The provider now exports under version 13, and a fresh client binding
  `base::phantom()` is accepted. The amendment is recorded in
  `struct-constructor-outcomes.json`, and the empty tuple and non_exhaustive
  fences are unchanged.
- **The private template-demands control**
  (`sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_keep_failed_attempt_and_peak_costs`).
  It failed on main from 2b0b5cf07 with "named lifetime unexpectedly passed
  its earlier catalog boundary". That commit added `DefKind::Lifetime` to the
  body-origin catalog's admitted kinds (`validate_definitions` in
  `resolve/body_origin/publication.rs`) for late-region free originals. So
  `fn declared<'a>(x:&'a u8)->&'a u8{x}` now reaches its template, as
  intended. rustc 1.98.0 accepts that function after the shared provider, and
  the unused-formal control too (`obj/parallel/template-demands/oracle/`).
  No source shape still reaches that boundary. Every transported
  declaration kind now maps into the admitted kinds, so the check only guards
  against forged rows. The test now records a survey of nine shapes, each of
  which is either accepted or rejected earlier under its own message:
  - a labelled loop, a labelled block, a body-local struct and a body-local
    `const` are rejected at export ("source declaration kind is outside the
    initial source profile");
  - a body-local `fn` is rejected with "source operation omitted a required
    nontrivial formation input";
  - a closure is rejected with "source closure typing requires actual source
    Copy and callable declarations";
  - a const generic is rejected with "source const parameters are outside the
    initial source profile";
  - `fn probe<'a, T>(x:&'a T)->&'a T` and the elided `fn probe(x:&u8)->&u8`
    are accepted.

  The test's subject is replaced by the acceptance path's cost assertions.
  Under exhausted steps, and again under exhausted storage, the named-lifetime
  template's attempt fails and is still charged past the limit. An honest
  attempt is ready, and its peak covers its retained bytes: 2,464 retained,
  peak 158,385, 1,229 steps. The `unused<T>` failed-attempt and peak assertions
  are unchanged.

## Evidence

Every run used the tree rebased onto `e07b0c511`. Receipts are under
`obj/qualification/fast/`. Each receipt passed and records the source hashes
of the committed tree. The candidate CLI is `obj/parallel/phantom/candidate/rustgm`
(sha256 `f0c7d9dd`), the fastcheck build of this tree. The baseline is the
same build of `e07b0c511` (`obj/parallel/phantom/baseline/rustgm`, sha256
`e75e8533`).

Non-native results:

| Selection | Tests | Receipt |
| --- | --- | --- |
| crate_linking, part 1 | 51 | `1794127805840998505` |
| crate_linking, part 2 | 51 | `1794128217953176666` |
| crate_linking, part 3 | 50 | `1794128387478933335` |
| conversion_inference downstream | 1 | `1794128482951175981` |
| private source-client cases, part 1 | 85 | `1794128521210286840` |
| private source-client cases, part 2 | 84 | `1794128615330493976` |
| private cases for this slice | 91 | `1794127568438354211` |
| course text pa1-pa12 | passed | `1794128824083663164` |
| course text pa13-pa19, pa21-pa24 | passed | `1794128878481752277` |
| course text pa20 | passed | `1794128949768931506` |

The three crate_linking parts are the inventory's 152 non-ignored tests,
`phantom_roles` and the amended `struct_constructors` gate included. The 91
private cases include `template_demands::tests::private_callable_templates_keep_failed_attempt_and_peak_costs`,
probed by name, the five `compiler_items::phantom` tests, both
`members::transaction_tests` and the header capacity and domain-budget
controls. Later assignments have no accepted text contract yet, and pa29's
contract is crate_linking. `make test-upstream` verified 128 pinned fixtures
and 126 Reference files. `make test-harness` found 464 roots in 13 binaries
and no orphan fixtures.

Native results (`--ignored`, reported separately):

| Selection | Tests | Receipt |
| --- | --- | --- |
| crate_linking native, part 1 | 19 | `1794128649409422463` |
| crate_linking native, part 2 | 19 | `1794128698921373914` |
| conversion_inference native | 1 | `1794128763172761907` |
| packet-envelope native client | 1 | `1794128793575301015` |

The two crate_linking parts are the inventory's 38 ignored tests. With the
non-native parts, that is all 190.

No new rejection. The identity run (`obj/parallel/phantom/identity/`)
compiled every pa29 spec provider with both CLIs under versions 10, 12 and 13,
then every fixture client against each export. That is 29 packets and 969 runs.
Only the `phantom-roles` packet under version 13 differs:

- the provider exports;
- the six declared clients now exit 0;
- `grow`, `marked-grow` and `mismatch` are rejected under their declared
  messages instead of the baseline's missing artifact.

Every other run's status, stderr and LowIR are byte-identical. Versions 10 and
12 accept 6 and 14 runs both before and after. Version 13 accepts 166 before
and 173 after. The bound proofs that type-generic-only consumers now run
changed nothing else.

Oracles, run before the candidate:

- `obj/parallel/phantom/oracle3/oracle3.log`: rustc 1.98.0 links the gate and
  `Opt` providers, and both clients exit 42 natively.
- `obj/parallel/template-demands/oracle/`: rustc accepts `declared<'a>` and
  `unused<T>` after the shared provider.
- The packet's nine outcomes, from the first rustc run.

Scaling, all asserted by the tests:

- `phantom_packets_and_fresh_consumers_grow_linearly` uses 6, 24 and 96
  records. It asserts exactly linear packet growth, and the client's LowIR
  stays within 64 bytes.
- `imported_marker_records_cost_linear_variance_work_and_need_the_marker_kind`
  asserts exactly linear variance steps at 6, 24 and 96. It also shows the
  formal turns bivariant once the marker is interned as a plain struct.
- `imported_generic_unit_values_prepare_storage_with_linear_work` covers the
  fix's cost. Total work at 6, 24 and 96 enums is 1,824, 2,238 and 3,894:
  exactly 23 per enum (`obj/parallel/phantom/measure-lib.log`).

## Files outside transport

For owner review:

- `dev/src/sema/declarations/lifetimes.rs`, `lifetime_variances`: one arm,
  the message for a bivariant source-free nominal;
- `dev/src/sema/header_attributes/declared_origins.rs`, `validate_header`:
  a struct header's aggregate kind may also be PhantomData;
- `dev/src/sema/compiler_items/phantom.rs`: `imported`, and the shared
  `formal` and `pointee_bounded` helpers that `declaration` now uses;
- `dev/src/sema/compiler_items/roles/admitted_headers.rs`, `trait_headers`:
  the PhantomData arm;
- `dev/src/types/aggregate/prepared_shapes.rs`,
  `prepare_imported_nominals` and `stage_structs`: the staged kind;
- `dev/src/support/crates/generics/ordinary_profile.rs`, `role_header`: the
  nominal arm under the language profile;
- `dev/src/sema/crates/typed_headers/staged_headers/definition_cells.rs`:
  `generic_nominals`, which replaces `regional_nominals`, and the shared
  `self_arguments` read by `nominal_regions`;
- `dev/src/sema/crates/typed_headers/staged_headers/commit_ports.rs`, the
  commit's flag update;
- `dev/tests/invariants/prepared_transactions.rs`: the new
  `prepare_imported_nominals` argument in both tests.

## Remaining gaps

- **The rest of the Pointee family.** `Pointee`, its `Metadata` type and
  `DynMetadata` stay fenced in every profile.
- **`Clone` and `Copy` for `PhantomData<T>`.** Generic trait impls on provider
  nominals are a separate transport.
- **Next prerequisites** of the caller-location chain, in the integrator's
  order:
  - NonNull's diagnostic role, with pattern types (tag 16) and
    `rustc_nonnull_optimization_guaranteed`;
  - auto traits with negative impls.

  Then come `Location`, the caller ABI and `PanicInfo`/`panic_impl`. Each
  exact shape is reported before it is used, and the pattern-type constraint
  encoding first.
