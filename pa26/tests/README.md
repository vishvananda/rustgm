# PA26 fixture development

PA26 remains planned. `operator-outcomes.json` declares the draft operator and
shared-index outcomes before reference generation; it is not an executed course
manifest. Every input belongs to the `spec/operators` family. The maintainer
`language_operators` integration test independently qualifies these outcomes with
the pinned rustc, checks full typed lowering, and runs native controls with both
rustc and the external CPPGM LowIR backend.

The controls distinguish different RHS and result types, generic bounds, aliases,
spelling-only lookalikes, method conformance, moved operands, shared-index writes
and receiver loans during index argument evaluation. Native programs additionally
check operand evaluation order and actual selected method execution.

No text sidecars are published yet. Release requires the remaining PA26 families,
reviewed complete outcomes and references, the cumulative source/native suites,
and the assignment activation reviewed by the integrator.
