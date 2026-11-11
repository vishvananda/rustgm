# Deref method receiver reductions

After the conditional const adapter checkpoint, original core first fails in
array iterator len/is_empty at file102:12256/12334. The actual receiver is
ManuallyDrop<PolymorphicIter<[MaybeUninit<T>;N]>>. Its len implementation is generic
over DATA:PartialDrop and does not require unsizing. The unchanged120s observation
remains partial; no original core/alloc/std artifact exists.

Nine ordinary exploratory controls first separated direct calls from the
bootstrap's missing Deref import. The seven relevant general-Deref library forms
belong to PA26, including the real ManuallyDrop positive. Eight direct/explicit
actual-source controls pass. Four wrapped array counterparts fail with either
ordinary or const Deref, with sized or unsized generic declarations. Further
minimization shows a concrete Hold<S>.get() also fails: generic clauses, arrays
and const effects are not required to expose the missing implicit receiver step.
All twenty source counterparts belong to PA31's actual-role contract.

Pinned Rust independently agrees with all27 final source hashes:23 accepts and
four rejections. The concrete native companion executes real shared/mutable
receivers and prints receiver23/41 through its C observer. RustGM e7b59bbd binds
compiler27cef3f64/3898 Rust hashes; it currently accepts source direct/explicit
and inherent-priority controls but rejects eight valid source implicit calls.
Ordinary bootstrap setup still blocks its positive counterparts. Early lookup
rejections do not qualify the source move/mutability opponents. No implementation
change, course oracle or candidate-native pass is part of this packet.

The existing receiver chooser explores built-in reference depths. Its separate
Target-owner discovery is gated to arbitrary Self types and supplies no executable
Deref adjustment. Extending that owner discovery alone cannot implement these
calls. Reuse the actual Deref/Target projection, scoped obligations and retained
call/lifetime facts; execute each real selected Deref or DerefMut call before the
final receiver borrow. The existing field access frontier already retains those
steps and their current source-call context. Share that machinery rather than
introducing a parallel proof or synthesized AST call.

Preserve receiver-level ordering: value/shared/mutable candidates and wrapper
inherent/trait priority must be tried before advancing through Deref. Choose
DerefMut only for the actual mutable consumer, retain mutable-place and move
opponents, and evaluate the original receiver once before explicit arguments.
Every projected result and call must retain its full const/source/lifetime proof.
Use a typed absent-candidate result; never suppress infrastructure, preparation
or canonical-query errors to continue receiver discovery. Bound cycles, retained
steps and work before publication. Check callable/capture, projection, region,
mutable indexing, field-chain and arbitrary-Self controls alongside the new cases.

Manifests and generated independent receipts are in the PA26 and PA31
`tests/spec/deref-methods` packets. Raw predeclared plans, all observations and
Rust native evidence remain under `obj/parallel/array-receiver/`. Committed files
are byte-identical to those observed inputs; their original paths are recorded.
The compiler and original upstream files are unchanged. Both whole assignments,
ordinary bootstrap, general imported contracts and original artifacts remain open.

Full42966631710 is running on the preceding semantic publicationa1c3d961c;
its fast job passed. Do not cancel or redispatch it for this input-only packet.
