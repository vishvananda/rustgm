# RustGM Programming Assignment 30: Foreign interfaces

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.

## Required implementation

C ABI, repr attributes, linkage and imported/exported declarations.

The [VaArgSafe/VaList source checkpoint](../docs/parallel/source-va-declarations.md)
completes actual ordinary declaration owners and preserves their real trait,
field, ownership and argument-ABI rules. Its two native observers exercise
ordinary role-bearing declarations; the genuine standard cursor observer and
complete marker/module source remain separate required contracts. Actual
c_variadic feature admission, variadic definition/init/read/copy/end/cleanup,
source unsafe/const proof and metadata must be implemented before claiming the
full standard VaList runtime.

The local implementation now supports
`rustc_pass_indirectly_in_non_rustic_abis` on the local repr(C) struct in
ffi/va_list.rs416 (LIB-111). This is a semantic argument ABI contract, not an
advisory marker. Validate its actual struct/feature/representation authority and
implement forced indirect non-Rustic arguments, transparent-wrapper propagation
and real copy/pointer behavior while preserving Rust ABI and return rules.
All 28 frozen source outcomes, five independent C observers and four freshly
built pinned Rust native comparisons pass. The original-core attempt clears this
marker. Older metadata profiles reject its unencoded policy; authenticated
transport and wider actual VaList/runtime consumers remain required. See the
[qualified checkpoint](../docs/pa30-indirect-arguments-checkpoint.md). This finite
family does not complete the assignment.

Opaque external types require genuine declaration identity and exact
`extern_types` feature/token authority. Their value size and alignment remain
unknown. Thin pointer metadata does not make the pointee Sized or MetaSized;
PointeeSized and metadata normalization are distinct source contracts. Do not
substitute an empty struct, fabricate fields or publish an unsupported foreign
type through an older metadata profile. See [the source design](../docs/foreign-type-source-design.md)
for the unqualified successor and independently observed contracts.

The maintainer callable checkpoint retains `Abi::{Rust,C}` and a variadic bit
independently of function safety and constant eligibility. Omitted `extern`
remains distinct syntax and selects C. Fixed-arity scalar C definitions and
function pointers use ordinary selected calls. Top-level safe-to-unsafe
coercions retain a checked lifetime proof view; nested callable equality and
calling-convention identity remain strict. Draft source outcomes are in
`tests/convention-outcomes.json`; this does not activate the assignment.

The maintainer scalar-import checkpoint retains actual foreign-block ownership
through macro expansion, inherits ABI and default unsafe permission, and emits
real declarations for selected imports. Five C-linked native controls cover
scalar imports, callbacks and ordinary/generic/constant pointer demand paths.
Draft outcomes are in `tests/import-outcomes.json`.

The maintainer exact-function-linkage checkpoint consumes retained `link_name`,
`unsafe(no_mangle)` and `unsafe(export_name)` attributes through typed registry
records. Shared roots retain otherwise unused exports; compatible exact import
and export aliases share one canonical LowIR symbol. Collision controls prevent
multiple bodies from replacing each other, including generated helper names.
Draft outcomes are in `tests/linkage-outcomes.json`; qualification and boundaries
are recorded in `docs/pa30-linkage-checkpoint.md`.

The maintainer foreign-static checkpoint retains separate safety and mutability,
real imported global declarations, exact exported storage and constant foreign
addresses. A single typed function/global namespace validates aliases and kind
collisions. Global export roots retain actual initializer and generic callback
dependencies in executable and entry-free modes. Draft outcomes are in
`tests/static-outcomes.json`; qualification and boundaries are recorded in
`docs/pa30-static-checkpoint.md`.

The maintainer aggregate-callable checkpoint classifies nonempty aligned
`repr(C)` structs once and retains INTEGER/SSE/none/MEMORY classes
for imports, exports and callbacks. Sparse object temporaries preserve ordinary
scalar record sizes. Native controls exercise mixed results, register rollback,
large values, padding and Drop. The nested-aggregate checkpoint adds recursively
validated `repr(C)` struct and fixed-array fields, including placement-aware mixed
eightbytes and MEMORY validation. Draft outcomes are in
`tests/aggregate-outcomes.json` and `tests/composite-outcomes.json`; see
`docs/pa30-aggregate-abi.md` and `docs/pa30-composite-abi.md` for the profile
and explicit boundaries. The variadic checkpoint adds direct and indirect C
calls with independently inferred extra arguments, explicit source promotion
checks, and actual scalar/record ABI classes. Fixed parameters retain ordinary
coercions; all arguments retain move, borrow and evaluation-order checks.
`tests/variadic-outcomes.json` and `tests/variadic-tail-outcomes.json` declare
42 source outcomes and 23 native comparisons. See
`docs/pa30-variadic-calls.md` for the typed transport, qualification and retained
boundaries. The fixed-callable metadata checkpoint preserves C ABI separately
from safety, linkage and lifetime slots across separately compiled crates.
Profile four reuses the existing scalar/reference and closed-record graphs;
`tests/foreign-metadata-outcomes.json` declares its independent source and native
controls. See `docs/pa30-foreign-metadata.md`. The named-storage checkpoint adds
owned/foreign static identity, safety, mutability and completed scalar/record/
reference values through profile five. Its source and native contracts are in
`tests/storage-metadata-outcomes.json`; see `docs/pa30-storage-metadata.md` for
validation and the remaining storage boundaries. The profile-seven checkpoint
now transports fixed arrays and plain C record representation across crate
boundaries, including mixed and MEMORY-class aggregate calls and function pointers.
`tests/aggregate-metadata-outcomes.json` includes a separate C caller; see
`docs/pa29-aggregate-metadata.md`. Profile eight now transports actual foreign
variadic declarations, aliases and compatible function pointers while retaining
the same call checks. `tests/variadic-metadata-outcomes.json` declares 21 source
outcomes and ten executable comparisons; see `docs/pa30-variadic-metadata.md`.
Additional record representations and the other unfinished contracts remain required.
The reviewed optional object backend
profile is selected explicitly; its native tests are always registered and run
with `--ignored` through the grouped maintainer suites. They fail if the profile
is missing. PA30's
release target and CI must configure that profile and run those tests before
activation. See `docs/pa30-import-checkpoint.md` and
`docs/pa30-callable-checkpoint.md` for qualification and remaining boundaries.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

The [borrowed-wrapper coercion reductions](../docs/pa26-borrow-coercion-reductions.md)
freeze the reached RefCell::replace conversion gap. The
[implementation checkpoint](../docs/pa26-borrow-coercions.md) qualifies39 actual
source checked-body outcomes and4 real source native/CTFE programs. Ordinary hooks
belong to PA26, the C ABI observers to PA30 and actual source roles to PA31.
Ordinary imports/native parity, complete composition and transport remain required;
this packet does not activate a whole assignment.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

repr(C); aggregate ABI; callbacks; exported globals; C interoperability.

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

The [unsized source observer](tests/spec/unsized-parameters/README.md) executes
a real ?Sized original, sized specialization and automatic destructor through
ordinary C observations7 then9, independently compared with Rust. Concrete
unsized parameter ABI, owned allocation handoff and cross-crate transport remain
required; PA26 owns the source permission and ownership rules.

The [source Deref/Clone checkpoint](../docs/pa26-deref-clone.md) fixes proof ownership at shared call sites and retains all ordinary, source and native contracts in `tests/spec/deref-clone/manifest.json`. This remains a finite development checkpoint; the whole assignment is planned.
