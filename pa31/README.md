# RustGM Programming Assignment 31 — Compiler and library bindings

Run the grading suite with `make -C pa31 test` (finite contract covering
`source-declarations`: 41 local, attribute/lint, and cross-crate metadata/native
transport checks).

Connect the compiler's typed language operations to declarations in the
selected source sysroot. Use the pinned Rust 1.98.0 source for Linux x86-64
with aborting panics. Complete the declaration work before integrating
the source bodies that use those declarations.

## Source declarations

- Establish explicit source sysroot and crate identities. Bind language items,
  diagnostic items, builtin macros and derives, and intrinsic declarations by
  validated source identity. Check the actual owner, signature, parameter
  kinds, feature permission, and duplicate or missing bindings. Transport
  the diagnostic-owned unwind-safety traits by their actual declarations so
  fresh clients retain the auto-trait roles.
- Keep compiler language requests, library API stability, const API
  eligibility, body policies, and unsafe permission distinct. Retain the
  exact macro expansion that grants an internal allowance. A declaration's
  name or an unrelated feature request cannot grant its permission.
- Preserve checked library feature relationships on the availability policy.
  A fresh client can request the feature named by `implied_by` to use that
  API. A request using `old_name` diagnoses the replacement.
- Check the stability and documentation attributes used by the pinned
  source. Distinguish current and scheduled deprecation and apply the
  scoped lint at the call, import or macro expansion. Preserve the exported
  macro definition's deprecation for fresh clients, including through aliases.
  A deprecation attribute on a re-export leaves the target's policy in force.
  For unstable trait
  implementations, use the stability of the types and traits named in
  the checked header. Validate `doc(cfg)`, `doc(auto_cfg)` and
  `doc(notable_trait)` where the source uses them. Check the placement, options,
  and format arguments of `diagnostic::on_const` and
  `diagnostic::on_unmatched_args`.
- Support the declarations used by the pinned library, including
  `no_core`, explicit prelude imports, internal attributes, nominal roles,
  trait aliases and compiler-provided traits. Use the expanded item identities
  and ordinary resolver scopes. Later edition-specific implicit prelude
  selection belongs to PA34.
- Compute which public declarations a fresh crate can reach. Follow public
  re-exports and the types exposed by public signatures, then apply scoped
  `unreachable_pub` settings to public declarations outside that reach.
  Include types exposed by checked associated-type implementations and use
  the actual `Debug` trait identity for `missing_debug_implementations`.
  Use the exported names and retained documentation attributes for
  `missing_docs`. For `multiple_supertrait_upcastable`, reuse typed direct
  bounds and dyn compatibility.
- Preserve exact declaration and member identities across aliases and imports.
  A source provider must supply the actual declaration. Keep an explicitly
  selected source provider free of duplicate bootstrap declarations.

## Source runtime

Reuse the constant machines, operator and iteration hooks, pointer and
intrinsic kernels, and C ABI developed in PA25–PA30. Connect them to the
actual source signatures and selected bodies.

- For an intrinsic call, check declaration authority, the selected typed
  operation, and its safety and const rules. The operation's producer and
  the scope that may call it are separate facts. Resolve language items,
  declared intrinsics (including SIMD intrinsics, `const_eval_select`, and
  direct intrinsics that also carry Rust fallback bodies), sizing fallbacks
  (`Sized`/`MetaSized`/`PointeeSized`), and compiler-synthesized shims
  (tuple/closure/function `Clone`, `FnPtr`, and drop glue) through the same
  role and intrinsic lookup whether the declaration is local or imported.
- Bind the actual callable traits, tuple elements, and associated result
  before implementing dependent callable intrinsics. Preserve tuple
  spreading and the selected normal or const invocation path. Pass
  ownership, region, and cleanup facts through the ordinary pipeline.
- Use real source definitions for operators, iteration, conversions,
  primitive and aggregate helpers, and builtin-backed derives and
  formatting. Reuse selected trait and associated item proofs.
- Establish the panic-abort entry and handler path using real library
  declarations and executed bodies. PA40 adds unwinding.
- Preserve the relevant declaration, feature, and body facts when importing
  checked metadata. PA29 introduces declaration artifacts. Preserve the order
  and ownership of type alias requirements, including dependent associated
  type projections and const parameters. Preserve associated constant owners
  and the construction and pattern rules for `#[non_exhaustive]` structs and
  enums. Fresh clients must be able to check these rules against the artifact.
  Retain the checked `#[rustc_as_ptr]` diagnostic marker on the function or
  method that declares it. Preserve a trait method's written `final` policy
  through metadata so a separate client uses the provided body and rejects
  an attempted override. Carry a specializing impl's parent, generic
  substitution and written method defaultness. Fresh clients use inherited
  methods and associated types through the provider's checked impls.
  Preserve both specialization-trait policies on their defining trait
  headers. Check the client's feature request when it supplies a new
  implementation of an imported always-applicable trait. When instantiating an
  imported generic body, evaluate its internal unstable features and edition
  rules under its defining provider's permissions rather than the client's
  feature list. Because an untracked trait method (such as `Into::into`,
  `Mul::mul`, or `BitXor::bitxor`) can dispatch to a `#[track_caller]`
  implementation in the provider, retain caller-site locations on symbolic and
  overloaded trait calls whenever `Location` is available.

## Check your implementation

Start with small source providers. Exercise valid and invalid language-item
owners, duplicate requests, signature mismatches, and incorrect caller
permissions. Check that macro-generated requests retain their origin. Then
compile separate consumers that select actual source operations, including
both runtime and const calls where permitted. Use CPPGM to link and run
the runtime controls. PA32 completes the build and execution of consumers
against the original `core` crate.
