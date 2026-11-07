# Original stdlib unit graphs

`core.json`, `alloc.json` and `std.json` are the unchanged raw Cargo 1.98.0
`--unit-graph` outputs behind [the frozen observations](../../stdlib-graph-outcomes.json).
Their SHA-256s equal each observation's `graph_sha256`, and
`scripts/stdlib_targets.py` refuses any other bytes. They keep the observing
machine's absolute paths. Package IDs bind only to the frozen local inventories
and registry archive pins, and source paths are relocated explicitly to the
caller's roots. The graphs are planning inputs for the target unit executor, not
evidence of any RustGM compilation.
