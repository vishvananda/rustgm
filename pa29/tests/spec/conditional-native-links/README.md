# Conditional native dependencies

`contract-v2.json` and `cases-v2.tsv` declare 47 authored source fixtures in
Rust 2021 and 2024 before qualification. Predictions use pinned Rust 1.98.0,
Linux x86-64, an explicit Allow lint cap and aborting panics. The native cases
also disable host debug assertions, matching the bring-up target. Build the
declared `selected.c` archive before metadata checks: rustc requires a selected
static library to be available even for its metadata-only invocation.

The positive and control programs execute Rust `main`, call the real selected
archive, and call a separately supplied C observer. Both must print `7 9\n`.
The false dependency names a nonexistent library: its declaration remains
visible, with its original unsafe contract, while its library is never linked.
`ffi_dependencies` runs this native pair in the maintainer linking suite.

The corpus covers feature requests, genuine crate permission versus internal
macro allowances, forwarded metadata, literal and predicate failures, duplicate
options, disabled owners in either attribute order, wrong attribute targets,
and independent cfg-query gates. V2 fixes only the oracle's archive setup;
source bytes and predicted outcomes are unchanged from V1. Repository-relative
fixture paths are a portability correction and have independent requalification.

The implementation retains an evaluated condition with the actual configured
attribute. Enabled `dylib` and `static` dependencies become a separate bounded
plan. An explicit `--native-links-output` serializes it for the external backend;
LowIR and foreign symbol names remain separate typed contracts. Dynamic-library
dependencies also have a versioned crate metadata extension. Static archive
bundling, other native kinds and modifiers, and broader imported transport
qualification remain required PA29 work. An unsupported publication must fail
instead of silently losing its native dependency. This packet does not release
the assignment or establish an original core/alloc/std artifact.
