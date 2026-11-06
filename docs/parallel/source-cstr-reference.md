# CStr independently frozen reference contract

This is a reference-only checkpoint on Source parent `fe32eaa1e`. It changes no
frontend, does not run RustGM and makes no candidate or PA completion claim.
Root observed the unchanged original core frontier at actual CStr; this packet
supplies independent sources and native expectations for a later implementation.

All 24 current sources match pinned Rust 1.98.0 under edition 2024, Linux x86-64,
panic abort, cap-lints allow and explicitly declared RUSTC_BOOTSTRAP=1. Inputs
and expected statuses were frozen before each corresponding reference run.
Ten accepted cases and fourteen rejections cover actual source methods, literal
forms, storage, ordinary type/privacy/representation/safety/lifetime/body/const
and borrow rules. Rust accepts unused extra-field and generic CStr role headers.
It also accepts the generic wildcard-literal metadata case, whose initially
uncertain contrary prediction is preserved. These are not semantic negatives or
permission to ignore a demanded actual target profile.

The no_core consumer retains exact source CStr representation/field/attributes,
as_ptr, to_bytes_with_nul and slice::as_ptr intervals. Authored ordinary support
declarations and the real synchronous ptr::drop_glue helper make its execution
independent of a substituted builtin CStr. Its raw-pointer C ABI observer prints
`CStr 65 2 0 195 169 255 92 1`, checking ordinary/empty/UTF-8/high-byte/raw literals,
one terminator, length patterns and same-static-value pointer consistency.

Two separately declared installed-standard-library companions print
`storage 8 65 0` and `construct 111 107 0`. They preserve byte-array versus CStr
const/static identities and actual views, and exercise only valid terminated
constructor inputs. Real Rust static libraries supply their actual standard
runtime dependencies. Neither CStr values nor fat references cross the C ABI.
Byte/C literal storage may have equal addresses; the contract requires correct
typed backing and views, not pointer inequality or equal-byte type substitution.

Complete pinned c_str.rs, slice/mod.rs and ffi/primitives.rs bytes are retained.
The exact drop_glue source fragment and every reached source interval have
byte-accurate offsets and hashes in source-intervals.json. Unused derive generation,
complete module/method closure, general from_ptr/strlen/constructors, CTFE and wire
transport remain requirements. Current pinned representation observations are
not a general CStr layout guarantee.

The adjacent JSON binds every source, observer, prediction, original, tool,
sysroot library and receipt. Final receipt:
`obj/parallel/source-transport/cstr-reference/reference-v4/receipt.json`.
Earlier v1-v3 receipts and original inputs remain in the same evidence root.
They preserve missing Copy/const-stability/drop_glue setup and standalone-object
standard-runtime link failures. These are setup/link failures, not semantic
parity negatives. All final sources, three native observers, harness and upstream
verification pass; private tests and candidate implementation were not attempted.
