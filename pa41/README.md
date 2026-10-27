# RustGM Programming Assignment 41: Target-specific closure

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA40. The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Target attributes and assembly contracts against explicit backend capabilities.

### Required core prerequisite

The original core source needs this assignment's assembly slice during PA32,
before the broader PA41 release. Track that implementation as LIB-045; do not
add another assignment or postpone a required core feature to this number.
PA31 authenticates the source builtin declaration and feature provenance;
PA27/PA28 supply the ordinary place, safety, memory and intrinsic rules. PA25
and PA29 retain const-evaluation and body-transport ownership.

For the first source path, preserve the original `_mm_stream_ps` function,
its `vps!`/`concat!` template production, `in(reg)` pointer, `in(xmm_reg)` real
`__m128` input, and `nostack`/`preserves_flags` options. Parsing its syntax does
not establish SIMD layout, target-feature compatibility, an exported body or
native emission. Complete each required boundary before claiming that consumer.

- Parse templates and operand expressions once. Preserve source spans, template
  pieces, names, positional references, modifiers and declaration ownership
  through macro expansion. A builtin's placeholder macro body cannot implement
  an assembly invocation.
- Check register classes and explicit aliases, input/output and tied-operand
  types, output places, initialization, Copy obligations, unsafe context,
  option combinations and actual feature availability. Evaluate operand
  expressions left to right and write outputs left to right after the effect.
  Overlapping output places receive the rightmost output last.
- Retain the real effect in checked CFG and instance data. Preserve memory,
  flags, register clobbers and divergence; `noreturn` has no normal successor.
  Const and symbol operands require real constant and relocation producers.
  Export and import must preserve the checked operation and revalidate its
  identities; an unsupported body protocol must fail explicitly.
- Implement the required SIMD representation and target-feature rules against
  the pinned compiler. A 128-bit vector is not two independent scalar arguments.
  Attribute presence alone proves neither the enabled instruction set nor call
  compatibility. Lower only independently proved Rust layout and ABI facts.
- Qualify a real backend operation for every demanded emitted instruction.
  The non-temporal store must preserve all128 bits and its actual memory
  semantics. An ordinary store, erased statement or frontend delegation cannot
  replace it. Keep unsupported backend forms distinct from Rust language errors.

### Broader target closure

Complete the pinned assembly operand, option, ABI-clobber and supported-target
contracts beyond the initial source path, including valid forms retained as
unsupported in earlier checkpoints. Inventory labels, symbols, target-specific
register/modifier rules, global/naked assembly and unwind restrictions against
the pinned Reference and compiler. State the supported target policy explicitly;
an unrelated architecture is not an excuse for a required selected-target gap.

Earlier library/crate milestones must already qualify their active target paths.
This assignment closes the general assembly/target contract, including explicit
unsupported-capability diagnostics. Separately scope any required CPPGM extension;
the roadmap does not authorize modifying the frozen code generator.

The maintainer implements compiler responsibilities in `dev/src/` and introduces
typed interfaces before text views. Follow [spec.md](../spec.md), the pinned Rust
Reference and CPPGM's efficient representation principles. Parse once, retain
completed facts and avoid speculative or repeated whole-graph work. The compiler
implementation remains idiomatic safe Rust; strict Clippy all/pedantic and the
syntax-aware file/architecture audit are required.

## Tool surface

Extend the existing `dev/rustgm --emit-lowir` view and explicit crate-build interfaces.
Earlier views remain available. Freeze deterministic ordering, diagnostics,
source/configuration identities and any new build-summary text contract before
activation. Whole-library acceptance includes actual linked execution through
the configured CPPGM backend; text output alone is insufficient.

## Tests to develop with the reference

- Asm constraints.
- Target features.
- Unsupported capability rejection.

Use text oracles for template/operand bindings, typed effects and CFG, target
facts and precise diagnostics. Independently observe accepted and rejected
source cases before generating references. Include operand/place side effects,
aliased outputs, uninitialized inputs, `noreturn`, foreign or changed source
owners, const/relocation contracts, and ordinary programs without assembly.
Measure row/byte growth and work on shared and repeated inputs. Retain actual
source/body/layout costs, including costs paid by ordinary bodies.

Native consumers must check the original demanded source body, full-width input
bits, destination and surrounding sentinels, and the required instruction's
presence. Validate its real Rust/C ABI against independent compilers where the
ABI crosses that boundary. Metadata acceptance or an instruction in disassembly
alone does not establish correct linked execution.

Place each fixture in the earliest assignment owning all features it exercises.
Keep original upstream bytes, attribution and checksums. Declare expected outcomes
before generating oracles; use pinned rustc independently and retain negative
controls. Register the complete selected dependency graph and native consumers,
not just reducers or successful submodules. Never hand-edit generated references.

## Completion and review

Publish the semantic/lowering boundary, nonempty `tests/manifest.json`, text
formats and generated references together after independent outcome review.
Until then, this assignment must fail explicitly and stays outside pass totals.
Run affected focused source/native controls during development. At publication,
run the required coherent checkpoint checks from
[TESTING_AND_REFERENCES.md](../TESTING_AND_REFERENCES.md), including prior active
suites, strict Clippy, file/architecture audits, harness checks and affected
performance measurements. Use the resource/storage guards and scoped cleanup.
Do not activate a stage merely because a reduced example or an empty suite passes.
