# RustGM Programming Assignment 27: Unsafe language

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA26. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Unsafe operations, raw pointers, unions, packed access and validity.

Track unsafe permission lexically in the typed body checker. An `unsafe` block
permits its descendant expressions and closures; a nested item starts its own
context. An unsafe function requires unsafe permission at every call site,
including calls through a function pointer and selected methods. Its body follows
the pinned `unsafe_op_in_unsafe_fn` policy. Unsafe permission does not bypass
ordinary typing, initialization, ownership, packed-reference alignment or constant
evaluation rules.

Interpret retained `allow`, `warn`, `deny` and `forbid` attributes for
`unsafe_op_in_unsafe_fn` and `static_mut_refs`. Their lexical inheritance crosses
closure and nested item boundaries independently of unsafe permission. An
explicit unsafe block satisfies a denied unsafe-operation lint. Descendant
`allow` or `warn` cannot weaken `forbid`; `deny` retains its prohibition. Keep
the ordinary no-policy path free of allocated per-node policy tables.

Keep raw pointer dereferences distinct from reference dereferences in the typed
place representation. Raw accesses have no reference lifetime or inherited loan
promise. Evaluate and check the pointer storage before selecting its referent;
reject writes through `*const T` and moves of non-`Copy` referents. Forming a raw
address does not itself create a reference or its alignment promise. The direct
`&raw const *pointer` cancellation does not read its referent, while accessing a
field through the pointer still requires unsafe permission.

Mutable static storage requires unsafe permission for reads and writes. Keep its
declaration mutability separate from the permission to form a reference: the
edition-2024 `static_mut_refs` lint denies explicit and implicit references by
default. A direct raw address does not create such a reference. Retain the types
of static places actually used by each body so assignment cleanup can run an old
value's destructor before replacing it. Constant evaluation has its own mutable
static access restrictions, including inside an unsafe block.

Union fields share storage and therefore overlap during borrow checking. Exactly
one field initializes a union; reading or borrowing a union field requires unsafe
permission, while assigning a field does not. Enforce the pinned allowed-field
contract and represent union layout directly, without a discriminant or a fake
struct sequence. Validate demanded constant results using symbolic allocations,
initialization, alignment and scalar validity; never obtain host addresses or
execute input-language unsafe operations in the compiler implementation.

The current runtime union checkpoint places every field at byte offset zero and
initializes only the selected field's storage. Different fields overlap for
loans; subfields within the same selected field retain ordinary disjointness.
Moving a non-`Copy` field consumes the containing union. Field eligibility uses
the canonical `ManuallyDrop` identity, and implicit field cleanup is suppressed
while an explicit union destructor still runs.

Constant union initialization and field reinterpretation remain explicitly
unsupported. The draft tests distinguish this valid-Rust boundary from invalid
boolean values and uninitialized-byte reads. Byte initialization, provenance and
constant promotion must be qualified before release, together with the remaining
packed-access, pattern and validity controls. This checkpoint does not activate
the assignment or establish generated text references.

The original unused invalid-boolean and uninitialized-byte constant inputs are
retained unchanged. Required initializer completion now rejects them through
the unsupported-union boundary, as it does their demanded counterparts. This
rejects the invalid programs but does not establish scalar validity or byte
initialization enforcement. Both demanded and unused variants must reject for
the appropriate validity reason before release.

Final constant storage follows logical aggregate reachability. A zero-length
repeat stores no fill value, so a raw pointer computed for its operand does not
escape through that array. The operand still executes and retains every const
eligibility and validity check. Nonzero repeats retain their dangling-pointer
checks, and discarded transient raw pointers do not become stored results.

Packed by-value copies, primitive compound assignment and raw address formation
remain distinct from creating a field reference. The qualified let-reference
and match-reference controls reject in the executable pipeline. Earlier type
inspection retains their alignment obligations under PA9's phase boundary.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

required unsafe contexts; alignment; raw borrows; union reads.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.
