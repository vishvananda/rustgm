# Exact enclosing identities for inline specialization

Generic inline constants now retain their complete enclosing callable instance,
including closure identity and every generic argument. The phase-neutral
`InlineInstance { source, enclosing: Context }` also represents selected
initializer contexts without inventing a callable. Registry rows cache their
nearest non-inline semantic owner at append time. A request from a different
closure in the same declaration is rejected before specialization. Identity
selection grants no const eligibility or execution permission.

Original anonymous templates keep `BodyKey::Anonymous(source)`. Materialized
bodies use `Owner::AnonymousInstance` and `BodyKey::AnonymousInstance`; callable
`Body.instance` stays unchanged. The execution request, catalog, active-frame
identity and completed-value cache include the complete inline instance and its
solved result type. Parameters absent from the callable result or signature
still distinguish instances. Nested inline operands inherit that full context.
Closed `InlineConstant` operands retain their existing path.

The provider builds each demanded original CFG and contract once from retained
typed facts. It proves the original template, substitutes complete enclosing
bindings, then performs ordinary dispatch, promotion, const eligibility and
region/ownership/loan/storage/destruction checks on each specialized body before
publishing execution Code. Missing bindings, changed result types, foreign owners,
cycles and changed completed values fail without granting cached permission.
The source template remains deferred rather than acquiring an instance's value.

The provider and ordinary template verifier retain separate immutable CFG arrays;
both copies are charged before handoff. Owned contract vector capacities and
nested proof payload are charged, excluding shared immutable Arc targets.
Catalog storage, active execution and completed registry entries retain their
existing report limits. Logical storage counters are conservative accounting,
not measurements of allocator heap or RSS.

The draft `constants 1` grammar adds:

```text
root inline-instance instance=FILE:NODE@callable:TYPE type=RESULT value=vID
root inline-instance instance=FILE:NODE@initializer:DEF:ARGS type=RESULT value=vID
```

Rows follow ordinary initializer and closed inline roots. Instance rows sort by
source, context kind, complete canonical context identity and result type. These
are local inspection identities, never external metadata IDs. Existing rows
retain their grammar. The independently object-qualified `const-arguments.rs`
example produces the following roots in repeated byte-identical CLI inventories:

```text
declaration source=0:11 kind=inline type=11 const=true status=deferred-template parent=0:12
root inline-instance instance=0:11@callable:23 type=11 value=v0
root inline-instance instance=0:11@callable:24 type=11 value=v1
value v0 type=11 integer bits=00000000000000000000000000000003 metadata=none
value v1 type=11 integer bits=00000000000000000000000000000004 metadata=none
```

Qualification covers 25 independent pinned Rust object outcomes: 14 supported
positive programs, nine rejections and two explicitly unsupported Rust-positive
cases. All 14 supported programs execute through the pinned CPPGM backend.
Controls include equal-signature distinct const/type arguments, recursion into
another instance, true cycles, nested inline bodies, array results, lifetime-only
owners, empty-environment generic closures, early/late demands, unused invalid
generic bodies and zero repeats. A repeat whose length becomes zero only after
specialization still evaluates its inline fault, matching full Rust object
qualification; a definition-time known-zero constant repeat suppresses value
execution. Ordinary runtime repeat operands evaluate once even at zero length.
All failed initial hypotheses remain in the evidence directory.

At 16/64/256 enclosing contexts, repeated demands retain one original inline CFG
and one template, publish exactly N checked instance bodies/values, and keep
lookup/storage counters within linear bounds. Three private invariants reject
foreign owners, wrong results, missing arguments, changed cached values,
exhausted registry storage and exchanged closure templates. The full 25-group
nested suite and seven closure-pointer groups passed after the compiler changes;
the subsequent test-only inventory/unused-body additions passed all five affected
specialization groups. Strict all-target/all-feature release Clippy, the
1,366-file architecture audit, layout and upstream checks pass. Native tests are
explicitly ignored in the backend-free Cargo lane and were run with
`--include-ignored` here.

Evidence is under `obj/parallel/pa25-inline-specialized/`: `qualified-*.log`,
`inventory-*.log`, object observations, native cases, inspection outputs and
`source-sha256.json`. Earlier consumer-exhaustiveness, style and phase/stem audit
failures remain recorded separately. The source digest is
`67e8d4f2aeb049f6d7079645370dbfd7d71b8dc4dd90d7fd7725d269e37e2edc`.

This is a coherent scalar/owned-value specialization slice, not PA25 completion.
At this checkpoint the original associated-initializer and generic captured-
environment fixtures remained unchanged, independently accepted by Rust, and
explicitly unsupported. The associated initializer fixture is now promoted by
[selected initializer execution](pa25-selected-initializers.md), with its original
bytes and independent object/native controls preserved. The generic captured-
environment frontier remains separately owned by the closure implementation.
Specialized conditional reference/output interfaces also remain unsupported;
no borrowed summary is reused as execution proof. General selected initializer
CFG execution, dynamic pattern/enum continuations, remaining slice/reference/
allocation validity families and released view/oracle coverage remain pending.
No full matrix, benchmark, oracle regeneration or assignment activation occurred
in this worker checkpoint. The earlier unexplained scalar RSS increase remains
unresolved; new wall/RSS qualification is pending integrator scheduling.


## Captured enclosing contexts

The original unchanged `inline-specialized/captured-context.rs` now passes types,
LowIR, inspection and native value checks. The retained closure body's proof
signature includes capture universals; it cannot be compared directly with the
selected public call signature. Captured anonymous contexts instead substitute
the complete enclosing arguments, normalize only canonical Self, and validate
that full Self against the selected closure instance using existing source-slot
matching. Runtime captured calls still perform full execution normalization;
empty-environment behavior remains unchanged.

The anonymous body retains its independent source/template, exact complete
context key and original/instantiated checks. This validation grants no callable
permission and imports no receiver universals or conditional source assumptions.
Nonempty Self-normalization proofs or repeated-slot equality debts reject with
an explicit lifetime-proof-transport boundary; this slice does not discard them
or claim their anonymous rebasing is implemented.

The implementation uses the shared helper split `cfaa63b6` (worker cherry-pick
`959dbbab`). Five private anonymous-context invariants pass, including malformed
public result/binder signatures, missing erased owner slots and distinct values
under complete unused type/const arguments. All 26 nested-evaluation tests pass:
27 independently qualified specialized inputs, including 18 accepted native
programs. The two new inputs cover erased type/const arguments and nested public
callable lifetime binders. Direct-path qualification also passes all 32
`environment_execution` and 30 `source_closures` tests, including native controls
and their existing scaling invariants. Strict all-target/all-feature release
Clippy, audit 1410/0, layout 38 and upstream 24+126 pass.

Evidence is under `obj/parallel/pa25-captured-inline/`. Full Self validation runs
on each anonymous materialization cache miss; no mixed many-capture/many-inline
performance claim or wall/RSS measurement is made for this correction. Nonempty
context-debt rebasing, specialized conditional output interfaces and the other
recorded PA25 completeness frontiers remain open. No references, assignment
activation or public refs changed.
