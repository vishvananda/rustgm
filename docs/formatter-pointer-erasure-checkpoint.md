# Formatter pointer formation and runtime ABI

Qualified source `b7efd4b8` continues the local transmute checkpoint without a
new representation or ABI framework. The pinned formatter's erasure block now
works through a real const constructor and actual runtime indirect calls.
This is a reduced operation/ownership consumer; genuine NonNull, Arguments,
Formatter, Location and PanicInfo still require their source prerequisites.

The previous conversion-time exact-signature restriction was a bounded unsupported
profile, not a Rust language rule. Pinned core `fmt/rt.rs:39–95` explicitly erases
`fn(&T,&mut Formatter)->Result` to an unsafe function taking NonNull, including in
const constructors. Its runtime invocation is at159–165. Pinned interpreter
`validity.rs:902–916` validates a genuine function target and leaves signature
matching as a FIXME; actual call compatibility is separate in `call.rs:169–255`.

Runtime transmute now performs the existing checked snapshot/store/load without
recovering an original signature or scanning value/local/store origins. The
ordinary typed destination supplies the indirect call ABI. No unconditional
runtime validity trap or stronger unsafe caller contract was added. The current
source catalog/header/callee/instance, size, unsafe, lifetime, move and Drop proofs
remain intact. Unresolved generic sizes still reject before publication.

CTFE function words retain their actual canonical original Function Instance and
WordTarget. The destination Function type stays in its existing typed Code slot.
A genuine target can be formed under a different function signature without
executing it or obtaining const/callable permission. Numeric/null/allocation and
vtable data do not become function targets. CTFE function-pointer invocation
remains rejected, even though formation from a nonconst formatter is valid.

The unnecessary structural Origins segment, scratch scan and runtime recovery
method were removed. The original SourceSelect implementation and its three old
adversarial tests are byte-identical to `fde44244`. Ordinary lowering gains no
new scan/cache/field. This removes O(values + locals + statements + projections)
preparation at an erased conversion; remaining word/target checks are bounded
kind/identity lookups. This is an algorithmic change, not a timing-matrix claim.

## Genuine observations and candidate execution

Only four new reducers were observed independently: eight pinned Rust1.98
metadata/object commands plus one reuse of the accepted object for native linking
and execution. All frozen predictions passed. The earlier28 observations were
not repeated. The new provider:

- constructs an erased pointer to a nonconst function inside a const fn;
- preserves that original target in the actual const value;
- invokes it at runtime, including through an ordinary borrowed owner;
- replaces a raw function address through a mutable alias and calls the new value;
- passes an opaque raw address through a real source function before invocation;
- forms an incompatible-signature pointer without invoking UB;
- retains actual lifetime escape, unsafe call and const invocation rejections.

Pinned Rust and the candidate both produced:

```text
5 19 5 23 9 7
```

The first two pairs prove real formatter return/output mutations;9 proves the
mutable-alias replacement was respected;7 proves runtime raw-address transport.
No compiler-written formatter, fake Arguments/NonNull or handler was installed.

The source is
`pa28/tests/spec/source-transmute/formatter-formation-and-runtime.rs`, SHA
`2b582e62fd5a146247347200cac8dbdfc3ba470ed698fd5c9d6213436976ea06`.
The C observer SHA is
`422429bc9dfad11ec572517398ff130d3c6f47ee40a014cfc351622acd4b70ff`.
Candidate LowIR SHA:
`9a12ff16d3ede2733726a31d384bff2b19730c2227fd7b176a3dcd0310d14e36`.
The unchanged backend SHA is
`ded1b6800ce1bdaeaad45cd28c2903d7c96de7cdc29261f6834869a6eaf525fc`.
Native stdout SHA is
`b9ed4473ef8beef3fd4dc007817ca0b62c768f333c94589a4216e6a7118434ca`;
stderr is empty and status0. Successful disposable native outputs were removed.

## Focused qualification and costs

The local successor completed in167.97 seconds, including formatting11.64,
architecture1.97, the missing private build152.88, six exact nonignored controls,
and actual native object/link/run. The guard admitted the unchanged12GiB cap,
150GiB reserve, three-job policy and standard CPU/RAM/task limits. Final managed
usage was5,523,832,832 bytes. No release build or broad suite was run.

The two strict Clippy passes were retained from the actual earlier163.21-second
scope: production59.90/private87.37. Its sole architecture failure was the new
`erased` filename stem reaching seven. The successor moves that100% byte-identical
cfg(test) file to `formatter_calls.rs` and updates only the cfg(test) path, preserving
its module alias. Exact content/module/source proofs authenticate retention;
there is no file/line/include-sensitive behavior. All other Rust, Cargo and
production inputs match. The failed receipt is retained separately. The first
nested-storage-lock launch also remains failed; it ran no compiler or tests.

All2,748 compiler/tool fingerprints in the actual passed receipt match before and
after execution. Body448/Catalog704/Resolution2056/Package256/Origins72/Constant32
were observed directly. No existing owner or wire field changed. This checkpoint
qualifies the library/native producer; it does not claim a newly built CLI.

## Remaining real source path

C's genuine NonNull pattern_type pointer, PhantomData and source constructors are
next prerequisites for the full tagged Arguments representation. Location needs
actual caller_location/track_caller propagation; PanicInfo and the real panic
handler/body path must follow. Real default/custom messages then exercise actual
formatting algorithms and Source Assert with single evaluation, lazy operands
and genuine const failure. No library algorithms belong in compiler factories.

Numeric-reference/ZST and genuine MaybeUninit/union CTFE profiles remain required.
Imported operation/body policy and public wire admission remain unchanged. The
existing24 bootstrap slots and original std Clone/size_of/imported must_use debts
remain untouched. Pinned CFI/KCFI alternatives are not claimed by this default
x86-64 profile. The draft PA28 suite remains planned, with no reference regeneration.

## Evidence

Own-worktree evidence:
`obj/parallel/pa31/transmute-function-abi-v1/`.

- New oracle: `oracle-receipt.json`; frozen plan
  `80fe606a66a8d955b2133f298a5a4faa7773a33387467c0bae505a55e6af624a`.
- Passed actual candidate:
  `obj/qualification/fast/1793033131909114545/receipt.json`, SHA
  `0cfc55db819e388670d04356df1ff0834e5b52f229d3a6bb1e4827d47df37dce`.
- Retained actual strict scope:
  `obj/qualification/fast/1793032494064474700/receipt.json`, SHA
  `93a35320283ebb9d328e54a2128b0271dec85bbd6de0964215a089364c49f40b`.
- The precise retention proof is `candidate-v2/plan.json`; no broader source
  equivalence or earlier failed runtime result is inferred.
