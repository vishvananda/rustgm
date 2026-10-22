# Impl-owned omitted lifetime inputs

This checkpoint seals the actual impl binder after typing both Self and trait
arguments, before registering a selectable header. The earlier pipeline
published Schema.captures after Self, then allocated omitted trait-input regions
in the same binder. Method substitution consequently read a stale count and
rejected valid reference impls with missing lifetime arguments.

The existing unfinished-header state owns the completion. Checker verifies the
exact active unfinished Impl, owner binder and matching context, then derives
its count from that binder. Declarations::finish_impl_regions lives beside the
semantic associated Inventory. It rejects another owner, stale Self schema,
decreasing/out-of-range counts and every already prepared header, including one
whose bodies are not yet validated. It adds no fields, enum variants, permission
or inference shortcut. Methods and templates consume only the sealed count.
Intervening projection environments retain their existing live-binder proofs;
they do not cache the provisional Schema.captures. Registry preparation and all
method/conformance/template publication follow the seal.

Independent Self and reference-valued trait arguments keep distinct slots,
including explicit placeholders, named/unused early slots, multiple and nested
inputs. Nested function-pointer binders remain local to that callable. The normal
member conformance, caller lifetime, overlap and original-body proof paths remain
responsible for rejection.

The independent matrix declares 25 sources: 19 Rust accepts and six rejects.
Eighteen accepted forms execute natively; the remaining unchanged accepted
FunctionItem inference case is required open work described below. The six
negatives distinguish method result elision, unpaid explicit result lifetimes,
caller escape, missing output lifetime and overlapping impls. Four private
invariants cover exact slot identities, nested callable separation, stale/wrong
owner rejection and prepared-but-unvalidated publication.

The original PA26 reference comparison same-named-trait.rs is unchanged (SHA-256
0020883752913f0c023d9fc09aa8a237066a2c193145576825a472289d667e13).
Its formerly failing omitted impl inputs now pass source and native execution;
its draft expected outcome is superseded accordingly. This is ordinary user
trait behavior with no canonical trait-name special case.

## Retained inference gap

The exact independently accepted source nested-callable.rs remains frozen under
obj/parallel/pa26/impl-elision/declared-v1. Passing id directly into a generic
trait argument selects a FunctionItem before the required fn-pointer conversion,
and fails with an unambiguous-matching-obligation diagnostic at byte 242. Both
an explicit fn-pointer local and a fully qualified trait-input type pass. The
immutable pre-fix b3f1e17b executable (SHA-256
3e34f9a813192a94629199a67fcf9d3b3d8806cf1cc10beff2298672e3ac761f)
reproduces the original failure and accepts both opposing forms. This is required
PA19 inference completion work; those supported controls do not replace it.
No inference change is included in this metadata checkpoint.

Independent declarations, Rust object outcomes and all failed observations live
under obj/parallel/pa26/impl-elision. First-v2 retains the initial source mismatch;
callback-v2 and declared-v2 retain pre-fix comparison and both independent
opposing qualifications. Second-v3 passes the 25 source classifications and
18 native programs, the original 61 reference classifications and now 25 native
programs, plus library Clippy. The source test explicitly preserves the one
Rust-positive inference gap rather than misclassifying it as a language error.

PA26 still requires concrete str and structural tuple/array/slice comparisons,
dot-call reference RHS inference, unsuffixed assignment-method RHS inference,
remaining primitive reference/unary forms, Deref, iteration, question-mark,
required derives and formatting. Slice len and explicit array-reference-to-slice
casts were completed in earlier checkpoints. Full combined qualification,
assignment activation and reference publication remain integrator work.

Closing-v4 passes the fourth private prepared-header invariant, strict library/
integration-test Clippy, architecture audit 1564/0, layout and pinned upstream
checks. Early unchanged active text is exact: PA19 1103/1103 and PA20 1381/1381.
No references were changed. Candidate executable SHA-256 is
fd57a8bb429eef06e62e20d17e64ff20c12f2a07c7ac873a07178b43f11af12b;
closing-v4/source-map.json records every production file hash. Full release
checks remain integrator-owned. At the next clean boundary the worker adopts
the published grouped maintainer harness before its next closing qualification.

The separate owning fixture payload is
obj/parallel/pa26/impl-elision/fixture-payload-v1/payload.patch, SHA-256
f0fead49ec90a585e1556357f4a67917683132100f352cfa044311fa570e244a.
It proposes 21 PA19 cases, one PA20 associated-output negative and one PA26
canonical-comparison case. Existing active manifests/oracles remain unchanged
in this compiler checkpoint; the integrator owns independent sidecar review.
The inferred FunctionItem source is retained separately as required work and
is not admitted as a passing implementation by that payload. Its SHA-256 is
2b0bd86513377dcc51154fba36cfacf7ba833f5553e5890b75bbefcf17672ed0.

After clean merge 07dfd0cd of integrator b0d85e28, grouped-v5 passes strict release
all-target/all-feature Clippy, registration (336 roots in 13 binaries), all four
exact elision/reference source/native selectors through maintainer_suites.py,
audit 1578/0 and layout. No compiler change was needed for grouped execution.
The first projections binary compiled in 78 seconds; repeated exact source/
native selections reused it. Full publication remains a separate frozen cut.

The later argument-local inference checkpoint closes the retained FunctionItem
trait-input source through ordinary contextual coercion, preserving the exact
original bytes and old failure evidence. See
[pa19-call-argument-context.md](pa19-call-argument-context.md). Its direct
returned-FunctionItem call boundary remains separate required PA18 work.
