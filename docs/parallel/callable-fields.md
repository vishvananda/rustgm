# Zero-slot nominal callable fields

Lane C's (ii), 2026-11-09, on `implement/callable-fields-20261109` from
`c8c2a8eda`. The integrator approved the notice, its request_ports trace and
two deviations: the constructor-payload arm, with forged rows (a) and (b), and
R2's shape for the associated opponent. (ii) lets a version-13 module-body
export carry a struct's or tuple variant's field whose type is a closed Rust fn
pointer with no bound region, such as core's `RawWakerVTable` fields, the
zero-slot `OsFunctions` fields and the `PhantomData<fn(T)>` variance markers.

## Before

- The exporter already emitted a nominal field's fn type as a CallableType
  binder owned by the nominal, with no parent and the type's bound-region
  count, in every profile.
- `frame_owners` required a function header to own every CallableType
  binder: "source shared call type requires an actual closed callable frame".
  Export self-validation stopped there, so R2's `field` opponent recorded it.
- The port plan's `Rewrite::Nominal` arm in `function_domain` answered "source
  nominal callable field requires its wider binder profile". Gate (a) made it
  unreachable.

## Rule (no wire change)

- **Gate (a), `frame_owners`.** Version 13 also admits a CallableType binder
  owned by a struct or enum, with no parent and no slots. `BinderIndex::new`
  takes the version-13 flag that `DomainIndex::new_mode` already carries:
  `body_domains` passes it, and `domains` and `conversion_domains` do not.
  Versions 12 and 10 keep the text above.
- **Gate (b), `rewritten_roots::function_domain`.**
  - `Rewrite::Nominal { owner }` forms the field's Function root in that
    nominal's header frame. A frame with slots keeps the old text, as the HR
    fence's port-plan half.
  - An application's target is read with `DomainIndex::target`. A function
    target keeps its checks. A called tuple-struct or tuple-variant
    constructor's payload is that nominal's fields, so it is formed in the
    nominal's header frame (accepted deviation 1).
  - The shared check then requires a CallableType frame with no parent and
    no slots, owned by exactly that function or nominal, with the Rust
    convention, not variadic and region independent: otherwise "source
    Function input lost its exact zero-slot Rust formation domain".
- **Tightening, `request_ports::signature_frame`.** A call signature is its
  own call site's CallSignature frame, or a parentless CallableType frame that
  a function header owns. A nominal's frame never types a call: "source Call
  signature belongs to another actual original context". Honest packets
  cannot change: the exporter forms every call signature in the calling body's
  function context.

## Fixture and evidence

[`pa29/tests/spec/callable-fields`](../../pa29/tests/spec/callable-fields/README.md)
holds the pointer-calls provider plus authored lines and 16 clients. Every
outcome was declared before its oracle and candidate run, and frozen before
the oracle (`pa29/tests/callable-field-outcomes.json`).

| Control | Result |
| --- | --- |
| 15 fresh clients | accepted; each exits 42 natively, as rustc's do |
| `narrow` | rejected: "required lifetime does not outlive its target"; rustc: "lifetime may not live long enough" |
| six generated opponents | each rejected at export by name |
| `module-branches-v12` over the branches provider plus `Slot` | gate (a)'s text, no packet; version 13 exports it |
| old reader, base pin `b612c926` | every client rejected with gate (a)'s text |
| baseline, base pin | the frozen provider rejected with gate (a)'s text |
| 11 forged rows | rejected by name |
| versions 10, 12 and 13 over a function frame retargeted to a nominal | gate (a)'s text twice, then the formation text |
| N = 8, 16, 32 | packet bytes exactly affine; restage linear; client work exceeds `u8` twins' by 110 per struct |
| self-demand probe, k = 1, 2, 4 | 14 header steps and 289 retained bytes per fn field over `u8` twins; no visits |
| flip survey against the base pin, 1,602 runs | among existing inputs only `gen-r2-field` flips; 58 compared packets are byte-identical |

- **Clients.** A struct's fn field called by a provider fn and an inherent
  method, built by a provider struct expression and assigned by the
  provider; two `unsafe fn` fields in RawWakerVTable's shape; a tuple
  struct's and a tuple variant's fn payload, each built by the client and by
  a provider constructor call; a generic `Apply<T> { f: fn(T) -> T }`; a
  `Pair` with the hand-written expansion of `#[derive(Clone, Copy)]`;
  `Send + Sync`; and the contravariant and invariant markers in their valid
  directions, including `fn([u8]) -> [u8]` under `T: ?Sized`.
- **Opponents, known rejects-valid.**
  - Higher-ranked, early-region and C fn fields keep "source anonymous
    callable requires its supported closed Rust frame" at export.
  - A record variant holding a fn, and core's `ArgumentType` shape, keep
    "source record variants require their wider profile": item (i).
  - A safe associated fn calling an unsafe fn field keeps "source unsafe
    associated Calls require their wider actual profile". An unsafe
    associated declaration instead hits "source nonordinary callable
    effect/safety/convention requires a wider profile", as Lane D found in F.
    Their widening is one item.
- **Forged rows.** Slot's frame with a parent, with one slot, or owned by a
  trait fails gate (a). Owned by another nominal or by a function, it fails
  gate (b). A pointer call's and a direct call's signature copied into Slot's
  frame fail the tightening. A pointer call typed by `Apply`'s `fn(T) -> T`
  row, and `Apply`'s field retyped over `Contra`'s `T`, are captured
  parameters. Row (a), a function's parameter frame owned by a nominal, fails
  the function's own header formation before any application. Row (b), a
  constructor payload's frame owned by another nominal, fails gate (b).
- **Survey detail.** The new packet and the (ii) generated providers are new.
  On the base pin the frozen provider, its clients, the associated opponent
  and the older control's version-13 export stop at gate (a). The other
  opponents' export fences come first on both sides, so their texts match.

## Recorded

- **Fixture amendment 1.** rustc 1.98.0 ICEs (`error performing operation:
  fully_perform`) on a match over an i32 value in this no_core environment,
  whose provider gives i32 no Copy impl. The `apply` and `pair` clients were
  rewritten to avoid that match after the oracle and before any candidate
  run.
- **Failed prediction, amendment 2.** An empty client's header work over N
  structs, each with a function over a reference to it, grew by 5,868 and then
  12,304: 504 over the declared bound. The declared `u8` twin family grows the
  same way (4,988 and then 10,544), so the term is a pre-existing cost of
  mapping function headers over distinct nominals; the twins carry no callable
  frame. The fn fields' excess over the twins is exactly 110 per struct.
- **Harness correction.** `store` owns three closed callable frames, so row
  (a) selects its public fn-pointer parameter's frame.
- **Notice departures, declared.** The tightening's rows use a closed copy
  bound in Slot's frame: a root over the nominal's own `T` in a calling body
  is captured first (row 7). `Pair` is a hand-written expansion.
- **The exporter's arena fence.** The source-nominals provider plus a fn
  field rejects with "source anonymous callable requires its actual prepared
  type arena" in every profile, on the base pin too. It is outside this item.
- **Process deviation.** The first older-profile exploration ran both
  binaries outside the resources/storage wrapper.
- **The HR item** must also cover nominal-owned callable frames with bound
  slots, not just pointer-call frames.
