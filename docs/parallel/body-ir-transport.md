# Typed body IR transport: stable generic nodes

Lane C checkpoint on `implement/body-ir-transport-20261107` (base `2947cc817`).
It is a transport slice, not a library milestone: no original-core artifact,
LIB-193 repair or whole-assignment claim follows.

## Design

**Profile.** Source envelope tag 10 gains version 13, `Source2024ModuleBodies`
(CLI `module-bodies-v13`). It grants the version-twelve capabilities, including
`branches()`, plus `body_nodes()`. Version 12 stays "version ten plus
branches". Body features no longer need a profile version each: version 13
fixes one wire vocabulary for every typed body IR form, and consumer support
grows behind it.

**Node encoding.** Four row families are self-delimiting nodes: values (`ty`
u32, then a node), statements (`site` u32, node), blocks (statement range and
`site`, then a terminator node) and projections (a base place operand and
`ty` u32, then a node). A node is `code: u8`, `arity: u8` (at most 6) and that
many operands. Each operand is `kind: u8` plus a fixed payload:

| Kind | Operand | Payload |
| --- | --- | --- |
| 0-8 | Value, Block, Local, Projection, Type, Declaration, Application, Region, Flag | u32 |
| 9 | Range | 2 × u32 |
| 10 | Bits | 16 bytes, little-endian |
| 11-13 | Place: Local, Projection, Temporary | u32 |
| 14 | Place: Static | always rejected by name |

Codes follow `dev/src/body/ir.rs` order. `Rvalue` uses 0-31 (Assembly through
Intrinsic), `Action` 0-12 (CheckInitialized through FutureTake), `Terminator`
0-5 (Jump, FalseEdge, Branch, Return, Unreachable, Panic) and `ProjectionKind`
0-12 (CaptureBoundary through FutureField). Flags carry closed sub-forms:

- Borrow: 0 immediate, 1 two-phase, 2 unique.
- Cast: 0 for a closed primitive `as`.
- Dereference mutability: 0 or 1.
- Field ordinal.
- Binary operator: 0-8 are Equal, NotEqual, Less, LessEqual, Greater,
  GreaterEqual, BitAnd, BitOr and BitXor; 9-15 are Add, Sub, Mul, Div, Rem, Shl
  and Shr.
- Binary lexical operand form: 0-4 are NotRequired, Neither, Left, Right and
  Both.

Locals, scopes, exits, temporaries, owned values, calls, openings and arguments
carry no variant vocabulary. They keep their version-twelve columns
byte-for-byte.

**Tables, constants and proofs.** Operands reference the packet's existing
tables by index. Type operands index the interned source type graph.
Declaration operands index the provider declaration table, application operands
the function application table, and region operands the original's region
rows. Scalar constants are inline Bits whose type is the value's own type. A
call's trait proof is its ordinary `CallRequest` row, with method application
and opening. No provider decision or proof is serialized: no trait selection,
cast or operator choice, overflow decision, borrow or dominance result. The
consumer derives each one again.

**Validation.** Decoding bounds each node's arity, requires known operand kinds,
reads fixed widths, and rejects every truncation or strict prefix. Each row is
then mapped in one of four ways:

1. A supported code with its exact operand schema becomes a row.
2. A supported code with any other schema reports malformed operands.
3. An allocated but unsupported code is rejected by name, for example
   `encoded Rvalue::IsVariant is not reconstructable by this consumer`. The same
   applies to unsupported flags, such as checked arithmetic operators, or
   two-phase and unique borrows.
4. An unknown code is an unknown form.

The rows then pass the unchanged version-twelve `ValidatedRows` pipeline:

- Ranges: operands precede their use, successors and places stay in their own
  body.
- Typing: Binary operands share one closed non-unit primitive type. Comparisons
  yield bool, and bitwise operations keep their bool or integer type.
- Owner and remap authority: the descriptor identity and the `--source-client`
  owner must match; foreign or swapped packets have no provider metadata.

Binary rows pass a profile gate in four places: the producer, the older-generation
encoder, both staging paths (`branch_profile`) and the supplied profile.

**Consumer reconstruction.** A fresh consumer rebuilds `Rvalue::Binary` from the
validated row (`rebuilt_operations`), including its operator and lexical operand
form. The cached-original check compares it exactly. Every instance then runs
the ordinary original pipeline: typing, initialization, moves, borrows,
storage, drops, promotion and const eligibility. Lowering chooses signed or
unsigned predicates from that instance's operand type.

**Version eleven.** It is not folded in. Version 11 has its own envelope and
construction (tag 13, `Construction::CompilerOperations`). Its additions are
header contract tables, not body IR families: associated types, projection
recipes, opaque futures, compiler-operation rows and Future roles. Its bodies
keep version-ten operation bytes. Folding becomes clean when version 13 needs
operator roles and associated types, which the Panic producer below requires.
At that point version 13 should absorb those tables as header columns under
tag 10, and version 11 should stay a compatibility decoder.

**Supported now.**

- Values: Unit, Aggregate, Scalar, Read, immediate Borrow, Function, Constrain,
  primitive Cast, Call, and Binary comparison or bitwise operations.
- Actions: CheckInitialized, Mention, Project, StorageLive, StorageDead,
  StorageEnd, Evaluate and Store.
- Terminators: Jump, Branch, Return, Unreachable and Panic.
- Projections: Dereference and Field.

Every other allocated form is rejected by name.

**Work and storage.**

- Decoding: each column checks its count against the remaining bytes with a
  per-row minimum (6 bytes per value, 6 per statement, 14 per block, 11 per
  projection). It reserves storage against the packet's aggregate retained
  budget, as version 12 does.
- Validation: work is charged per row and per operand, two type lookups for
  each Binary.
- Production: the existing per-value charge applies.

All three costs are linear in the decoded rows. The arity bound is a format
constant, not a ledger. No limit from another domain is borrowed.

## Panic producer: blocked before body transport

There is still no genuine `Panic` producer from real source. Checked
arithmetic, compound assignment and indexing are rejected before body transport.
Probes are under `obj/parallel/body-ir/probe/`. The diagnostics are exact:

1. `+`, `+=` and `[]` need their language declaration: `source binary operator
   requires its actual language declaration`.
2. Exporting a declared operator trait fails: `source compiler language contract
   transport is not implemented`. Only the version-eleven Future roles are exempt
   (`authored_traits::require_role_transport`).
3. Operator traits need `Rhs = Self`. Without it: `source binary Rhs changed its
   genuine declaration or Self default`. With it: `source parameter defaults are
   outside the initial profile`.
4. `Add` and `Index` declare `type Output`, and associated types transport only
   in version 11.
5. Consumers bind no imported operator role. `recipe_identity` admits only
   Future roles.
6. The rustc oracle needs `panic_const_add_overflow`, and through
   `#[track_caller]` the `panic_location` lang item. This calls for a
   cfg-gated rustc-only shim.

The body side is no longer missing. Consumer lowering of a primitive Binary
already emits the overflow guard and the shared abort exit. The Binary node,
the row form and the consumer rebuild are part of this checkpoint, and checked
operators 9-15 are allocated and rejected by name.

Proposed next steps:

1. Operator role tables, starting with AddAssign, which has no `Output`, plus
   consumer catalog binding.
2. Parameter-default transport.
3. Checked Add, Sub and Mul rows with their guard cause.
4. The oracle shim.
5. `Add` with `Output`, which is the version-eleven fold point.

Finding outside transport: RustGM accepts the negative range pattern
`-5..=-1` with no `neg` language item, but rustc 1.98.0 rejects it
(`requires neg lang_item`). This checkpoint records it and does not fix it.

## Gap list toward `Option::map`

- Enum match with payload binding: `enum` declarations are refused with `source
  declaration kind is outside the initial source profile`. Transport needs
  nominal enum headers (variants, discriminants, layout), followed by
  IsVariant, Select and BindView values, FalseEdge exits and guard tables.
  These codes are allocated and rejected by name.
- Panic: see above.
- Closures and `FnOnce`: transport needs closure entries, captures and
  CaptureBoundary projections. `FnOnce` needs the same role and associated-type
  transport as Panic.
- Still open from version twelve: drops, impl-method calls, references in free
  originals, constants, and SSA snapshot bodies with two-way exits. The SSA
  dominance proof is deferred until a real source body needs it, and those
  bodies are still rejected explicitly.

## Evidence

Receipts are under `obj/parallel/body-ir/`. Nothing was hand-edited or blessed.
The fixtures and outcomes (`pa29/tests/spec/source-body-ir/README.md`,
`pa29/tests/source-body-ir-outcomes.json`) were declared before any candidate
run of a frozen file.

- **Independent Rust 1.98.0** (`rustc/receipt.txt`): both providers compile.
  Native exits are 3, 41, 44, 32, 36, 21, 23, 24, 38, 27, 30 and 14, and 21 for
  the wire provider. The opponents fail with E0308 and E0277.
- **Candidate** (`crate_linking::node_transport`): every declared outcome matches.
  The packet has version bytes `[13, 0]` and round-trips. Version-twelve and
  version-ten exports of the Binary provider are rejected and name version 13.
  Comparison counts per consumer (2 per range, 1 per literal) were recorded
  after the first observation; an initial guess that bool literals emit no
  comparison was wrong and was corrected.
- **Re-encoding**: the source-branches providers, re-exported as version 13,
  give byte-identical consumer LowIR for all eight positive clients and
  round-trip their packets. The packets grow by 301, 38 and 10 bytes over
  version 12.
- **Wire controls**: all fifteen declared mutations are rejected with their
  declared diagnostics. A relabelled version and every strict prefix fail to
  decode, and foreign or swapped owners have no provider metadata.
- **Baseline `2947cc817` against candidate** (`baseline-compare/receipt.txt`,
  baseline SHA-256 `35732e8b…0eef`, candidate SHA-256 `83810ea6…7cd6`):
  - Byte-identical: version-twelve exports of provider, wait and spin; the
    version-ten export of spin; and the version-ten export of the
    source-nominals provider.
  - Changed: only the older-profile diagnostic for a Binary value.
- **Scaling**: literal-arm decisions of width 6, 24 and 96 add exactly 233 wire
  bytes per arm, with one Binary node and one `cmp eq` per arm. LowIR grows
  less than fivefold for each fourfold step. At width 96 the provider takes
  0.03 s and the consumer 0.20 s, and the export is deterministic.
- **Native** (`--ignored`): all twelve clients, the wire client and the eight
  re-encoded version-twelve clients return their exits. The version-twelve
  native regression also passes.

Pins: the baseline `obj/parallel/body-ir/baseline` (main `2947cc817`) and the
candidate are pinned. `obj/parallel/body-branches/baseline` is unpinned and
pruned; its source head and SHA-256 (`b9cbb8a0…1dec`) are kept.

Checks:

- Focused `make fastcheck` passed assignment metadata, full format,
  architecture, strict Clippy for the CLI and `suite_linking`, the build, the
  four new tests, and the four version-twelve and source-nominal regressions
  (`obj/qualification/fast/1794031670659831761`).
- Native fastcheck passed on the final binary (`.../1794031834393587728`). Two
  admission refusals for a heavy slot were capacity waits.
- `make test-upstream` passed.
- Earlier attempts failed and were fixed:
  - Repeated filename stems in the architecture audit (`.../1794030775606379829`).
  - Clippy: line limits, bool fields, by-value generic, `new`, and
    `format!`-collect (`.../1794030862335444585`, `.../1794031023895587672`).
  - The bool comparison count (`.../1794031151686977454`).

Pending for scheduled CI: full regular and native suites, private library
tests, harness, all-target Clippy and release builds. No private test was added.
