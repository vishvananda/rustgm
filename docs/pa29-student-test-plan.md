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

1. Use all four unit manifests as the baseline. The current drafts contain
   262 cases. Run their text oracles and independent Rust and native checks on
   one immutable compiler. Review disagreements at the first diagnostic or
   failed observation.
2. Recover the ordinary Rust cases declared in
   `pa29/tests/generic-contract-outcomes.json` and
   `pa29/tests/generic-id-outcomes.json`. Keep distinct tests for runtime and
   const instantiation, private generic helpers, lifetime and const binders,
   generic calls through a relay crate, and same-name providers with distinct
   identities. Check wrong-type, trait, lifetime, and const uses. A declared
   valid program stays a positive when the compiler cannot yet publish it.
3. Exercise the typed-body path first with one ordinary provider and then
   with several. Start with the existing combined provider and compile each
   client in a fresh process. Add the relay's own generic body and two
   providers with the same name and distinct disambiguators. Verify that an
   incorrect trait from one provider cannot satisfy a bound from the other.
   Preserve compatibility coverage for the earlier artifact versions.
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

## Ordinary transport qualification

A single ordinary provider now publishes the checked body rows and supports
fresh-client calls with lifetime and const binders, private helper calls and
trait dispatch. The suite uses that provider for nine cases, including five
independent rejections. Two cases load providers that share a crate name
and differ by disambiguator. They check dispatch to each provider and
rejection of an implementation for the other provider's trait. The typed
re-export loads the base by transitive metadata and calls its original
generic declaration through the relay alias. The full relay exports its own
templates for a runtime call, a const call and a trait-constrained call. A
fresh client executes all three, including const evaluation of the array
before lowering. All 241 cases pass the exported student entry point, and
the independent Rust and native suites pass for all four components. A
replacement dependency that adds a trait bound causes the relay to fail
when the fresh client checks the defining header.

The first shared reduction from the PA32 replay uses a const function in an
array type and an array repeat count. PA29's source-bound artifact carries the
evaluated length in the canonical type. A fresh client checks the imported
function at compile time and runs it with two distinct byte values. The
enum reduction checks a provider's computed discriminant against its
neighbor in a fresh client. Both cases use the canonical values in the
artifact. PA29 now carries the checked eligibility for direct casts of
imported enums. A fresh client observes explicit, implicit and compile-time
discriminants. Separate clients reject a payload variant and an empty tuple
variant when the declaration uses explicit discriminants.

PA29 also carries legacy const argument positions on their checked function
headers. The frozen call forms pass the independent Rust checks. The native
client exercises two interleaved const operands in a direct call, compile-time
evaluation through a renamed import, explicit generic arguments, and an
instantiated alias. The runtime-valued const operand fails in a fresh client.
The student export passes all 253 cases on one immutable compiler, and the
independent Rust and native suites pass for all four components.

Use a common typed artifact model for those requirements. Give a declaration
reference an artifact owner as well as a declaration coordinate. Preserve
definition, type, const and lifetime identities before rebuilding the body in
the consumer. Keep bootstrap language operations and source sysroot bindings
as explicit inputs to this remapping. Source-specific permissions retain their
actual owner and provenance. Preserve the earlier wire profiles while adding
coverage for the common model.

The current student export passes all 262 cases. Independent Rust and native
checks pass across the four components. The initializer-scope regression
includes native observations and separate rejections for a hidden function
and type.

## Reductions from the standalone build

| Owning assignment | Provider and fresh-client checks |
| --- | --- |
| PA29 | Qualified. Cast an imported fieldless enum and observe its explicit and implicit discriminants. Reject a payload enum and the ineligible empty tuple form. |
| PA29 | Qualified. Call a provider that declares legacy const argument positions through the direct path, with explicit const arguments, and through an alias. Check compile-time and native values, reject a runtime-valued const operand, and reject duplicate or out-of-range metadata positions. |
| PA31 | Qualified. Compile a blanket impl and a specialization in the provider. In a client with no specialization feature, link and observe the blanket method, override, inherited const argument and inherited associated type. Reject a provider that overrides the final parent method. Transport the checked specialization-trait policies and test them with fresh clients. |
| PA31 | Qualified. The shared availability metadata carries checked library feature relationships. Fresh clients check a direct `implied_by` request, reject reverse and chained implication, and diagnose the previous name of a feature. Native results agree with pinned Rust. |
| PA31 | Qualified. Fresh clients invoke a deprecated exported macro directly and through an alias. Check warning, deny, allow, and expect policies, the scheduled release, and both current and deprecated targets of annotated re-exports. Pinned Rust and native observations agree. |
| PA31 | Qualified. Carry resolved import policies through canonical binding order. Compare metadata across the existing availability and ordinary providers and renamed imports across multiple modules. |
| PA31 | Qualified. An inherent comptime method appears before trait impls in source. The reader resolves the implementation owner by declaration identity, and the fresh client agrees with pinned Rust. |
| PA29 | Qualified. Compile a provider with constant items inside free and inherent constant initializers. Check imported lengths and native reads, including a renamed type path. Reject a client that tries to read the hidden inner item. |
| PA29 | Qualified. A closure with a named parameter in an evaluated `const _` publishes under both profiles. A fresh client evaluates the provider's const function and calls a provider's nested function at runtime. |
| PA29 | Qualified. A separately compiled client calls the handler stored in a provider's function-local static, verifies the static's address identity, and evaluates a named constant whose initializer has a local binding. |
| PA31 | Qualified. Import the two diagnostic-owned unwind-safety auto traits. Observe the owned and shared cases, reject the provider's negative impl for a mutable reference, and reject a diagnostic row with a foreign owner. |
| PA31 | Qualified. Import the restricted `Pointee` and `TransmuteFrom` headers. Observe the pointer-metadata and options cases and reject an explicit `Pointee` implementation. |
| PA31 | Qualified. Publish an `extern "rust-call"` implementation of `FnOnce` and call its body with two different inputs in a fresh native client. |
| PA31 | Qualified. Compile methods whose `Self` captures a lifetime and whose `Pin<&mut Self>` or `Pin<&Self>` receiver introduces a separate borrow. A fresh native client reads through both. |
| PA31 | Qualified. A fresh client instantiates a const generic function that calls a private fixed-arity C-ABI definition with a record. It exercises the call at compile time and in native code. PA30 owns the C calling convention. |
| PA29 | Qualified. Compile a trait implementation with an elided lifetime and a const argument in `Self`. In a fresh client, call the method with two distinct const instantiations and observe their different results. |
| PA29 | Qualified. A fresh bootstrap client calls an ordinary provider's shared, mutable and consuming closures. The compiler rechecks the remapped closure's input and result types and actual receiver permission. |
| PA31 | Qualified. Import methods with two receiver reference layers. Fresh native clients check replacement of both shared and mutable inner references. |
| PA31 | Qualified. A closure borrows its captured reference binding and mutates through it. The fresh source client observes the result after the unique immutable borrow is rebuilt. |
| PA29 | Qualified. Two evaluated `const _` blocks each declare a struct and function with the same names. A fresh client observes their distinct behavior and reads values from a named constant and a static with their own nested declarations. Two independent clients reject attempts to use the hidden function or type through the crate. |
| PA32 | The original build cleared the receiver, body operation, and namespace checks. The next failure is a labelled universal alias in a quantified lifetime proof. The original artifact and all 18 fresh consumers remain open. |
