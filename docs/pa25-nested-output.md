# Nested result and formation interfaces (draft)

This checkpoint is being qualified; it does not activate PA24 or PA25. The
generic anonymous-body specialization boundary remains separate.

## Identity and proof contract

Nested inference retains its original actual result. Fresh parent witnesses
transport child-private result regions across the inference interface. They do
not equate a borrow with static storage or with a captured lifetime. The formal
closure signature and ABI remain unchanged. A retained internal result, when
different, drives the exact typed CFG return checks; a separate ordinary
coercion obligation proves that result against the formal signature. Explicit
returns, branch returns and diverging executable roots use the solved result
term, not the type of the executable root.

A semantic Link records the exact child source, its parent, the interface
revision and region-slot bridges. The conditional region proof may export only
relations between mapped enclosing/interface regions and static storage. Local
storage, hidden receiver lifetimes and unexportable input-only universals cannot
be exported. The complete original child constraint graph is proved again under
exactly those relations. The parent must independently prove every mapped
relation. There is no execution permit or positive result cache attached to a
conditional summary.

Each summary retains its exact BodyKey, source and immutable Link identity.
Specialized linked bodies are an explicit boundary in this slice. Unrelated
generic instances retain the existing proof path and may share diagnostic
sources without becoming ambiguous participating parents.

Capture precision also needs source-input formation facts. Each actually used
capture root supplies its exact opened child type and its original parent type,
using the retained Environment mapping. The parent must prove well-formedness of
the parent type even if the closure is never invoked. Only the corresponding
child type is assumed well-formed in the child. Formation-only links still run
the complete child proof and require parent discharge even when their output
condition list is empty.

## Ownership and execution

Participating original CFGs are lent to one bottom-up region-proof batch before
the ordinary incremental verifier drains them. The batch indexes parent/child
dependencies once. All participating bodies must succeed before its proofs are
retained; each retained proof is consumed once by the existing ownership, loan,
storage and destruction checks. No CFG is reconstructed for this batch, body
order is preserved, and unconsumed original proofs prevent publication of a
checked program.

Inline constants share their root syntax with the parent's const expression,
but the two ownership contexts have different completed facts. An inline body
with exported result regions retains its own NodeFacts and adjustment selections.
The CFG builder consults that snapshot only at that anonymous body's exact root.
Parent publication resets the previous root adjustment and publishes its own
selection. Execution requests use the typed inline body's retained result;
ordinary non-inline requests continue using their expression facts. Lender
identity and exact result checks remain mandatory.

## Bounded work and storage

The semantic interface registry permits at most 65,536 ports, 65,536 links and
1,000,000 combined slot/formation entries. It charges retained record, map-entry
and boxed-array payload against 64 MiB. Charges follow support::retained's
logical payload contract, excluding allocator headers/control bytes and shared
immutable pointees already charged to their original owner. Anonymous root
snapshots are charged to the existing inline registry before boxing.

The proof scheduler visits each body/dependency once and preserves deterministic
input ordering. Per-body graph work includes conditional discovery and the full
second proof under the discovered hypotheses; both share the graph work limit.
Port witnesses are allocated once and reused across inference refreshes.

## Qualification

The first complete parent-discharge probe accepted borrowed outer references,
borrowed mutable referents returned as shared references, moved FnOnce reference
results, joined borrows, explicit/branch/diverging returns and a closure mixed
with unrelated generic instances. Owned-capture and FnMut receiver escapes still
failed ordinary lifetime proof. The first inline-root snapshot probe reached the
checked lender and exposed execution's use of the parent expression type;
the typed result lookup correction subsequently passed inferred and explicit
inline-reference return controls.

All original observations are retained under
obj/parallel/pa25-nested-output. The aggregate field-return probe exposed a
separate lifetime-only receiver issue: the mixed generic table has no entry for
an owner whose only parameters are lifetimes. The receiver now validates the
exact nominal owner and uses the existing lifetime declaration arity and slot
substitution for that case. The mixed type/const path is unchanged. The
unchanged field-return probe and original once-partial-cleanup source now pass.

The sixth focused batch passed all-target/all-feature checking, 15 portable
nested/inline tests (two native groups intentionally ignored), and all ten
independently qualified adjusted receiver outcomes. These include method-deep
and call-mutable, which previously lost the source root's well-formedness
relations after capture precision. Port/root invariants cover 8/32/128 distinct
closures. The malformed receiver unit's first filter matched zero tests and is
not counted as an observed pass from that earlier batch. Its corrected exact
filter later ran one test successfully.

Strict release all-target/all-feature Clippy and the architecture/file audit
passed (1,343 Rust files, zero findings). The final source/native gate passed all
17 nested/inline tests with both native groups explicitly enabled, and all nine
capture execution groups including the adjusted endpoints. The two graph tests,
two registry identity/budget tests and one malformed receiver test passed. Native
comparison includes 14 output programs and the ten existing inline programs,
with independent Rust execution and deterministic LowIR comparison. Upstream
verification passed all 24 pinned fixtures and 126 Rust Reference files.

The ordinary returned-field control is owned by PA16 at
`tests/spec/thin-native/returned-lifetime-field.rs`. Its explicit accepted
LowIR/native manifest entry is included, but references and the generated
receipt remain pending integrator qualification. Assignment layout is therefore
pending; the worker has not generated references or claimed layout success.
The exact final program wrapper and all 14 output native values passed again.
The focused region/caller/closure regression batch passed 82 tests with four
existing backend tests ignored. The two original inferred-inline reference
fixtures passed unchanged-byte CLI and native checks; their draft outcomes now
record acceptance. The original local `&u8` annotation case remains an uncovered
valid-Rust boundary, with its source and acceptance evidence unchanged.

Evidence is retained under `obj/parallel/pa25-nested-output`, including the
failed/canceled initial probes, `eighth-clippy.log`, the `ninth-*` qualification
logs, `tenth-*` regressions, `original-frontiers/outcomes.json` and `source.json`.
No oracle changes are authorized by this document.

Generic anonymous specialization and participating specialized output links
remain unsupported. Conditional output graphs containing projected obligations,
membership choices or verification alternatives retain explicit boundaries in
this initial slice. This checkpoint does not claim PA25 completion. The earlier
unexplained scalar RSS increase also remains unresolved; no performance claim
is inferred from these functional results.

The final Rust source-set hash is `2979a0ae06dc9e7a316bff1967c366f2d27dd3cb7cbdb541e7cb5dd1b8592293`
(canonical compact JSON of sorted relative `dev/**/*.rs` paths to file SHA256).
The qualified debug frontend is `3e5bebcbe06fe733c286da50438388b75d3870f2d126b21d7782d1692ef3c279`.
The final three-job trial receipt is
`obj/parallel/resources/480586077eb447db8e13deb725792baa.json`:
return zero, no quota/reserve error, worker storage 9,460,555,776 bytes within
50 GiB. This functional receipt is not a compiler performance measurement.
