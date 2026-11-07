# Enum-and-match transport: enum headers, constructor tests and downcasts

This is a Lane C checkpoint in `rustgm-async-work`, based on `9f3c0c64b`. It
is a transport slice, not a library milestone. It makes no claim about an
original-core artifact or a whole assignment.

## Design

**Provider enums.** Under version 13 (`module-bodies-v13`), a no-core
provider may export the following enums:

- fieldless enums;
- `#[repr(<integer>)]` enums with explicit non-negative discriminants;
- generic enums with tuple variants, such as `Opt<T> { None, Some(T) }`.

Each enum is a nominal header with no record fields and the Rust
representation. Each variant is a kind-11 declaration, and its parent is its
kind-10 enum. Only the variant's value-namespace constructor is published.

`authored_export/sum_headers.rs` builds the enum table from the completed
shape. It rejects the following by name:

- record variants;
- negative discriminants;
- lifetime parameters;
- `must_use` enums;
- enums without variants.

**Wire additions** (version 13 only):

- **Declaration kinds.** Kind 10 is an enum and kind 11 is a variant.
- **Enum columns.** These follow the associated-result columns, and only when
  the declaration table names a kind-10 declaration. There are three columns:
  - headers: `(enum, tag 0 | [1, integer scalar], variant range)`;
  - variants: `(declaration, ordinal, shape 0 unit | 1 tuple, 16-byte LE
    discriminant, field range)`;
  - fields: `(variant, ordinal, type, site)`.
- **Body IR node 14.** This is `Rvalue::IsVariant [Place, Declaration
  (variant)]`, a `bool` constructor test.
- **Body IR node 3 with arity 2.** `[Range(fields), Declaration(variant)]` is
  a variant aggregate. Arity 1 remains the struct aggregate.
- **Body IR node 28, flag 1.** This is a fieldless enum's integer cast. Flag 0
  remains the scalar cast.
- **Field projection with a variant owner.** A field projection whose owner
  declaration is a variant is a downcast field, `(place as Variant).n`.

Older codecs and profiles reject all of these by name. Packets without an
enum keep their exact bytes.

**Wire validation.** These checks are in `generics/sum_contracts.rs` and the
port plan. Header rows must follow declaration order and cover the variant
and field columns contiguously. The validator then checks that:

- every variant names its enum parent, dense ordinal, unique name and own
  field range;
- a tag is an integer scalar;
- each discriminant fits the tag, or `isize` when there is no tag;
- discriminants are unique;
- a data enum without a tag keeps its ordinal discriminants;
- a unit variant carries no fields.

The port plan adds three checks:

- An aggregate's variant must belong to its actual enum type.
- A constructor test must return `bool` over a place of the variant's enum.
- An enum cast requires a fieldless enum and an integer result.

**Consumer reconstruction.** A fresh consumer trusts no provider layout or
match proof:

- `mapped_records` turns each header into an imported enum shape.
  `prepare_imported_nominals` re-derives the layout, including the nullable
  policy: no tag, two variants and payload arities 0 and 1. It pays per
  shape, variant and nullable policy.
- `prepare_external_nominals` installs the variants and their payload
  fields, paying per variant, field and constructor.
- A unit variant's cell has the enum type. A tuple variant's cell is the
  constructor function item `fn(payload..) -> Enum`.
  `mapped_members::variant` re-reads each installed variant's enum, ordinal
  and payload.
- `rebuilt_operations` rebuilds the three value forms as an
  `Aggregate { variant }`, `IsVariant` and an `EnumToInteger` cast. These are
  then checked by the consumer's own typing.

**Re-proved matches.** `transported_originals/variant_facts.rs` runs on every
rebuilt body that contains a constructor test or a downcast field:

- **Facts.** A `Branch` on a live test yields a positive fact on its `yes`
  edge and an exclusion on its `no` edge. Facts flow down the tree of
  single-real-predecessor edges, so a `FalseEdge` contributes only its real
  successor, and a merge starts with no facts.
- **Places.** Facts are keyed by hash-consed canonical places.
- **Kills.** A fact dies when its root local is stored, re-bound or
  re-scoped. Facts reached through a dereference, and facts on locals that
  are ever mutably borrowed, also die at every call and every store through a
  dereference.
- **Downcasts.** Every downcast field use must be proved by a live fact for
  its variant.
- **Exhaustiveness.** Every reachable `Unreachable` exit must follow an
  impossible variant path or a never-typed value. A missing arm is therefore
  rejected as `transported match is not exhaustive over its enum`.
- **Cost.** The work is one step per block and statement plus one step per
  fact, charged to canonical body work. Bodies without tests charge nothing.

**Closed-nominal formation.** A version-13 provider now requests a
well-formedness input for body value and local types that contain nominals.
Local obligations deduplicate these requests. Before this change, a body over
a closed nominal type failed export with `omitted a required nontrivial
formation input`. Packets that exported before are unchanged.

**Fences.** Two forms stay rejected at export by name:

- A called tuple-variant constructor (`Opt::Some(x)`) fails with `source call
  target has no staged actual function header`. Payloads use the braced form
  `Opt::Some { 0: x }`.
- A match guard fails with `source guard facts require a wider transport
  profile`.

## Evidence

Receipts are under `obj/parallel/enum-match/`. Nothing was hand-edited or
blessed.

The fixtures are in `pa29/tests/spec/enum-match/`. Their outcomes are in
`pa29/tests/enum-match-outcomes.json` and were declared before any oracle or
candidate run. The provider reuses lines 1-88 of
`spec/callable-roles/provider.rs` and keeps their attribution. The outcome
file records five fixture deviations, all forced by the fences or gaps
below:

- the braced constructor;
- a `ref` binding instead of `match &o`;
- a free `map` instead of an inherent method;
- no guards;
- nested literal matches instead of comparison operators.

**Independent Rust 1.98.0.** The oracle ran with `-Cpanic=abort` and
`--cfg rustc_oracle` (`rustc/receipt.txt`):

- `fieldless`, `tagged`, `generic`, `mapped` and `by-ref` all exit 42.
- `wrong-enum` and `wrong-payload` fail with E0308.

**Candidate.** `crate_linking::match_transport` checks the following:

- The packet carries `[13, 0]`, round-trips and holds eight constructor tests.
- All five clients reach LowIR in fresh consumers.
- Both opponents reject with `incompatible types`.
- An enum-only provider rejects at v12 and v10 with `source declaration kind
  is outside the initial source profile`.
- The called-constructor and guard controls are fenced.

**Forged wires.** Each is rejected in a fresh consumer:

| Forgery | Diagnostic |
| --- | --- |
| `pick`'s second test `Two::B` relabelled `Two::A` | `transported match is not exhaustive over its enum` |
| `or`'s `Some` test relabelled `None` | `transported match is not exhaustive over its enum` |
| `or`'s `Some` and `None` tests swapped | `source downcast field use is not dominated by its variant test` |
| `pick`'s first test relabelled `Opt::None` | `source variant belongs to another actual enum` |
| `Tag::High` discriminant 39 relabelled 3 | `source enum has a duplicate discriminant` |
| `Tag::High` discriminant 39 relabelled 295 | `source enum discriminant exceeds its representation` |
| `Opt::Some` discriminant relabelled 5 | `source data enum discriminants require an integer representation` |
| `Opt::Some` shape relabelled unit | `source unit variant carries payload fields` |
| `Opt::Some` shape byte 2 | `unknown source variant shape` |
| Version relabelled 12 | `unknown source declaration kind` |

Every strict prefix fails to decode.

Two expectations were declared before they were observed, and both were
reworked after the first run:

- **Older-profile control.** The full provider stops at v12 and v10 on its
  callable-role aliases first. The outcome file records that diagnostic, and
  the enum control now uses an enum-only provider.
- **Swapped-test control.** The declared single swap (`Some` relabelled
  `None`) is rejected as non-exhaustive. It is kept as its own control, and
  the full swap reaches the downcast diagnostic.

**Amended frozen control.** In `source-body-ir-outcomes.json` and
`node_transport`, "binary node code 14" was declared as `encoded
Rvalue::IsVariant is not reconstructable by this consumer`. It is now marked
`amended`, because code 14 is now a supported constructor test whose
relabelled Binary operands are malformed: `body IR value node has malformed
operands`. A new "code 15" control (`encoded Rvalue::Validate is not
reconstructable by this consumer`) keeps an unreconstructable allocated code
under test.

**Native.** The ignored native test runs fresh consumers through CPPGM, which
was not modified. All five clients exit 42 with empty stdout, as with rustc.

**Byte equality.** Two binaries are pinned:

- the baseline from main `9f3c0c64b` (`baseline/`, SHA-256 `5e85f88e…`);
- the candidate (`candidate/`, SHA-256 `d2dba6cc…`).

`identity/receipt.txt` compares 395 files, covering exports, LowIR, statuses
and diagnostics for every fresh consumer of these exports:

| Profile | Exports |
| --- | --- |
| v13 | source-nominals, callable-roles, source-body-ir provider and wire, language-roles, associated-results |
| v12 | source-branches provider, wait and spin |
| v10 | source-branches spin, source-nominals |

No file differs.

**Scaling.** Fieldless providers with 6, 24 and 96 `Wide` variants and a
constant-arm `rank` match grow exactly linearly in packet bytes. A fresh
client's LowIR grows at most linearly. Header validation, imported shape
preparation and variant facts all charge per row.

**Gates on the final tree.**

| Gate | Result | Receipt |
| --- | --- | --- |
| `make fastcheck`, 4 exact linking tests (first run) | 2 passed; 2 expectation mismatches, reworked above | `obj/qualification/fast/1794053642654766417` |
| `make fastcheck`, the 2 reworked tests | passed | `obj/qualification/fast/1794053995598274762` |
| Private fastcheck, 6 exact lib cases (external nominal batches, owned slots, decoded contracts, body-policy wire) | passed | `obj/qualification/fast/1794054129361677846` |
| Native fastcheck (`enum_match_instances_execute_natively`) | passed | `obj/qualification/fast/1794054420545716791` |
| `make fastcheck`, 11 linking tests (new tests, node transport, roles, results, nominals) | passed | `obj/qualification/fast/1794054494751102359` |
| Byte-identity compare | identical | `obj/parallel/enum-match/identity/receipt.txt` |
| `make test-upstream` | passed: 127 fixtures, 126 references | `obj/parallel/enum-match/test-upstream-1.log` |

## Gap list toward `core::option::Option::map`

1. **Called tuple-variant constructors** (`Some(f(x))`). A constructor
   function item has no staged function header, so it is fenced at export.
2. **Inherent methods on enums** (`impl<T> Option<T> { fn map(self, ..) }`).
3. **Const callable bounds.** Core's `map` is a `const fn` with `F: [const]
   FnOnce(T) -> U + [const] Destruct`, which needs const conditions in
   transported bodies and the `Destruct` role.
4. **Option's own attributes.** These include its lang items (`Option`,
   `Some`, `None`), diagnostic items, stability, derives and `must_use`
   enums, which are fenced.
5. **Guards and their body forms.** Guards need `GuardStart` and `GuardUse`,
   guard `BindView`s and `Select`. They are fenced by name.
6. **References.** These cover `match &o`, reference parameters (region
   formals in transported signatures) and snapshot dominance for borrowed
   scrutinees.
7. **Consumer-side variant naming.** A consumer cannot yet write or match
   `base::Opt::Some(..)`. Clients reach variants only through provider
   functions.
8. **Wider enum shapes.** These are record variants, negative discriminants
   and lifetime-generic enums.
9. **Unexplained `Unreachable` exits.** In a body that tests variants, an
   `Unreachable` exit that no variant exclusion or never value explains is
   rejected conservatively.

## Pending

These checks were not run here:

- Full release and native suites, all-target strict Clippy and the harness.
  Scheduled CI owns these. No integration root or suite was added
  (`match_transport` is inside `crate_linking`), so `make test-harness` was
  not needed.
- `--pa pa24` was not run, because no closure or body inference shared with
  single-crate compilation changed.
- `make test-backend` was not needed, because the bridge and the backend
  contract are unchanged.
