# Renamed unstable library features

PA31 source policy owns old_name syntax and library-feature identity. PA40 owns
its hard request diagnostic; PA29 owns authenticated relationship metadata.
These are required compilation rules, independent of deferred advisory lints.
PA31 remains planned, and original core/alloc/std artifacts remain required.

Contract-v4 freezes 44 programs against Rust 1.98.0 revision
88d9e12ae178fab0fb5cc050a94da85685d449ea. It binds primary attribute parsing,
lib-feature collection and request checking sources. Three genuine fresh rustc
consumers exercise an actual provider rlib: the canonical request succeeds, the
former request fails as renamed, and an ungated use fails. RustGM still requires
complete metadata relationships before it can publish/import that producer.

A former name never enables the current API. Its declaration is valid without a
request, including an empty or otherwise nonidentifier string. A request for a
former name is a hard error, not a warning suppressed by a lint cap. Only the
first unstable declaration supplies a feature's former name; later declarations
still retain their own policy data. Const attributes also define library features.
The packet preserves canonical-name precedence in collisions, duplicate keys,
invalid forms, configuration, nested owners, source-provider mode and ordinary
body execution. CURRENT_RUSTC_VERSION uses the pinned language release rather
than the host used to build RustGM. Conflicting stable/unstable definitions and
inconsistent stable release versions reject.

Feature identity is projected once from actual policy declarations. Implied-by
links and former-name links share the existing paid cold relationship vector,
with an explicit kind and exact owning policy. They have different validation:
implied_by needs a genuine feature definition; old_name creates none. Neither
becomes a global or transitive predicate grant. Root requests are indexed once
when former-name links are present; semantic uses do not rescan alias declarations.
The existing inventory, policy, ordinary body and syntax layouts stay unchanged.
Old metadata profiles reject unencoded relationships.

The candidate table and its binding select 41 local/source-provider source cases;
the three imported reference programs remain required metadata-consumer work.
The native observer calls actual ordinary code and prints 7. The maintainer tests
also check both relationship kinds, prior implication controls, failed growth,
source publication fences and work growth with 16/64/256 declarations/requests.

Earlier contracts and diagnostics remain intact. The v1 native setup omitted the
observer's required public stability annotation; setup-v2 supplies it without
changing the predicted result. The first imported setup used its provider's crate
identity; v3 assigns the fresh consumer a distinct identity. All38 v3 observations
pass. V4 adds six feature-definition consistency controls and passes all44.
Commands, diagnostics, hashes and failed observations remain under
obj/parallel/renamed-features-v1/oracle-v1 through oracle-v4. Candidate text tables
are bound to contract-v4 rather than hand-editing generated text oracles.
