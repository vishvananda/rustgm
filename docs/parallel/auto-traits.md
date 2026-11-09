# Auto traits

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/auto-traits-20261108`. C1 started on pushed main `b47b51e79` and
landed as `89cb62ee5`. C2 landed as `c9fb67cc3` and C3 as `bfbf43049`,
and C3b is developed on main `bfbf43049`. It is item 5 of the
[core census plan](../core-census-plan.md). It is a transport slice, not a
library milestone, and makes no claim about an original-core artifact or a
whole assignment.

The integrator approved one item in three commits, which may be joined, and
then added C3b and C2b:

- **C1** (`89cb62ee5`): exact empty auto declarations, the auto-marker roles,
  and `Send`'s diagnostic identity.
- **C2** (`c9fb67cc3`): auto impls, positive and negative, and the
  structural selection lift for imported custom auto traits
  ([C2](#c2-auto-impls)).
- **C3** (`bfbf43049`): unsafe traits that are not auto, and every trait
  impl's own safety and polarity
  ([C3](#c3-unsafe-traits-and-non-auto-negative-impls)).
- **C3b**, this commit: knowable absence relative to the crate whose
  coherence is proved ([C3b](#c3b-knowable-absence)).
- **C2b**, after C3b: an imported nominal's transported where-clauses as its
  formation requirements ([C2b](#c2b-imported-nominal-formation)).

C2c, core's `CoerceUnsized` impl for `pattern_type!(*const T is !null)`
(`pat.rs:80`), moved to Lane A and is done in census batch 16.

The sections before [C2](#c2-auto-impls) record C1.

Core declares its markers as

```rust
#[rustc_diagnostic_item = "Send"]
pub unsafe auto trait Send {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[lang = "unsafe_unpin"]
pub unsafe auto trait UnsafeUnpin {}
#[lang = "unpin"]
pub auto trait Unpin {}
```

and `panic/unwind_safe.rs` declares `UnwindSafe` and `RefUnwindSafe` the same way,
each with only a diagnostic item. Before this slice, version 13 stopped export
with "source UnsafeUnpin contract transport is not implemented". Without
UnsafeUnpin it stopped with "source Send diagnostic contract transport is not
implemented", and without both with "source const/auto/unsafe traits require a
wider explicit profile".

## Wire rule (version 13)

The integrator approved these shapes before code (Q1, Q2):

- **No new trait-header tag or bit.** The trait header's existing `AUTO` (4)
  and `UNSAFE` (1) flags are admitted only for an exact empty auto declaration:
  no written parameters, supertraits, members, region captures or bounds, and
  a single unsized, unqualified `Self` slot. `CONST` stays rejected with
  "source const/auto/unsafe traits require a wider explicit profile".
- **Role-bound markers keep their local role's flags.** Freeze, Send, Sync and
  UnsafeUnpin are unsafe auto. Unpin is safe auto. Sync, Unpin and UnsafeUnpin
  travel as spelled language role rows (tag 255), as every non-compact role
  does, and Freeze keeps its compact role.
- **`Send`'s diagnostic identity is the second 254 row**:
  `[254][u32 4]["Send"][u32 artifact][u32 declaration]`. It follows the
  language rows, after `NonNull`'s row when both exist, and the closed table is
  `DiagnosticRole::ALL = [NonNull, Send]`. A consumer requires the provider's
  own trait, with exactly `UNSAFE | AUTO`, at most once, and installs it in its
  own `LangItem::Send` slot.
- **UnwindSafe and RefUnwindSafe get no identity (Q2).** They are ordinary
  custom auto traits on the wire.
- **Older profiles fence by name, verbatim.** Versions 10 and 12 keep main's
  exact diagnostics for each declaration (see the generated controls).

### Exporter

`authored_export/authored_traits.rs`:

- `Stage::traits` lifts the UnsafeUnpin fence under version 13 and keeps it
  verbatim below. Send's role row is skipped under version 13, because its
  identity travels in the 254 row; below version 13 it keeps main's error.
- `Stage::diagnostic_roles` writes the closed table in order: NonNull, then
  Send. Each older profile keeps its own verbatim message.

### Reader and consumer

- `generics/schema_rows.rs`: `DiagnosticRole::Send`, spelled `Send`.
- `generics/ordinary_profile.rs::diagnostic_bindings` checks a Send row's
  declaration is a trait with exactly `UNSAFE | AUTO` ("imported Send
  diagnostic item requires its unsafe auto trait declaration") and returns it.
  `bindings` records the Send, Sync, Unpin and UnsafeUnpin declarations, and
  `trait_headers` routes every `AUTO` header through
  `free_protocols.rs::automatic`.
- `generics/free_protocols.rs`: `freeze` and `automatic` share `empty_auto`,
  the exact-empty re-proof, charged at 8 steps per header. Freeze keeps its
  texts. `automatic` requires `AUTO` with no flag beyond `UNSAFE | AUTO`
  ("source auto trait requires its exact empty auto declaration"), then each
  role-bound marker's exact unsafety ("source Sync role requires its exact
  auto declaration flags", and likewise for Send, Unpin and UnsafeUnpin).
- `compiler_items/binding_views.rs::reserve_roles` installs each 254 Send row
  as the consumer's own `LangItem::Send` role, and `diagnostic_roles`
  re-proves it as a trait definition over the consumer's own remap.
- `roles/admitted_headers.rs::trait_headers` admits Send, Sync, Unpin and
  UnsafeUnpin with `free_marker`, as Freeze is (the `FREE_MARKERS` arm), with
  each role's exact local flags.

## The auto-marker accessor (condition 1)

`Catalog::auto_marker(role, validated)` in `compiler_items/roles.rs` answers
only Freeze, Send, Sync, Unpin and UnsafeUnpin ("auto-marker lookup requires
an auto-marker role" for any other). It returns the role, its definition and
its origin. A provider answers exactly as `language` does, or as
`validated_language` when `validated`. A source client answers from the roles
its admitted descriptor re-proved, and when `validated` only after their
completed header admission. `Catalog::language()` is not widened: it still
answers `None` in every client.

The approved shape named four sites in `selection/search/freezing.rs` and one
in `validation.rs`. Making the declared clients pass found a sixth: the body
evidence consumer `roles/freezing.rs::predicate`. The integrator approved it,
together with the payer change below.

### Every `language(...)` caller for the five markers

`grep -rn '\.language(' dev/src` finds 214 calls. Only the callers below can
receive Freeze, Send, Sync, Unpin or UnsafeUnpin, either literally, through a
marker list, or through a loop over every role. Each one has one disposition.

Moved to `auto_marker` in C1:

| Site | Function | Lookup |
| --- | --- | --- |
| `traits/selection/search/freezing.rs` | `bound` | validated; its origin picks the payer |
| `traits/selection/search/freezing.rs` | `object_marker` | identity |
| `traits/selection/search/freezing.rs` | `Search::source_auto_kind` | identity, then validated |
| `traits/selection/search/freezing.rs` | `has_source` | identity |
| `declarations/validation.rs` | `Checker::require_shared_static_sync` | validated Sync |
| `compiler_items/roles/freezing.rs` | `predicate` (`freeze_predicate`) | identity, then validated |

Deferred to C2, and moved there:

- `declarations/traits/implementations.rs::validate_source_impl_policy`. Its
  `freeze_impls` gate reads `language(Freeze)`. A client's impl of an
  imported Freeze is an auto impl, so it is gated with C2's (b)-(d) rules, not
  by transport. C2 reads `auto_marker(Freeze, false)`.

Provider-only by design:

- `associated.rs::owner_kind` validates a local auto trait's header when it is
  one of the crate's own role or diagnostic declarations. It reads the
  resolve-level reservations, which only a source core fills. A client
  rejects every local lang item and diagnostic item by name (the duplicate
  controls below).
- `traits/selection/search/negative_equalities.rs::custom` runs its marker
  loop only after `automatic_traits::predicate` accepted a local owner
  (`owner.source`). An imported marker never reaches the loop, and in a client
  a local auto trait is never a marker. C2's selection lift lets `predicate`
  accept an imported owner, so C2 moves this loop to `auto_marker(role, false)`.
- `compiler_items/roles.rs::validate` is the provider's own role validation
  over `TRAIT_ROLES`. A client admits imported roles in
  `admitted_headers::trait_headers` through `role()`.
- `compiler_items/roles.rs::validated_language` and `auto_marker`'s provider
  arms.
- `crates/authored_export/authored_traits.rs`: `Stage::traits` (the
  UnsafeUnpin fence) and `Stage::diagnostic_roles` (the Send row). The exporter
  publishes a source core's own declarations.
- `compiler_items/binding_views/compact.rs::new` encodes the source core's
  reserved roles.

No site is left neither moved nor deferred.

## Paying for a client's marker bound

`selection/search/freezing.rs::bound` interns a marker's zero-argument
application for each structural child, for example `u8: Sync` under
`&u8: Send`. A source core pays with `source_payments`, which requires its
completed local header producer. A source client never has one, so every
client structural proof failed with "source retention lacks its completed
local producer".

An `Imported` role now pays through `imported_application`. It charges
`crate::sema::crates::charge_source_headers`, the installed header budget,
exactly the steps and bytes the observed interning attempted. This is the
existing split in `search/builtins.rs::classify_waiting_materialization`. The
charge is applied right after the interning, because the payer cannot be
borrowed during it. At most five distinct rows can be interned this way, one
per marker. The source-core path is unchanged.

The control `freezing::payment_tests::imported_marker_bounds_pay_the_header_budget_once_per_marker`
(`dev/tests/invariants/client_marker_payments.rs`) builds the honest packet, analyzes
the `answer` client in-process, and then calls `bound` for each marker three
times. It asserts:

- each role is `Imported`;
- each call's header-budget work and bytes equal the attempted interning:
  `1 + growth.work` and `growth.bytes` when the row is absent, `(1, 0)` when it
  is present;
- the first round interns all five rows;
- the later rounds find every row present and return the same bound.

## Fixtures and outcomes

`pa29/tests/auto-declaration-outcomes.json` was declared at 12:45:17Z, before
any rustc or candidate run of these files. It holds 11 outcomes: the provider
export and ten clients. The provider is the pa29 shared prefix (lines 1-147)
plus the marker declarations, `Pair<T>`, `Raw { p: *const u8 }`,
`Either<A, B>` and `answer`.

| Client | rustc 1.98.0 | RustGM |
| --- | --- | --- |
| send-structural | exit 42 | exit 42 |
| sync-static | exit 42 | exit 42 |
| local-static | exit 42 | exit 42 |
| pin-markers | exit 42 | exit 42 |
| bounded-param | exit 42 | exit 42 |
| local-holder | exit 42 | exit 42 |
| answer | exit 42 | exit 42 |
| unbounded-param | E0277 | "trait obligation requires an unambiguous matching assumption or implementation" |
| duplicate-lang | E0152 | "compiler bindings require an explicit source-core provider" |
| duplicate-send | duplicate diagnostic item | "diagnostic identity requires its rustc_attrs source provider" |

`local-holder` holds `Pair<Raw>`. Without core's `!Sync for *const T`, a raw
pointer is structurally Sync, and rustc agrees.

The duplicate-identity control (condition 2) is the 12:49:06Z amendment,
declared after main-CLI probes of its bodies and before any rustc or
candidate run. `duplicate-lang` declares its own `#[lang = "sync"]` beside the
imported one: rustc gives E0152. `duplicate-send` declares its own `Send`
diagnostic item: rustc reports the duplicate, because
`rustc_interface/src/passes.rs` computes `all_diagnostic_items`
unconditionally. Both are oracle-run in
`obj/parallel/auto-traits/oracle/c1b`. RustGM rejects each by name before any
catalog lookup, so the outcome is deterministic whatever the provider
carries.

`unbounded-param` records `rustc_observed` beside its declared text. rustc
says "the trait bound `T: Send` is not satisfied in `Pair<T>`". The declared
wording is std's `rustc_on_unimplemented` message, which this provider's Send
does not carry. The code and span match.

Generated controls, in `crate_linking::marker_transport`:

- `older_profiles_fence_auto_declarations_by_name` exports lines 1-57 plus each
  of six declarations under versions 12 and 10, and asserts main's verbatim
  text. Each export is accepted under version 13.
- spec/static-sync's plain-static client still fails with "requires `sync`
  lang_item" (`crate_linking::synced_statics`).

Wire controls (`forged_auto_declarations_reject_in_fresh_consumers`). An
`UnwindSafe` unsafety variant first calibrates the fixed-width (66-byte) trait
header table. Then:

| Forgery | Fresh consumer |
| --- | --- |
| `AUTO` set on Add's header | "source auto trait requires its exact empty auto declaration" |
| `UNSAFE` cleared on Sync | "source Sync role requires its exact auto declaration flags" |
| `UNSAFE` set on Unpin | "source Unpin role requires its exact auto declaration flags" |
| `AUTO` cleared on Send | "imported Send diagnostic item requires its unsafe auto trait declaration" |
| the Send row moved to `Raw` | the same |
| the Send row duplicated | "duplicate source diagnostic identity" |
| `Send` respelled `Senx` | "unknown source diagnostic identity spelling" |
| the Send row deleted | accepted: Send is then an ordinary custom auto trait |

## Flip survey (condition 3)

Imported Freeze, and now Send, Sync, Unpin and UnsafeUnpin, prove through the
role path in every source client. The integrator required a list of every
outcome this changes. Only rejected→accepted flips matched against rustc were
allowed. Any accepted→rejected flip would stop the work for a report.

The survey (`obj/parallel/auto-traits/flip/`) ran two CLIs:

- the baseline, `b47b51e79` (`obj/parallel/auto-traits/baseline/rustgm`, sha256
  `557d664a`);
- this tree on the same base (`obj/parallel/auto-traits/candidate/rustgm`,
  sha256 `91de2846`).

Each exported every pa29 spec provider under versions 10, 12 and 13, then ran
every fixture client against each export: 31 providers, 93 exports and 1,035
runs. Only the new `auto-declarations` packet under version 13 differs:

- the provider exports, where the baseline stops at "source UnsafeUnpin
  contract transport is not implemented";
- `send-structural`, `sync-static`, `local-static`, `pin-markers`,
  `bounded-param`, `local-holder` and `answer` are rejected→accepted, each
  matching rustc's exit 42;
- `unbounded-param`, `duplicate-lang` and `duplicate-send` stay rejected,
  under their declared messages instead of the baseline's missing artifact.

Every other run's status, stderr and LowIR are byte-identical, including the
30 existing packets, each of which exports Freeze. There is no
accepted→rejected flip. Versions 10 and 12 accept 6 and 14 runs before and
after. Version 13 accepts 178 before and 186 after.

The survey covers the pa29 packets. The linking suite and the private
source-client cases (see Evidence) assert every other source-client outcome,
and none of them changes.

## Evidence

Every run below used the tree rebased onto `4a1ac00c4` (commit `b61f24357`),
except the flip survey, which isolates this slice on its `b47b51e79` base.
Receipts are under `obj/qualification/fast/`, and each passed.

Non-native results:

| Selection | Tests | Receipt |
| --- | --- | --- |
| this slice's crate_linking, nonnull and static-sync cases | 9 | `1794145577737841628` |
| the two payment controls | 2 | `1794145438151865590` |
| linking suite, part 1 | 47 | `1794146222265856099` |
| linking suite, part 2 | 47 | `1794146346287091247` |
| linking suite, part 3 | 46 | `1794146450655975977` |
| linking suite, part 4 | 49 | `1794146527516620217` |
| linking suite, part 5 | 48 | `1794146605354325205` |
| lib cases of the touched modules | 123 | `1794146642487248130` |
| private source-client cases, part 1 | 85 | `1794146789260317442` |
| private source-client cases, part 2 | 84 | `1794146825245603157` |
| course text pa1-pa12 | passed | `1794146880731697687` |
| course text pa13-pa19, pa21-pa24 | passed | `1794146943419373850` |
| course text pa20 | passed | `1794147014004405415` |

The five linking parts are the suite inventory's 237 non-ignored tests,
listed from the built `suite_linking` binary. That includes the three
non-native `marker_transport` tests. The 123 lib cases are every test under:

- `compiler_items::binding_views`, `compiler_items::roles`,
  `crates::authored_export` and `support::crates::generics`;
- in `selection::search`: `freezing`, `negative_equalities`,
  `auto_cycle_tests`, `no_drop`, `marker_readiness_tests`,
  `marker_evidence_tests`, `pinning_tests` and `nominal_tests`.

`make test-upstream` verified 128 pinned fixtures and 126 Reference files.
`make test-harness` found 464 roots in 13 binaries and no orphan fixtures.

Native results (`--ignored`, reported separately):

| Selection | Tests | Receipt |
| --- | --- | --- |
| linking suite native, part 1 | 25 | `1794147144366574272` |
| linking suite native, part 2 | 24 | `1794147385727557219` |

The two parts are the inventory's 49 ignored tests. They include
`auto_declaration_clients_execute_natively`, in which the seven accepted
clients exit 42 with empty stdout. With the non-native parts, that is all 286.

Oracles, run before the candidate:

- `obj/parallel/auto-traits/oracle/c1.log`: rustc 1.98.0 gives the eight
  first-declared outcomes, and the seven accepted clients exit 42 natively;
- `obj/parallel/auto-traits/oracle/c1b`: the same, plus E0152 for
  `duplicate-lang` and the duplicate diagnostic item error for
  `duplicate-send`.

One process slip, recorded here: while splitting the selection lists, a shell
loop deleted five just-written split intermediates under
`obj/parallel/auto-traits/` with a variable in an `rm` command. That breaks
the literal-path rule. The targets were only those five scratch files, and the
`.txt` selections were written first. Every later removal used literal,
inspected paths.

## Owner review (condition 4)

At the join, owners review:

- `traits/selection/search/freezing.rs`: `bound`, `imported_application`,
  `object_marker`, `Search::source_auto_kind` and `has_source`;
- `compiler_items/roles/freezing.rs::predicate`;
- `declarations/validation.rs::Checker::require_shared_static_sync`;
- `compiler_items/roles.rs::Catalog::auto_marker`;
- `compiler_items/roles/admitted_headers.rs`: `trait_headers` (`FREE_MARKERS`)
  and `free_marker`;
- `traits/selection/search/automatic_traits.rs::Search::automatic_result`, at
  C2's selection lift.

## C2: auto impls

C2 is the second commit of this item. It was developed on `89cb62ee5`, where
C1 landed, and rebased onto pushed main `6cd9f9e82` with no overlapping file.
It transports positive and negative impls of a packet's own auto traits, and
it lifts the imported custom auto-trait structural fence.

### Wire rule (C2)

The integrator approved these shapes before code, and again before commit:

- **Version 13 only; no new tag, bit or row.** An impl header's existing
  `unsafe` and `negative` bytes are admitted when its trait application names
  an exact empty `AUTO` trait of the same packet. Those traits are the
  role-bound Freeze, Send, Sync, Unpin and UnsafeUnpin, and custom auto traits
  such as UnwindSafe and RefUnwindSafe.
- **`constant` stays rejected, and so do non-auto unsafe or negative impls**,
  until C3. Admission says "source const/negative/unsafe implementation
  requires a wider explicit profile" (`ordinary_profile.rs`) and staging says
  "source impl requires its wider conformance/effect profile"
  (`staged_headers/impl_headers.rs`). Versions 12 and 10 keep both verbatim.
- **Each consumer re-proves** `unsafe == trait UNSAFE && !negative`, an empty
  member range, the auto Self rule and, for a negative impl on a nominal, that
  it is always applicable. An imported negative auto impl completes as
  `NegativeValidated`.
- **Q5, a foreign auto-trait impl** (a trait application naming another
  artifact), is fenced by name: "imported foreign auto-trait impl requires its
  multi-packet profile".

### Reader and admission

- `generics/free_protocols.rs::automatic_implementation` charges 2 steps. It
  rejects a foreign trait application with the Q5 text, and answers whether
  the trait header has `AUTO`. `ordinary_profile.rs::validate_with` admits a
  negative or unsafe impl only when it answers yes. The exporter
  self-validates through the same path, so the provider exports.
- `staged_headers/impl_headers.rs::prepare` admits the same rows through
  `automatic`. That requires the version-13 profile, the provider's own trait
  application and an `AUTO` trait header.

### Consumer

`portable_impls.rs::validate_imported_trait_implementation` now branches when
the trait is an imported `AUTO` trait. That trait is re-checked by name as its
packet's version-13 declaration. `validate_imported_auto_implementation` then
does five things:

1. It re-reads the row (`Graph::automatic_implementation`, `origin_ports.rs`).
   It requires:
   - the version-13 profile;
   - the provider's own trait application (the Q5 text);
   - an exact `AUTO`, non-constant trait;
   - `unsafe == UNSAFE && !negative` ("selected impl safety/member kinds
     disagree with its actual trait");
   - an empty member range ("auto or negative trait implementations cannot
     declare members").

   It answers the row's polarity. The registry holds the same polarity:
   registration reads it from this row (`polarity.rs::imported_status`), and
   `polarity` re-reads it whenever it answers.
2. It applies the local auto Self rule (`automatic_domains.rs`). The trait is
   the impl's own packet's, so the rule is the declaring crate's local rule.
   - The source and imported paths now share `automatic_subject` through a
     `Site`. A dynamic, projection or opaque Self is rejected.
   - A generic Self must be Sized. The imported proof runs in the impl's
     installed environment. Its goal is the imported Sized role's own
     installed Self bound, the one decoded original sizing reads.
   - Nothing is interned, so nothing is paid. Retained evidence is rejected.
3. For a negative impl on a nominal, it re-proves coverage
   (`negative.rs::negative_coverage`).
   - Each of the impl's declared inputs must be one of its nominal's inputs
     instantiated at Self.
   - Self's arguments must be the impl's distinct own parameters
     (`negative_parameters`, unchanged).
   - The impl's declared inputs now come from `declaration_inputs::declared`.
     It adds imported inputs and is identical for a source owner.
   - A source client installs no imported nominal's predicates (C2b).
     Instead, the imported nominal's inputs come from
     `declaration_inputs::all`, with outlives omitted as formation omits
     them.
   - A non-ADT negative Self is skipped. rustc skips it when the impl's crate
     has `auto_traits`, and declaring the packet's own auto trait required
     that feature.
4. For a positive impl, it proves its requirements in its installed
   environment (`validate_imported_requirements`, unchanged).
5. It completes the registry entry as `Validated` or `NegativeValidated`, by
   polarity.

### Selection lift (g)

`automatic_traits.rs` no longer answers `false` for an imported custom auto
trait in `predicate`. `automatic_result` drops "imported custom automatic trait
structural profile is not implemented".

`predicate` and `require_automatic_header`'s imported branch both call
`Inventory::imported_automatic_trait` (`Graph::automatic_trait`,
`owner_members.rs`). It returns an error by name for an owner whose packet is
not version 13 ("imported automatic trait requires its version-thirteen
profile"), or that is not an auto trait ("imported automatic trait requires
its actual auto trait owner"). The owner's own authentication (`require_owner`)
compares its `AUTO` flag with the trait header first. A Provided profile owns
no imported auto trait.

### Silent fallback (C1 carry-over)

`freezing.rs::object_marker` and `has_source` now return `Result`. Each reads
`auto_marker(role, false)?`, with no `.ok().flatten()`. A role lookup's
integrity failure is an error, never "not a marker" or "no source marker".
`auto_obligations.rs` reads `has_source` once per proof, and `objects.rs`
reports the error at its path.

### `language(...)` callers, updated

- `implementations.rs::validate_source_impl_policy`, deferred in C1. The
  `freeze_impls` gate now reads `auto_marker(Freeze, false)`. Before, it read
  `language(Freeze)`, which answers `None` for an imported Freeze, so a
  client's Freeze impl never met the gate.
- `negative_equalities.rs::custom`, C1's "provider-only by design". Its
  marker loop now reads `auto_marker(role, false)`, because `predicate` now
  admits imported owners and an imported marker reaches the loop.

The other C1 dispositions are unchanged.

### Consumer checks that admission keeps unreachable

These checks stay as defense in depth. The integrator required each to be
reached by a test where that is cheap, and listed otherwise.

Tested by direct calls on honest graphs
(`portable_impls::fence_tests::imported_auto_checks_reject_ordinary_traits_and_older_profiles_by_name`,
`dev/tests/invariants/polarity_fences.rs`). The `source-branches` provider
exports under versions 12 and 13 and declares only ordinary traits. The test
calls each imported trait's and trait impl's automatic check and asserts its
own text:

| Check | Version 13 | Version 12 |
| --- | --- | --- |
| `Graph::automatic_trait` | "imported automatic trait requires its actual auto trait owner" | "imported automatic trait requires its version-thirteen profile" |
| `Graph::automatic_implementation` | "imported automatic implementation requires its exact auto trait" | "imported automatic implementation requires its version-thirteen profile" |

`automatic_trait` now checks the profile before the auto flag, so a
version-12 call reaches the profile check.

Unreachable via admitted packets, untested:

- "imported foreign auto-trait impl requires its multi-packet profile", in
  both admission (`free_protocols.rs`) and the consumer (`origin_ports.rs`).
  `membership_rows.rs` requires every impl's trait application to name this
  packet before either runs. The Q5 wire forgery stops there ("source
  implementation requires its exact supported trait header").
- "auto or negative trait implementations cannot declare members", in the
  consumer. `membership_rows.rs` requires an impl's member links to match its
  trait's members, and an admitted auto trait has none. The widened-range
  forgery stops there.
- "imported automatic Sized evidence requires its wider profile". The honest
  Sized blanket control below runs this proof, and it retains no evidence.

Reaching the first two would need a test-only mutable row accessor on an
admitted descriptor, which sits in an immutable `Arc` inventory. That is more
than a small helper.

Removed as subsumed, not left as dead code:

- the comparison "imported automatic implementation changed its actual
  polarity". The registry's polarity and the row's are the same byte: see
  consumer step 1.
- "imported automatic trait changed its actual auto declaration", and the
  header re-read before it. `require_owner` already rejects an owner whose
  `AUTO` flag differs from its header ("imported trait changed its actual
  Self/flags/owner mapping").

### Non-ADT negatives (integrator decision)

Coverage of an imported negative impl on a non-ADT (for example `!Send for
*const T`) is trusted to the provider, on the grounds that a same-packet AUTO
trait means the provider compiled with auto_traits, and primitive impls are
provider-only.

### C2b: imported nominal formation

C2 found an accepts-invalid gap that predates it. `nominals::requirements`
reads only `generics.predicates`, and a source client installs no imported
nominal's predicates. So `fn hold(_r: base::Rc<u8, NotAlloc>)` is accepted
(the `89cb62ee5` CLI, `obj/parallel/auto-traits/explore/c2wf`), where rustc
gives E0277.

The integrator assigned it to Lane C as item C2b, after C3 and before 5b:

- **Scope.** An imported nominal's formation requirements include its
  transported where-clauses. They are paid from the installed header budget,
  as C1's marker bounds are.
- **Controls.** rustc's E0277 opponent above, a positive with a satisfying
  argument, and a source-local nominal control that stays unchanged.
- **Temporary workaround.** C2's imported negative coverage reads the
  nominal's transported inputs directly (`negative.rs::imported_nominal_inputs`).
  C2b removes it, so coverage and formation read one source.

### Fixtures and outcomes (C2)

`pa29/tests/auto-impl-outcomes.json` was declared at 14:56:06Z, before any
rustc or candidate run of these files. It holds 27 outcomes: the provider
export and 26 clients. The provider is `#![feature(negative_impls)]`, the
auto-declarations provider, and a tail of authored auto impls shaped as
core's and alloc's (see the packet README).

Three fixture changes were approved before the oracle:

- `Loc` holds a literal file name, so `HERE` goes.
- `Foo` loses its raw field, so `foo-other` proves the root-type rule and
  nothing else.
- `Rc` holds `T`, so `rc-send` rests on the negative impl alone.

| Client | rustc 1.98.0 | RustGM |
| --- | --- | --- |
| explicit-send, loc-static, unwind-structural, cycle, local-cell, local-impl, local-specialized | exit 42 | exit 42 |
| cell-sync, raw-send, foo-other, rc-send, pinned, unique-mut, cell-ref-unwind, raw-holder, pair-cell, local-specialized-other, local-negative | E0277 | "trait obligation requires an unambiguous matching assumption or implementation" |
| cell-static | E0277 | "shared static type must implement Sync" |
| arguments-send | E0277 | "conditional negative implementation requires scoped debt payment" |
| foreign-pointer | E0117 (E0751 printed first) | "library trait implementation requires a local type" |
| foreign-reference | E0321 | "foreign automatic trait requires an actual local nominal implementation target" |
| safe-impl, unsafe-negative, unsafe-unpin-impl | E0200, E0198, E0199 | "trait and implementation safety declarations disagree" |
| freeze-impl | E0658 | "explicit source Freeze implementations require freeze_impls" |

- `arguments-send` is integrator decision 1. rustc's status matches, but its
  reason differs. The refutation of `!Send for Arguments<'_>` stays
  conditional, because its header names a region (`refutation.rs:106`). Q6's
  root-type rule already removes the structural candidate. So this is a
  diagnostic and negative-reasoning gap, not a false acceptance.
- `pair-cell` replaces the drafted `pair-unwind`
  (`Pair<&'static mut u8>: UnwindSafe`). Its structural child into
  `!UnwindSafe for &mut T` is a second witness of the same gap.
- The (c) rule needed no new text. Main's `automatic_domains.rs` already
  applies rustc's local and nonlocal auto Self rules, and C2 reuses its texts.
- Four scratch clients are accepted: `Rc<u8, Global>: Unpin`,
  `PhantomPinned: Send`, `Cell<u8>: UnwindSafe` and
  `Rc<u8, Global>: UnwindSafe`. So each negative-impl rejection above rests on
  its negative impl.

Generated controls (`crate_linking::polarity_transport`):

| Control | rustc | RustGM |
| --- | --- | --- |
| Rc's `!Send` strengthened with `T: Sync` | E0367 | "negative implementation header-strength proof is not implemented" |
| `impl !Send for Bar<u8>` | E0366 | "specialized negative header legality is not implemented" |
| `impl<T: ?Sized> Mark for T`, `Mark` a local auto trait | E0321 | "automatic implementation generic subject must be Sized" |
| `impl<T> Mark for T`, and a client requiring `u8: Mark` and `Cell<u8>: Mark` | compiles; exit 42 | exported; accepted |
| `impl !Freeze for X` under versions 12 and 10 | compiles | "source const/negative/unsafe implementation requires a wider explicit profile"; version 13 exports, and its answer client is accepted |
| a client library with `unsafe impl base::Send for Holder` and a public `Holder` | | "nominal trait, member or non-exhaustive contract is outside the metadata profile" |

- The Sized blanket control is an amendment. It was declared after the
  integrator's C2 review and before its rustc and candidate runs, as the
  honest counterpart of the generic-Self forgery.
- The last row is Q5's reachability probe. The same library with a private
  `Holder` exports only its functions. So no consumer can observe a client's
  foreign auto impl, and no new fence is needed.

Wire controls (`forged_auto_impls_reject_in_fresh_consumers`):

- Making `UnwindSafe for &T` negative moves one byte. That calibrates the
  52-byte impl header table, and every tail row's flags are checked.
- The trait application table is the one offset at which every tail impl's
  application names that impl's own Self type.
- Each forged packet is consumed by `explicit-send`, and the honest packet is
  accepted.

| Forgery | Fresh consumer |
| --- | --- |
| `unsafe` set on `!Send for *const T` | "selected impl safety/member kinds disagree with its actual trait" |
| `unsafe` cleared on `Send for Loc` | the same |
| `negative` cleared on `!Sync for Cell` | the same |
| `Send for Foo<u8>` made negative and safe | "specialized negative header legality is not implemented" |
| `!RefUnwindSafe for Cell<T>`'s Self changed to `T` | "automatic implementation generic subject must be Sized" |
| Rc's own `A: Allocator` retargeted to `A: Send` | "negative implementation header-strength proof is not implemented" |
| `Send for &T`'s Self changed to `T` | "overlapping trait implementation headers require coherence proof" |
| `!Send for Rc`'s member range widened by one link | "source implementation omits an actual trait member mapping" |
| `!Send for Rc`'s application moved to artifact 1 | "source implementation requires its exact supported trait header" |
| version relabelled 13 to 12 | "unknown source declaration kind" |

Four declared wire expectations met an earlier rejection. Each observation is
recorded beside its declaration, and two replacements keep the declared texts:

- `Send for &T` with Self `T` is a blanket Send. It overlaps the packet's
  other Send impls, so coherence rejects it first. The replacement uses
  RefUnwindSafe's only impl.
- Shortening Rc's input range orphans a clause row, which admission rejects
  ("source clauses have incomplete owning header coverage"). The replacement
  retargets the bound's application.
- A widened member range takes another impl's link, which admission rejects.
- The version-12 decoder rejects a version-13 declaration kind before any
  fence. The Q5 forgery likewise fails admission before the named fence.

The integrator accepted these amendments, on the condition about unreachable
checks above.

Scaling (`portable_impls::scaling_tests`, in-process):

- N is 8, 16 and 32. Each provider is the packet's provider plus
  `S000 { next: Unique<S000> }` and `S{i} { inner: S{i-1}, next: Unique<S{i}> }`,
  with zero-padded names. A client requires `S{N-1}: Send`.
- Packet bytes grow exactly linearly: 50,840, 52,624 and 56,192.
- The client's header-budget work (34,745, 35,777 and 37,841) and its
  inference steps (1,315, 2,523 and 4,939) each grow from 16 to 32 by at most
  twice their 8-to-16 increment plus 64.
- The first draft measured canonical steps. They stay 16,096 at every length,
  because a body's solved goals report their search steps as inference steps.
  The drafted "one coinductive hit per cycle" has no counter to observe. The
  linear step bound is its evidence, recorded as an amendment.

### Shared output folders (condition 8)

The race met while developing C2 was its own. `polarity_transport` and
`marker_transport` both published into `auto-traits/frontend-provider-PID`.
Tests of one binary run concurrently, so one overwrote the other's packet. C2's
labels are now `impls-*`, so the race is fixed in C2's own new file.

The integrator then asked for a grep of main's tests for any other pair. None
was found:

- **Packet folders.** All 128 `Packet::new` and `Packet::publish` calls,
  wrappers included, use distinct `(group, label)` pairs across functions.
  The one repeated pair, `node_transport`'s version-12 and version-13 loop,
  runs sequentially inside one function.
- **Literal folders.** No literal `obj/parallel/...` string (163 of them)
  appears in two functions.
- **Parameterized helpers.** These were grouped by binary and argument:
  `observed_program::compare`, `host_objects::check_cases*`,
  `native_references::check_*cases*`, `crate_artifacts::build*` (14 tags)
  and `reconstruction_work::compare`.
  - Each repeated argument (`union-borrow-native`,
    `implicit-assignment-demand-v1/native` and
    `selected-index-captures-v1/native`) repeats within one test function.
  - Wrappers taking a folder argument (`variadic_calls`, `hidden_values`,
    `iterators` and `composites`) pass distinct literals.
- **`qualify::qualify`.** Name collisions exist: for example `assignment-add`
  appears in both `language_operators` assignment tests. They are harmless,
  because `qualify_at` already isolates each test thread's directory.

So C2 has no separate race hunk.

### Flip survey (C2)

The survey (`obj/parallel/auto-traits/flip2/`) ran two CLIs:

- the baseline, `89cb62ee5` (`baseline/rustgm`, sha256 `d97cd78e`);
- C2 on that base (`candidate/rustgm`, sha256 `f6e547df`).

The candidate predates the rebase and two later source edits: the removal of
the two subsumed checks above, and the reorder in `automatic_trait`. Neither
edit changes an admitted packet's outcome, and the integration controls
assert all 27 outcomes on the final tree.

Each CLI exported every pa29 spec provider under versions 10, 12 and 13, then
ran every fixture client against each export: 32 providers, 96 exports and
1,116 runs. Only the new `auto-impls` packet under version 13 differs:

- The provider exports, where the baseline stops at "source
  const/negative/unsafe implementation requires a wider explicit profile".
- The seven accepted clients are rejected→accepted, each matching rustc's
  exit 42.
- The nineteen opponents stay rejected, now under their declared messages
  instead of the baseline's missing artifact. That includes `freeze-impl`'s
  `freeze_impls` text.

Every other run's status, stderr and LowIR are byte-identical. That includes
`auto-declarations` and every packet that exports Freeze, Send or Sync.

- There is no accepted→rejected flip.
- Versions 10 and 12 accept 6 and 14 runs before and after. Under both, the
  auto-impls provider stops at the same earlier gate in either CLI ("named,
  associated or generic alias transport requires its wider profile").
- Version 13 accepts 186 runs before and 194 after.

### Evidence (C2)

Every run below used the final tree on `6cd9f9e82`. Receipts are under
`obj/qualification/fast/`, and each passed.

Non-native results:

| Selection | Tests | Receipt |
| --- | --- | --- |
| auto-trait source and object regressions (`auto_markers`, `automatic_sources`, `auto_sources`, `principal_auto_sources`) | 13 | `1794157462680335183` |
| linking suite, part 1 | 49 | `1794157693223908798` |
| linking suite, part 2 | 48 | `1794157764184973565` |
| linking suite, part 3 | 48 | `1794157850622850781` |
| linking suite, part 4 | 48 | `1794157945271367588` |
| linking suite, part 5 | 48 | `1794158105932999415` |
| lib cases of the touched modules | 180 | `1794157217608785461` |
| private source-client cases, part 1 | 85 | `1794158200322054941` |
| private source-client cases, part 2 | 84 | `1794158236866668950` |
| course text pa1-pa12 | passed | `1794158423718631953` |
| course text pa13-pa19, pa21-pa24 | passed | `1794158529328923205` |
| course text pa20 | passed | `1794158602934725074` |

- The five linking parts are the suite inventory's 241 non-ignored tests: C1's
  237 plus the four non-native `polarity_transport` tests.
- The 180 lib cases are every test under the modules C2 touches or reads:
  `declarations::traits::implementations` (including `fence_tests` and
  `scaling_tests`), `traits::implementations`, `crates::typed_headers`,
  `associated`, `declaration_inputs`, `nominals` and
  `support::crates::generics`, plus `freezing::payment_tests`,
  `negative_equalities` and `auto_cycle_tests` in `selection::search`.
- The private source-client cases are C1's two parts, unchanged.

`make test-upstream` verified 128 pinned fixtures and 126 Reference files.
`make test-harness` found 464 roots in 13 binaries and no orphan fixtures.

Native results (`--ignored`, reported separately):

| Selection | Tests | Receipt |
| --- | --- | --- |
| linking suite native, part 1 | 25 | `1794158273163184274` |
| linking suite native, part 2 | 25 | `1794158334929366028` |

The two parts are the inventory's 50 ignored tests: C1's 49 plus
`auto_impl_clients_execute_natively`, in which the seven accepted clients exit
42 with empty stdout. With the non-native parts, that is all 291.

Oracles, run before the candidate:

- `obj/parallel/auto-traits/oracle/c2`: rustc 1.98.0 gives all 27 declared
  outcomes, and the seven accepted clients exit 42 natively;
- `obj/parallel/auto-traits/oracle/c2gen`: E0367, E0366 and E0321, and the
  older control compiles;
- `obj/parallel/auto-traits/explore/c2sized/oracle`: the Sized blanket
  provider compiles and its client exits 42.

### Owner review (C2)

At the join, owners review:

- `support/crates/generics/free_protocols.rs::automatic_implementation` and
  its `ordinary_profile.rs::validate_with` caller;
- `sema/crates/typed_headers/staged_headers/impl_headers.rs::automatic`;
- `sema/crates/typed_headers/mapped_source/origin_ports.rs::Graph::automatic_implementation`
  and `owner_members.rs::Graph::automatic_trait`;
- `declarations/traits/implementations/portable_impls.rs`:
  `imported_automatic_owner` and `validate_imported_auto_implementation`;
- `declarations/traits/implementations/automatic_domains.rs`: `Site`,
  `automatic_subject`, `automatic_sized_subject` and `imported_sized_bound`;
- `declarations/traits/implementations/negative.rs`: `negative_coverage`,
  `validate_imported_negative_header` and `imported_nominal_inputs` (C2b's
  workaround);
- `traits/selection/search/automatic_traits.rs`: `predicate`,
  `require_automatic_header` and `Search::automatic_result`, the selection
  lift;
- `traits/selection/search/freezing.rs`: `object_marker` and `has_source`,
  now `Result`;
- `declarations/traits/implementations.rs::validate_source_impl_policy` and
  `negative_equalities.rs::custom`.

## C3: unsafe traits and non-auto negative impls

C3 is the third commit of this item, developed on main `078d3f238`. It
transports unsafe traits that are not auto and every trait impl's own safety
and polarity, so core's negative impls of traits with members and its unsafe
marker and method traits export.

### Wire rule (C3)

The integrator approved these shapes before code:

- **Version 13 only; no new tag, bit or row.**
  - `trait_headers` admits `UNSAFE` on a trait that is not auto. `CONST`
    stays rejected.
  - The impl loop admits an impl's `unsafe` and `negative` bytes for every
    trait impl (`trait_application.is_some()`). `constant`, and modifiers on
    inherent impls, stay rejected.
  - Both read one fact, `Bindings::languages` (renamed from `automatic`,
    `profile == BindingProfile::Languages`).
  - Versions 12 and 10 keep "source const/auto/unsafe traits require a wider
    explicit profile" and "source const/negative/unsafe implementation
    requires a wider explicit profile".
- **The ledger's zero-member rule, every version**
  (`membership_rows.rs::application`). A negative impl maps no member; one
  that maps any is "auto or negative trait implementations cannot declare
  members". A positive impl keeps the member-count rule, "source
  implementation omits an actual trait member mapping".
  - Declared older-version text change: a negative-only provider whose
    negative impl's trait has members, exported under v12 or v10, now stops
    at the impl fence instead of the ledger.
- **Unsafe-impl methods** (the item-4 amendment, as corrected). Under version
  13 `conversion_profile::method` admits a method of an unsafe impl, through
  an `unsafe_impls` argument that `associated_method` passes from the same
  `Bindings::languages` the impl loop reads. `primitive_method` keeps
  refusing unsafe impls: it refuses every method with a receiver, so
  `haystack` was always a conversion method. Under version 13 every primitive
  method is also a conversion method, so its `Ok(false)` falls through. The
  receiver-less `make` of both unsafe Searcher impls proves it. Negative and
  constant stay rejected in both families.
- **Each consumer re-proves safety for every imported impl**
  (`Graph::implementation`, `origin_ports.rs`), literally
  `required_unsafe = trait.UNSAFE && !negative; unsafe == required_unsafe`,
  whose doc comment names E0199, E0200 and E0198. It runs at staging,
  reservation and conformance, and its text is "selected impl safety/member
  kinds disagree with its actual trait".
- **Non-auto negatives complete as `NegativeValidated`**
  (`portable_impls.rs::validate_imported_trait_implementation`), after the
  auto branch and before method planning, with no requirement, member or
  coverage proof, as rustc does. They are never positive evidence
  (`refutation.rs`, unchanged).
- `Graph::empty_module_trait` admits `UNSAFE` under version 13.

### Removed as subsumed

Each was approved under C2's rule: an earlier check rejects the same rows on
every path.

- `staged_headers/impl_headers.rs::prepare`'s modifier clause and its
  `automatic` helper: admission's impl loop decides the same bytes first.
- `free_protocols.rs::automatic_implementation` and both Q5 texts ("imported
  foreign auto-trait impl requires its multi-packet profile"): the ledger
  rejects a foreign trait application first ("source implementation requires
  its exact supported trait header").
- `Graph::automatic_implementation`'s Q5, safety, members and constant
  checks: safety moved to `Graph::implementation`, which it calls first; the
  ledger owns members; admission owns constant. Its version-13 and exact-auto
  checks remain, so `polarity_fences`' four direct-call texts pass unchanged.
- `portable_impls.rs`'s "negative imported conformance requires its wider
  profile": every negative now completes before that point.

### Corrections recorded before code

- **Correction A.** The recorded "same-packet same-head coherence pair limit"
  named the wrong path: `contract_completion.rs:90` is the Provided-contracts
  consumer only. In the ordinary consumer, the coherence walk re-proves an
  imported packet's same-head pair, and `disjointness.rs::absence_closed`
  closes absence only for a trait local to the consumer. Core's
  `impl<T: Error + ?Sized> Error for &T` beside `impl !Error for &str` is
  therefore a generated control, declared rejected; C3b closes it.
  - C3b corrected this: the consumer re-proved the imported pair with no
    conditions at all, not with conditions it could not close
    ([imported conditions](#imported-conditions-in-coherence-correction-found-by-the-candidate)).
- **Correction B.** A trait-impl method on a nominal Self already exports
  under version 13 (`conversion_profile::method`). The real limit is a
  trait-impl method under an impl that captures a lifetime (`&T`, `&mut T`),
  which fails in the exporter with "source function early/captured callable
  frame transport is not implemented". Core's `DerefMut for &mut T` and
  `Error for &E` need it lifted. The provider's negative impls on references
  map no member, and its method impls are on `u8` and `Counter`, so clients
  stay unrestricted.
- **`may_dangle`.** `#[may_dangle]` is rejected before export ("unresolved
  attribute macro is not in path scope"), so no export fence is reachable. It
  is recorded under the open item "may_dangle frontend dropck (blocks alloc)"
  with its three parts: resolving the attribute, the unsafe-Drop safety rule,
  and the dropck opt-out. No fence is added.

### Fixtures and outcomes (C3)

`pa29/tests/unsafe-negative-impl-outcomes.json` was declared at 19:16:33Z,
after the rustc oracle and baseline probes and before any candidate run. It
holds 17 outcomes: the provider export and 16 clients. The provider is
`#![feature(negative_impls)]`, the lifetime-nominals provider, and a tail of
authored impls shaped as core's (see the packet README).

The receiver-less `fn make() -> u8` of `Searcher`, and its two unsafe impls,
are integrator condition 2, declared before the oracle as fixture version 4.
Every earlier outcome was unchanged.

| Client | rustc 1.98.0 | RustGM |
| --- | --- | --- |
| trusted, local-trusted, exclusive, error-fault, local-negative | exit 42 | exit 42 |
| error-str, iterator-slice, fuse-plain, searcher-fault | E0277 | "trait obligation requires an unambiguous matching assumption or implementation" |
| deref-shared, clone-mut | E0277 | "conditional negative implementation requires scoped debt payment" |
| safe-searcher, unsafe-plain, unsafe-negative | E0200, E0199, E0198 | "trait and implementation safety declarations disagree" |
| deref-positive | E0751 | "overlapping trait implementation headers require coherence proof" |
| negative-coherence | exit 42 | "overlapping trait implementation headers require coherence proof" (recorded gap) |

- Every candidate text was first observed by compiling the client's body with
  the provider as one source core on the baseline CLI. On main the provider
  export itself stops at the ledger.
- `deref-shared` and `clone-mut` share `arguments-send`'s reason: the
  refutation of a negative impl whose header names a region stays
  conditional.
- **`negative-coherence` is a frontend gap, independent of transport.** It
  fails with every item local, too. `pay_negative` (`overlap/inputs.rs`)
  pays only Sized premises, so a `?Sized` (MetaSized) negative impl's premise
  is never paid from the intersection's own MetaSized assumption. The
  narrowest repro is the provider's lines 2-58 plus one line:

  ```rust
  pub trait A {} impl<T: ?Sized> !A for &T {} pub trait B {} impl<T: A> B for T {} impl<T: ?Sized> B for &T {}
  ```

  | Negative impl's `T` | `B for &T`'s `T` | rustc 1.98.0 | RustGM |
  | --- | --- | --- | --- |
  | `?Sized` | `?Sized` | accepted | overlap text |
  | `MetaSized` | `MetaSized` | accepted | overlap text |
  | `PointeeSized` | `?Sized` | accepted | passes coherence |
  | `PointeeSized` | `PointeeSized` | accepted | passes coherence |
  | `Sized` | `Sized` | accepted | passes coherence |

  - The baseline gave this table. On the C3 candidate the `?Sized` and
    `MetaSized` rows keep the overlap text, and the other three export.
    The `Sized` row is a single-crate composite over the full provider.
  - PA31's `ordinary-negative-mode.rs` spells the same shape against real
    core and passes.
  - The integrator assigned the repair to Lane B, after chain-bytes. It is
    not a core blocker today, and it flips this client.

Generated controls (`crate_linking::modifier_transport`):

| Control | rustc | RustGM |
| --- | --- | --- |
| the provider plus `impl !Error for &str {}`, and an answer client | compiles; exit 42 | the export is accepted; the client is rejected with "overlapping trait implementation headers require coherence proof" (C3b) |
| the foundation plus `pub unsafe trait TrustedFused {}` | compiles | versions 12 and 10: "source const/auto/unsafe traits require a wider explicit profile", verbatim as on main; version 13 exports |
| the foundation plus `impl<T: PointeeSized> !Clone for &mut T {}` | compiles | versions 12 and 10: "source const/negative/unsafe implementation requires a wider explicit profile"; version 13 exports |

- The last row is the declared older-version text change. On main the ledger
  rejected that provider first under all three versions ("source
  implementation omits an actual trait member mapping").
- The full provider is not an older-version trait-fence control. Under
  versions 12 and 10 it stops first at "named, associated or generic alias
  transport requires its wider profile", as observed on main for the provider
  without its negative impls.

Wire controls (`forged_impl_modifiers_reject_in_fresh_consumers`):

- Making `Plain for Counter` negative moves one byte. That calibrates the
  52-byte impl header table, and every tail row's flags are checked.
- Making `Iterator` unsafe moves one byte, which calibrates the trait header
  table. That variant is generated in the test's output folder and is not a
  fixture. The honest packet's TrustedFused and Searcher carry `UNSAFE`, and
  Plain and Error do not.
- Each forged packet is consumed by `trusted`, and the honest packet is
  accepted.

| Forgery | Fresh consumer |
| --- | --- |
| `unsafe` set on `Plain for Counter` | "selected impl safety/member kinds disagree with its actual trait" |
| `unsafe` cleared on `TrustedFused for Counter` | the same |
| `unsafe` set on `!Searcher for Fault` | the same |
| `UNSAFE` cleared on the TrustedFused and Searcher trait headers | the same |
| `negative` set on `Searcher for u8`, which maps its members | "auto or negative trait implementations cannot declare members" (admission) |
| `negative` cleared on `!DerefMutLike for &T`, which maps none | "source implementation omits an actual trait member mapping" (admission) |
| version relabelled 13 to 12 | "unknown source declaration kind" |

Every declared outcome held on the first candidate run.

### A C2 control's text (declared before the candidate)

C2's `rc-member` forgery widens `!Send for Rc`'s member range over one
existing link. C2 observed the ledger's count text, before its own declared
"auto or negative trait implementations cannot declare members". C3's
zero-member rule runs first for a negative impl, so the control now gives the
declared text. That change was declared before the candidate run and is
recorded beside the control in `auto-impl-outcomes.json`. The consumer's
member check it backed is removed as subsumed (above).

### Flip survey (C3)

The survey (`obj/parallel/auto-traits/flip3/`) ran two CLIs:

- the baseline, `89cb62ee5` (`baseline/rustgm`, sha256 `d97cd78e`);
- C3 on `078d3f238` (`candidate/rustgm`, sha256 `f35ec380`), the final
  source tree.

Each CLI exported every pa29 spec provider under versions 10, 12 and 13, then
ran every fixture client against each export: 33 providers, 99 exports and
1,167 runs.

- Against C2's candidate outputs (`flip2/b`), every run of the 32 earlier
  packets is byte-identical in status, stderr and LowIR. So C3, and the
  Lane A and B commits it was rebased over, change no earlier packet's
  outcome. Against the baseline, `auto-impls` differs exactly as C2's survey
  recorded.
- In the new packet under version 13, the provider and the five accepted
  clients are rejected→accepted, each matching rustc's exit 42. The eleven
  opponents stay rejected, now under their declared messages instead of the
  baseline's missing artifact.
- Under versions 12 and 10 the new provider stops in both CLIs at "named,
  associated or generic alias transport requires its wider profile", byte for
  byte.
- There is no accepted→rejected flip. Versions 10 and 12 accept 6 and 14
  runs before and after; version 13 accepts 186 before and 200 after (C2's
  194 plus six).

### Evidence (C3)

Every run below used the final source tree, on `078d3f238`. Receipts are under
`obj/qualification/fast/`, and each one passed.

The focused run came first: receipt `1794165958734543023`, 6 tests. Those were
the three non-native `modifier_transport` tests, `polarity_transport`'s forged
and frontend tests, and `polarity_fences`.

Non-native results:

| Selection | Tests | Receipt |
| --- | --- | --- |
| lib cases of the touched modules | 217 | `1794166593368191022` |
| auto-trait source and object regressions (C2's) | 13 | `1794166638010305683` |
| linking suite, part 1 | 49 | `1794166697505251444` |
| linking suite, part 2 | 49 | `1794166800647009458` |
| linking suite, part 3 | 49 | `1794166945528830683` |
| linking suite, part 4 | 49 | `1794167120752580611` |
| linking suite, part 5 | 48 | `1794167202143352611` |
| private source-client cases, part 1 | 85 | `1794167271729332335` |
| private source-client cases, part 2 | 84 | `1794167402636457147` |
| course text pa1-pa12 | passed | `1794167615954520541` |
| course text pa13-pa19, pa21-pa24 | passed | `1794167663321503171` |
| course text pa20 | passed | `1794167742423160844` |

- The 217 lib cases are every test under the modules C3 touches or reads:
  `declarations::traits`, `traits::implementations`, `crates::typed_headers`,
  `associated`, `declaration_inputs`, `nominals` and
  `support::crates::generics`, plus C2's `selection::search` extras.
- The five linking parts are the suite inventory's 244 non-ignored tests:
  C2's 241 plus the three non-native `modifier_transport` tests.
- The private source-client cases are C1's two parts, unchanged.

`make test-upstream` verified 128 pinned fixtures and 126 Reference files.
`make test-harness` found 464 roots in 13 binaries and no orphan fixtures.

Native results (`--ignored`, reported separately):

| Selection | Tests | Receipt |
| --- | --- | --- |
| linking suite native, part 1 | 26 | `1794167442918997482` |
| linking suite native, part 2 | 25 | `1794167538574612700` |

The two parts are the inventory's 51 ignored tests: C2's 50 plus
`modifier_clients_execute_natively`, in which the five accepted clients exit
42 with empty stdout. With the non-native parts, that is all 295.

Oracles:

- rustc 1.98.0 first ran over the scratch fixture, before the outcomes were
  declared and before any candidate run. It gave every declared outcome: the
  provider compiles, six clients exit 42 natively (the five accepted and
  `negative-coherence`), six give E0277, and the others give E0751, E0200,
  E0199 and E0198.
- For this record it ran again over the committed bytes, which are identical
  to the scratch fixture, with the same results:
  `obj/parallel/auto-traits/oracle/c3` (`c3.log`, provider sha256
  `96075b0ed139d472`).
- `obj/parallel/auto-traits/oracle/c3gen`: the generated controls are built
  as the test builds them. The pair provider compiles and its answer client
  exits 42, and the unsafe-only and negative-only controls compile.
- The negative-coherence variants and the narrowest repro were run on the
  baseline before C3. That record is in the C3 entry of
  `docs/library-build-blockers.md`.

### Owner review (C3)

At the join, owners review:

- `support/crates/generics/source_validation/header_ledger/membership_rows.rs::application`:
  the zero-member rule for negative impls, in every version;
- `support/crates/generics/ordinary_profile.rs`: `Bindings::languages`, the
  impl loop, `trait_headers`' `UNSAFE` admission and `associated_method`'s
  gate;
- `support/crates/generics/conversion_profile.rs::method`: its
  `unsafe_impls` argument;
- `sema/crates/typed_headers/mapped_source/origin_ports.rs`:
  `Graph::implementation`'s safety rule, `applied_trait_flags`, and the
  reduced `Graph::automatic_implementation`;
- `sema/crates/typed_headers/mapped_source/owner_members.rs::Graph::empty_module_trait`;
- `declarations/traits/implementations/portable_impls.rs`:
  `validate_imported_trait_implementation`'s negative completion,
  `imported_negative`, and `imported_receiver_environment` without its
  polarity fence;
- the removals in `staged_headers/impl_headers.rs::prepare` and
  `free_protocols.rs`.

## C3b: knowable absence

C3b is the fourth commit of this item, developed on main `bfbf43049` and
rebased onto `2a27e11ca`, which carries the v3 backend lock. It proves
coherence absence for the crate whose coherence a pair proves, as
rustc 1.98's `trait_ref_is_knowable` does, so core's same-head pair
`impl<T: Error + ?Sized> Error for &T` beside `impl !Error for &str` is
accepted by its consumers. There is no wire change.

### Rule (C3b)

The integrator approved this design before code:

- **The perspective** (`implementations/perspective.rs`). A pair's
  perspective is `Packet(a)` when both impls were imported from origin-catalog
  artifact `a`, and `Local` otherwise: a client's own pair, a client impl
  against an imported one, and, until a multi-packet profile records which
  packet is downstream, a pair across two packets. `overlap::instantiated`,
  `inherent` and `iteration::possible` compute it from the two owners;
  `reference_comparison` (a source impl against the canonical blanket) is
  `Local`. `Packet` holds the artifact's ordinal as a `u16`: a catalog holds
  at most `MAX_DEPENDENCIES` (1,024) artifacts, and a larger ordinal is an
  error, never a truncation.
- **The purpose carries it.** `Purpose::Coherence` and
  `Purpose::NegativeCoherence` hold the perspective, and every frame
  completion's `coherence_absence` passes it to `absence_closed`.
- **Under a packet's perspective the consumer's own source impls are never
  candidates** (`append_candidates`). Absence is decided over what that
  packet could see, as its own coherence decided it. Other packets' impls
  stay candidates, which can only add presence.
- **`absence_closed` is knowable** when, as before, the inputs are closed or
  the goal is concrete, and one of:
  - the perspective owns the trait (`Local`: today's source-owner check;
    `Packet(a)`: imported from `a`);
  - the trait is a closed language trait (`associated/language.rs`,
    unchanged);
  - the trait is `#[fundamental]` (`header_attributes::fundamental_policy`).
    rustc 1.98's are `PointeeSized`, `MetaSized`, `Sized`, `FnOnce`, `FnMut`,
    `Fn` and `Coroutine`, and its fundamental nominals are `Pin`,
    `FieldRepresentingType` and alloc's `Box`;
  - the perspective owns any input root, Self and then the trait arguments,
    after peeling `&`, `&mut` and `#[fundamental]` nominals
    (`disjointness.rs::owns_input`, one unit of work per visited term).
- **Imported conditions.** Coherence reads an imported impl's where-clauses
  from the same original inputs selection does (see the correction below).
- **Q1, any input root.** rustc takes the first owned input with no
  uncovered parameter before it. Closed inputs have no uncovered parameter,
  so any owned input is that first one and the order is moot. A widening to
  open inputs must restore the order; the doc comment says so.

### Fixtures and outcomes (C3b)

`pa29/tests/knowable-absence-outcomes.json` was declared at 20:27:56Z, after
the rustc oracle and baseline probes and before any candidate run. It holds
14 outcomes: the provider export and 13 clients. The provider is C3's
provider plus `impl !Error for &str {}` (see the packet README).

| Client | rustc 1.98.0 | Baseline (C3 CLI) | C3b |
| --- | --- | --- | --- |
| answer, answer-local-impl | exit 42 | overlap text, unlocated | exit 42 |
| local-error-ref, local-mut-ref, local-copy, trait-argument | exit 42 | overlap text, unlocated | exit 42 |
| foreign-self, uncovered | E0119 | overlap text, unlocated | overlap text at offset 101 |
| tuple-self, nonfundamental-wrapper | E0119 | overlap text, unlocated | overlap text at offset 115 |
| local-uncovered | E0119 | overlap text, unlocated | overlap text at offset 108 |
| orphan-blanket, orphan-str | E0210, E0117 | "library trait implementation requires a local type" at 52 | the same |

- The overlap text is "overlapping trait implementation headers require
  coherence proof". On the baseline every non-orphan client fails at the
  imported pair, which has no source key. Under C3b each opponent's failure
  is located at its own client impl, and the integration test asserts each
  offset.
- Against the C3 packet, which has no pair, the baseline accepted `answer`
  and `answer-local-impl` and rejected the four local-ownership clients at
  their own pairs: the knowable-absence gap for a client's own types.
- `nonfundamental-wrapper` uses the provider's existing plain
  `pub struct Slot<T> { pub value: T }`, which is not `#[fundamental]`.
- `answer-local-impl` is integrator condition 3: an unrelated local
  `impl base::Error for Local {}` beside the pair, which must not enter the
  packet's proof.
- Under versions 12 and 10 the provider stops at "named, associated or
  generic alias transport requires its wider profile", as C3's does.

### C3's pair control (declared before the candidate)

C3's generated pair control in `crate_linking::modifier_transport` declared
the answer client rejected, as the gap C3b closes. Its `c3b_amendment` in
`unsafe-negative-impl-outcomes.json`, recorded before the first C3b
candidate run, declares the export and the client accepted. The test now
checks the client with `artifacts::checked`.

### Imported conditions in coherence (correction found by the candidate)

The first candidate run rejected every non-orphan client at the imported
pair, still unlocated. A debug trace on `answer` found the cause. The pair's
perspective was `Packet(0)`, as designed, but neither imported impl
contributed a condition: `Inputs::conditions` and `reserve_traits` read
`facts.generics.predicates_in_scope(owner)`, which holds only source
predicates. `absence_closed` was never reached, and the pair stayed possible.

- **Correction to C3's Correction A.** The consumer re-proved an imported
  pair with no conditions at all, not with conditions it could not close.
  `impl<T: Error + ?Sized> Error for &T` overlapped `impl !Error for &str`
  because its `T: Error` was missing.
- **The fix, approved by the integrator.** Coherence now reads the same
  imported original inputs selection does. Both loops read
  `declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)`:
  the source table, then the imported owner's original inputs
  (`Linkage::original_inputs`). Selection's impl conditions
  (`selection/instantiation.rs`) read the declared view of the same inputs.
- Every consumer now re-proves imported pairs with their real where-clauses.
  That can only exclude more pairs, so the flip survey's accepted flips are
  each checked against rustc.

Generated controls (`absence_transport::generated_controls_read_imported_conditions`)
were declared at 22:29:45Z, after the first candidate run and before their
rustc oracle and any candidate run of them. The seal provider is the packet's
provider plus `Seal` with `impl<T: ?Sized> !Seal for &T`, `Mark` with
`impl<T: PointeeSized> !Mark for &T`, and the blankets
`impl<T: Seal + ?Sized, S> Holds<T> for S` and
`impl<T: Mark + ?Sized, S> Keeps<T> for S`. Each blanket's condition names the
provider's own trait with a `?Sized` parameter.

| Control | rustc 1.98.0 | RustGM |
| --- | --- | --- |
| condition-holds: `impl base::Error for Local {}` and `impl base::Error for &Local {}`, so the imported blanket's `Local: Error` holds | E0119 | overlap text at the client's `&Local` impl |
| the seal provider | compiles | exports under version 13 |
| negative-metasized: a `with_negative_coherence` client, `impl base::Holds<&'static str> for Local {}` | exit 42 | accepted (declared rejected; a declaration error, below) |
| negative-metasized-param: `impl<U: ?base::Sized> base::Holds<&U> for Local {}`, with the feature | exit 42 | overlap text at the client impl (Lane B's MetaSized gap) |
| negative-pointee-param: `impl<U: ?base::Sized> base::Keeps<&U> for Local {}`, with the feature | exit 42 | accepted |
| negative-pointee: `impl base::Keeps<&'static str> for Local {}`, with the feature | exit 42 | accepted |
| stable-pointee: that impl without `with_negative_coherence` | E0119 | overlap text at the client impl |

- `condition-holds` is the integrator's non-disjoint control. An honest
  packet cannot carry an overlapping pair of its own, because the provider's
  coherence rejects it before export, so the control pairs the imported
  blanket with a client impl. Its rejection is at byte 96, the client's
  `&Local` impl.
- **A declaration error that rustc confirms.** `negative-metasized` was
  declared rejected as Lane B's gap. The candidate accepted it (status 0),
  and rustc's oracle, run before the candidate, exits 42. The `?Sized`
  negative impl's premise there is `str: MetaSized`, which is concrete, so
  the control never reaches `pay_negative`'s intersection-parameter gap. It
  is not a behaviour change. The amendment keeps `declared_before_run` beside
  the amended outcome, and the control stays as a check that an imported
  `?Sized` condition does not break.
- **`negative-metasized-param` is the parametric repro for Lane B.** The
  negative impl's premise `U: MetaSized` is covered only by the client impl's
  own `U: ?Sized`, and `pay_negative` pays only Sized premises. It is a known
  rejects-valid, and Lane B's MetaSized fix flips it. Both parametric
  controls were declared at 22:41Z, before their oracle and candidate runs.
  Their first oracle run stopped at E0405, because a `no_core` client must
  spell `?base::Sized`. The texts were corrected before any candidate run,
  with the declared outcomes unchanged; the JSON's `spelling_note` records it.

### Scaling (C3b)

`tests/invariants/absence_scaling.rs`, mounted in `overlap::disjointness`,
appends N generated traits to the provider, each with
`impl<T: E + ?Sized> E for &T {}` and `impl !E for &str {}`, for N = 8, 16
and 32. An empty client consumes each export. Packet bytes, the client's
original header-budget work and the coherence steps of re-proving every
same-trait pair grow by at most a constant per pair: each doubling stays
within `2 * previous + 64`.

### Limits and recorded items (C3b)

- **Conservative limits.**
  - A pair across two packets is proved for the consumer until a
    multi-packet profile records which packet is downstream.
  - `dyn` of an owned trait and foreign types are never owned inputs.
  - A packet cannot carry a `#[fundamental]` struct today (export stops at
    "general fundamental policy requires authenticated trait/nominal
    metadata transport"), so an imported nominal is never peeled. Imported
    fundamental traits are the sizing traits, through their transported
    FUNDAMENTAL flag.
- **Diagnostic quality** (not fixed in C3b). An imported pair's coherence
  failure is unlocated (`input:`). C3b accepts honest pairs, but a dishonest
  pair in a forged packet would still be reported that way.
- **Out of scope.** MetaSized negative coherence stays Lane B's item;
  `pay_negative` is unchanged. `negative-metasized-param` records the gap
  with imported conditions.
- **The search query's layout.** Because a coherence purpose carries its
  perspective, `Query` grows by one word, from 680 to 688 bytes in the test
  build. `tests/invariants/sharing_sizing.rs` now asserts
  `664 + size_of::<Vec<usize>>()` in place of 656, with a comment giving the
  reason. `Node` (160) and `Frame` (1,312) are unchanged. A `u32` ordinal
  gives the same 688.

### Lane B's `overlap/inputs.rs` delta

Lane B's MetaSized item rebases onto this signature change:

- `Inputs::excluded` gains a trailing `perspective: Perspective` parameter.
- `Inputs::has_absent_local_trait` gains a trailing
  `perspective: Perspective` and passes it to `absence_closed` and
  `may_overlap`.
- `may_overlap` and `may_overlap_with` (`search/coherence.rs`) gain
  `perspective: Perspective` just before the `pay` closure.
- The three closures that call `pay_negative` are unchanged inside; each
  call site passes the one extra argument beside its closure: `excluded`'s
  negative branch (its own `perspective`), `overlap.rs::reference_comparison`
  (`Perspective::Local`) and `overlap/iteration.rs::possible` (the pair's
  perspective).
- `pay_negative`'s signature and premise logic are unchanged.
- Body changes, with no further signature change:
  - `Inputs::conditions` and `Inputs::reserve_traits` each iterate
    `declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)?`
    instead of `facts.generics.predicates_in_scope(owner)`. The loop bodies are
    unchanged; each item's `.value` is the same `Clause`.
  - `use crate::sema::declaration_inputs;` is added.
  - `has_absent_local_trait`'s doc comment now names the perspective.
- `negative-metasized-param` is the parametric repro for Lane B's MetaSized
  item (above).

### Flip survey (C3b)

The survey (`obj/parallel/auto-traits/flip4/`) ran two CLIs:

- the baseline, C3's `531b1f839` (`candidate/rustgm`, sha256 `f35ec380`);
- C3b on `bfbf43049` (sha256 `16c1adb8`), with the imported-conditions
  correction, before the rebase.

Each CLI exported every pa29 spec provider under versions 10, 12 and 13, then
ran every fixture client against each export: 34 providers, 102 exports and
1,209 runs. The expected flips were declared before the survey, and every
accepted flip had to match a rustc outcome from an oracle run declared
before it.

- **Only `knowable-absence` under version 13 differs.**
  - Six clients go from rejected to accepted, each matching rustc's exit 42
    in the frozen oracle: answer, answer-local-impl, local-error-ref,
    local-mut-ref, local-copy and trait-argument.
  - The five E0119 opponents stay rejected. Their text moves from `input:`
    to their own impl: foreign-self and uncovered at 101, tuple-self and
    nonfundamental-wrapper at 115, local-uncovered at 108.
  - The orphan clients are byte-identical.
- In the other 33 packets every run's status, stderr and LowIR is
  byte-identical. So reading imported conditions, and the perspective, change
  no earlier packet's outcome.
- There is no accepted→rejected flip and no unconfirmed acceptance.
  - Version 13 accepts 201 runs before and 207 after.
  - Version 12 accepts 14 runs both times, and version 10 accepts 6.

- **On the final tree.** The final candidate (`cand3b/rustgm`, sha256
  `7cc3e3ef`) was built after the rebase onto `2a27e11ca` and the `u16`
  ordinal. It reran the same survey, declared byte-identical to the
  pre-rebase candidate's outputs before it ran, and it was: `diff -rq` over
  all 3,670 files (every run's status, stderr and LowIR) is empty. The
  upstream commits since `bfbf43049` change no pa29 spec outcome.

### Evidence (C3b)

Every run below used the C3b tree rebased onto `2a27e11ca`. Receipts are under
`obj/qualification/fast/`, and each one passed. The backend was first
reconfigured to the v3 lock: `setup_backend.py` configured all three locked
tools, and `make test-backend` passed its LowIR, object and assembly
(`movntps`) handshakes.

The focused run is receipt `1794182673981448852`, 8 tests:
`absence_transport`'s two non-native tests, `modifier_transport`'s two
non-native tests, and four lib cases. The lib cases are the scaling test,
`future_tests`' absence case, `search::coherence`'s cutoff case and the
sizing test. It ran after the last source edit, a doc comment in
`perspective.rs`. The batch below ran on the tree just before that comment,
with identical code.

Non-native results:

| Selection | Tests | Receipt |
| --- | --- | --- |
| lib cases of the touched modules | 409 | `1794179988417894818` |
| `declaration_inputs`' lib cases | 2 | `1794181902711437560` |
| auto-trait, inherent-overlap, impl-parameter and arbitrary-method roots | 19 | `1794180063331985699` |
| tail-overlap, specialization, projected-condition and call-trait roots | 8 | `1794180163621159709` |
| linking suite, part 1 | 50 | `1794180325839883901` |
| linking suite, part 2 | 49 | `1794180627997622391` |
| linking suite, part 3 | 49 | `1794180737614682794` |
| linking suite, part 4 | 49 | `1794180829910078375` |
| linking suite, part 5 | 49 | `1794180899725256456` |
| private source-client cases, part 1 | 85 | `1794180986163705941` |
| private source-client cases, part 2 | 84 | `1794181026720219038` |
| course text pa1-pa12 | passed | `1794181241549094416` |
| course text pa13-pa19, pa21-pa24 | passed | `1794181290273560078` |
| course text pa20 | passed | `1794181364192735120` |

- The 409 lib cases are every test under `sema::traits` (326),
  `sema::declarations::traits` (61) and `resolve::body_origin` (22). They
  include the scaling and sizing tests. `declaration_inputs`, which coherence
  now reads, ran its two cases separately.
- The five linking parts are the suite inventory's 246 non-ignored tests:
  C3's 244 plus `absence_transport`'s two.
- The private source-client cases are C1's two parts, unchanged.

`make test-upstream` verified 128 pinned fixtures and 126 Reference files.
`make test-harness` found 464 roots in 13 binaries, 44 stages and no orphan
fixtures.

Native results (`--ignored`, reported separately):

| Selection | Tests | Receipt |
| --- | --- | --- |
| linking suite native, part 1 | 26 | `1794181066598271714` |
| linking suite native, part 2 | 26 | `1794181134091580700` |

The two parts are the inventory's 52 ignored tests: C3's 51 plus
`absence_clients_execute_natively`, in which the six accepted clients exit
42 with empty stdout. With the non-native parts, that is all 298.

Oracles (rustc 1.98.0):

- The fixture oracle ran before the outcomes were declared and before any
  candidate run, and gave every declared outcome. The provider compiles
  (sha256 `1aac461a2424ec09`). The six accepted clients exit 42 natively
  with no warnings. foreign-self, tuple-self, nonfundamental-wrapper,
  local-uncovered and uncovered give E0119, orphan-blanket E0210 and
  orphan-str E0117. The committed `.rs` files are byte-identical to the
  oracle's inputs.
- `obj/parallel/auto-traits/oracle/c3bgen` holds the generated controls,
  built as the test builds them (`oracle-c3bgen.sh`, `oracle-c3bparam.sh`).
  The provider and seal provider compile; condition-holds and stable-pointee
  give E0119; negative-metasized, negative-metasized-param,
  negative-pointee-param and negative-pointee exit 42. The test's generated
  sources are byte-identical to the oracle's.

### Owner review (C3b)

At the join, owners review:

- `sema/traits/implementations/perspective.rs` and
  `resolve/body_origin/declarations.rs::declaration_artifact`;
- `sema/traits/implementations.rs`: `Purpose`'s perspective and
  `append_candidates`' packet filter;
- `sema/traits/implementations/overlap/disjointness.rs`: `absence_closed`'s
  gates, `owns_trait` and `owns_input`;
- the perspective at each pair: `overlap.rs` (`instantiated`, `inherent`,
  `reference_comparison`), `overlap/iteration.rs`, `overlap/inputs.rs`
  (Lane B's delta, above) and `selection/search/coherence.rs`;
- `overlap/inputs.rs::conditions` and `reserve_traits`: imported impls'
  original inputs in coherence (the correction above);
- `tests/invariants/sharing_sizing.rs`: the `Query` assertion amended from
  656 to 664 plus the test log, one word for the purpose's perspective, and
  `Perspective::Packet`'s `u16` ordinal.

## Remaining gaps

- **C2b** (Lane C, next, before 5b): an imported nominal's transported
  where-clauses are not formation requirements in a client, so
  `fn hold(_r: base::Rc<u8, NotAlloc>)` is accepted against rustc's E0277.
  C2's `negative.rs::imported_nominal_inputs` is its temporary workaround.
- **Lifetime-capturing trait-impl methods** (a core transport blocker,
  unowned; a candidate for the multi-packet item): "source
  function early/captured callable frame transport is not implemented" for
  core's `DerefMut for &mut T` and `Error for &E`.
- **Negative coherence over a `?Sized` negative impl** (Lane B, after
  chain-bytes; not a core blocker): `pay_negative` pays only Sized premises.
  It flips C3's `negative-coherence`. It rebases onto C3b's
  `overlap/inputs.rs` signatures ([delta](#lane-bs-overlapinputsrs-delta)).
- **C3b's limits** ([C3b](#limits-and-recorded-items-c3b)): pairs across two
  packets are proved for the consumer until multi-packet; `dyn` and foreign
  types are never owned inputs; imported `#[fundamental]` nominals are never
  peeled; an imported pair's coherence failure is unlocated.
- **Conditional refutations' reasons.** `!Send for Arguments<'_>`,
  `!DerefMutLike for &T` and `!Clone for &mut T` stay conditional
  refutations, because their headers name a region. That is a
  negative-reasoning gap for owner selection (Lanes A and B), not a false
  acceptance.
- **Untested defense in depth.** The imported Sized evidence check (see C2),
  and `Graph::implementation`'s "imported conformance requires its own
  packet's actual trait", which the ledger's same-packet trait rule keeps
  unreachable.
- **Item 9** owns impls of another artifact's traits, behind the ledger's
  same-packet trait rule.
- dyn + Send waits on Lane B's tag 15.
- Recorded open items: `may_dangle` (frontend dropck; it blocks alloc):
  resolving the attribute, the unsafe-Drop safety rule and the dropck
  opt-out.
