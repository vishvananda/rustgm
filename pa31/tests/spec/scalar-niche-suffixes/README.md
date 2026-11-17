# Scalar niche suffixes

These authored source-core reductions exercise the zero-based integer interval
used by core's CodePointInner and the nested Option<bool> storage used by alloc.
The copied no_core setup retains the attribution from bool-niches; see root
NOTICE and third_party/sources.json. Every input and declared result is in
outcomes.json, written before independent or candidate execution.

The bounded range reserves 0x110000, while nested bool enums reserve 2, 3 and 4
in order. An exhausted u8 suffix forces a physical outer tag. Invalid payloads
remain invalid; pointer provenance and permission are separate from these scalar
encodings. Nonzero-start partial intervals and char niches retain their existing
wider ABI boundary. The three accepted programs must print 41 59 via observe.c.
