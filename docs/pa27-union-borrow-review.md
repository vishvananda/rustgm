# Direct scalar union references during constant evaluation

This bounded PA27 slice supports direct `&u.field` and `&mut u.field` for the
existing initialized scalar-Image profile. Borrow dispatch passes the original
typed UnionField capability to a private union_reference helper. That helper
forms the canonical union projection through the existing evaluator and shares
checked-reference validation. It preserves an existing raw-base anchor, and
creates no synthetic RawAccess or unsafe authority. Raw-only wrappers retain
their anchor requirement. Common validation now explicitly checks the selected
leaf footprint stays within its root allocation before alignment and access.

Source unsafe, mutability, ownership, lifetime and packed-reference proofs remain
independent. Mutable reconstruction retains existing writable per-pointer authority
and ordinary nonstatic storage; shared results remain monotonically restricted.
Initialization and bool/char validity use the current Image bytes and masks.
No public enum, pointer representation, active member, backend or structural
Reader change is introduced.

Twenty-two full-object outcomes distinguish nine supported cases, nine
independently rejected controls and four accepted-Rust boundaries. Shared and
mutable locals, separate-scope cross-field aliases, static demand, closed generic
families, const-call reference returns and Copy snapshots are supported. Both
packed-source cases have physically aligned backing storage: type inspection
succeeds, while executable/constants inspection rejects their source reference.
The second uses a raw base and proves the anchor does not erase that source rule.
Missing unsafe, immutable mutable borrow, invalid bool/char, uninitialized selected
bytes and escaping reference controls reject for their independent reasons.

Write-only partial initialization and invalid-bool repair through mutable
references remain accepted-Rust eager-profile boundaries, as do pointer-bearing
and unit/zero-sized union families. These are not language validity rejections.
The original direct-Borrow fixture becomes supported unchanged. No production
correction was needed after the initial implementation; the two raw-base cases
were independently declared as a follow-up and retained beside the initial 20.

Four ignored-by-default native groups compare local aliases/raw bases, generic
calls and snapshots, static/scalar references and scaled constants against pinned
rustc. Five existing private invariants and thirteen affected source/scaling tests
pass. At N=8/32/128 ordinary borrowing takes 485/1709/6605 steps and 44/68/164
slots; union borrowing takes 947/3443/13427 steps and 45/69/165 slots. The bound
allows linear growth and constant +1 slot overhead. Native scaled values equal
the ordinary controls. No throughput-neutrality claim is made.

Strict all-target/all-feature Clippy and the 1,371-file audit pass. The final
receipt includes fresh independent object/view hashes, upstream verification,
source and compiler fingerprints and ordinary scalar output equality. No active
reference, assignment activation, backend change or full-release claim occurs.

Remaining PA27 profiles include deferred unused/read-only/shared-derived and
write-only reference validity, mutable/foreign/cyclic-static reference formation,
numeric nonnull ZST provenance, general reinterpretation/offsets, pointer/reference/
function/aggregate and zero-sized union families, shared-derived interior writes,
the default runtime invalid-reference-casting lint and full packed/pattern/release
qualification. PA28 still needs its memory/numeric families, broader volatile and
atomic widths/methods/fences and concurrent/release qualification. Neither stage
is complete.
