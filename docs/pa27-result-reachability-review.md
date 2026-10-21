# PA27 zero-length constant result storage

The constant interpreter followed the fill recipe of every repeat while checking final allocation liveness. A zero-length array therefore rejected a raw pointer operand that referred to a finished local, although the array stored no pointer. The correction traverses repeat elements only when the length is nonzero. Operand execution, whole-CFG const eligibility, initialization and nonzero result checks remain unchanged.

## Independent outcomes

Thirteen hypotheses were declared before pinned Rust 1.98/edition 2024/x86-64 object compilation. All thirteen independent outcomes matched those hypotheses. The original declaration and initial compiler observations are retained under `obj/parallel/pa27/validity-liveness/{predeclared,results}.json`; the tracked hypothesis receipt and fixtures remain unchanged after qualification.

| Case | Rust object status | Initial types / LowIR | Corrected types / LowIR |
| --- | ---: | --- | --- |
| `dangling-raw-repeat` | 1 | 1 / 1 | 1 / 1 |
| `expired-raw-read` | 1 | 1 / 1 | 1 / 1 |
| `integer-raw-address` | 0 | 0 / 0 | 0 / 0 |
| `packed-copy-compound` | 0 | 0 / 0 | 0 / 0 |
| `packed-match-reference` | 1 | 0 / 1 | 0 / 1 |
| `packed-raw-address` | 0 | 0 / 0 | 0 / 0 |
| `packed-reference-pattern` | 1 | 0 / 1 | 0 / 1 |
| `packed-value-pattern` | 0 | 0 / 0 | 0 / 0 |
| `transient-local-raw` | 0 | 0 / 0 | 0 / 0 |
| `transient-returned-raw` | 0 | 0 / 0 | 0 / 0 |
| `zero-raw-repeat-function` | 0 | 1 / 1 | 0 / 0 |
| `zero-raw-repeat-local` | 0 | 1 / 1 | 0 / 0 |
| `zero-raw-repeat-nested` | 0 | 1 / 1 | 0 / 0 |

The two packed reference-pattern cases deliberately retain PA9's inspection boundary: type inspection publishes reference-alignment obligations; the executable pipeline checks them and rejects unaligned references. Packed copies, primitive compound assignment and raw address formation pass. This correction does not move layout proofs into earlier inspection.

All three empty raw-pointer repeats now pass. The nonzero repeat still rejects with `constant pointer refers to expired storage`; pinned Rust rejects a dangling pointer in a final constant. Discarded transient raw pointers and integer raw addresses already agreed and remain accepted.

The zero-repeat invalid-read control still evaluates its operand. Pinned Rust rejects the expired-storage read with E0080. The current compiler rejects through its explicit `raw pointer dereference in constant execution is not implemented` boundary. That outcome is retained as uncovered validity behavior and does not establish expired-read enforcement.

## Validation

- Strict all-target/all-feature Clippy passes.
- Six affected test binaries pass: reachable_storage, constant_storage, constant_locations, initializer_lifetimes, unsafe_unions and atomic_storage.
- The new source test independently compiles every complete input to an object and checks both frontend modes with separately declared phase outcomes.
- Three native controls pass under pinned rustc and the unchanged locked CPPGM backend: empty pointer storage, discarded transient/numeric pointer values and packed copies. Both execute successfully with empty streams; the native input prefix and complete qualified-input hashes are recorded in the draft outcomes.
- Rust audit reports 1,272 files and zero findings; layout and pinned upstream checks pass.
- A nonexistent make test-layout target stopped the initial shell batch after its successful audit; only the missing layout/upstream checks were rerun using check_layout.py and make test-upstream. Passing compiler checks were not repeated.

No active reference sidecars changed and no assignment was activated. Constant union byte initialization, pointer provenance, scalar validity and raw dereference remain separate unfinished PA27 work.

The subsequent symbolic raw-read slice replaces the expired-read unsupported boundary with the existing allocation-liveness check. The original zero-repeat invalid-read input is unchanged; it still evaluates the operand and now rejects because its storage has expired. The unsupported result above records the original reachability checkpoint.
