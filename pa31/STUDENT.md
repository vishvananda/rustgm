# RustGM Programming Assignment 31 — Compiler and library bindings

This is a draft handout. The assignment's grading suite will accompany
its release.

Connect the compiler's typed language operations to declarations in the
selected source sysroot. Use the pinned Rust 1.98.0 source for Linux x86-64
with aborting panics. Complete the declaration work before integrating
the source bodies that use those declarations.

## Source declarations

- Establish explicit source sysroot and crate identities. Bind language items,
  diagnostic items, builtin macros and derives, and intrinsic declarations by
  validated source identity. Check the actual owner, signature, parameter
  kinds, feature permission, and duplicate or missing bindings.
- Keep compiler language requests, library API stability, const API
  eligibility, body policies, and unsafe permission distinct. Retain the
  exact macro expansion that grants an internal allowance. A declaration's
  name or an unrelated feature request cannot grant its permission.
- Support the declarations used by the pinned library, including
  `no_core`, explicit prelude imports, internal attributes, nominal roles,
  trait aliases and compiler-provided traits. Use the expanded item identities
  and ordinary resolver scopes. Later edition-specific implicit prelude
  selection belongs to PA34.
- Preserve exact declaration and member identities across aliases and imports.
  A source provider must supply the actual declaration. Keep an explicitly
  selected source provider free of duplicate bootstrap declarations.

## Source runtime

Reuse the constant machines, operator and iteration hooks, pointer and
intrinsic kernels, and C ABI developed in PA25–PA30. Connect them to the
actual source signatures and selected bodies.

- For an intrinsic call, check declaration authority, the selected typed
  operation, and its safety and const rules. The operation's producer and
  the scope that may call it are separate facts.
- Bind the actual callable traits, tuple elements, and associated result
  before implementing dependent callable intrinsics. Preserve tuple
  spreading and the selected normal or const invocation path. Pass
  ownership, region, and cleanup facts through the ordinary pipeline.
- Use real source definitions for operators, iteration, conversions,
  primitive and aggregate helpers, and builtin-backed derives and
  formatting. Reuse selected trait and associated item proofs.
- Establish the panic-abort entry and handler path using real library
  declarations and executed bodies. PA39 adds unwinding.
- Preserve the relevant declaration, feature, and body facts when importing
  checked metadata. PA29 introduces declaration artifacts. The bootstrap
  sequence also requires transport for the source bodies and policies it uses.

## Check your implementation

Start with small source providers. Exercise valid and invalid language-item
owners, duplicate requests, signature mismatches, and incorrect caller
permissions. Check that macro-generated requests retain their origin. Then
compile separate consumers that select actual source operations, including
both runtime and const calls where permitted. Use CPPGM to link and run
the runtime controls. PA32 completes the build and execution of consumers
against the original `core` crate.
