# Optional PA30 LowIR object bridge

The executable bridge keeps its reviewed `lowir` and `lowir2native` tools,
configuration and invocation sequence. The optional object profile separately
pins CPPGM revision `3438f5aea24d8fcc9a0113316f7e27c890cd5ac6` and its
`cppgm++` driver, used only with serialized LowIR and `-c -O0`. It validates
host-object ABI facts and emits a host ELF object. The earlier executable
profile retains revision `905bd7e4412a2d77d88694fe6abc7f3e1c27bce1` and its two
original hashes; those tools were not relabeled with the newer checkout revision.
No Rust source is passed to CPPGM. The bridge rejects inputs whose suffix is
not `.lowir` before selecting or invoking the multi-input driver.

The reviewed driver SHA-256 is
`ded1b6800ce1bdaeaad45cd28c2903d7c96de7cdc29261f6834869a6eaf525fc`.
The optional lock profile declares the exact target, ELF format and LowIR-to-object
capability. Configuration verifies both its checkout commit and binary bytes, copies
the driver to `obj/backend/pins/<sha256>/cppgm++`, verifies the copy, makes it
read-only/executable and retains a storage pin marker. Execution hashes the
immutable copy again. It never executes a live rebuilt checkout driver.

After the ordinary executable backend has been configured, run these commands
through the repository's storage/resource guard:

```sh
python3 scripts/backend.py --configure-object /path/to/reviewed/cppgm
python3 scripts/backend.py --emit-object input.lowir -o output.o
python3 scripts/backend.py --test-object
```

Object configuration preserves the existing executable tool paths. The default
bridge validates only its original two required tools; the object profile
requires the optional third tool explicitly. An unavailable or changed object
tool is an error, never a fallback to another frontend or backend.

The object smoke test compiles hand-written LowIR, links it with an independent
C translation unit using host `cc`, then executes the result. It covers integer
and floating exports, a direct import, an indirect callback and imported/exported
globals. Every object build independently checks the ELF header for ELF64,
little-endian encoding, relocatable object kind and x86-64 machine identity.
Unit boundary controls verify suffix rejection before invocation,
preservation of the original tools, isolation from a later live driver rebuild
and rejection of a modified immutable pin.

This bridge is infrastructure for the unfinished PA30 foreign registry and
aggregate ABI lowering. It does not activate PA30 or supply Rust semantic facts.
No CPPGM implementation or existing executable backend hash is changed.

The focused profile gate passes both boundary units, the original executable
smoke before and after optional configuration, the C object handshake and CLI
Rust-input rejection. The boundary units are discovered by the ordinary
`test-harness` target: all 50 harness tests and `test-backend` pass. A separate
`readelf --file-header --wide` qualification independently confirms ELF64,
little-endian x86-64 relocatable output. Commands, logs, object hashes and format
evidence remain in `obj/parallel/pa30/bridge/` (`final.json`, `harness.log`,
`object-format.json`, `readelf.stdout` and the focused profile receipts).

The first optional configuration attempt correctly rejected the live checkout's
revision against the older executable profile. The reviewed separate optional
profile records that newer provenance explicitly; default commit and hashes
remain unchanged.

Default configuration now also copies both original tools to hash-named immutable
pins after checking the original commit and both lock hashes. It retains an
already configured optional driver. A live checkout rebuild cannot change a
configured executable bridge; changed pin bytes still fail before execution.
The regression test executes the configured validator/linker after overwriting
both live tools, then verifies rejection of a modified pin, a mismatched commit,
a changed live binary and an existing corrupt pin without replacing local
configuration. The updated focused gate passes all eight bridge tests, all 56
harness tests, the 38-stage layout check and the original exit-42 backend smoke.
No lock hash or CPPGM source changes are involved.
