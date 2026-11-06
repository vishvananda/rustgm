# Reborrow reference successor

These are independent Rust observations, with no RustGM candidate or production
edit. The exact actual no_core role/support prefix comes from the existing
`pa31/tests/spec/reborrow-source/source.rs`. The additional PhantomData support
retains its pinned declaration and uses real ordinary Clone/Copy bodies.

Every version's source, observer, prediction and raw diagnostics remain intact.
`successor-reference-v1.json` in the same fixture packet binds their hashes,
commands and receipts, plus the previously completed local-negative evidence.
All direct runs used the existing resource/storage guards with 90-second outer
deadlines, 16 GiB output caps and the unchanged 150 GiB reserve/two-job policy. Rust
was installed Rust 1.98.0, edition 2024, x86_64-unknown-linux-gnu, panic=abort,
cap-lints=allow, bootstrap 1 and two compiler threads. No native RustGM artifact
is claimed.

## Qualified current observations

- `successor-native-v3.rs` passes metadata, genuine object generation, C linking
  and execution. Output is `successor 17 23 19 31 23 59 1`. Two-data-field nominal
  values preserve their Copy tag across repeated concrete and explicitly typed
  generic calls. A real `locate(&raw mut holder).inner` operand calls the C marker
  exactly once; its original pointee is written. Retained real Clone bodies mark
  an accidental call, and the observer rejects any such call.
- `successor-inferred-v1.rs` rejects E0382. An unconstrained generic target cannot
  gain an implicit reborrow merely because its type becomes known afterward.
- The same-ADT generic return in `successor-relay-v1.rs` rejects E0515 in both
  metadata and object modes. The initial accept prediction was disproved. This
  is a negative; it is not a runtime-positive target.
- `successor-static-unused-v1.rs` accepts. Its demanded partner rejects E0597.
  The ordinary unused implementation cannot supply the actual operation's field
  lifetime proof.
- `successor-unsafe-v2.rs` rejects solely E0133 for dereferencing the real raw
  source pointer. The implicit operation does not erase source unsafety.

## Retained unqualified attempts

Native v1 produced E0716 for an inner constructor temporary and ICE101 during
borrow checking of `zero`. Native v2 named the inner values and removed that zero
operation, but ICE101 remained in the single-data-field `nested` operation.
Neither source was executed or counted as semantic parity. Native v3 removes
only the demanded nested function/observer call; genuine nested/zero declarations,
impls and checked ordinary Clone bodies remain. Their runtime behavior remains
required work and is not proved by unused declaration acceptance. Unsafe v1 also
had the temporary-lifetime issue; its exact bytes remain alongside v2.

## Existing fourteen local negatives

The evidence selects eleven real ordinary failures from local v1, shared-write
E0506 and raw-source E0133 from local v2, and overlapping-borrow E0499 from
local v3. This yields fourteen qualified ordinary negatives without rerunning
them. All actual source hashes and error causes are bound individually.

The three local v1 std-unavailable E0433 setup failures are preserved. Localv2's
overlap input instead failed E0308 because its helper expected an Exclusive
while `keep` returned `&mut u8`; that is not a borrow-conflict proof. Localv3
corrected only those helper parameter types and reached the intended E0499.

## Unchanged source MIR

A separate pinned `--emit=mir`, mir-opt-level 0, include-spans=yes observation
passes for unchanged `source.rs`. Its artifact remains in
`obj/parallel/reborrow-successor/mir-v1/source.mir.txt`, SHA256
`a99fb753aed8929f0c3e417ad4bc508f12786faa46147a22cbbc19660c4f0354`.
The constructor directly initializes nominal local_2 after borrowing through
argument_1. There is no nominal Reborrow on that let initializer. Both exclusive
calls and the shared call reborrow/coerce shared from actual local _2, each before
moving its separate result into the actual callee. The exact source spans and
command are bound in the JSON; no candidate source-place structure is inferred.

Nested/zero runtime, broader guards, closures, Drop, const and authenticated
transport remain required. Root owns candidate test registration and joined
compiler qualification; this finite reference does not release PA31 or build
the original core/alloc/std artifacts.
