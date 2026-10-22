# Frozen atomic backend controls

These original RustGM maintainer inputs qualify the external backend, not
RustGM's Rust API bindings. `contract.json` registers every input with its exact
hash and independently declared outcome. The existing grouped, ignored
`intrinsic_backend` harness runs them:

```sh
python3 scripts/storage.py run --prune-under obj/parallel/pa28 -- \
  python3 scripts/maintainer_suites.py run intrinsic_backend --release --ignored
```

The 27 LowIR cases comprise 19 valid native programs returning 42, five
Rust-invalid ordering probes that only validate LowIR, and three rejected
numeric memory-order encodings. Invalid contracts never reach native emission
or execution. The independent Rust phase separately checks 20 valid programs,
five `invalid_atomic_ordering` rejections, and an ABI observation. It invokes
only the pinned Rust compiler, never RustGM. These are not frontend rejection
fixtures: backend acceptance of numeric orders does not discharge Rust's
operation-specific ordering rules.

The scalar cases cover signed and unsigned 8/16/32/64-bit storage, all five
RMW orders, legal load/store orders, high bits, wrapping arithmetic, exchange,
and both compare-exchange results. LowIR `atomic_add_fetch` returns the updated
value; Rust `fetch_add` returns the old value. A failed compare-exchange updates
the expected-value scratch; a successful comparison leaves it unchanged.
The historical U32 LowIR control remains at `../atomic-u32.lowir` unchanged.
There is no distinct unsigned 64-bit LowIR scalar: that case deliberately uses
the existing `i64` bit representation.

The pointer case qualifies a physical pointer word and byte-delta add-fetch,
with separate Rust `fetch_byte_add(8)` and `fetch_ptr_add(1)` observations. The
boolean case uses only valid 0/1 bytes and makes no AtomicBool add claim. The
ABI observation also checks isize/usize size/alignment; it is not independent
execution coverage of every AtomicIsize/AtomicUsize API.

Each native artifact is disassembled only after the shared strict ELF64 LE,
x86-64 executable-load-segment checks. The harness verifies exact memory RMW
counts and operand widths. Thread SeqCst emits one `mfence` in this frozen
profile; the other exercised thread orders and all signal fences emit none.
The validated LowIR still retains each fence. These deterministic uncontended
programs establish encoding/result semantics and the inspected lowering only.
They do not establish concurrency correctness or general compiler reordering
properties. Target cfg advertises atomics 8/16/32/64/ptr, not 128.

The original input bytes and outcomes were settled before promotion in
`obj/parallel/pa28/atomic-capabilities/declared-pointer-literals.json`
(SHA-256 `1ce801c47f5517b3a38872cfdf15853f035746942ba8f7e62dcff8e959b125e7`).
Final backend qualification is `fourth/qualification.json` under that root
(SHA-256 `4e4b4ccc37aff5f1f68c91f76339ff3c77b6501c8d5c7bcc037670e6e8c375ae`),
with `instruction-widths.json` and `abi/qualification.json` retained beside it.
Earlier receipts retain the disproved nonexistent AtomicPtr `fetch_add` source
and two rejected LowIR spelling attempts (`obj<32 x 8>` and `null`). Those
superseded attempts are not promoted fixtures. The settled `obj<32x8>` and
`nullptr` bytes are preserved exactly. The additional metadata in the permanent
manifest records those prior observations; it does not retroactively claim
that the promotion/harness itself was already qualified.

Attribution: the fixture programs are original maintainer work. The vocabulary
and result conventions come from frozen [CPPGM commit
905bd7e](https://github.com/vishvananda/cppgm-extended/tree/905bd7e4412a2d77d88694fe6abc7f3e1c27bce1),
especially `pa8/lowir.md`, `dev/src/lowir/text_parser_body.cpp`,
`dev/src/lowir/validator.cpp`, and `dev/src/native/select/emit_memory.cpp`.
Their exact hashes are in the manifest; inherited CPPGM attribution remains in
the repository `NOTICE` and `shared/CPPGM-NOTICE`. No upstream source is copied
into these inputs. The independent source contract is [Rust commit
88d9e12 atomic.rs](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/sync/atomic.rs),
including `fetch_ptr_add` and `fetch_byte_add`.
