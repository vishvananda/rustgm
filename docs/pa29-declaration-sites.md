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

## Compact common-owner correction

The measured16-byte candidate above is preserved in commit53d72269 and the
`declaration-second` receipt. The approved correction now stores one vector of
`OwnerRow::{Declaration,Body}`, with distinct sealed eight-byte declaration and
body handles over the same catalog-local row identity. Lookup validates the row
kind before any body/declaration API can consume it. Common imported ProofSite
locations do not grant either body identity or proof authority. BodySite and
DeclarationSite constructors remain distinct and validate the exact owner.

The enum measures40 bytes, exactly the existing body-row payload size; its
separate declaration payload measures24. No duplicate declaration binding vector
is retained. The cold Artifact header is136 bytes, Catalog168 and Slot8. ProofSite
returns to12 bytes. Actual candidate located predicates and builtin uses are
20→24, Obligation28→32 and LifetimeProof120→128; Body448, Contract184, canonical
Predicate12 and Value24 stay unchanged. No live proof/body field migrated yet.

At8/32/128 declaration+body pairs the actual owned vector/map/string payloads
(including the catalog header, excluding shared immutable identity pointees and
allocator metadata) are1971/5259/18411 bytes. Conservative retained charges are
4147/13843/52627 bytes and cover these measured capacities. Complete publication
charges five visits per pair plus fixed28 for the frozen fixture; warm indexed
lookup remains four visits per request independent of unrelated prior rows.

`owner-rows-second` passed strict all-target/all-feature debug Clippy, the audit,
registration and12 focused controls. Its thirteenth test stopped in setup because
`typed-rows` is an invalid crate disambiguator, before exercising authority. The
only correction was that private fixture spelling to `typed_rows`;
`owner-rows-third` reran the exact authority control successfully. Production
bytes and the12 passing controls were unchanged. The frozen binary also ran the
layout/retained-payload observers with uncaptured output, retaining its binary
hash and originating receipt in `frozen-layout-provenance.json`. No compiler
candidate, full Cargo/native/text sweep or wire profile is claimed.

## Permanently frozen shared metadata

The resolver's already cold Imported record owns CatalogSlot. After complete
installation it permanently freezes the catalog before publication. Semantic
Linkage shares the same immutable metadata-only Rc when one exists; no original
Template, Contract or proof payload enters resolve. No-import/default slots
allocate neither Rc nor catalog, and old closed imports leave the slot empty.

Freeze is a retained catalog state, not a conclusion from Rc uniqueness. Every
installation entry checks it before touching incoming data. Dropping all shared
borrowers, obtaining unique mutable Rc access, or repeating freeze never permits
reinstallation. Failed attempts retain observer work. Partial/unfrozen catalogs
cannot be shared. Ordinary exact epoch/row/kind validation still rejects foreign
catalogs with numerically equal rows and definitions.

Measured Catalog remains168 bytes and CatalogSlot8. Resolver Imported changes
136→144 and semantic Linkage224→232; both live only in preexisting optional/cold
owners. Value remains24. CatalogSlot charges its16-byte Rc control block once
before allocation, in addition to its actual catalog/header/storage budget.
Standalone catalog tests do not pay an unallocated shared header.

`frozen-catalog-first` stopped at the existing import_crates function's102-line
Clippy limit. A natural Imported constructor now initializes its owned cold state
without extending the import driver. The coherent `frozen-catalog-second` run
passed format, strict all-target/all-feature debug Clippy, architecture audit,
registration and one debug batch of17 exact private tests plus two unchanged
scalar-import semantic contracts. The library/group targets were each built
once; no full maintainer/native/text sweep ran. Its frozen binary also records
the actual cold/hot layouts in layout.log. Real source-free original validation
and executable generic import remain the next live consumers.

## Bounds-carrying body locations

Imported BodyOrigin now contains a sealed ImportedBodyOrigin: the checked body
ID plus its actual nonzero site count. Only a checked catalog row can mint it.
Composing BodySite with this handle checks the actual bounds in constant work;
it does not require a catalog pointer on every body. An authorizing original
lookup still revalidates current catalog epoch, body row kind, exact expected
owner and the unchanged site count. A stale retained location can describe its
old site, but cannot validate or resolve in a replacement catalog. Distinct
sealed declaration constructors remain unchanged.

The count is not encoded as count+1. Every executable body has a real root
site, including an empty source function with its actual CFG block; a declaration
without a body has no body row. The focused empty-function control exercises the
ordinary source pipeline and body builder before checking the one-site imported
partition. The future executable importer must check actual body contents and
site assignments as well as the catalog; this dormant metadata remains no proof
or execution permission.

Actual layout observations remain ImportedBodyId8, ImportedBodyOrigin12,
BodyOrigin12 and ProofSite12, with candidate Body448 and Contract184 unchanged.
Value24, canonical Predicate12, OwnerRow40, Catalog168, Artifact136 and Slot8
remain unchanged. No new allocation or retained owner row was introduced.

The first closing guard passed format and all-target/all-feature strict Clippy,
then stopped on the filename-stem audit because imported_sites.rs was a seventh
imported-prefixed file. Renaming only that private test to bounded_sites.rs
preserved behavior. The corrected guard passed audit, registration and all19
exact private cases in one debug library target, plus the uncaptured containing
layout observation. It covers real empty functions, altered bounds, foreign and
retired catalogs, exact current validation, quotas, cold slots and constant-work
composition. Evidence: obj/parallel/pa29-executable-originals/bounded-origin-first
and bounded-origin-second (source maps, checks, batch receipt and layout.log).
No wire tag or executable generic import has been enabled by this checkpoint.
