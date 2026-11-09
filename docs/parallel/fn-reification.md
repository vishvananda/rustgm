# Fn-item reification

Lane C's fn-item reification item, 2026-11-09, developed on `e0bc74592` and
rebased onto `dcb956cf8`. The integrator approved the design notice and its
four scope changes. R1 lets a version-13 module-body export carry a fn item
coerced to a closed Rust fn pointer, which is rustc's `ReifyFnPointer` plus its
`UnsafeFnPointer` step when a safe item meets an `unsafe fn` target. R2, calls
through a fn pointer, is a separate commit.

## Before

`value_rows.rs` rejected `Rvalue::Reify` with the generic "source rvalue
requires a complete wider transport row". The body builder emits Reify for
`AdjustmentKind::FunctionPointer`. Three consequences:

- the nonnull-patterns layout witness's single-crate control could not be
  built;
- core's bodies reify fn items for fn-pointer fields and callbacks;
- core's `fmt/rt.rs` `argument_new!` reifies `<T as Display>::fmt` under a
  rigid `T` inside the const fn `new_display<T: Display>`.

## R1: value code 23

- **Wire.** `(23, [Value(item)])`, read as `ValueKind::Reify(u32)`, and admitted
  only under the body-node profile. Value codes follow the typed body IR's
  Rvalue order. The value's type is the target's `Type::Function` row. No
  coercion decision is serialized.
  - Its operand is a preceding code-16 Function value of `FunctionItem` type.
  - The application row is the one direct calls already write. Each consumer
    re-selects a trait method target from it, as for a direct call.
  - Existing packets keep their bytes, because export fenced Reify before.
  - The legacy column codec names the profile it needs ("source function
    reification requires the version-thirteen body IR profile"). Version 12
    export stops at the generic text, before any column is written.
- **Export** (`value_rows.rs::reify`). The operand must be the function-item
  value that precedes it, of the same declaration, and the result must be a fn
  pointer. Kept fences, each by name, each accepted by rustc:
  - a closure: "source closure reification requires its closure transport".
    This fence is defensive: closure construction stops first, at the generic
    rvalue text.
  - a tracked function: "source tracked function reification requires its
    caller-location ABI". rustc reifies a `#[track_caller]` function through a
    ReifyShim, which belongs with the Location/caller-ABI item.
  - a constructor: "source constructor reification requires its shim
    transport" (Q3, Branch B, below).
  - a higher-ranked or C pointer type keeps its type-row text, "source
    anonymous callable requires its supported closed Rust frame".
  - A fn pointer over a type parameter is formed only conditionally, so its
    reification requests formation, which the consumer proves again.
- **Consumer domain** (`operation_domains.rs`). The operand is a preceding
  Function value, and the value's type is a `Function` row. Otherwise: "source
  reification lost its function item or pointer type". The operand's Function
  value already charged `function_use` and its formation.
- **Consumer re-proof** (`applied_signatures.rs::reified`). The item's applied
  signature is the template a direct call opens. It is compared with the
  pointer as a client's own `let f: fn(..) = item;` compares it:
  - convention and variadicness are equal;
  - `unsafe` may be added, never removed;
  - the admitted pointer domain binds no region, so the item opens no late
    slot and interned parameter and result nodes compare exactly.
  - Otherwise: "reified function pointer differs from its item's signature".
- **Rebuilt operations, lowering and instances.** The consumer rebuilds
  `Rvalue::Reify` and reuses its local lowering and instance demand. An
  imported non-generic item is emitted from its transported body as a direct
  call's would be. A generic instance (`id::<u16>`) is instantiated by the
  consumer.
- **Profile.** `ordinary_profile.rs` admits Reify among version 13's regional
  values.

## Q3: constructor reification

Both branches were declared before the probe. The probe (the R1 build with
the exporter's constructor fence removed) passed export, and three fresh
clients each emitted their own constructor shim. Its single-crate controls
stopped first at R2's indirect-call fence, and two of them also at "source
place transport supports locals and reference projections only". So R1 takes
Branch B and fences constructors by name. R2 re-probes them with entries that
bind the call result to a local first. If all four declared steps pass, R2
admits constructor reification with its declared opponents and a native
client.

## Fixture and evidence

[`pa29/tests/spec/reified-items`](../../pa29/tests/spec/reified-items/README.md)
holds the callable-roles provider plus authored lines, and nine clients. rustc
1.98.0 accepts each and it exits 42:

- a safe item;
- a generic instance, and a formation-requesting generic;
- an inherent method, and a trait method;
- a rigid-`Self` method in a const fn, shaped as `argument_new!`, at a client
  type and at a provider type;
- a safe item reified to `unsafe fn`, and an unsafe item.

`make` returns `fn(T) -> u8` over `fn show(self)`. core's `fn(&T, ..)` is
higher-ranked, which stays fenced.

Every outcome was declared before its oracle and candidate run
(`pa29/tests/reified-item-outcomes.json`).

| Control | Result |
| --- | --- |
| nine fresh clients | accepted; each exits 42 natively, as rustc's do |
| ten generated provider opponents | each rejected at export by name; rustc accepts each |
| `module-branches-v12` | rejects the reification with the generic rvalue text and writes no packet |
| old reader, the C2b pin `4c63e7bb` | rejects every client with "encoded Rvalue::Reify is not reconstructable by this consumer" |
| seven forged rows | rejected by name: non-Function operand, non-pointer type, changed parameter, changed result, dropped `unsafe`, convention C, another original's binder |
| N = 8, 16, 32 reified pairs | packet bytes exactly affine; restage and client work grow by at most a constant per pair |
| flip survey against the C2b pin | only the new packet differs |

- **Generated opponents.** Two of them needed oracle care:
  - The tracked opponents declare the oracle-only `Location` lang item that
    rustc's reify shim needs. This amendment was recorded before any
    candidate run.
  - The explicit cast `hold as fn(u8) -> i32` is `Rvalue::Cast`, not Reify. It
    is rejected with "source cast operand requires a closed primitive type", a
    recorded gap.
- **Forged rows.** The convention-C and foreign-binder forgeries stop at the
  existing Function domain ("source Function input lost its exact zero-slot
  Rust formation domain") before the re-proof is reached.
- **The nonnull amendment.** Recorded before its run: the layout witness's
  single-crate control now builds, and its entry and `hold` equal the fresh
  `Nanoseconds` witness's.

## Recorded

- **The fmt chain's next dependencies stay fenced.**
  - `argument_new!`'s transmute to `unsafe fn(NonNull<()>, &mut Formatter<'_>)
    -> Result` needs the transmute intrinsic transport.
  - Calling the stored formatter needs R2.
  - The pointer type itself is higher-ranked, so it needs the HR/C anonymous
    callable item that follows 5b. Its witness is `fn(&T, &mut Formatter<'_>)
    -> Result`.
- **Target-feature functions.** Lane D's F admits only `unsafe`
  target-feature exports, so a reification can only produce an `unsafe fn`
  pointer, which rustc allows. If F later admits safe target-feature
  functions, a coercion to a safe fn pointer must be rejected.
- **Explicit fn-item casts** (`item as fn(..)`) lower as `Rvalue::Cast` and
  stay fenced by the scalar-cast text.
- **Order after R1** (integrator, 2026-11-09): 5b-1, then R2. 5b-1 is
  developed on K's join commit `fe860d6be` and qualified once it is on
  origin/main. If K needs an amendment, R2 goes next instead.
