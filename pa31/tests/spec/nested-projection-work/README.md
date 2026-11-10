# Nested projected-size work

These authored reductions require source-defined sizing roles, owned by PA31.
`P<T>` stores `<T as Tr>::Out`, and the blanket `Tr` implementation returns T.
All eight inputs are accepted by rustc 1.98.0 in edition 2024 and by RustGM.
`outcomes.json` lists every source and its frozen SHA-256. The generic reductions
need only the three sizing roles; closed reductions also need Clone, Copy and
LegacyReceiver for rustc's concrete-type queries. No upstream fixture bytes or
checksums are changed.

Generic depths 12 and 16 retain the former repeated-proof regression; depths 31
and 32 guard the fixed path beyond it. Closed 12/31 check concrete applications.
The source's language result stays acceptance; work and depth budgets are tested
separately by private live budget/preload controls. A formerly predicted nominal
depth 32 rejection was disproved: after reuse these sources need no deep recursive
proof stack and remain accepted. The actual nested-owner guard remains 93.

Private invariants assert exactly d normalizations through 31 in ordinary and
placeholder contexts, for both sized and unsized results, exact nested-work
conservation and fixed hot layouts. The measured work envelopes through 31 are `80*d*d` for true and `180*d*d` for
false results; the
original affine total-work target remains an explicit optimization follow-up.
PA31 is planned: this packet is not a graded assignment release or a text oracle.
