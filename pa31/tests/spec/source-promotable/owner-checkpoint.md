# Direct const-policy owner regression

Full renamed-feature CI41742788707 exposed an older test diagnostic expectation.
The original regular-fallback.rs declares promoted_value both unstable (word)
and stable (the other functions). Rust correctly rejects this conflict before
RustGM's later promotable-owner check. The original file, contract, hash and
negative outcome remain unchanged.

regular-fallback-owner.rs changes only word's unstable feature name to a distinct
promoted_unstable_value. It still lacks direct rustc_const_stable/unstable and
still must fail. This retains the intended guard against treating inherited or
regular API policy as a direct const-policy witness. The maintainer test now
checks both the original earlier conflict and the new actual owner boundary.
No negative case became accepted and no expected error check was removed.

owner-contract-v1.json freezes both source hashes and rejection predictions.
The pinned Rust 1.98.0 reference reports E0711 for the original conflict and E0717
for the new missing paired const attribute. The independent receipt is
obj/parallel/default-bodies-v1/promotion-owner-oracle/receipt.json, SHA-256
299b894274bfae46c451ba53ff710ac527085b4a698c5606c67049f8053af166.
This is harness coverage repair, not a relaxation of feature consistency or
promotion permission. The original promotion execution/borrow requirements and
all other source-promotable contracts remain required.
