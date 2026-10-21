# PA30 aggregate callable checkpoint

This planned-assignment checkpoint lowers fixed x86-64 C calls with nonempty,
aligned `repr(C)` structs whose immediate fields have scalar integer, float,
thin pointer/reference or function-pointer representation. It retains object
spans and INTEGER/SSE/none/MEMORY classes in one cached typed ABI plan. Imported
functions, exported bodies and indirect callbacks consume the same plan. The
backend assigns registers and rolls a whole argument back to the stack when
its classes cannot all fit; RustGM does not flatten it into scalar arguments.

`abi::classification::Plans` derives classes from prepared aggregate layouts
and field types. Cache hits charge one signature lookup and retain no new
field data. Canonical LowIR object signatures are interned by full structure,
including classes and noreturn. Distinct object temporary identities and a
sparse per-function arena preserve the ordinary scalar arrays. On this 64-bit
build, scalar Type/Value/Operand remain 1/12/8 bytes, Operation/Instruction
remain 32/40, and Signature remains 12. Function grows from 192 to 200 bytes
for the optional object arena. Empty ordinary call scratch vectors allocate
nothing; object arguments allocate only on the object path.

Incoming object parameters and object results are copied directly with the
pinned `copyobj` object-source contract into ordinary body snapshots. Object
values are never cast to scalar temporaries or addressed with `addr`. Source
body, ownership, loan and destruction proofs remain unchanged. Validators
reject invalid object/result/span identities, scalar/object crossing and
incompatible same-name declarations before serialization.

`pa30/tests/aggregate-outcomes.json` declares thirteen source outcomes: three
accepted programs, seven Rust-valid profile boundaries and three language
rejections. The C programs exercise sixteen Copy struct shapes, a consumed
Drop struct, mixed callback results, integer/SSE whole-argument rollback,
large MEMORY arguments/results, and padding-only eightbytes. Independent
Rust 1.98.0/C execution and the pinned object backend match across three native
programs. A C-returned function pointer is invoked by Rust through the selected
object signature; thin reference and function-pointer fields retain their
ordinary typed contracts. Four private controls and sixty affected source/native
regression groups pass. Strict all-target/all-feature release Clippy,
architecture audit (1390 files, zero findings), layout, upstream pins and both
backend handshakes pass. Receipts, including earlier failed attempts, remain
under `obj/parallel/pa30/aggregate-design/`. The optional object binary and its
lock are unchanged.

Run the source checks normally and select native interoperability explicitly:

```sh
python3 scripts/storage.py run -- cargo test --manifest-path dev/Cargo.toml --test aggregate_calling
python3 scripts/storage.py run -- cargo test --manifest-path dev/Cargo.toml --features object-interop-tests --test aggregate_calling -- --include-ignored
```

This is not PA30 completion or activation. Rust-layout, packed, empty, union,
enum, nested aggregate and array-field values remain explicit boundaries in
this callable profile. Variadic calls, aggregate crate metadata transport and
other target ABIs remain unfinished. Ordinary Rust aggregate calls preserve
their existing internal ABI. Matching release pins and frozen shared/aggregate
performance inputs will be prepared after functional qualification; exclusive
measurement is reserved for the integrator window.
