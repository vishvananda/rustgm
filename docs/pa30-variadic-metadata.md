# PA30 variadic declaration metadata

This maintainer checkpoint transports C variadic signatures across real crate
boundaries. PA30 remains planned; this is not assignment release qualification.

The signature owns a typed Rust/fixed-C/variadic-C convention independently of
safety. Profile eight is the outermost envelope, present only if a signature is
variadic. Public names, aggregate graphs and named storage retain their existing
inner encodings. Removing the envelope and clearing the variadic bit reproduces
the exact fixed-signature bytes. Decoding rejects redundant or reordered
envelopes, Rust variadics, unknown flags, truncation and unsupported const bits.
No body transport or new execution permission is introduced.

A public foreign-block declaration can be published only through its matching
typed import registry entry. It keeps its external symbol and declared safety.
Separate compilation imports it through the same ordinary call and ABI paths
used locally, with one shared type graph and indexed remapping. Aliases preserve
defining identity. Public foreign functions remain imports during lowering.

`pa30/tests/variadic-metadata-outcomes.json` freezes two provider crates, ten
accepted clients and nine rejected clients. Pinned Rust independently qualifies
all 21 sources. Ten CPPGM-backed executables match the Rust/C controls for zero
and spilled integer arguments, spilled floating arguments, pointers, safe foreign
calls, aliases, mixed aggregate results, record variadic tails, callbacks and
constant function pointers. Negatives retain unsafe-call, callable-convention,
explicit C promotion, fixed-argument count, move, loan and const-call rules.
The C provider is the existing unchanged PA30 variadic provider.

Codec tests verify envelope composition, exact legacy bytes, malformed headers
and complete prefix truncation. Import widths 8, 32 and 128 retain one type node,
one signature per declaration, two roots per signature and linear metadata size.
No new integration root or backend profile is needed.

Focused evidence is retained under `obj/parallel/pa30-variadic-metadata/`:
`independent-second` qualifies all source and native expectations;
`focused-fourth` passes format, strict all-target Clippy, file audit, all 43 codec
tests and import scaling; `focused-fifth` passes candidate source and native
comparisons. Earlier observations remain recorded: the original moved-field
wildcard is valid Rust and remains a positive fixture; a separate arithmetic
read is the independently rejected move case. The codec fixture initially omitted
its public static binding, and one expected diagnostic substring used the wrong
wording. Both failures and their explanations are retained; no production
validation rule was relaxed.

Combined release tests, all active text suites and integration qualification
remain required before publication. Generic bodies, const bodies and additional
unsupported callable/type contracts retain their existing assignment owners.
