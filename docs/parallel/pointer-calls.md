# Calls through fn pointers

Lane C's R2, 2026-11-09, on `implement/pointer-calls-20261109`. Checkpoint 1
factors `value_rows.rs`'s `emit()` into families, byte-identically; checkpoint
2 is the item. Both are rebased onto `645df0a92`. The integrator approved the
notice, its rulings, the option-(a) scope ruling and Q3's Branch A, and Lane D
reviewed the codec. R2 lets a version-13 module-body export carry a call
through a closed Rust fn pointer: rustc's `TerminatorKind::Call` whose callee
operand is a `FnPtr` value. It also admits constructor reification.

## Before

- The exporter wrote every `Rvalue::Call` as a direct call, and its request
  ports required a function-item callee: "source Call requires its actual
  direct function-item application". R1's `indirect` opponent recorded that
  text.
- R1's Q3 probe exported reified constructors, but its single-crate controls
  stopped at that fence, so R1 fenced constructors with "source constructor
  reification requires its shim transport".
- core's `Argument::fmt` calls the formatter that `argument_new!` stores.

## Wire: a second shape of value code 30

- `(30, [Type(signature), Value(callee), Range(arguments), Flag(safety)])`,
  read as `ValueKind::PointerCall`, admitted only under the body-node
  profile. A direct call keeps `(30, [Type, Value, Range])`. The read matches
  the whole operand-kind slice, so the shapes cannot be confused (Lane D's
  ruling a: one stable code per Rvalue form).
- The flag is the request codec's: 0, 1 and 2 are Safe, ExplicitUnsafeBlock
  and UnsafeFunction, in the order the request and asm codecs use. Any other
  value is "unknown source call safety context".
- No request row: a pointer call selects no target. Its site is its one
  evaluation, and the value names everything a consumer re-proves. Request
  coverage stays exact: every direct Call owns one request, and a pointer
  call owns none.
- Existing packets keep their bytes, because export fenced the form before.
- The legacy column codec and the supplied-profile check name the profile they
  need: "source pointer call requires the version-thirteen body IR profile".
  Versions 12 and 10 keep the direct-call text, before any column is written.
- Older readers reject the node with "body IR value node has malformed
  operands", the same text an operand-swap forgery gets; the two cannot be
  told apart. A packet that also holds a target-feature row reports F's flags
  text first.

## Export

- `value_rows.rs`: a `Call` whose callee's type is a fn pointer becomes
  `PointerCall`, in the callable family that checkpoint 1 factored.
- `obligation_rows/request_ports.rs`: the call must be in a runtime original
  context ("source pointer Call requires a runtime original context"). Its
  lexical safety comes from the same `SourceCallUse::call_safety` a direct
  call uses. A safe callee always records Safe, inside an unsafe block or
  not, because only unsafe calls retain a discharge record.
- Constructors: the exporter's constructor fence is retired. Each consumer
  defines its own constructor instance, as the Q3 client's own
  `rg.27.i2` in `some.lowir` shows.

## Consumer

- **Domains and ports.** `operation_domains.rs` owns the callable family,
  `ssa_ports.rs` orders the callee before the arguments, and
  `frame_owners.rs` and `ingested_domains.rs` read the signature as a direct
  call's.
- **Re-proof** (`request_ports.rs::pointer_call`, then
  `value_requirements.rs::pointer_inputs`):
  - the callee's type row must be a fn pointer: "source pointer Call lost its
    fn-pointer callee";
  - the signature must be a fn pointer with the callee's unsafety, no bound
    regions and one parameter per argument, and then equal convention,
    variadicness and node-by-node parameters and result: "source pointer Call
    signature differs from its callee's pointer type";
  - its frame must be this body's call signature or a parentless callable
    type that a function owns: "source Call signature belongs to another
    actual original context". (ii) added the function-owner condition, so a
    nominal's field frame never types a call;
  - the original must be runtime;
  - then argument and result relations, shared with direct calls.
- **Safety, checked twice with direct Call's own matrix and texts.**
  `call_safety` is factored out of the direct `call()` and serves both. The
  rebuilt-body check (`safety_requests.rs`) applies `matching`, the existing
  Call match moved into a function, to each pointer call's rebuilt
  `Rvalue::Call`. So a safe callee claiming ExplicitUnsafeBlock is rejected,
  as for a direct call.
- **Rebuild and lowering.** `rebuilt_operations.rs` rebuilds `Rvalue::Call`
  on the pointer value, and local lowering calls the pointer as it does for a
  client's own pointer call.

## Q3 re-probe: Branch A

Declared before its build and run, with R1's four steps unchanged and entries
that bind the call result to a local first. The probe binary was the R2 tree
minus the constructor fence.

1. The provider exports. The packet holds three code-23 nodes over code-16
   Function values and no other new form.
2. The `ctor`, `variant` and `some` fresh clients are accepted. For
   `some::<u8>`, the client defines the constructor instance itself.
3. The single-crate controls are accepted.
4. Each exits 42 natively.

R1's `struct-ctor`, `variant-ctor` and `generic-ctor` opponents were amended
to exported before their next run (reified-item amendment 2).

## Fixture and evidence

[`pa29/tests/spec/pointer-calls`](../../pa29/tests/spec/pointer-calls/README.md)
holds the reified-items provider plus authored lines, and 13 clients. rustc
1.98.0 accepts each and it exits 42. Every outcome was declared before its
oracle and candidate run (`pa29/tests/pointer-call-outcomes.json`).

| Control | Result |
| --- | --- |
| 13 fresh clients | accepted; each exits 42 natively, as rustc's do |
| 11 pointer-call nodes | 7 Safe, 3 ExplicitUnsafeBlock, 1 UnsafeFunction |
| six generated opponents | each rejected at export by name |
| `module-branches-v12`, `module-nominals-v10` | the direct-call text, no packet |
| old reader, base2 `a9d8ac32` | every client rejected with "body IR value node has malformed operands" |
| four fresh-CLI wire forgeries | three same-width kind swaps are malformed operands; flag 3 is unknown |
| private codec test | honest rows round-trip; a resized Range and flag 3 reject at the node |
| 12 forged rows | rejected by name |
| N = 8, 16, 32 pointer calls | packet bytes exactly affine; restage and client work grow by at most a constant per call |
| flip survey against base2, 1,524 runs | among existing inputs, only R1's four amended opponents flip; every other status, text and packet byte is unchanged |

- **Clients.** A pointer parameter, two nested calls through one pointer, a
  generic `fn(T) -> T`, a body that reifies an item and calls it, an unsafe
  pointer in an unsafe block of a safe fn and of an unsafe fn, a bare call
  under `#[allow(unsafe_op_in_unsafe_fn)]`, a safe pointer in an unnecessary
  unsafe block, the fmt-shaped twin (a free unsafe fn calling its formatter
  parameter), a safe pointer call in an inherent associated fn, and the
  three constructors.
- **Opponents.**
  - Higher-ranked and C pointers keep "source anonymous callable requires
    its supported closed Rust frame".
  - An unsafe pointer call in an associated fn kept "source unsafe
    associated Calls require their wider actual profile" (ruling 4). It
    exports since the associated-unsafe item
    ([guarded-methods.md](guarded-methods.md), outcomes amendment 3).
  - The stored-formatter witness, a record variant holding an `unsafe fn`,
    stops at "source record variants require their wider profile".
  - A field call `(s.f)(7)` stopped at "source shared call type requires an
    actual closed callable frame". It exports since (ii) (outcomes
    amendment 2).
  - These five are known rejects-valid: rustc accepts each. The last two are
    pre-existing nominal-header fences outside R2's body scope (option a).
    Exploration showed base2 rejecting the same shapes with the same texts
    over a provider without pointer calls, so they are not flips.
  - A const fn calling a pointer: RustGM's sema text is "non-const calls are
    not allowed in const functions". rustc rejects it too, with the
    unnumbered "function pointer calls are not allowed in constant
    functions"; the declared E0015 was a failed prediction of the code only.
- **Survey detail.** The new packet and R2's own generated inputs are new on
  both sides. On base2, R2's associated, field and const-fn opponents stop
  first at the frozen provider's own pointer calls, so only their texts
  differ, and the older control exports under version 13 only on the
  candidate. Its version-12 and version-10 texts match.
- **Forged rows.** A function-item callee, a request row on a pointer call,
  u16 parameters, dropped `unsafe`, each of the four rejected safety
  combinations of the matrix, a cyclic callee, and a constructor
  reification's pointer with a changed payload or result type.
  - Failed prediction (outcomes amendment 1, recorded after fc7 and before
    any further run): the payload forgery `fn(u16) -> P` stops at "source
    operation omitted a required nontrivial formation input", not the
    re-proof. Its nominal result is not intrinsically formed, and the
    forgery supplied no formation input. A twelfth row adds the WellFormed
    obligation a consistent forger would supply, and it reaches "reified
    function pointer differs from its item's signature", as the result
    forgery does.

## Recorded

- **Process deviation.** The first compiled candidate's fastcheck (fc2)
  observed the `indirect` flip before its amendment was written; it is not
  evidence. The first test run after the amendments is fc7, and fc8 passes
  every selection.
- **Test file names.** The architecture audit's repeated-stem rule allows
  six `pointer*` files, so the tests are `called_pointers.rs`,
  `called_pointer_rows.rs` and `called_pointer_wire.rs`.
- **The fmt chain.** core's `ArgumentType` stores its formatter in a record
  variant whose field is a fn pointer. It needs two blockers ahead of HR/C,
  and R2 alone does not unblock it:
  - "source record variants require their wider profile";
  - nominal callable fields: `frame_owners` requires a function header to
    own each callable-type binder ("source shared call type requires an
    actual closed callable frame"), and the port plan keeps "source nominal
    callable field requires its wider binder profile".
- **Associated unsafe calls.** Widening "source unsafe associated Calls
  require their wider actual profile" for direct and pointer calls together
  was a separate item before HR/C. Version 13 now carries it
  ([guarded-methods.md](guarded-methods.md)).
