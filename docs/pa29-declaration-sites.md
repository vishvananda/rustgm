# Declaration and clause site identity before live migration

The private resolver catalog now retains actual imported declaration ownership
independently of body ownership. A trait or impl Self declaration can own clauses
without any executable body row. ImportedDeclarationId is sealed to its unique
live catalog and exact remapped declaration. DeclarationSite is a separate
checked ordinal from BodySite. Neither diagnostic labels nor equal numeric IDs
in another catalog establish ownership or proof.

Publication checks the complete declaration remap, real source-free declaration
kind and a nonempty ordered partition of declaration site labels. It preflights
owned storage before staging/publishing. Invalid declaration rows cannot publish
bodies, remaps, artifacts or retained bytes. Site lookup is indexed: declaration,
exact validation and site lookup together take exactly four logical visits per
request at widths 8/32/128. Failed lookup/publication work is retained.

This does not yet authenticate an artifact declaration against the real metadata
importer's complete external remap/header. The executable importer must do that
before enabling any header/body use. The catalog remains proof-neutral: it
contains no Checked, Permit, Proved, selected caller or execution permission.
No profile tag or existing payload bytes change.

The measured three-variant candidate has ImportedDeclarationId8,
DeclarationOrigin12, DeclarationSite4 and ProofSite16 bytes. Actual containing
models are Body448→448, Contract184→184, Obligation28→36, LifetimeProof120→128,
canonical Predicate12 unchanged, located Predicate20→28 and builtin use20→28.
Value24 and lazy CatalogSlot8 remain unchanged. Catalog header remains168 and
its cold artifact header grows112→160 for declaration bindings/labels.
These are measured candidates: no live Body/Contract/Predicate field migrated.
The located-predicate growth is material; a shared catalog owner-row identity
with distinct sealed body/declaration wrappers is proposed before broad migration.

Qualification uses the unchanged b6 consumer base plus dd5 binary provenance.
The first strict run retained two wiring failures (unused re-export and a test
semicolon). The corrected frozen run passed format, strict all-target/all-feature
debug Clippy, architecture audit, registration and all 12 exact debug identity
and layout controls. The focused batch built the library test target once; no
full Cargo/native/text sweep or compiler candidate acceptance is claimed.

Evidence is `obj/parallel/pa29-executable-originals/declaration-second/`:
source-map.json, checks.json, batch/receipt.json and layout.log. The independent
executable generic source family is recorded in
[its source contract](pa29-generic-contract-sources.md); real cross-crate execution
remains required after the ownership and proof consumer migration.
