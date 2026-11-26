# PA29 student test plan

PA29 teaches separate compilation on small crate graphs. Each provider is
compiled to metadata and LowIR. A fresh client uses the metadata and links
with the provider's objects. PA32 uses this same path for pinned `core`.

| PA29 component | Student coverage |
| --- | --- |
| `declarations` | Crate and symbol identity, renamed imports, transitive dependencies, namespaces, visibility, and declaration-only clients. |
| `value-metadata` | Constants, shared storage and references, records and arrays, and foreign declarations. Native observations check the values and addresses clients actually use. |
| `provided-profile` | A separate client instantiates a provider's default method. Controls exercise the wrong owner, invalid arguments, and incompatible metadata. |
| `generic-transport` | Checked generic templates, including private helper bodies, traits and associated types, callable values, const expressions, regions, and the context that owns each declaration and body. Clients instantiate and evaluate them after reload. |

1. Establish the baseline with all four unit manifests. The current drafts
   contain 218 cases. Run their text oracles and independent Rust and native
   checks on one immutable compiler. Review disagreements at the first
   diagnostic or failed observation.
2. Recover the ordinary Rust cases declared in
   `pa29/tests/generic-contract-outcomes.json` and
   `pa29/tests/generic-id-outcomes.json`. Keep distinct tests for runtime and
   const instantiation, private generic helpers, lifetime and const binders,
   generic calls through a relay crate, and same-name providers with distinct
   identities. Check wrong-type, trait, lifetime, and const uses. A declared
   valid program stays a positive when the compiler cannot yet publish it.
3. Exercise the current typed-body artifact path on at least one small
   provider and fresh client. Include a body form that requires the current
   profile and a source-level opponent. Preserve the existing compatibility
   cases. Cover the common declaration and body remapping in coherent
   batches once the observations identify shared gaps.
4. Verify deterministic artifact bytes, dependency inventory, and rejection
   of missing, incompatible, and malformed metadata at the compiler's public
   entry points. Reuse maintainer controls for byte comparisons and packet
   mutations. Keep language behavior and ordinary graph cases in the student
   suite, with all inputs declared in its manifest. Require the intended
   diagnostic and successful provider setup for every rejection.
5. Compose the full PA29 manifest from its qualified components. Check
   layout, placement, and oracle provenance, then run the student PA29 entry
   point against the immutable compiler in a local export. Record the
   independent results before changing component publication status.
6. Return to PA32 and publish the original `core` artifact locally. Reduce
   shared serialization failures into PA29 fixtures and rerun the affected
   PA29 checks before retrying the pinned source. Place source binding and
   target-specific reductions in the earliest assignment owning all their
   features. PA32 retains the full artifact and fresh-consumer integration
   checks.

## Remaining ordinary transport

The retained ordinary provider combines four requirements that can be tested
separately: a lifetime binder on a generic callable, a const binder and its
array length term, a trait and dispatched method, and a generic body that
calls into a dependency. A transitive generic re-export already checks
namespace remapping. The relay body must also retain the dependency call in
its template and prove each new invocation against the callee's header.

Use a common typed artifact model for those requirements. Give a declaration
reference an artifact owner as well as a declaration coordinate. Preserve
definition, type, const and lifetime identities before rebuilding the body in
the consumer. Keep bootstrap language operations and source sysroot bindings
as explicit inputs to this remapping. Source-specific permissions retain their
actual owner and provenance. Preserve the earlier wire profiles while adding
coverage for the common model.
