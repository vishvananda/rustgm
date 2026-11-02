# Forced indirect argument checkpoint

Source `1244ebfa2` implements the local argument ABI contract that blocked
original core at ffi/va_list.rs:416 (LIB-111). It validates the actual struct,
word form, duplicate and representation rules and the defining rustc_attrs
feature or macro allowance. The policy is retained in typed nominal
representation; recognizing the attribute does not grant call or body permission.

The existing optional aggregate cache memoizes transparent-wrapper policy once
per completed layout node. Ordinary declarations with no marked family skip
this query and allocate no cache. Argument classification retains the original
nominal type ID; normal return classification keeps its independent existing
cache entry. Repr20, Definition36, Plans88 and Passing16 remain unchanged.
No Node, Body or backend vocabulary field was added.

Forced arguments use a real pointer to a fresh owned snapshot. Incoming pointers
copy into the callee's actual local storage. Existing moves, Copy, Drop, loan,
unsafe and constant eligibility checks run before lowering. Physical signatures
containing only scalars and pointers use ordinary LowIR scalar signatures;
typed source calling conventions still select argument conversion. Returns keep
their normal register/object classification. Older named-record metadata rejects
the unencoded policy before publication.

## Qualification

All compiler gates bind the same 3,298 source/tool fingerprints and remain below
the 300-second development budget, including admission.

| Gate | Result | Receipt SHA-256 |
| --- | --- | --- |
| Private, Clippy and static checks | 13 exact controls, including all 28 frozen source outcomes; 135.81s | `335f0930ea4fe8854e6d9f8866272f0930245c17e3037a5c4596b8e7908075ff` |
| Native | Five original C observers and four freshly built pinned Rust libraries; 34.90s | `2138bbd20d1d2b7cb210633495037befea3861a969fcacffdfb53882d17b177d` |
| Production and text | 21 exact controls, PA12 125 and PA18 502; 227.17s | `557a38d0e4bdc24d33c865a8a01443b76f8cc0f0f05a10db33eff0f720f6c987` |

Fastcheck receipts are under obj/qualification/fast/1793625570819171011,
1793625721881601990 and 1793625767909862667. The native test is persistent and
runs in the existing ignored/native CI lane. Exports, transparent wrappers and
normal returns execute 7; caller-copy and register-pressure programs execute 16.
Fresh pinned Rust matches all four supported programs. Its original marked-return
codegen ICE remains a reference limitation: the unchanged positive now passes
the independent C observer with the candidate, without becoming a Rust rejection.

Shared transparent-chain widths 8/32/128 pay work 41/161/641 and retained bytes
1648/5872/22768. Each new node is summarized once; repeated signatures cost one
lookup with zero retained growth. Flat ordinary classification stays cold. Failed
work/storage admission publishes no policy cache. Existing composite DAG, memory,
nullable, ordinary argument and transparency/privacy controls pass. Initial failed
layout and signature tests remain retained; the common record expansion was fixed
instead of relaxing its assertion.

## Original core and remaining work

The new immutable CLI is
`4e18000fbc3315c69eb7fb02822311c921cdfbb094a8c062e09ed366faa0eeea`.
The unchanged original-core types attempt clears LIB-111 and reaches
`rustc_skip_during_method_dispatch(array, boxed_slice)` on IntoIterator,
library/core/src/iter/traits/collect.rs:280 (LIB-112), file172 byte8793.
It takes 23.80s with 375 loading opens. All 722 original source hashes, target,
cap Allow and finite budgets are unchanged. Core receipt
obj/parallel/forced-indirect-abi-v1/core-candidate-v1/receipt.json has SHA
`e978c0ef6f171e8a3d04bb672430b509fd3f8f03bff43133156440cee6a2978e`.

Authenticated representation transport remains required, including reached
nominal/body/callable policies through broader metadata graphs. Wider forced
callback/variadic/packed/large-value and actual VaList runtime consumers still
need their owning independent contracts. Continue source checking at LIB-112,
then complete semantic checking, real core metadata/object generation and an
independent core consumer. No original core, alloc or std artifact, additional
whole assignment or complete PA30 release is established by this checkpoint.
