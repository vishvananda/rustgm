# Trait alias clause requirements and shared Self preparation

Typed aliases remain unimplemented. This checkpoint establishes additional
independent semantic requirements and extracts the real rigid Self producer so
an alias can later use it without becoming an associated trait owner. It does
not close LIB-031 or produce a core artifact.

[Independent run40253717374](https://github.com/vishvananda/rustgm/actions/runs/40253717374)
executes 24 metadata compilations, eight native builds and eight actual native
executions against the pinned rustc in editions2021/2024. All12 hypotheses match,
all native programs print `7\n`, and there are zero infrastructure failures.
The40 commands take0.806 seconds. These are reference-compiler observations;
RustGM did not execute them. The earlier44-command seed is retained separately,
including associated equality; it was not repeated.

The new [frozen sources](../pa31/tests/spec/trait-aliases/semantic-v1/contract.json)
establish the following obligations:

- An alias where-clause can imply a bound on a parameter other than Self, and a
  use whose actual argument violates that bound rejects.
- Missing alias arguments and an invalid default reject even when the alias is
  unused. Full declared formals cannot be reduced to parameters mentioned by a
  selected body.
- A quantified alias and an alias beneath another quantifier preserve their
  actual lifetime binders. Type-outlives where-clauses are usable premises;
  a short reference cannot satisfy an alias requiring static Self.
- Empty aliases permit unsized Self. Their generic type parameters still have
  their own implicit Sized requirement unless explicitly relaxed.
- A repeated shared diamond is accepted. Nested alias contracts must retain
  shared dependencies instead of flattening duplicate ancestor clauses at every
  declaration.

The frozen plan SHA is
`fb68dcc1f5ea62aa336a32562f562a4a7a9211e5728071154af97bc99a574e8b`.
Artifact12621489365 ZIP SHA is
`4b96a4046d6c750f5685a9f2e35cd85d4fd5230930fea7149b4228f7a3d7868f`;
binding SHA is
`226c888f971ffdc174fa5b7a0a4104f2cd0d31d3b3984718f9c2b06448c1113a`.
The integrator verifies every source and raw output hash before using the results.

The existing trait implementation now calls `reserve_lexical_self` in
`declarations/self_schema.rs`. The shared routine retains the original binder,
rigid parameter, parent/formal registration and completion order. Its actual
declaration owner installs the schema through a typed callback before Self
becomes complete. The trait callback still installs its ordinary associated
schema. An alias consumer can install its separate canonical contract there;
there is no kind flag, fake trait or implicit Self:Alias premise.

[Run40256280863](https://github.com/vishvananda/rustgm/actions/runs/40256280863)
qualifies six exact groups, strict production/private Clippy, formatting and
the global architecture audit in129.55 seconds. The genuine source control
retains the unused lifetime/type/const formals and their original slots, checks
Self is a separate unsized parameter, and verifies an owner rejection cannot
publish completion. Existing default transfer/cost, alias identity, captured
trait-method conformance and fresh consumer controls also pass. No persistent
fields or layouts change. This run executes no native or course-text sweep.

All2,662 source fingerprints match the hosted snapshot. Receipt SHA is
`6f0339af5282d54756135c913ff7801134df9430e2ebf96ab750b7c38b0fa777`;
binding SHA is
`65f646b2e8717769d2b5eabbfb4b6de6af4917b81831cb5436a213761ce94c3f`.
The first static failure remains retained: filename ownership and a test using
a nonexistent usize constant. Both lint modes were gathered and no tests ran
after that failure. The successor uses the canonical primitive identity and
cohesive Self/formal helpers, without audit or lint allowances.

Next, install genuine per-alias canonical headers and clause dependencies under
this producer, extend the bound worklist to multiple actual clauses, and connect
projection/object consumers. Preserve unused declaration validation, complete
arguments, quantifier depth, original/normalized views and cumulative budgets.
Imported transport remains a separate required consumer of those real contracts.
