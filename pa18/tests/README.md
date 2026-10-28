# PA18 fixture ownership

The manifest declares the complete source, exit status, rustc classification and
native outcome before oracle generation. These are original RustGM fixtures
promoted from maintainer tests or authored for the public generic contract.
`maintainer_origin` records that provenance. No upstream diagnostic or IR is
used as a RustGM oracle.

There are 502 cases: 315 accept and 187 reject. Accepted cases include 305
executed programs, three deliberately divergent code-generation-only cases,
four pattern inspections and three library object checks.
The former method, associated-item and opaque boundaries belong to their later
owning assignments; PA21 accepts the migrated opaque-return source.
Growing generic demand requires rustc object generation;
metadata alone cannot demonstrate the expected recursion rejection.

Clusters separate callable/constructor/destructor execution, aliases and default
queries, generic declarations, argument inference, bounds, aggregates, template
ownership/regions/loans, scalar const operands, Drop conformance, entry checks,
demand and later-stage boundaries. Accepted unused-body fixtures intentionally
prove a source body that need not have an executable instance. Runtime fixtures
include in-source value and cleanup counter assertions.

The two `moved-pa12.rs` / `moved-pa17.rs` sources retain their original bytes and
now belong to their earliest complete feature owner. Their obsolete earlier
rejection oracles are removed, and PA18 generates accepted LowIR oracles only
after independent qualification. Every required `.rs` input is in the manifest.

`shared-call-reborrow/` adds four library LowIR controls (three accepts, one reject)
for generic reference heads, generic value arguments, Copy bounds and mutable
reference moves. They are byte-identical relocations from the initial ordinary
freeze to their earliest type-generic owner; the directory contract binds that
evidence. Plain reference contexts remain in PA16 and source-owned providers in
PA31. Reviewed text oracles are integrator-owned.

The generic loan-view attempt is retained as an explicit inspection boundary;
these cases use the existing defining-template/LowIR pipeline. Accepted generic
libraries export no runtime root and their empty object checks claim no execution.
The ordinary/source general call profiles carry separate real native traces.
