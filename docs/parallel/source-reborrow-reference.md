# Reborrow and CoerceShared source mechanism

The pinned marker module declares safe empty Reborrow and safe empty
CoerceShared<Target: Copy>: Reborrow, both behind feature reborrow. These are
compiler operations, not methods; there is no Reborrow derive macro in these
declarations. Existing Clone/Copy derives and PhantomData remain real dependencies.

The primary implementation is compiler/rustc_hir_analysis/src/coherence/builtin.rs
at reborrow_info (503) and coerce_shared_info (602). Reborrow implementations need
one impl-owned lifetime and a struct application with one lifetime argument.
Every data field must have actual Reborrow or Copy proof; mutable references are
the intrinsic base case. Only actual PhantomData fields are excluded.

CoerceShared implementations need one impl-owned lifetime, struct source/target
applications with one lifetime argument each and matching zero-or-one data-field
counts. The mutable-reference/shared-reference base case uses equality after
peeling all reference layers. Otherwise matching fields need real Copy, while
different fields need genuine nested CoerceShared plus ordinary region proof.
Target Copy and parent Reborrow remain ordinary source trait obligations. These
finite implementation restrictions must not become blanket declaration rules.

Typeck coercion.rs (953/978) emits GenericReborrow(Mut) for the same ADT and
GenericReborrow(Not) for a proven different-ADT CoerceShared relation. MIR building
uses the actual source place/temporary and Rvalue::Reborrow(target, mutability,
place), in builder/expr/as_rvalue.rs (437) and into.rs (915). No Clone call is
introduced. Borrow-set construction (305) uses the actual source place and target
lifetime argument; type_check/mod.rs add_generic_reborrow_constraint (2470)
retains liveness, source/target relations and outlives constraints. Exclusive and
shared uses have different read/write/move restrictions. Promotion explicitly
rejects this operation in promote_consts.rs (584).

All paths above refer to Rust commit
88d9e12ae178fab0fb5cc050a94da85685d449ea. This is primary-source research for a
future shared RustGM producer/consumer, not an external compiler implementation.
The source packet and frozen outcomes live in pa31/tests/spec/reborrow-source.
Reference observations, corrections and native receipts will be bound alongside
this note; no candidate has been observed.

The final packet has 93 distinct sources. All 42 unchanged UI cases match their
source expectations. In total, 82 source outcomes are qualified (33 accepts and
49 rejections); ten ICE cases are explicitly unqualified, and one obsolete
`impl const` parse input is setup-only. The corrected `const impl` opponent reaches
the genuine non-const-trait error, with a separate valid syntax positive. All nine
unchanged crash tests still ICE; none is counted as a semantic negative.

Both actual-declaration/no_core and installed-standard-library C observers print
`reborrow 23 23 47 47 0 99 31`. They prove real operand writes/reads, implicit
coercion avoiding Clone, explicit execution of the genuine Clone body, and actual
const evaluation. Ten original upstream run-pass executables also pass. The
initial standalone standard object lacked its real null-panic dependency; that
failure is retained, and the successful observer links the genuine Rust runtime
static library. No placeholder panic or Clone body is substituted.

Several paired results distinguish declaration validity from operation behavior:

- Unused nested-reference peeling and source-field-to-static target implementations
  accept. The nested demand fails E0597; the mutable-to-static demand ICEs and
  remains unqualified. Same-Copy-field static application demands also fail E0597.
- An unused implementation from a real `&'a u8` data field to `&'static u8` accepts
  when the target's own lifetime remains in actual PhantomData. Its matching-life
  control accepts; its demanded use fails E0597. This does not permit dropping the
  eventual operation/caller's lifetime obligations.
- Multiple data fields with real Copy/Reborrow evidence are valid. Concrete and
  explicit generic targets reborrow twice. The inferred generic-call target was
  initially predicted to reborrow but actually moves (E0382); the original input
  and correction are retained. A reborrow cannot be inserted retrospectively.
- Simple transparent reference aliases accept. The associated-projection field
  case rejects E0277, even after replacing only the outer alias field syntax with
  literal references. A blanket alias-spelling rejection is not supported.
- Altered unused unsafe and extra-method trait headers accept; wrong Reborrow
  generic arity rejects. These declaration results do not grant a standard
  operation profile or implementation evidence.

Raw observations and all failed hypotheses remain under
`obj/parallel/source-transport/reborrow-reference`, with receipt hashes, exact
tools/sysroot/primary-source fingerprints and native artifacts in the adjacent
JSON. Harness and upstream verification pass. Full derive/source closure,
unqualified bug profiles, broader operations/Drop/evaluation-order/const behavior
and authenticated transport remain required. No frontend edits, RustGM candidate,
whole-module or PA completion claim is made by this reference-only checkpoint.
