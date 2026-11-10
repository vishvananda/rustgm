# Overflow check query

Five complete sources and the C observer are frozen before implementation.
Pinned Rust1.98 accepts four and rejects the wrong-result declaration under both
-Coverflow-checks=yes and no. The oldc52c555a CLI reaches the original-core
unsupported constant-intrinsic diagnostic in direct/inherited/native; the
ordinary same-name control accepts and the malformed declaration rejects.

The actual intrinsic returns true in CTFE under both independent flag profiles.
At runtime its result follows the generation policy. RustGM currently has a
fixed checked arithmetic profile, so the native observer must print `1 1 1\n`.
Independent enabled checks give the same; disabled checks give `1 0 0\n`, proving
that CTFE and runtime are separate decisions. The provided false body cannot
substitute for either intrinsic result. The inherited word supplies no call,
const, source transport or unsafe permission. Runtime reification and imported
policy transport remain independently proved requirements.

The first malformed-result probe forced evaluation and produced a reference
compiler ICE. That failed probe remains unqualified under obj; the frozen
opponent declares the invalid signature without evaluating it and gives ordinary
E0308 under both independent profiles. PA31 owns the typed intrinsic identity;
no earlier ordinary same-name source reproduces the blocker. No grading
activation or text oracle is generated.
