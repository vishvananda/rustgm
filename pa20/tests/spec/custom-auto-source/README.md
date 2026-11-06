# Custom auto traits and original UnwindSafe: independent references

This finite PA20 packet supplies independently frozen source obligations for the
actual custom-auto frontier in pinned Rust 1.98.0. PA31 owns unstable declaration
and diagnostic-item authority; PA29 owns authenticated transport. This packet's
`manifest.json` lists its independent reference sources. It does not change the
stable PA20 text-oracle manifest or claim a RustGM candidate implementation.

`original-unwind-safe.rs` preserves the complete upstream
`library/core/src/panic/unwind_safe.rs` byte for byte, revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, under MIT OR Apache-2.0. Its checksum
and exact declaration/implementation intervals are in `provenance.json`.
The first five companions embed exact original UnwindSafe, RefUnwindSafe,
AssertUnwindSafe and minimal reference/raw/wrapper/UnsafeCell impl intervals.
They also retain the actual UnsafeCell declaration from core/src/cell.rs.
The dependency shell supplies ordinary no_core sizing, Clone/Copy and genuine
PhantomData declarations. Other cases use an ordinary custom `Auto` trait with
no standard diagnostic identity. No path spelling or role bit supplies evidence.

Predictions in v1 were frozen before any Rust observations. Two written
where-clause controls were separately frozen in v2 before observing them.
The solver owner then requested one-fault declaration controls, frozen separately
in v3/v4; the original combined restrictions remain unchanged. All 34 sources
match in metadata and object modes: 68 checks, with eleven accepted sources and
twenty-three intended ordinary negatives. There are no setup
failures, compiler crashes, timeouts or corrected predictions. The exact compiler,
source hashes, commands, diagnostics and raw receipts are bound by
`reference-v1.json`. All four batches used installed rustc1.98, edition2024,
x86_64-unknown-linux-gnu, panicabort, capallow, bootstrap1 and two compiler threads.
Every batch used a 90-second outer deadline, immediate resource admission, explicit
16 GiB output cap and unchanged shared two-job/150 GiB reserve policy.

## Decisive contracts

- `original-headers` checks the actual diagnostic auto traits and unused real
  implementation dependencies. `original-demands` proves aggregate/tuple/array
  evidence and actual conditional reference/raw-pointer implementations. Owned
  UnsafeCell is UnwindSafe, while its RefUnwindSafe negative is retained. Real
  AssertUnwindSafe implementations permit the wrapper's actual bounds.
- `original-mut-reference`, `original-cell-negative` and `original-shared-cell`
  reject E0277. Standard UnwindSafe's explicit reference rules remain distinct
  from arbitrary custom-auto reference composition.
- `structural-positive` checks structs, enums, unions, tuples, arrays, references,
  raw pointers, real PhantomData and function pointers. Each real component
  supplies its ordinary proof. `where-premise` additionally accepts a function
  pointer returning a negative-auto type: its return type is not an owned
  capture of the pointer value.
- `field-negative`, `raw-negative`, `reference-negative`, `zero-array-negative`
  and `phantom-negative` reject E0277. Zero runtime array elements and zero-sized
  physical storage do not erase logical auto-trait requirements.
- `generic-premise` and `projection-premise` accept actual caller and associated
  result bounds. Their missing-premise opponents reject E0277. `where-premise`
  accepts a written where-clause; `where-no-inversion` rejects E0277 because
  `Wrap<T>: Auto` does not prove `T: Auto`.
- `lifetime-premise` accepts an ordinary local reference under the actual generic
  bound. `lifetime-escape` still rejects with the lifetime diagnostic. Auto
  evidence supplies no static lifetime or loan permission.
- `explicit-positive` checks a real manual impl overriding a negative field,
  plus a legal unsafe impl of an unsafe auto trait. `conditional-positive`
  satisfies its real prerequisite. `conditional-negative` rejects E0277 for
  missing `u8: Allowed`: a relevant explicit generic impl suppresses automatic
  fallback even when its own requirement cannot be proved.
- `recursive-positive` accepts the structural auto cycle. `recursive-negative`
  rejects its negative sibling; `recursive-condition-negative` rejects the
  sibling's failed explicit-impl prerequisite. An active cycle cannot publish
  completed evidence before all other children and debts succeed.
- `declaration-restrictions` checks E0567/E0568/E0380 for generics, supertraits,
  where bounds and associated items. `implementation-safety` checks E0198/E0199/
  E0200 for unsafe negative impls, excess unsafe and missing unsafe respectively.
- The seven `decl-*` sources separate feature-off E0658, generic E0567, supertrait
  E0568, member E0380, and const-auto's precise uncoded rejection. An isolated
  `where Self: Copy` header rejects E0568; its matching empty header accepts.
  These tiny cases use installed std/core so removing `auto_traits` does not
  conflate the intended gate with a no_core foundation's Freeze declaration.

These traits have no runtime methods. Metadata/object evidence directly exercises
the contract; this packet runs no native observer and makes no unwind-runtime
claim. Original atomic/Unique/NonNull adapters, AssertUnwindSafe method bodies,
closures, generated futures/coroutines, arbitrary trait-object/foreign-type and
higher-ranked profiles, full cross-crate coherence and authenticated transport
remain separate required work. Header admission cannot manufacture any of them.

## Primary architecture references

The pinned `rustc_ast_passes/src/ast_validation.rs:958-995,1396-1406` rejects the
ordinary invalid declaration forms. Trait selection's
`candidate_assembly.rs:738-825` withholds automatic candidates for unknown generic
or alias subjects, and suppresses structural candidates when relevant manual
impls exist. `traits/select/mod.rs:1262-1277` requires the cycle to be coinductive;
its constituent producer at2319+ derives actual field/component goals.
The vendored Rust Reference at
`doc/rust-reference/src/special-types-and-traits.md:136-159` specifies automatic
composites, function pointers, logical PhantomData and manual-impl precedence.
Use one typed component frontier and current caller's ordinary proof graph;
completed structural data is not a cache of permission across environments.
