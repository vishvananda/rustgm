# Source Assert implementation work

LIB-029 remains open. This branch starts its private typed condition transaction;
it does not enable Assert or qualify runtime/constant assertions. The existing
sixteen independent observations are in [source-assert-observations.md](source-assert-observations.md)
and are reused without another oracle run.

The private parser consumes one actual classified expression prefix and keeps the
custom-message suffix for the real panic macro. A consumed transaction transfers
the condition into an owned draft with one monotone projection over a verbatim
source interval. Each token retains its own hygiene/origin/jointness, and opaque
fragments use the existing registry and authenticated relocation before the new
interval transfer. Allocation-free span preflight reuses the exhaustive syntax
transfer dispatch; invalid intervals publish no package state. Parser/transfer
attempts and temporary coexistence remain charged on failures.

This kernel is staged for the actual resolver-owned source consumer. Its narrow
private-entry dead-code expectations are temporary and must disappear when that
consumer is installed. Ordinary source does not call the kernel or allocate its
storage. Assert is still unavailable; frozen bootstrap slots remain24.

Required next work, in implementation order:

- PA4/PA8/PA31: exhaustive bounded parsed-expression default-message writer and
  authenticated prepare/consume integration; separate defining `panicking::panic`
  and edition-sensitive custom-message branches.
- PA19/PA20/PA26/PA31: real source Not trait/method/associated Output selection.
- PA25/PA26/PA31: checked real panic roles and CTFE instance redirection/failure;
  initializer work must execute normally, including both unchanged alignment
  obligations. Prior handlers returning21/7 are routing evidence only.
- PA28/PA31/abort subset of PA39: actual reached source abort intrinsic signature,
  effect/proof checks and existing supported native abort boundary. CPPGM stays
  unchanged. Unwinding remains separate.
- PA26/PA31: actual dynamic formatting constructors, borrows, ordinary runtime
  formatter/handler path and type checking of a true assertion's message branch.
- PA29: source macro and panic-policy metadata transport remains closed; no wire
  permission or fake defining-crate root is introduced here.

True-only or expansion-only results cannot close LIB-029. Candidate closure needs
real provider true/false runtime and CTFE, overloaded Not, lazy/dynamic message
arguments, exact default text, genuine source handler identity, edition/provenance
opponents and transactional budget/layout evidence. Rich generic-assert capture
remains explicitly unsupported until its actual separate source policy is added.
