# AtomicU32 swap and fetch_add

This checkpoint adds the exact canonical `AtomicU32::swap` and `fetch_add` methods. Both are safe runtime-only intrinsics and retain ordinary inherent receiver ownership, shared storage permissions and function-pointer reification. The existing typed `Rvalue::Intrinsic` payload is unchanged. No spelling-based call dispatch, source-level unsafe exemption or const write permission is introduced.

LowIR gains typed `AtomicExchange` and `AtomicAddFetch` operations plus `RmwOrder`. The writer uses the already qualified pinned backend vocabulary `atomic_exchange` and `atomic_add_fetch`; the backend is unchanged. Rust swap returns the captured exchange result. Rust fetch_add returns the old value by pure wrapping u32 subtraction of its operand from the captured add-and-fetch result. It performs no further read of the atomic object.

| Rust Ordering | Rust tag | LowIR order code |
| --- | ---: | ---: |
| Relaxed | 0 | 0 |
| Release | 1 | 3 |
| Acquire | 2 | 2 |
| AcqRel | 3 | 4 |
| SeqCst | 4 | 5 |

All five orderings are legal for these RMW operations. The existing source-sensitive invalid-order lint still applies independently to load/store. A shared typed permission helper selects operation-specific legality; bounded shim switches translate Rust tags and preserve the invalid-tag abort path. Typed RMW builders initially accept u32 only.

Twenty-one independently qualified full-object source controls match three frontend views. They include direct and variable orderings, aliases, UFCS, reified methods, safe-to-unsafe function-pointer narrowing, shared/nested statics, wrapping inputs and ordinary type/arity errors. Direct const calls, retained skipped branches and function-item aliases reject specifically as runtime-only intrinsics. Explicit reified pointer calls retain their separate non-const-call rejection. Packed receiver borrowing rejects at executable checking; earlier type inspection retains its alignment obligation and remains status zero.

Four independent native groups verify every ordering, old/new values, wraparound, reified/UFCS calls, ignored effects and shared/nested static storage. Scaling controls at 8/32/128 callers retain four canonical atomic methods and exactly two demanded RMW shims. Their bodies contain five exchanges, five add-and-fetch operations and five pure wrapping subtractions. Each shim loads only its u8 Ordering argument outside the atomic operation: there is no additional atomic-object load, store or object copy. Body work remains below `1024*(callers+1)`. The ordinary no-Atomic input retains byte-identical types and LowIR output to the immutable Image qualification. These are bounded structural/output checks, not a throughput or concurrency claim.

All five focused source/native binaries pass (23 tests), as do strict all-target/all-feature Clippy, file audit (1,327 files, zero findings), assignment layout and pinned upstream verification. Exact input/artifact hashes, diagnostics, phase statuses and evidence paths are in `pa28/tests/atomic-rmw-outcomes.json`; raw gates are under `obj/parallel/pa28/rmw-final/`. Earlier incomplete gates and the corrected test-only assertion compile error remain separately recorded. The all-dev-Rust source-map SHA256 is `cef517affabf04c4eacac50a475cad61478470462487a24c2aed4446f6c44c01`.

No assignment activation, reference generation, full matrix or performance study occurs here. Compare-exchange and additional intrinsic families remain separate work; concurrent-execution stress and combined release/performance validation remain integrator-owned.
