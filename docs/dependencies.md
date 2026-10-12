# Initial dependency review

The compiler uses `unicode-ident = 1.0.24`, pinned in Cargo.lock, with its
default feature set. It supplies Unicode XID_Start/XID_Continue property lookup;
it does not perform tokenization, parsing, macro expansion or semantic analysis.
Its MIT/Apache-2.0 and Unicode data terms are recorded by the package's license
files. The compiler forbids authored unsafe code; third-party internals are a
separate reviewed boundary. NFC interning is not implemented in PA1 and needs
its own dependency/data review at PA6.

Development audit dependencies, isolated in `obj/audit-venv`, are tree-sitter
0.25.2 and tree-sitter-rust 0.24.0. They parse Rust implementation source for
file/function/nesting and architecture audits; they are never linked into the
compiler. Standard-library Python runs the course harness and backend adapter.

Pin the compiler toolchain, Unicode property version and dependencies together
when expanding the language profile. No rustc_lexer, syn, rust-analyzer parser
or rustc frontend crate implements the maintainer solution's assigned phases.

PA2 adds no dependency. Decimal floats use Rust's standard `f32::from_str` and
`f64::from_str`, exactly the conversion operations named by the pinned Reference.
Typed literal decoding, suffixes, checked u128 accumulation and escapes are
authored here. The standard parsers are value conversions, not a delegated
Rust language frontend. Only independent test tools invoke rustc.

PA6 adds `unicode-normalization = 0.1.25` (MIT OR Apache-2.0), with its default
`std` feature and Unicode 17.0.0 normalization tables. Its only role is NFC
identifier normalization; ASCII interning bypasses normalization. The lockfile
pins `tinyvec 1.10.0` (`alloc`, which enables `tinyvec_macros`) and
`tinyvec_macros 0.1.1`, both Zlib OR Apache-2.0 OR MIT. Package license texts and
registry checksums accompany the pinned distributions in Cargo's cache.

Review covered normalization/composition, Hangul decomposition and the enabled
small-vector features. `unicode-normalization` has narrow unsafe scalar
construction internally: its Hangul arithmetic constructs values inside the
specified Hangul ranges. `tinyvec` forbids unsafe code. RustGM's own source
continues to forbid unsafe code; this dependency review does not assert that
all transitive implementation code is safe Rust. Neither dependency supplies
Rust syntax or semantic analysis. Inline invariants cover decomposed Latin,
Hangul, raw identifiers, lifetimes, table growth and repeated interning.
