# Pure and readonly assembly with vector outputs

The manifest declares seven native observations before execution. Fresh metadata
clients load all 128 input bits, observe changed memory on a second call, copy a
vector through separate input/output registers, zero an inout, and write two
aliased outputs in source order. Every vector write preserves adjacent C storage, and by-value calls preserve
all source-vector bytes.
A scalar readonly load observes two memory values; ordinary code returns 99.

The complete original SSE4.1 `_mm_stream_load_si128` function is inserted verbatim
from pinned Rust 1.98.0 source, including its target feature and original template.
See `../../upstream/rust/sse41.rs`, `third_party/sources.json` and
`shared/RUST-NOTICE`. Pinned CPPGM rejects `movntdqa`; `runtime_client.rs` retains
expected-success original execution. The qualified `load` is an authored ordinary
`movdqa` control. No original opcode or source byte is replaced.

Version 13 retains the pure-option fence. A wrong-type client must reject for its
argument type. Raw-row opponents additionally require a real output and a valid
pure memory contract before importing a body.
