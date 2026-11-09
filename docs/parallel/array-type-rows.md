# Array type rows (B2 T)

Lane D's slice T, 2026-11-09, on `implement/array-type-rows-20261109` from
origin/main `1aa288ad1`, under Lane C's arrays design notes T1-T6. The
integrator approved the S notice, which puts T before S as two commits. T
moves whole `[T; N]` values through version-13 module-body transport. It is a
transport slice, not a library milestone, and makes no claim about an
original-core artifact or a whole assignment.

## Before

Version 13 rejected every array type at export with "source type form
requires an explicitly supported transport row". Tag 14 was reserved for
arrays in the raw-pointer and slice slice.

## Row

- **Tag 14, `Array`:** `[u8 14][id element][term length]`. The length is K's
  const term written in place: `[u8 0][id ty][u128 bits]` for a value or
  `[u8 1][id slot]` for a const parameter. Any other term tag fails with
  "unknown source const term".
- It is read only under `body_nodes()`. Every older reader fails with
  "unknown source function type recipe".
- The reader appends each term to `ColdColumns::array_lengths`, dense in
  type-row order, and the row names it by position. The writer rejects a
  column out of order ("source array lengths are not in type-row order") or a
  surplus term ("source array length has no type row").
- **Width.** `s::Type` stays 20 bytes and the port-plan node 20 bytes.
  `ColdColumns` grows by one Vec, from 200 to 224 bytes, pinned in
  `term_codec.rs`. Each tag-14 read reserves `size_of::<ConstTerm>()`.

## Exporter

`recipe_emission::type_row` emits the row only under `body_nodes()`, after
the element row. A local `Kind::Array` writes a value term on the usize row;
`Kind::ArrayTerm` writes K's `const_term` for its const. Each array type
therefore has one wire form.

## Validation and consumer

- **Validation** (`node_rows`). The element row may not be `str` or a slice
  ("source array element requires a sized type"). A value term must be on
  the usize row ("source array length term is not usize") and canonical for
  usize ("non-canonical source const value"). A slot must name a usize const
  parameter ("source array length slot is not a usize const parameter"), and
  puts the row in that parameter's domain, as a type parameter does.
- **Port plan.** `Node::Array` joins the element and length nodes, and is
  formed only when its element is. Signature comparison pairs the elements
  and the lengths. A captured receiver's Self compares array lengths exactly
  (`frame_captures::same_self`).
- **Consumer** (`demanded_types`). The element is proved sized from the
  consumer's own facts (`Store::is_sized`); no sizedness travels. A value
  folds to `Kind::Array` with its u64 length and a slot to `Kind::ArrayTerm`
  over the consumer's own const parameter.

## Boundary with Lane C's arrays item

The integrator's ruling (c), as reviewed by Lane C:

- **T owns** array types in signatures, fields and impl headers, and
  whole-value Read, Store and copy of an array-typed place. That includes the
  witness's authored `impl<T: Copy, const N: usize> Clone/Copy for [T; N]`
  headers (T5).
- **Lane C keeps** list and repeat rvalues, indexing, element patterns,
  unsizing, general `CoerceUnsized`, and core's `array/mod.rs` impls with
  their bodies and values. In T each fails at export by name:
  - list and repeat rvalues: "source rvalue requires a complete wider
    transport row";
  - an element pattern's constant-index projection: "source projection
    transport requires its wider explicit contract";
  - unsizing `&[u8; 4]` to `&[u8]`: "reference unsizing requires its actual
    source CoerceUnsized declaration", checked locally before export;
  - indexing `a[i]`: "source binary operator requires its actual language
    declaration", checked locally because the foundation has no `Index`
    item.
- **T6.** Arrays are not `#[fundamental]`, so `[Local; N]` is never an owned
  input, and C3b's knowable absence is unchanged.

## Fixture and outcomes

[`pa29/tests/spec/array-types`](../../pa29/tests/spec/array-types/README.md)
holds one provider and nine clients. Every outcome was declared in
`pa29/tests/array-type-outcomes.json` at 2026-11-09T09:31:35Z, before the
rustc oracle and before any T code.

| Control | Result |
| --- | --- |
| provider | exports under `module-bodies-v13`, a plain `[13, 0]` packet |
| five accepted clients | each exits 42 natively with empty stdout, as rustc's do: `[u8; 4]` and `[u8; 0]` values, `pass` at `[u8; 4]` and `[bool; 2]`, `Buf` and `Grid` fields, `Wide<u8, 2>` and `Wide<bool, 1>`, and whole-array copies, `base::Clone::clone` and the imported `copied` and `store` bodies |
| four opponents | each rejected as its local twin is: "incompatible types" three times (rustc E0308), "local may be uninitialized or moved at this read" (E0382) |
| list, repeat, element pattern, unsizing, indexing | each rejected at export with the text above, and no packet is written |
| `module-branches-v12`, `module-nominals-v10` | the provider stops at its aliases; `source-nominals` plus `keep` gives "source type form requires an explicitly supported transport row" and exports under v13 |
| old readers | the comment-fence pin `3960ec90` and the base pin `06b40f6b` reject T's packet with "unknown source function type recipe" |
| forged `[u8; 4]` row | term tag 2, bits above usize width, the element row as the term's type, and relabelled version 12, each rejected by name in a fresh consumer |
| forged rows | a slot length naming a type slot and the `str` row as an element, each rejected by name; writer order and surplus errors in `term_codec.rs` |
| 6, 24 and 96 array functions | packets of 41,600, 53,660 and 101,900 bytes, exactly 670 bytes per function; a client naming none emits the same 60-byte LowIR for all three |
| byte identity | the base pin `06b40f6b` (built from `1aa288ad1`) and the candidate `ca3ba92f` publish identical packets and LowIR for transparent-records, panic-attributes, macro-contexts, const-parameters, pointer-slices, named-constants, pa41 asm-transport and the native.rs prelude |
| flip survey, base pin against candidate | 497 runs per binary over 66 spec directories (53 exports, 444 clients): the only 10 differences are this packet, whose provider flips to export and whose clients give their declared outcomes |
| stream witness | "source nominal representation requires its wider complete contract", from both pins and the candidate |

- **Oracle.** rustc 1.98.0 gives every declared outcome but one
  (`obj/parallel/array-types/rustc`). The indexing control was declared
  accepted, and rustc rejects it with E0608, because without an `Index`
  item rustc indexes nothing. The failed prediction is recorded and the
  control is unchanged.
- **Indexing text.** With the integrator's approval, T's first development
  CLI (`5f97486d`) ran the indexing control once, before any other T run.
  It rejected it with "source binary operator requires its actual language
  declaration", the baseline's local text, which is frozen as observed.
- **Amendment 1.** The first receipts run showed that `3960ec90`, recorded
  as built from `1aa288ad1`, rejects the named-constants provider. It is the
  comment-fence candidate built before the fence was rebased onto 5b-1. It
  stays the approved old-reader pin. Byte identity and the survey were rerun
  against a base pin built from `1aa288ad1` itself, as recorded in the
  outcomes before that run. Against `3960ec90`, the survey's only other
  differences were the 20 named-constants runs, which are 5b-1's.
- **Focused checks.**
  - `make fastcheck` passed 5 private tests and 10 linking tests
    (`obj/qualification/fast/1794219738417137265`): T's codec, row and
    integration controls, plus the pattern-type, const-parameter, named
    constant and inherent-receiver neighbours.
  - The native root passed 2: T's clients and the named-constant clients
    (`obj/qualification/fast/1794219931532489650`).
  - `make test-upstream` verified 128 pinned fixtures and 126 Reference
    files.
- **Factoring.** Three functions reached Clippy's 100-line limit. Each was
  factored, not allowed:
  - `node_rows` gained `push_term`, which const arguments share;
  - the port plan's one-child recipes, now including references and arrays,
    finish in `single_child`;
  - the consumer's array kind is built in `array`.
- **Receipts.** `obj/parallel/array-types/receipts` (against `3960ec90`)
  and `receipts-base` (against `06b40f6b`). The pins are
  `obj/parallel/array-types/candidate` and `base`.

## Remaining

- **S** follows: `repr(simd)`'s representation byte 3 and the `movups` xmm
  control. The stream witness stops, before and after T, at `__m128`'s
  `repr(simd)`: "source nominal representation requires its wider complete
  contract".
- **Array values and element access** stay with Lane C's arrays item, as
  listed above.
