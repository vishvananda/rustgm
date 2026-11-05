# First authored future runtime implementation contract

Root approved these joins on2026-11-05 after the v11 symbolic checkpoint.
This is a no-await executable milestone toward full PA42/43, not a replacement
for await, cancellation, recursive cleanup or real authored-body transport.
Implementation starts from4cd080c84; the independently frozen packet is
`pa43/tests/spec/future-runtime-v1/`.

* Append `Kind::Future { identity: OpaqueId, arguments: GenericArgs }`. The identity
  is the existing actual declaration/ordinal, with complete application captures.
  No ClosureId, nominal AST/DefId, or tuple identity is fabricated.
* Wrap each existing private type-store opaque row with its unchanged public
  contract and optional prepaid boxed shape. The shape retains canonical formal
  GenericArgs, the actual parameter type ListId and Output. The shared capture
  substitution kernel is extracted from existing opaque substitution. A shape
  supplies type/storage data and grants no execution permission.
* Keep the one authored AsyncBody in existing Prepared/templates/Checked ownership.
  Its owner/key and real `fn(inputs)->Output` signature differ from the public
  constructor. A generated constructor uses the actual public function/signature
  and explicit checked construction provenance. It cannot replace or masquerade
  as the deferred authored Original.
* `FutureEntryKey { future: TypeId, operation: Poll|Drop }` owns generated runtime
  identity. Poll/drop have genuine separate signatures and body owners. Actual
  Future::poll trait selection retains its real member FunctionItem through the
  existing Call/Reify path and existing generated-method registry. A scheduler
  binding selects the distinct generated entry; no new Rvalue/direct-call law or
  competing registry is introduced. Drop demand selects its own typed entry.
* Authenticate source Poll/Ready/Pending/Context declarations and exact demanded
  operation schemas together. Ready/Pending use their actual enum/variant owners,
  fields and layouts. Unused alternative declarations retain ordinary validity;
  a demanded operation requires real Pin, independent receiver/context lifetimes,
  actual Output equality, safe Rust ABI and complete canonical arguments.

Source body typing publishes capture shape only from authenticated actual
parameters/application/body Output. Body construction produces the actual raw
parameter locals and deferred pattern/body operations once. Existing ownership,
region, initialization, cleanup and instance proof must complete before a family
can execute. Per-application materialization borrows the checked original and
produces one owned candidate, consumed by the state transform. Constructor moves
raw inputs once; first poll executes the parameter patterns and body. Anonymous
frame slots have typed capture/state projections and share physical sequence
layout kernels without becoming tuples or invented Members fields.

Unpolled drop destroys captures in their actual order. Completion transfers the
actual output into Ready, destroys body locals/parameter remnants in source order,
and then marks completion; completed drop cannot repeat those effects. Logical
capture loans survive according to their real types even when runtime fields have
moved out. Futures remain !Unpin/!Copy/!Clone; capture Send/Sync uses actual stored
types, not the poll Context or output type. Shape, generated instance and checked
family readiness stay distinct. Real Kind/Owner/opaque Entry/Body sizes and every
owned allocation are measured/charged before qualification.

Compiler AsyncDropInPlace remains exclusively its authenticated semantic recipe;
the written placeholder cannot enter authored runtime. Its feature-off original
core declarations remain legal. Full suspension liveness, saved locals, self
references/pinning, panic/cancellation paths, recursive async cleanup, broader
async forms and body/region/export recipes remain required. Wider argument graphs
use indexed canonical formals and memoized transformations, with no repeated
whole-graph fold per field.
