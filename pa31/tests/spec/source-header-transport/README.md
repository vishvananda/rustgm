# Source-language header transport hypotheses

These are independently declared preparation inputs, not an activated suite.
`contract.json` retains initial, revised and corrected hypotheses and their
source/upstream-slice hashes. Earlier mismatches remain evidence; their expected
hypothesis values are not silently rewritten as a language contract.

The authoritative current positive controls have `-qualified` provider names.
Both retain the real full Clone declaration and provided clone_from body, along
with real sizing/Copy/Destruct/Freeze/receiver declarations and explicit user
primitive impls. The semantic cutout explicitly removes API stability attributes;
the paired original-attributes provider preserves their actual attachment.
The signature-only control omits clone_from and is expressly not full upstream
Clone header coverage.

Independent qualification is recorded under
`obj/parallel/pa29-source-headers/independent{,-second,-third}`. The third run has
ten matching Rust outcomes, including six native consumers returning7 and two
unsized E0277 controls. The earlier errors include missing feature requests and
roles, const_destruct, and an explicitly corrected attribute assembly. No target
compiler is used to implement parsing or semantics.

See [the interface review](../../../../docs/pa29-source-header-interface.md).
