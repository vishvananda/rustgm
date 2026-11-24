# RustGM Programming Assignment 23 — Trait objects

Build on [PA22](../pa22/README.md) to check trait object formation and
lower vtables, object upcasts, and dynamic calls. Use Rust 1.98.0,
edition 2024, on Linux x86-64.

## Required implementation

Extend PA22's unsized storage and coercions with trait-object types, their proof
obligations, vtables and dynamic calls. Preserve typed phase boundaries and the
existing LowIR backend contract.

- Canonicalize the principal trait, complete type/const arguments, associated
  equalities, quantified lifetimes, object lifetime and auto-trait set. Support
  marker-only objects and aliases. Distinguish user traits from builtin markers
  by resolved identity. Retain the binding time of defaulted object lifetimes,
  including defaults supplied by container parameters and supertraits.
- Check dyn compatibility before forming an object. Reject incompatible
  supertraits, associated constants, unrestricted generic methods, required
  generic associated types and forbidden uses of `Self`. Respect `Self: Sized`
  opt-outs. By-value `Self` methods cannot be invoked through borrowed objects.
- Prove concrete-to-object coercions and object upcasts through the ordinary
  trait, equality and lifetime machinery. Upcasts require the principal's own
  supertrait evidence; an unrelated implementation for an object is insufficient.
  Dropping markers is permitted, while adding one requires retained principal
  evidence. Preserve full quantified and associated-type obligations.
- Permit nominal struct-tail reference `as` casts through the existing object
  coercion proof and selected terminal conversion. Preserve single evaluation,
  prefix storage, virtual dispatch, mutability and borrowed object lifetimes.
- Support shared/mutable references and raw-pointer metadata, nominal unsized
  tails, constants and promotions. Compute dynamic tail alignment from the
  selected table without changing the established packing and validity checks.
  Raw dereference semantics remain owned by PA27.
- Dispatch shared/mutable receiver methods, inherited and default methods,
  associated argument/results, and lifetime-generic methods. Reified methods use
  ordinary callable identities. Prove additional method conditions at the call
  or reification site; an absent conditional method slot is no proof of a bound.
- Implement structural `Send`, `Sync`, `Unpin`, `UnwindSafe` and `RefUnwindSafe`
  obligations for the supported language, including explicit local nominal
  implementations and inherited object markers. An explicit implementation
  replaces the nominal family's structural default; its conditions and unsafe
  declaration rules still apply. Negative impls are outside the PA23 profile.
- Auto-trait demands on return-position `impl Trait` may require a completed
  defining body. Retain the pending proof, finish that body once, and close its
  hidden lifetimes using the actual CFG and borrow proof. Keep private hidden
  types out of ordinary trait/equality permissions. Distinct precise capture
  lists must retain separate identities. Cyclic defining dependencies diagnose;
  structural coinduction cannot turn an opaque reveal into a completed proof.

The pinned language reference covers [trait objects](../doc/rust-reference/src/types/trait-object.md),
[dyn compatibility](../doc/rust-reference/src/items/traits.md#dyn-compatibility),
[lifetime defaults](../doc/rust-reference/src/lifetime-elision.md), and
[coercions](../doc/rust-reference/src/type-coercions.md).
Owning library pointers such as `Box` belong to PA26. This assignment implements
borrowed objects and their destruction metadata without requiring an owning
object pointer, closure call traits, procedural macros or a Rust standard library.

## Runtime and text contract

Use PA11's `rustgm --emit-lowir` interface and input/output options.
Success exits 0 with deterministic LowIR and empty standard error.
A source error exits 1 with a diagnostic and empty standard output.

An object reference occupies 16 bytes, with the data address first and the vtable address second.
The course table starts with destructor address, concrete size and alignment,
then method slots and demanded upcast-table pointers. A null destructor entry
means no destruction work; a null conditional method entry remains uncallable
without proof. Use this course ABI for LowIR. Reuse the aggregate and
out-pointer calling convention, instance discovery, and destructor recipes.

Build tables lazily from selected concrete implementations. Deduplicate by the
complete runtime object shape and concrete type. Reserve identities before
following table dependencies, and drain dependencies iteratively. Runtime lifetime
erasure follows proof; it cannot strengthen an object's lifetime or marker set.

## Implementation and performance

Keep source parsing, resolution, semantic proof, CFG construction and LowIR
lowering separate. Consume typed decisions, never rendered views or reparsed
source. Reuse the existing trait-selection, region, promotion and cleanup engines.
Retain canonical queries and completed work across readiness waits. Keep dense
ordinary body scratch; suspend only the waiting body's touched state. Preserve
completed constant-pattern batches when restoring an ancestor body.

An orchestration layer may request an early defining CFG and borrow proof, then
retain that CFG and contract for ordinary lowering. Consume appended lifetime
facts once and reuse constant-definition indexes. Test long chains of demanded bodies and growth in unrelated definitions.
Check that completing one body resumes its dependents without rescanning
the entire crate.

## Run the tests

Run the PA23 test target from the repository root.

```sh
make -C pa23 test
```

[The manifest](tests/manifest.json) covers object formation, upcasts,
dynamic dispatch, and destruction metadata. Use the
[PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
to run examples that exercise inherited and default virtual methods.
