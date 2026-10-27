# RustGM Programming Assignment 23: Trait objects

Status: **implemented**. The maintainer reference and manifest-listed text
suite implement the contract below. Prerequisite: PA22. The source profile is
Rust 1.98.0, edition 2024, Linux x86-64.

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
  declaration rules still apply. Stable Rust's negative-impl syntax is excluded.
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

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier views remain available.
Success exits 0 with deterministic LowIR and empty stderr; diagnosed rejection
exits 1 with empty stdout. Signals, panics, timeouts and exit 86 fail the test.
Committed `.ref`, `.ref.stderr` and `.ref.exit_status` sidecars are byte oracles.
`reference.json` records the source and input identities used to generate them.

An object reference occupies 16 bytes: data address followed by vtable address.
The course table starts with destructor address, concrete size and alignment,
then method slots and demanded upcast-table pointers. A null destructor entry
means no destruction work; a null conditional method entry remains uncallable
without proof. This is the course ABI, not a claim about rustc's unspecified
vtable layout. Reuse the existing aggregate/out-pointer calling convention,
ordinary instance discovery, destructor recipes and unchanged CPPGM backend.

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
facts once and reuse constant-definition indexes. Measure chains of demanded
bodies and growth in unrelated definitions; a cache-hit scan can still be
quadratic. Keep actual work/storage counters alongside frozen-binary A/A and
paired ABBA measurements, and verify identical shared outputs. `spec.md` owns
the detailed architecture and performance requirements.

Use safe idiomatic Rust. Clippy `all` and `pedantic` warnings are errors. Keep
lint exceptions narrow and justified; do not use blanket allowances or split
files mechanically to evade the syntax-aware audit. Compiler sources must not
read environment variables. The file, function, nesting and phase checks in
`scripts/rust_audit.py` apply independently of the text suites.

## Tests and completion

See [tests/README.md](tests/README.md) for the published matrix. Every fixture
has a declared outcome, independently checked with pinned rustc before reference
generation. Runtime controls compare RustGM through CPPGM with rustc; compile-only
controls make no claim that their unused function was executed. Maintainer tests
also inspect and directly invoke a table's destructor header through typed LowIR.

Run `make test-pa23`, `make test-native-pa23`, `make test-rustc`, all active earlier
byte/native suites, the complete release Cargo and native maintainer suites,
formatting, strict Clippy, architecture/file audits, harness, upstream integrity
and the owning performance checks. Follow `TESTING_AND_REFERENCES.md` and use the
storage guard with scoped cleanup. Qualification and reviewed representation
changes are recorded in [the release evidence](../docs/pa23-release.md).
PA24 and PA25 remain separate assignments and cannot be counted as passing here.
