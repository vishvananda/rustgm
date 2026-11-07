# Called constructors and imported variants

This is a Lane C checkpoint in `rustgm-async-work`, based on `00d41a267`. It
is a transport slice, not a library milestone. It makes no claim about an
original-core artifact or a whole assignment. It completes the first and third
enum-methods steps (called tuple-variant constructors, and consumer naming of
imported variants). Inherent enum methods remain open; see the gaps below.

## Design

**No new wire tag.** A called constructor `Opt::Some(x)` travels exactly as the
local IR has it: a `Function` value whose type is the function item of a
function application, a `Call` value, and a complete call request. The only
new fact is that, under version 13 (`module-bodies-v13`), the application may
name a kind-11 tuple variant instead of a function header. Older profiles
cannot carry variant declarations at all, so they still reject the provider by
name and keep their exact bytes.

**Provider export.** `obligation_rows/request_ports.rs` gives a constructor
call no late-region opening, because a constructor has no function header and
binds no late region. `lower/transported_source.rs` skips the synthesized
constructor bodies of local enums when it collects originals. They are not
authored code, and every consumer synthesizes its own. Before this change, a
monomorphic tuple variant (for example `Pair::Two`) made the whole export fail
with `source declaration has no reserved provided original`.

**Wire validation.** `generics::sum_contracts::Constructor::of` turns a
variant target into its validated variant row and its enum's nominal header.
It rejects a unit variant (`source unit variant has no called constructor`).
The validators then treat the constructor as a function target:

- `type_domains/node_rows.rs` counts the application's arguments against the
  enum's complete type slots.
- `domain_index.rs` adds a private `Target` with two cases, function header or
  constructor. The constructor case uses the enum's scope, inputs and Self,
  with no parent.
- `port_plan/applied_{debts,uses,signatures}.rs` read the target's scope and
  premises. The constructor's template signature is its payload field types,
  then the enum's Self, with safe Rust flags and no variadic.
- `body_ports/request_ports.rs` requires a constructor call to be safe, with
  no opening or method, no late binder, and parameter and argument counts
  equal to the payload length. Otherwise it fails with `source constructor
  Call changed its actual payload or safety`.

**Consumer import.**

- `staged_headers/template_ports.rs` registers each imported tuple variant in
  the consumer's callable templates, over its enum scope's own parameter IDs
  (`insert_constructor`, or `insert_monomorphic` for a non-generic enum). This
  is the same template a local constructor gets.
  `definition_cells::constructor_signature` builds the signature shared by
  the cell and the template, paying per payload field.
- `decoded_original::function` maps a constructor application to its variant
  declaration (`DefKind::Variant`). `scope_inputs` gives a variant no inputs
  and no parent scope, as a local constructor has. The enum's premises arrive
  through the well-formedness of the enum type.
- `binding_publication::variant_bindings` binds each imported variant once,
  in the value namespace of its enum's member scope. It pays one step per enum
  and per variant, and rejects a variant outside that scope or any duplicate
  key. Consumer paths (`base::Opt::None`), constructor calls
  (`base::Opt::Some(5)`) and patterns resolve through that binding only. They
  then use the local enum, pattern, guard and binding-mode code. Exhaustiveness
  is checked against the installed, re-authenticated variant table.
- `decoded_bodies::finish` records each constructor that a rebuilt body
  references, as local typing does. `build/constructors.rs` then synthesizes
  every referenced imported constructor as a template, reusing the local
  constructor body and contract builders. An imported variant has no syntax
  in the consumer, so its synthesized body is anchored at the consumer's crate
  root. Concrete constructor bodies and contracts stay local-only.

**Costs.** The work is linear. It charges one step per enum, variant and
payload field for templates, cells and bindings, one step per application and
call request during validation, and one step per rebuilt value when collecting
references. The 6/24/96 control below measures this.

## Evidence

Receipts are under `obj/parallel/enum-methods/`. Nothing was hand-edited or
blessed.

The fixtures are in `pa29/tests/spec/enum-methods/`. Their outcomes are in
`pa29/tests/enum-methods-outcomes.json`, and were declared before any oracle
or candidate run of a frozen file. The provider reuses lines 1-88 of
`spec/enum-match/provider.rs` unchanged.

**Independent Rust 1.98.0** (`rustc/receipt.txt`, `RUSTC_BOOTSTRAP=1
-Cpanic=abort --cfg rustc_oracle`; the provider object is linked). The oracle
took three rounds:

- **Round one.** Seven cases matched their declared outcomes. `guarded` hit a
  rustc internal compiler error (`fully_perform`).
- **Round two.** The provider gained `impl Clone/Copy for bool`. `guarded`
  still hit the same error at its `match rank(..)` scrutinee, whose type `i32`
  has no `Copy` impl in the no_core provider.
- **Round three.** `rank` returns `u8`. Every case now matches its declared
  outcome:
  - `called`, `mapped`, `named`, `guarded` and `by-ref` all exit 42;
  - `non-exhaustive` fails with E0004, `wrong-payload` with E0308 and
    `wrong-arity` with E0061.

The outcome file records the rounds. No declared outcome changed.

**Candidate.** `crate_linking::constructor_transport` checks the following:

- The packet carries `[13, 0]` and round-trips.
- All five clients reach LowIR in fresh consumers.
- The three opponents reject:
  - `non-exhaustive` with `non-exhaustive match`;
  - `wrong-payload` with `incompatible types`;
  - `wrong-arity` with `call argument count mismatch`.
- The provider rejects at v12 and v10 with `named, associated or generic alias
  transport requires its wider profile`.
- A native run (`--ignored`) of all five clients exits 42, matching rustc.

**Forged wires.** Each is rejected in a fresh consumer of `called`:

| Forgery | Diagnostic |
| --- | --- |
| Both `Opt::Some` applications renamed to the unit `Opt::None` | `source unit variant has no called constructor` |
| Both renamed to `Pair::Two` | `source application omitted or added actual unused type formals` |

Every strict prefix fails to decode.

**Amended frozen control.** The enum-match `called` generated control
(`provider.rs` plus `wrap`) used to be rejected by name with `source call
target has no staged actual function header`. It now exports, and a fresh
consumer of `wrap` reaches LowIR. `enum-match-outcomes.json` marks the
amendment. The `guarded` control, a provider-body guard, is unchanged.

**Scaling.** Each step generates 6, 24 and 96 one-field `Wide` variants, a
`pick` that calls every constructor, and a `rank` match. The packet grows
exactly linearly, and the client LowIR grows at most linearly.

**Byte identity.** `identity/compare.sh` exports the earlier fixture packets
under v13, v12 and v10, together with every fresh consumer of each, and now
also includes the enum-match provider and its clients. All 420 files are
identical between the pinned main baseline and the candidate
(`identity/receipt.txt`).

**Checks.** Every check ran through `make fastcheck` with
`--prune-under obj/parallel/enum-methods` on the final candidate
(`candidate/rustgm.sha256`). Receipts are under `obj/qualification/fast/`.

| Check | Result | Receipt |
| --- | --- | --- |
| `constructor_transport` (3), `match_transport` (3), `node_transport::unsupported_malformed_and_relabelled_nodes_reject_in_fresh_consumers` | 7 passed; format, architecture and strict Clippy clean | `1794059054685907618` |
| Native `--ignored` `constructor_instances_execute_natively` and `enum_match_instances_execute_natively` | 2 passed | `1794059169762180592` |
| Private lib cases: `sema::members` transactions (2), `generics::source::scopes` (2), `provided_clients::contracts`, `lower::transported_source::publication_tests` (2) | 7 passed; private Clippy clean | `1794059200671275381` |
| `--pa pa1` .. `pa12` | all pass | `1794059425205504759` |
| `--pa pa13` .. `pa19`, `pa21` .. `pa24` | all pass | `1794059486429148304` |
| `--pa pa20` | 114 pass; exactly the four known Lane A `assoc-bounds/{wide16,deep16,wide64,deep64}.rs` failures (`source header attribute producer exceeds storage budget`) | `1794059554125723235` |
| `make test-upstream` (`scripts/upstream.py --verify`) | 127 fixtures and 126 Reference files verified | `test-upstream.log` |
| Byte identity, baseline `52e867ea` vs candidate `91697758` | 420 compared, 0 differing | `identity/receipt.txt` |

Pending for scheduled CI: full release regular and native suites, all-target
strict Clippy, the harness and the independent suites.

## Gaps toward `core::option::Option::map`

- **Inherent impls on enums.** These are not exported. A provider with
  `impl<T> Opt<T> { .. }` fails with `source impl owner has no completed
  canonical conformance record`. Exporting them needs:
  - inherent `ImplHeader`s (no trait application) and member links (no trait
    member);
  - consumer inherent buckets and method lookup;
  - `self` and `&self` receivers.
- **Reference parameters in provider bodies.** A provider original with late
  region formals (`fn is_some<T>(o: &Opt<T>)`, `&self`) is rejected with
  `source original region formals require their complete wider body profile`.
  Only conversion-role methods admit them today. Consumer-written reference
  parameters and `match &o` over imported enums work.
- **Guards in provider bodies.** These stay fenced (`source guard facts
  require a wider transport profile`). The proposed wire form is local kind 3
  (Guard), terminator node 1 (FalseEdge), action nodes 2/3/4/6 (GuardStart,
  GuardUse, BindView, Select), and guard columns after `calls`. Consumer-written
  guards over imported variants work.
- **Real `Option::map`** also needs const callable bounds (`[const]
  FnOnce`), lang and diagnostic items, stability attributes, derives,
  `must_use`, closures in provider bodies, and consumer-written callable
  bounds.
- **Imported tuple structs** remain fenced: `Constructor::of` admits only enum
  variants.
