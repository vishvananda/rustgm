# Actual source offset member selection

The genuine upstream `offset_of!` macro now reaches the existing source
intrinsic and layout kernel. The unchanged provider in
`pa31/tests/spec/source-offset-of/provider/` produces native
`1 4 8 0 0 8 16`, matching its eight pinned Rust metadata/object/link/run
observations across editions 2021 and 2024. This completes the finite original
macro consumer that the earlier [direct kernel](pa31-reached-offset-kernel.md)
left fenced. PA28 and PA31 remain incomplete.

Builtin syntax retains its actual source type and member operands. A cold plan
in the existing call-context owner authenticates the source call, body, node,
module and checked intrinsic. Member selection validates the real ADT,
variant/field identities, ordinals and owner intervals. It checks source privacy
without autodereferencing. The enum feature requires the compiling crate's own
request even when the original macro supplies an intrinsic allowance.

Container types participate in the existing local annotation and const-argument
dependency scheduling. This preserves the actual inline parent's generic
environment rather than substituting a similarly named formal from another
owner. Temporary-scope analysis treats the builtin as a value leaf; its member
metadata does not read a runtime container. Lowering emits an ordinary function
item, two checked u32 operands and a call to the existing reached kernel.

The finite profile selects one field, including `Variant.field` for an enum.
Fixed scratch bounds rejected nested paths. Name interning and immutable lookup
charge raw/Unicode normalization, attempted work and geometric storage growth.
There is no new result cache, type-store census, environment fallback or bound
premise. Operation remains 16 bytes and NodeFacts 80 bytes; Body, Code and
Rvalue do not grow.

Worker gate `1793198480285940493` passes eight exact private controls. Root's
joined gate `1793199147503376522` passes thirteen controls in 256.31 seconds,
including source mutations, enum/privacy/API opponents, name-work scaling,
same-instance fields and comparison/lint controls. Production/CLI gate
`1793199619799548310` passes ten lexer controls and all PA6/PA12 text cases in
184.91 seconds. Strict Clippy, formatting and architecture checks pass in their
selected compilation modes.

Root's actual CPPGM native observation is
`obj/parallel/offset-members-integration-v1/native-observed-v1/receipt.json`,
SHA-256 `7697c7d056d9e6705473a7e1629a0c48318a5cbffede9f19b633eaebb13c448c`.
The unchanged original provider has SHA-256
`e99eb43843e7289ebd66825d0b5d7e23fba9477131bb837aa2dc09951b8afeca`.
Root's subsequent CLI/header, 260 harness, registration, placement and upstream
checks are recorded in
`obj/parallel/offset-members-integration-v1/observed-v1/receipt.json`, SHA-256
`feb734c82128decc62ae31820f76ba49dcecd71a1cdc0e421d79766ae50e4a03`.

The later hidden-glob integration also passes the actual macro control and all
eight audited storage assertions, together with its eight lint/lineage controls:
17 exact tests in 205.24 seconds at `1793200728407559823`. Dense policy nodes
remain four-byte IDs; the measured additional declaration storage belongs to
cold crate-level policy rows and request progress.

Nested offset sums, DST/static alignment, bare tuple and external source
contexts, broader niche/layout parity and authenticated source metadata remain
required. These reducers establish actual macro execution; they do not establish
an original core, alloc or std artifact. Original-core observations in the older
offset receipt precede hidden-glob integration and must not describe a later CLI.
