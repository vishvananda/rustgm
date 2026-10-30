# Forced indirect argument contract

LIB-111 is the current unchanged-core blocker, on the local struct in
`library/core/src/ffi/va_list.rs:416`. This is draft PA30 foreign-representation
and PA31 source-authority work. It neither activates PA30 nor implements the
attribute. `contract.json` freezes all 28 source inputs, five C observers and
predictions before execution; its primary-source intervals include the original
core declaration and pinned parser, layout and x86-64 classifier.

The pinned attribute accepts actual structs under rustc_attrs, has word form and
hard duplicates, and conflicts with repr(transparent) on the same declaration.
An unmarked transparent wrapper inherits its marked payload's argument mode.
Non-Rustic arguments use an ordinary pointer, distinct from SysV stack MEMORY
byval. Return classification and Rust calling conventions remain separate.
The actual caller must preserve value-copy, move, drop, borrow and call-safety
semantics; an inert marker or erased argument cannot qualify this requirement.

Independent qualification matched all 28 metadata outcomes and 27 object
outcomes. Four C-linked native programs passed: exported argument7, transparent
wrapper7, caller-copy16 and register-pressure16. All programs preserve checked
arithmetic and use the real pinned Rust runtime. The supported subset's receipt
is `b67ee04518e01483b1f93ff8f54e52e6a257c043171f3c29232385926961317d`.

`native-return.rs` remains a required positive, with native7 still unqualified.
Metadata succeeds, but pinned Rust object/staticlib generation ICEs at
rustc_ty_utils/src/abi.rs:469: its sanity checker expects indirect mode for the
return while its x86-64 classifier changes arguments only. The original failed
receipt and complete diagnostics are retained. The program is not relabeled
invalid, the expected success is unchanged, and the complete packet is not
claimed to pass. `qualification.json` records this limitation explicitly.

Candidate implementation and every candidate test remain pending. Add the actual
typed representation policy through the existing owner and layout machinery;
keep argument/return cache identities correct, preserve incoming/outgoing object
snapshots and qualify actual C imports, exports, pointers and variadic tails.
Old metadata profiles must reject unencoded ABI intent before publication. A
new authenticated representation capability and full original VaList/runtime
consumers remain required. Existing LowIR pointers and object copies should be
investigated before proposing any backend vocabulary change.
