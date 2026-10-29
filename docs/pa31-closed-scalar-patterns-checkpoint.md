# Closed inclusive scalar pattern consumer

Commit929b90e28 implements the complete local closed-inclusive consumer selected
by the original Nanoseconds declaration. The captured pattern stays a typed AST.
Plain captured endpoint bindings become expression names only in the fresh
builtin tree, with their original symbols, hygiene and spans. Ordinary typed
constant requests and continuations evaluate literal, signed and named endpoints.
No rendered syntax is reparsed and no source helper body is invented.

An opaque RangeId identifies a paid cold interval containing the actual base
type and target-width inclusive endpoint bits. Signed intervals can wrap in raw
bit order. Cross-store equality, hashing and normalization use interval contents,
not local IDs. Initialization, pointer provenance and base validity, including
Unicode scalar validity, precede refinement checks. Lowering preserves the real
base representation. Only genuine NonNull selects the existing nullable-pointer
ABI rule; scalar integer enum niches remain explicitly unsupported.

Store472, Kind24, Constructor16, Node36 and Body456 remain unchanged. The already
cold arguments storage is224bytes and owns the optional boxed interval interner.
Cold/warm identity, paid interning, failed payment and cross-store normalization
have focused controls. No wall-time speedup is claimed.

The declared frozen source contract is
`pa31/tests/spec/source-scalar-patterns/contract.json`. Eight Rust sources cover
the source Clone/Copy/native consumer and seven genuine rejections: below/above
the interval, invalid char base, uninitialized union read, pointer bits, reversed
range and missing feature. Eleven independent pinned-Rust operations pass.
Candidate LowIR validates and executes through the frozen CPPGM backend, matching
the independent stream `0 7 999999999 -2 65` with status zero. Wider original
comparison, hashing and formatting methods are not replaced by the fixture.

Private8 passes132.84s; actual receipt1793269802164695571 has SHA256
`5a2f7d1da3abd451eca2e5394bd101e96a37ffcaf0aeb7de41fef4dcf9226e76`.
All3155 compiler/test/harness fingerprint entries and all11 frozen fixture hashes
exactly match Root after the normal merge. Root reuses that actual private/native
evidence without claiming a repeated execution. Independent proof SHA256 is
`268f41dc2a48dcd95a16c7b68cd69b1d09842f2c81fa6e976cdb036e4c071655`;
native proof is
`7667b95b78f0e29af5dab5bcefdc047925f233ff3e19925742040d58b756222e`.

Root production/CLI3 passes231.05s, including production strict Clippy, formatting
and architecture; receipt1793270074251489059 has SHA256
`9a57b4e974931510ca04b9ae948411302eaff2ece4336c395332f0ac41f0e89d`.
Private strict Clippy is covered by the identical worker snapshot. Authenticated
proof/fixture reuse,272 harness controls and pinned upstream verification pass
in20.58s; receipt SHA256 is
`bf3974fb5b74891df46df50553a79d134e5c727e6fce8ee430459ade99b2ccb4`.
The first Root reuse adapter failed before executing checks because it assumed
the predecessor's `files` schema. The corrected adapter validates this contract's
actual `cases`, original selection and foundation hashes; the failed receipt
remains retained. Worker oracle, naming, cold-timeout and real endpoint failures
also remain recorded in its authenticated handoff.

Qualified CLI1e9301db17a751f09745e8cf376ee9c9e3669737117f3f5d40173c07283266d4
now passes the original closed-range expansion and stops at
`file45:6972: open, exclusive and alternative pattern types require a wider profile`.
That is an expansion/image offset, not an asserted physical source byte.
The observation takes11.36s with374 opened sources; all722 original input hashes
and target arguments remain unchanged. The same explicit finite profile uses
work14,500,624, attempted1GiB and retained64MiB. Receipt SHA256 is
`14ea7a04abafba720f68c1108449eb7d8115b9ff9d361d42dca67c32bc8a8ac0`.
Strict separately retains the deferred lint-expectation failure at68 opens,
SHA256`f4661ec1bffde1f06a83d59c43efead7427adecbf4e01eb451db51c6c06552a0`.
Neither opened-file counts nor an error observation establish a crate artifact.

Next is the genuine RangeMin/RangeMax/RangeSub declaration and source-call join
for open/exclusive and signed two-arm wrapping forms. Role registration grants
no implementation, const-call or API permission; generated PatTyRange use needs
its authenticated internal desugaring allowance. Actual char predecessor/panic
behavior must come from the source body. Generic endpoints, integer enum niches,
refinement metadata, complete original methods and all core/alloc/std artifacts
remain required. No whole assignment is released by this checkpoint.

The preceding public55dcc81d3 full CI40855585622 is entirely green, including
release/course, native, debug and independent Rust. Full CI for this new scalar
consumer is pending publication; local checks retain their exact finite scope.
