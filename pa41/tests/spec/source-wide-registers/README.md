# Wider vector register classes and actual payload widths

The outcome manifest declares the AVX2-host runtime and AVX-512 object-only
contracts before execution. The three original 256-bit non-temporal stores run
with every byte checked, source preservation and surrounding sentinels. Authored
256-bit copy/load and 128-bit payloads in YMM registers have the same guards.
The complete original eight streaming functions are inserted verbatim from the
three pinned files under `../../upstream/rust`; see `third_party/sources.json`
and `shared/RUST-NOTICE` for source identity and licensing.

A payload keeps its actual 128-, 256- or 512-bit storage width independently of
its YMM/ZMM register view. Narrow operands never acquire wider snapshot reads.
The AVX-512 client is independently object-qualified and lowered to an actual
backend object, including 128/256-bit payloads in ZMM views. This host lacks
AVX-512, so its expected-success runtime remains pending with a C observer.

Original non-temporal `vmovntdqa` load runtime acceptance remains expected
success, separately blocked by the pinned backend's missing opcode. The authored
ordinary loads are distinct controls; no original instruction is replaced.
Version 13 retains the YMM-class fence. A wrong-type client must reject for its
argument type. No generated reference oracle is changed.

`forged_client.rs` is an honest compile-only control under a stronger caller.
The in-process forgery changes the defining body's YMM operands to ZMM while
keeping its AVX-only header. Fresh import must reject that requirement even
though the caller declares AVX-512.
