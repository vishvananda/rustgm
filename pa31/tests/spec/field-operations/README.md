# Demanded Field contracts and native references

This finite independent PA31 packet reuses the exact original no_core Field v4
foundation in `../field-source/original-headers-v4.rs`, including the genuine
Send diagnostic identity, actual representing struct, unsafe Field trait,
Base/Type declarations, Copy/Clone bodies and provided OFFSET initializer.
The full original module remains unchanged at `../field-source/original-field.rs`;
its Rust revision, license and exact interval hashes are recorded there and in
this packet's `provenance.json`. Ordinary u32/usize Clone/Copy implementations
are added where needed by the consumer bodies. The native pointer adapter uses
the exact original arith_offset intrinsic declaration/attributes, not an authored
replacement body.

Predictions were frozen before Rust observations. The separately frozen
`proof-only.rs` companion ran first: it requires the actual concrete Field proof
and consumes both Base and Type through genuine typed functions. The original
default OFFSET declaration remains present and checked, but there is no OFFSET
demand or static initializer. This is an initial implementation stage; it does
not replace the required complete offset/native scope.

The bounded reference batch qualifies twenty sources in both metadata and object
modes, forty observations: nine positives and eleven ordinary negatives. It also
builds and executes a real no_core object with the C observer. Three additional
invalid raw demands cause six compiler aborts and are explicitly unqualified.
There are no setup failures. `manifest.json` lists every input and settled
classification; `reference-v1.json` binds source/tool/command/artifact/diagnostic
hashes and raw receipts. No RustGM candidate or compiler edits occur here.

## Qualified operations

- `proof-only` checks genuine representing identity and exact Base/Type without
  offset execution. `generic-premises` checks actual abstract Field assumptions,
  generic field substitution and complete Base/Type equalities. Missing Field
  assumptions reject E0277.
- `wrong-full-application` rejects E0308 for Generic<u8> versus Generic<u32>.
  Equal nominal owner/field ordinals do not erase complete argument identity.
- `lifetime-positive` retains the reflected reference lifetime;
  `lifetime-escape` rejects the ordinary lifetime extension.
- `sized-field-unsized-base` and `unsized-field` reject E0277. Both original Base
  and Type contracts require Sized. `packed` also rejects E0277.
- `enum-union-unused` accepts valid raw enum/union descriptions, while each
  demanded Field opponent rejects E0277. Description formation is distinct from
  the narrower compiler-provided implementation.
- `noncopy-result` rejects E0507: the representing Self's Copy supertrait does
  not confer Copy on the selected field type.
- `foreign-members-unused` accepts actual member tags on another trait.
  `foreign-members-demand` rejects E0277 for that real unrelated parent, and
  `nominal-impostor` rejects E0277 despite matching generic kinds/physical fields.
- `alternate-unused` and `alternate-demand` both accept. Their safe Field trait
  has no original supertraits and has an ordinary u8 OFFSET default of7; Rust
  still permits the genuine builtin Type proof and its actual ordinary constant
  demand. These are separate required consumer semantics, not Rust negatives.
  A deliberately narrower implementation must retain that explicit boundary.

## Genuine native observer

`native.rs` passes metadata and object generation with installed rustc1.98.
The real Rust object links with `native.c` and executes successfully:

```text
field 4 23 29 37 41 47 31 1
```

The first value comes from the actual Field::OFFSET provided initializer and
field_offset CTFE for a repr(C) record. The observer independently compares it
to C offsetof. Rust-struct and tuple offsets are not guessed: the same real
initializer supplies each offset, and the generic projection reads the distinct
runtime value stored at that location. Further calls consume a generic record,
a lifetime-bearing reference payload, and a callback-produced field value.
The callback must run exactly once. None of the Rust exported results substitutes
a hand-coded offset or constant result for the typed operation.

The helper takes a real reference to F::Base and returns the actual F::Type
reference in the same lifetime, using the declared offset and genuine pointer
intrinsic inside explicit unsafe code. The C caller supplies valid storage for
the raw-pointer input. No lifetime, write, alias or ownership waiver is added.

## Preserved unqualified raw applications

`invalid-raw-unused` accepts raw aliases with out-of-range indices or a scalar
base. Their three demanded companions panic inside pinned rustc's field-info
lookup and abort with SIGABRT(-6), contrary to the predicted exit101 convention.
The six observations and ICE files remain under
`obj/parallel/field-operation-reference/v1`, with hashes bound in the reference.
They were never executed or counted as ordinary Rust negative parity. A compiler
implementation must validate its coordinates before indexed access; these crashes
do not authorize either an ordinary builtin proof or a claimed reference rejection.

All direct work used one immediate-admission shared resource/storage wrapper,
90-second outer deadline, explicit16 GiB output cap, unchanged150 GiB reserve and
two-job global policy. Inner work took10.069 seconds and outer status was0.
Rust was1.98.0, edition2024, x86_64-unknown-linux-gnu, panicabort, capallow,
bootstrap1, two threads; object and native hashes are in the receipt.

Root owns the actual source-backed evidence, projection, default-body dispatch,
const-only intrinsic and layout implementation. Builtin field_of expansion,
privacy/name resolution, actual field access, full original Debug/reflection and
remaining method bodies, alternate demanded profiles and authenticated transport
remain required. This reference packet neither grants those permissions nor
completes a whole assignment or original library artifact.
