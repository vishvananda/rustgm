# Source Debug field arrays

This planned PA31 cluster owns the source-provider boundary for PA26's Debug
recipe. `contract.json` froze inputs, options and predictions before independent
Rust execution. It is a cluster sidecar, not an activated assignment manifest.

The nine record, tuple and enum inputs cover five, six and seven fields. Five
uses the specialized helper; six and seven use the array helper. Record variants
carry ordered field labels; tuple variants carry values only. Each value is a
shared Debug reference, with an extra reference on the last field. The generic
`unsized-tail.rs` checks that this final indirection supports a `[u8]` tail.
`labels.rs` checks raw-identifier labels without the raw prefix and Unicode
labels. `missing-debug.rs` expands but fails semantic checking with E0277: array
lowering does not waive any field's Debug requirement.

`native-rust.rs` prints six independently predicted lines using ordinary Rust
std. This is independent recipe/output evidence. It does not execute RustGM's
source Formatter or supply a source-provider native consumer.

`source-payloads.rs` is an authored seven-field payload for the actual source
Debug provider. It deliberately supplies no substitute Formatter, generic helper
signature or library body. Root owns its composition with genuine existing
source foundations and the resulting candidate semantic/native consumer. The
existing `source-location/panic-debug-declarations.rs` fragment is incomplete for
this runtime family; merely appending these payloads cannot qualify formatting.

The four `upstream-*.rs` fragments preserve exact bytes from Rust revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. They are not standalone crates.
`contract.json` records the full primary hashes and half-open byte intervals:

- `upstream-fn-ptr.rs`: unchanged seven-field FnPtr declaration and attributes,
  `library/core/src/mem/type_info.rs`, lines327–354.
- `upstream-struct-helper.rs` and `upstream-tuple-helper.rs`: actual Formatter
  method signatures and bodies, `library/core/src/fmt/mod.rs`, lines2562–2579
  and2702–2717. Their builders, iterator and Result dependencies remain real.
- `upstream-recipe.rs`: compiler derive recipe, including cutoff5, static names,
  slice of shared dyn Debug references and final-field indirection,
  `compiler/rustc_builtin_macros/src/deriving/debug.rs`, lines91–211.

These upstream portions retain Rust's attribution and MIT/Apache-2.0 licensing;
see the repository's NOTICE and third-party source attribution. The authored
reducers are distinct from the original FnPtr and do not replace its types,
attributes, complete module dependencies or full original-core acceptance.

Independent evidence lives in
`obj/parallel/debug-field-arrays-rust-v1/` in the fixture worker. All27 commands
matched their frozen status:12 expansions,12 metadata checks, a native build and
run, and upstream verification. The first observer incorrectly compared literal
Unicode bytes with rustc's escaped unpretty strings. Its failed receipt and raw
bytes remain retained. A second read-only review decoded those same string
literals and confirmed the ordered labels without rerunning Rust or changing
inputs or predictions. Source-provider semantics/runtime, full Formatter,
failed-payment/scaling controls and the original core artifact remain pending
this cluster's compiler consumer and complete PA release qualification.
