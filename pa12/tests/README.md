# PA12 fixture ownership

The manifest declares 107 cases before reference generation: 40 clean native
programs, eight native bounds panics, 12 layout vectors and 47 source rejections
or explicit later-stage boundaries. Every input is discovered; no fixture uses
implicit stdout, status or runtime expectations.

`aggregates`, `arrays`, `enums`, `representation`, `bounds`, `packages`, `reject`
and `boundary` exercise `--emit-lowir`. `layout` explicitly selects the declared
supplementary `--emit-layouts` mode. Its metadata contains independently chosen
sizes/alignments/offsets and names rustc can inspect. Unspecified Rust field order
and transparent zero-sized offsets have explicit comparison exceptions.

The fixtures are authored for RustGM; no new upstream text is imported here.
Required partial-move/pattern/lifetime tests belong to PA14/PA13/PA15, respectively.

Nominal lifetime and nested-reference executable boundary cases moved to PA15
and PA16 when those contracts became independently testable.
