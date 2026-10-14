# PA18 fixture ownership

The manifest declares the complete source, exit status, rustc classification and
native outcome before oracle generation. These are original RustGM fixtures
promoted from maintainer tests or authored for the public generic contract.
`maintainer_origin` records that provenance. No upstream diagnostic or IR is
used as a RustGM oracle.

There are 425 cases: 262 accept and 163 reject. Accepted cases include 259
executed programs and three deliberately divergent code-generation-only cases.
Four rejections reserve general methods, associated types and opaque types for
later assignments. Growing generic demand requires rustc object generation;
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
