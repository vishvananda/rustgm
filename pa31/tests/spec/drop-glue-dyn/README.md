These maintainer fixtures retain source-declared `drop_glue`, Drop/Destruct,
Freeze and Unsize/CoerceUnsized identities. The provider declarations are reduced
from Rust 1.98.0 revision `88d9e12ae178fab0fb5cc050a94da85685d449ea`, including
the actual helper contract in `library/core/src/ptr/mod.rs`. They retain the
same attribution and MIT/Apache-2.0 source pin as the adjacent `drop-glue/`
fixtures and [source-helper review](../../../../docs/pa31-source-drop-glue.md).
User-defined record types, observations and C clients are maintainer controls.

`contract.json` records immutable bytes and independently declared outcomes.
The original dyn source is unchanged. Rust constructs every wide reference;
the C clients pass valid live thin storage and do not assume a C fat-pointer ABI.
The method-before-fields case tests the complete concrete cleanup entry, rather
than a direct call to Drop::drop. Null entries are distinct from absent proof.

The original more-general reified pointer is a language rejection (E0308).
The separately qualified fixed `'static` object-bound source is a positive.
The historical provider without any actual DropGlue declaration is retained in
the independent evidence: metadata accepts, object generation requires the lang
item. That observation does not ban ordinary same-spelled functions.

These fixtures do not qualify Box ownership/deallocation order, unsized tails,
CTFE cleanup, general DispatchFromDyn, or full upstream Drop/Pin/default methods.
PA31 remains planned; this directory does not activate an assignment.
