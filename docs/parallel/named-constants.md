# Named constants (5b-1)

Lane C's item 5b-1, 2026-11-09, developed on origin/main `28e2b19fb`,
qualified on `306b82377` and rebased onto Lane D's A (`f62db7da9`), which lands
first. The integrator approved the design notice and its corrections 2a-2f. 5b-1 lets a version-13 module-body export carry named const
items of primitive and unit types, at module scope and inside function bodies,
and lets exported bodies read them. Associated consts, trait-const conformance
and const-parameter values are 5b-2. Range patterns are 5b-3.

## Before

`authored_export.rs` rejected every `DefKind::Const` with "source declaration
kind is outside the initial source profile". Core's bodies read many module
and item-local consts, so every exported body that named one was fenced
before its own transport began.

## Rows

- **Declaration tag 13, `Constant`.** Read only under `body_nodes()`. Older
  readers, including the base pin, reject it with "unknown source declaration
  kind".
- **Constant header column** (`free_constants_wire.rs`). It follows the enum
  variant columns, inside the body-nodes block of both availability codecs.
  - It has no count and no declaration field: row `k` belongs to the `k`-th
    tag-13 declaration, so a packet without constants keeps its exact bytes.
  - Each row is `ty | value`, where value is `0` or `1 | term`, with K's term
    codec in place (2a). A u8 row is 26 bytes. Any other presence byte fails
    with "invalid source constant value presence".
  - The row holds its own term, so K's dense argument-owned term column and its
    invariant are unchanged. The cost is that const terms live in two places.
- **Value code 17** is `(17, [Declaration(const)])`, read as
  `ValueKind::Constant(DeclarationRef)`. The legacy column codec names the
  profile it needs ("source constant read requires the version-thirteen body
  IR profile"). `supplied_profile.rs` repeats that text in A's table of
  body-only kinds, and `ssa_ports.rs`'s `dependency()` lists code 17 among
  the kinds with no operand.

## Export

- `exported_kind` maps `DefKind::Const` inside the version-13 nominal block:
  - an unnamed `const _` exports no row (2b);
  - an associated owner, inherent or trait, fails with "source associated
    constants require their wider profile";
  - otherwise the declaration is a `Constant`.
- `evaluated_constants.rs` writes one row per constant declaration, in
  declaration order. It classifies the completed type first:
  - a primitive or unit continues;
  - a reference, including `&str`, fails with "source reference const value
    requires its wider profile";
  - a raw or fn pointer fails with "source pointer const value requires its
    wider profile" (2c);
  - anything else fails with "source aggregate const value requires its wider
    profile".
  - The value is the completed evaluator value, and its bits come from the
    legacy exporter's `named_values::value_bits`, unchanged. The header and the
    term share one type row.
- A free const's initializer is not an original: its value travels evaluated.
  `collect_body` skips `Owner::Initializer { kind: Const }` when the const is
  evaluated, free and either has a constant row or is unnamed. Every other
  initializer keeps its relocated-view text.
- Code 17 (`value_rows.rs`). A read of a declaration with no local row fails
  with "source constant read of another crate's declaration requires its wider
  profile". It has no witness until the multi-packet item. A local row that is
  not a constant fails with "source constant read names no validated constant
  declaration".
- Availability adds `DefKind::Const` to the source-origin kinds, so a public
  const carries its regular policy.
- Module bindings (`implementation_domains.rs`). Under version 13 a constant
  binds in its module's value namespace. Candidate run 1 found this site:
  every module-constant provider stopped at "module binding target requires
  a wider declaration profile". The notice had not listed it.

## Validation (every consumer)

`free_constants.rs` follows the sum and results contracts:

- `require_profile`: constant rows or tag 13 under an older profile fail with
  "unknown source declaration kind";
- `headers`: one pass binds each tag-13 declaration to its row. A surplus or
  missing row fails with "source constant rows lack their declaration", and
  `Constant` joins the kinds that must have a typed header;
- `validate`, in order, each O(1) per row:
  1. the parent is a module or a function. A trait or impl parent gives the
     associated text; anything else gives "source constant header lost its
     free owner";
  2. presence 0 fails with "source free constant requires its value", and term
     tag 1 with "source const value over a const parameter requires its wider
     profile" (2e);
  3. the term's type is the header's: "source constant value changed its
     declared type";
  4. K's `const_value`, unchanged: "source const value type is not a
     primitive row" and "non-canonical source const value".
- Code 17 in `operation_domains.rs`: the declaration has a constant header,
  and the read's type is the same scalar as the header's. Otherwise "source
  constant read names no validated constant declaration" or "source constant
  read changed its declaration's type".
- `value_requirements.rs` admits code 17 beside Scalar, and
  `ordinary_profile.rs` admits it among version 13's regional values. A
  constant can claim no language role.
- The new `HeaderRef::Constant` joins the no-input groups. `scope_inputs.rs`
  gives it no parent scope, as for variants and constructors: a free const
  inherits no generics, even inside a generic function.

## Consumer

- **Resolve.** `Constant` maps to `(DefKind::Const, Namespace::Value)`, and the
  namespace catalog accepts it with its constant header.
- **Graph roots.** `ingest_roots.rs` demands each constant's header type. The
  lazy ordinary graph maps only demanded roots, and candidate run 2 found that
  an f32, f64, i8 or u128 constant's type is not otherwise demanded ("source
  type recipe has an unresolved actual dependency").
- **Cells.** Each row installs its header type as the declaration's cell, and
  readiness checks that cell against the header: "source constant readiness
  lost its installed value type".
- **Seeding** (`imported_values::seed`). The legacy loop is unchanged. Each
  source artifact's constant rows are then resolved through the import
  catalog, rebuilt with `ScalarValue::new`, added to `imported_constants` and
  completed in the evaluator. One shared helper completes both kinds. Every
  existing reader path then serves client reads, client consts, array lengths,
  patterns and const evaluation.
- **Rebuilt bodies.** `rebuilt_operations.rs` rebuilds code 17 as
  `Rvalue::Constant` of the consumer's own definition, through
  `mapped_members.rs::constant`, which requires a constant header. Lowering and
  const evaluation already handle an imported constant.

## Fixture and evidence

[`pa29/tests/spec/named-constants`](../../pa29/tests/spec/named-constants/README.md)
holds four providers and sixteen clients. Every outcome was declared before the
oracle, the base-pin probe and any candidate run
(`pa29/tests/named-constant-outcomes.json`). Eight amendments are recorded
there, each before the run it governs.

| Control | Result |
| --- | --- |
| four providers | each exports under `module-bodies-v13` |
| `unnamed-provider` and `unnamed-twin` | byte-identical packets (2b) |
| twelve fresh clients | accepted; each exits 42 natively with empty stdout, as rustc's do |
| `const-call` | rejected with "implementation validation has no source scheduling context", a failed prediction (amendment 5) |
| three opponents | each rejected as its local twin is: "incompatible types" twice, "private value name `SECRET`" |
| nine generated provider opponents | each rejected at export by name; rustc accepts each |
| `module-branches-v12`, `module-nominals-v10` | "source declaration kind is outside the initial source profile"; no packet |
| old reader, the base pin `aecc5503` | rejects the packet with "unknown source declaration kind" |
| seventeen forged rows and bytes | each rejected by name |
| N = 8, 16, 32 constants | packets of 4844, 5540 and 6932 bytes, exactly 87 bytes per constant; validation work grows by at most a constant per constant, and the decoded packet yields exactly N constant rows |
| flip survey, the base pin `aecc5503` against the candidate `32a51ce1` | 1,401 runs over 39 packets: every difference is in `spec/named-constants` under version 13, where `provider.rs` and its ten accepted clients flip to accepted; every other export, packet and client outcome is identical |

- **Candidate runs.** Run 1 found the module-binding site and run 2 the
  graph-root demand, both listed under Export and Consumer above. Neither
  changed a declared outcome. Run 3 found `const-call`. Run 4, by the
  qualified candidate on `306b82377`, repeats run 3 line for line. The base
  pin was built from `28e2b19fb`; `306b82377` changes only two test files.
- **`const-call`.** The fence is existing and independent of 5b-1. In
  `obj/parallel/auto-traits/constants/probe-cc`, the base pin rejects a client
  `const COUNT: usize = base::four();` over a provider with no constant, only
  `impl Clone for usize`, `impl Copy for usize` and `pub const fn four() ->
  usize { 4 }`, and accepts it without those impls. The candidate accepts
  `const-call` itself over a provider that reads `LANES` without the Copy
  impls. Those impls are amendment 2's accommodation of rustc's match
  lowering. 5b-1 is not widened.
- **Forged rows.** Four amendments govern them, each recorded before its
  run:
  - The writer forgery is a surplus row, which validation's ledger rejects
    with the same text before any writer runs (amendment 4).
  - Their file was renamed `evaluated_constant_rows.rs`, because the
    architecture audit limits a filename stem to six files (amendment 6).
  - The provider declares no struct, so amendment 4 put `INNER` under a type
    parameter instead. The existing lexical forest rejects that parent first,
    with "source lexical child has a foreign kind, gap or duplicate ordinal".
    That failed prediction is kept with its amended text (amendment 7).
  - Only a struct or enum parent reaches "source constant header lost its
    free owner". Its witness is the provider plus a generated `pub struct
    Holder;`. `INNER` is renumbered as the struct's next lexical child,
    because the unit constructor is already child 0 (amendment 8).

## Recorded

- **"implementation validation has no source scheduling context".** A
  client's const evaluation of an imported const fn fails whenever the
  provider declares primitive Copy impls, with or without constants. Core
  declares exactly those impls, so the fence blocks alloc and std clients that
  const-evaluate anything imported from core. The integrator made it Lane C's
  next item, starting with a read-only diagnosis and notice.
- **Cross-crate constant reads.** A provider body that reads another crate's
  constant keeps "source constant read of another crate's declaration requires
  its wider profile" until the multi-packet item.
- **Two term copies.** A constant's value is a const term in its header row,
  beside K's argument-owned term column. A later profile can share them if
  core's constants make the duplication measurable.
- **`value_rows.rs`'s `emit()` is at Clippy's 100-line limit.** The next
  rvalue arm, expected in R2, needs a real factoring of the value-kind
  translation, not a mechanical split.
- **Order after 5b-1** (integrator, 2026-11-09): the scheduling-context fence,
  then R2, then 5b-2 and 5b-3, then HR/C, Location, PanicInfo, arrays and
  multi-packet.
