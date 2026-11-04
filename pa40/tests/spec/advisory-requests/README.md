# Documentation and optional provenance requests

This unpublished PA40 packet recognizes `intra_doc_pointers` and
`strict_provenance_lints` without granting language, unsafe, trait, float or ABI
permissions. The active `contract-v2.json` and `cases-v2.tsv` bind36 original
fixtures in the 2021 and 2024 editions, their declared outcomes, primary-source
hashes and a C observer. These course-authored fixtures are not upstream copies.

The pinned compiler uses intra_doc_pointers in Rustdoc's raw-pointer link
resolution. Ordinary Rust compilation accepts the same doc strings without the
feature and does not resolve them into program bindings. Rustdoc link checking,
feature-sensitive rendering and metadata transport remain separate requirements.

The provenance feature enables an optional lint whose default is Allow. It does
not change the legality or execution of exposed-provenance pointer/integer casts.
Ordinary and explicit Allow profiles preserve those operations. Under an explicit
Allow cap, standard allow/warn/deny/forbid/expect syntax is admitted through the
existing policy parser; unsupported expectations remain witnessed deferred debt,
not fulfilled producer events. Existing unsafe and primitive feature checks remain.

Strict explicit provenance warn/deny/forbid/expect profiles stop at a missing
producer rather than silently accept an unenforced policy. Required Rust successes
for warn and expect remain positive unfinished cases; their current failures are
not reclassified as invalid Rust. Exact producer events, diagnostic rendering,
expectation fulfillment and their imported transport remain required. This packet
is not whole PA40 completion or an original core/alloc/std artifact.

The C observer calls ordinary code and performs an actual pointer-to-integer-to-
pointer round trip followed by a valid unsafe read. Both annotated and attribute-
free programs must print `7 9`; candidate LowIR must preserve the same execution.
The independently declared native profile has panic=abort and debug_assertions=false,
matching bring-up. The initial v1 metadata predictions all passed; its default
checked object required Rust's panic runtime when linked directly to the C
observer. Version2 records the matching native options explicitly. It changes no
fixture bytes or required metadata outcomes. Only version2 is active.
