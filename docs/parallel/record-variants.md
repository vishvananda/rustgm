# Record variants

Lane D's item (i), 2026-11-09, on `implement/record-variants-20261109`,
rebased from main `0bcba9e3f` onto `a6be80084`, Lane C's (ii), and then
onto `198fdea9b`, item B, census batch 21 and Lane C's associated-unsafe
item. The first commit is the i128 slot control and its blockers line; the
second is the item. The integrator accepted the notice and its rulings: the
declaration row, the namespace checks, the wire, the flips and the survey.
It also accepted amendment 3, the three deviations, the size pin and the
stage-flag fix, required the first rebase and amendment 4, and confirmed
amendment 4's flip after its runs. A version-13 module-body
export now carries enums with record variants (`V { a: T, .. }`), which
version 13 had fenced with "source record variants require their wider
profile".

## Before

- `sum_headers.rs` stopped an exported enum with a record variant, and
  `authored_export.rs` stopped a type-namespace variant with no constructor,
  with that text.
- A unit or tuple variant's one declaration row names its value-namespace
  constructor, and its type-namespace definition has no row. A record variant
  has no constructor, so it had no row at all.
- Variant fields were positional and carried no names.

## Wire

- **Shape byte 2 is Record** in the variant row `(declaration, ordinal,
  shape, discriminant, fields)`. Only body_nodes() profiles write enum
  columns, so only they can write it.
- **A dense name column** follows the field rows exactly when some decoded
  variant row has shape 2: each record variant's field names, in
  variant-row and ordinal order, written as nominal field names are, with no
  count. The record variants' field ranges imply it. A packet without a
  record variant keeps its exact bytes.
- In memory, `EnumColumns.names` is empty unless some variant is a record
  variant. Then it holds one slot per field row: a record field's name, or
  empty for a positional field.
- **The declaration row keeps kind Variant.** For a record variant it names
  the variant's type-namespace definition. The shape is the one source of
  the namespace: Type for a record variant, Value for a unit or tuple
  variant.
- **Body IR is unchanged.** A variant aggregate lists its fields in
  declaration order, a constructor test names the variant row and a downcast
  field projects by ordinal.
- An older reader stops at the first record variant's shape byte with
  "unknown source variant shape". Nothing decoded before the enum columns
  consults a variant's shape.

## Export

- `sum_headers.rs` maps `Shape::Record` to shape 2. The export's first record
  variant gives every field row so far an empty name slot and marks the
  stage, and every later field row pushes its spelling, or an empty slot for
  a positional field. A field whose name does not match its shape is still
  "source variant field changed its position".
- `authored_export.rs` publishes a record variant's own type-namespace
  definition as its declaration row; a missing constructor entry is "source
  variant lacks its actual constructor".
- `namespace_emission.rs`: a re-exported record variant's target already has
  a declaration row, so its `pub use` is an ordinary type-namespace binding.
  A re-exported unit or tuple variant keeps its value-namespace binding.

## Consumer

- **Validation** (`sum_contracts.rs`, `table_storage.rs`):
  - names are canonical path components (`validate_component`, "invalid
    canonical metadata path component"), unique within their variant
    ("source record variant has a duplicate field name"), and a positional
    variant's slots are empty ("source positional variant field carries a
    name");
  - the column covers every field row ("source record variant names have
    incomplete field coverage");
  - storage accounting counts the name column and its bytes.
- **Namespace, checked at three places:**
  - module binding rows: `implementation_domains.rs` derives the expected
    namespace from the target's shape, so a record variant bound in Value or
    a tuple variant bound in Type is "module binding changed its target
    namespace";
  - constructor applications: a function application naming a record
    variant is "source record variant has no called constructor";
  - value cells: `definition_cells.rs` requires a value cell for every
    variant except a record variant, and readiness requires its absence, so
    `E::V` names no value.
- **Installation.** The resolver's `Records` marker installs a record
  variant's row in its enum's type namespace, in `packet_declarations.rs`,
  `scope_rows.rs`, `module_remap.rs` and `binding_publication.rs`.
  `mapped_records.rs` maps the variant to `Shape::Record` with its field
  names interned under the nominal name budget, and `members.rs` and
  `mapped_members.rs` install and re-check those names.

## Fixture and evidence

[`pa29/tests/spec/record-variants`](../../pa29/tests/spec/record-variants/README.md)
holds the provider, two accepted clients and four opponents. Every outcome
was declared before the rustc oracle, the base-pin baseline and any
candidate run (`pa29/tests/record-variant-outcomes.json`).

| Control | Result |
| --- | --- |
| provider, `module-bodies-v13` | exports a 23,230-byte packet; decode then encode reproduces it |
| `construct`, `nul` | accepted by fresh consumers; each exits 42 natively with empty stdout, as rustc's do |
| four opponents | the same texts as their local twins, whose stderr equals the base pin's |
| re-export control | `Pair` binds the value namespace, `Rect` the type namespace, each naming its own row; its client is accepted |
| `module-branches-v12`, `module-nominals-v10` | the base pin's texts and stderr bytes, no packet |
| old reader, base pin `0680f6ad` | every client rejected with "unknown source variant shape" |
| wire | shape bytes 3, 4 and 255 are unknown; every strict prefix fails to decode |
| ten forged rows | rejected by name, the two namespace swaps among them |
| N = 6, 24, 96 record variants | packet bytes exactly affine; client LowIR within the declared bound |
| amendment 4: two record-variant opponents | export, each with a native witness that exits 42, as rustc's does |
| amendment 4: `argument-type` | "source anonymous callable requires its supported closed Rust frame" |
| flip survey, 1,920 runs per binary | only the seven new `spec/record-variants` runs under version 13 differ |

- **Candidate.** `cand4` (`0b096be3`), built from the rebased item
  `d83e61437`, whose dev/src the item commit keeps. The base pin is
  `0680f6ad`, built from `7047a05d9`, whose dev/src is `198fdea9b`'s.
  Evidence: fastcheck `1794252988341863603` (16 integration and 12 private
  controls: (ii)'s `fielded_callables` and `input_plan::roots` invariants and
  Lane C's `guarded_methods` root and `guarded_member_rows` invariants among
  them) and `1794253163557520234` (five native tests, R2's, (ii)'s and the
  associated-unsafe item's included).
- **The union on `198fdea9b`.** Lane C's associated-unsafe item moved each
  packet's `associated` opponent out of the lists this item moves
  `record-variant` out of. Both now leave and are checked as exports. Every
  declared run, the old reader and both witnesses on `cand4` and the new base
  pin are byte-identical to `cand3`'s and the `a6be80084` pin's: status,
  stderr, LowIR and packets.
- **Survey.** 73 spec directories under versions 10, 12 and 13 (204 exports
  and 1,557 clients), `callable-fields` and `guarded-methods` among them,
  plus 159 committed single-crate inputs with a record variant. The seven
  differences carry `cand3`'s hashes. `same-crate-variants.rs` and every
  other local input keep their status and stderr bytes, and every existing
  packet keeps its bytes. Against the `a6be80084` survey, only the stderr
  offsets of the two edited test sources moved, on both binaries alike. The
  amendment-4 opponents are generated, so `called_pointers`,
  `fielded_callables` and `braced_variants` check them.
- **Before the rebases.** On `0bcba9e3f`, `cand2` (`3b5d0c5d`) passed the
  same controls without (ii)'s, and its survey of 1,810 runs per binary
  changed only the new packet. On `a6be80084`, `cand3` (`9af607a3`) passed
  fastchecks `1794250650057581843` and `1794250731909863826`, and its survey
  of 1,863 runs per binary, `callable-fields` among them, changed only the
  new packet.
- **Size pin.** `ColdColumns` grows from 224 to 248 bytes, one `Vec` for the
  name column; `term_codec.rs` pins the new size.

## Recorded

- **Deviations from the notice.**
  - `PartialEq` is not in the packet. An exported `==` needs the comparison
    and `Not` language declarations, which version 13 does not transport;
    base-pin probes of tuple twins found each fence. Comparison and Not role
    transport is its own item.
  - Clients copy through the imported `Copy` impl instead of calling
    `.clone()`: a method call through an imported `Clone` impl is "no
    applicable method for this receiver", a pre-existing fence of fresh
    consumers that no version-13 fixture exercises.
  - The notice's `pub use` of a record and a tuple variant is a generated
    re-export control, so the frozen packet holds only declaration bindings.
- **Amendment 1** (before any candidate run): a no_core twin needs a
  source-core configuration and therefore an export profile, so the twins
  are compiled in the provider's own export form. Under the default profile
  the base pin rejected them before reaching the variant fence, a failed
  prediction of "the base pin and the candidate agree".
- **Amendment 2** (after the first candidate run, before any further run):
  the candidate rejects the nul twin's export with "source place transport
  supports locals and reference projections only". The twin's body matches
  directly on `NulError::NotNulTerminated`, and base-pin probes p7 to p9
  locate the fence there: a pre-existing body-transport fence on a
  unit-variant scrutinee, outside (i). The cross-crate nul client is
  unaffected, because a client body is not exported.
- **Amendment 3** (after fc8, the first run of the integration controls,
  before any further run): enum-match's forged `unknown-shape` row wrote
  shape byte 2 on `Opt::Some`. That byte is now Record, and the row reports
  "truncated metadata artifact", because its name column runs past the
  packet's end. The control moves to byte 3, which stays "unknown source
  variant shape", and `enum-match-outcomes.json` records the move. It is a
  forged byte, not a survey input: a failed prediction of "every other
  outcome is unchanged".
- **Candidate defect found by fc8.** When an export's first record variant
  preceded every field row, the producer left the name column empty, so the
  scaling control's `V000` field lost its name ("invalid canonical metadata
  path component"). The stage now marks its first record variant. The
  scaling expectation is unchanged.
- **Process note on amendment 3.** It was written after fc8 had already
  run the forged byte-2 row, not declared before that run. The integrator
  accepted it, because the accepted wire ruling assigns byte 2 to Record.
- **Amendment 4, the (i) x (ii) interaction** (declared 18:44:48Z, after
  the rebase and the new base pin and before any rebased candidate run, with
  code-reading predictions, all observed as predicted):
  - (ii) declared four outcomes that (i) changes: its `record-variant` and
    `argument-type` opponents, control 0's `record-variant` clause and
    limits/1. Pointer-call amendment 4, written on `0bcba9e3f`, predicted
    the `field` opponent's text, which (ii) retired.
  - A record variant's fields take a tuple variant's export path, so a
    zero-slot fn field binds a CallableType frame its enum owns. A variant
    aggregate's operands and a downcast field both form through
    `Rewrite::Nominal` with that enum as the exact owner.
  - The integrator approved the flip to export by name, on the recorded
    rustc status 0 and a native witness that calls through the field. R2's
    stored formatter (pointer-call amendment 5) and (ii)'s record-variant
    opponent (callable-field amendment 4) export. W1 builds and matches the
    imported `Kind::Placeholder` and calls `formatter(value, 22)` in the
    client; W2 calls the imported `stored_fmt`. Both exit 42 natively, as
    rustc's do, and the base reader rejects both packets with "unknown
    source variant shape".
  - `argument-type` keeps the higher-ranked fence: its formatter's elided
    `&mut` is a late-bound region.
- **Renumbering on `198fdea9b`.** Lane C's associated-unsafe item landed
  its own amendment 3 in both outcome files (recorded 18:11:59Z), and those
  keep their numbers. This item's amendments take the next free numbers in
  their recorded order, with their timestamps unchanged: pointer-call 3
  (17:48:25Z) and 4 (18:44:48Z) become 4 and 5, and callable-field 3
  (18:44:48Z) becomes 4. Each records its former number. Record-variant
  amendments keep theirs.
- **Test file names.** The architecture audit's repeated-stem rule rejected
  two more `record_*` files ("repeated filename stem record: 8 files"), so
  the tests are `braced_variants.rs` and `braced_variant_rows.rs`.
- **What (i) unlocks in core 1.98.** Two enums have record variants:
  - `fmt/rt.rs` `ArgumentType::Placeholder`, whose formatter
    `unsafe fn(NonNull<()>, &mut Formatter<'_>) -> Result` is higher-ranked
    and waits for HR, as (ii)'s `argument-type` opponent shows;
  - `ffi/c_str.rs` `FromBytesWithNulError::InteriorNul { position }`: its
    header, its construction in `CStr::from_bytes_with_nul` and its Clone
    and Copy derives. PartialEq needs comparison role transport, and Debug
    and Display need the fmt chain.
