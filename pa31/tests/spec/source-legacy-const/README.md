# Local legacy const-generic declarations

This phase validates the actual `rustc_legacy_const_generics` free-function
contract: rustc_attrs (or real macro allowance), a nonempty unsuffixed integer
index list, const-only generic formals, matching index count and original call
arity. Source values are retained in their original order, without inventing
uniqueness or sortedness. Named source identity and actual typed function
signature/owned const parameters are joined before the contract is completed.

The marker is not const eligibility, safety, API permission or execution.
Same-crate calls retain ordinary const-generic syntax and are NEVER rewritten;
local aliases and function pointers receive no legacy calling privilege.
Explicit generic arguments likewise select the ordinary call path.

The pinned compiler only rewrites a directly resolved external function path
without explicit generic arguments. Actual external argument extraction,
anonymous-constant ownership/evaluation and metadata index transport remain
required. Existing metadata exports fence the real source word rather than lose
that contract. `pa29/tests/legacy-const-call-outcomes.json` preserves required
Rust-positive external calls and their source hashes; it is not a language
negative or a fake imported provider.

`contract.json` freezes independent outcomes before execution. The ordinary
module-provider fixture preserves the qualified NoCore/stable API foundation;
its marked function has a genuine identity body. It tests declaration/transport
fences, not full original stdarch execution. Whole original source/runtime and
all wider const-generic/imported consumers remain requirements.

The independent pinned Rust successor matched all 91 metadata/object outcomes,
including the required cross-crate graph. The first run matched 85/87: its elided
reference prediction was wrong because Rust introduces an outer lifetime generic.
That source and failed prediction remain recorded. The typed declaration check
rejects outer lifetime binders while retaining static references and nested
higher-ranked function-pointer arguments. Two additional attribute-envelope
controls independently match all four metadata/object outcomes: a generated
`#[$meta]` uses its authentic macro envelope, while a forwarded whole item
retains its caller's gate requirement.

Joined candidate qualification passes40 focused controls, strict Clippy and
six affected text suites. Local runtime/const/unsafe calls and a generated meta
attribute each produce7 through the locked backend and independent Rust. Header80,
Node36 and Body456 retain their widths; the cold Marker is136 bytes and shared
Promises232. Width8/32/128 costs216/864/3456 work and12320/49184/196640 bytes.
The external legacy protocol and all broader assignment requirements remain open.
