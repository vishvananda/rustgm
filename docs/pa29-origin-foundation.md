# Private source-free ownership foundation

This dormant prerequisite follows the approved source-free design on frontend
`46d8b82a` plus parent port `4eb1a20f`. It adds private resolver ownership and site
records, without migrating Body, Contract, Value, proof, execution or template
fields. No importer invokes it yet. It does not encode generic bodies, reserve a
wire tag, authorize execution or complete PA29. The first executable family in
[the transport design](pa29-source-free-originals.md) remains required.

## Ownership and publication

`resolve::body_origin` keeps source NodeKeys intact. ImportedBodyId is an opaque
8-byte compiler-local epoch/row pair. Checked atomic epoch allocation never wraps
or reuses a retired catalog identity; debug formatting reveals neither component.
Epochs are not serialized, rendered as symbols or obtained from the environment.
Catalog/site lookup rejects foreign or retired epochs and unknown rows/sites.

A lazy CatalogSlot allocates neither a catalog nor an epoch until demanded. No
slot has been added to a live declaration owner in this checkpoint. The resolver
catalog is metadata only; future decoded Template/Contract pairs belong in the
pipeline/body owner, keyed by this identity. Local templates retain their existing
representation; no duplicate Arc layer or proof graph is introduced.

Installation validates a complete dense declaration remap against real resolver
Definition records: valid source-free declarations, compatible named-body kinds,
no reused definition slots. The already validated artifact owner forest supplies
canonical original parent/child identities. Each numbering unit must be the body
itself or its exact parent's unit. Ordered nonempty operation ranges partition the
label table; provider paths/spans are separately bounded and validated. Provider
labels never construct FileId, NodeId or NodeKey.

All checks and scratch allocations precede publication. Failure retains consumed
work but appends no artifact, row or map entry. Retained collection capacity and
owned String capacity are charged; shared immutable Arc pointees follow the
existing retained-allocation exclusion. Conservative map/vector growth and
publication scratch are preflighted under the existing 16,000,000-byte policy.
The default body/declaration/artifact/work quotas reuse existing crate limits.
Queries are indexed and charge visits on successful and failed lookup.

This is not a checked-import seal. Exact defining crate/declaration ordinal to
resolver-ID authentication must come from the actual generic import reservation
producer, not merely node=None or a matching kind. Full header/result/binder/unit
validation, derived operation obligations and ordinary original/instance proof
remain mandatory before any future consumer is enabled. No existing permission,
SourceComputation, Caller, Checked, Proved or Code is produced here.

## Measured layouts and work

All sizes are Linux x86-64 with pinned Rust 1.98. The candidate containing models
first reproduce the actual source-record sizes/alignment, then substitute only
the proposed field type. They are measurements, not migrated production records.

| Record | Existing | Candidate |
| --- | ---: | ---: |
| Body | 448 | 448 |
| Contract | 184 | 184 |
| Obligation | 28 | 32 |
| LifetimeProof | 120 | 128 |
| Value | 24 | 24, unchanged |

ImportedBodyId is 8, BodyOrigin 12, BodySite 4 and ProofSite 12 bytes. The original
proposal's guessed 16-byte ProofSite was conservative. Catalog header is 168,
body row 40, artifact descriptor 112, and lazy slot 8 bytes. Actual later error,
header, Prepared/Checked and Code layouts must be checked when those fields are
migrated; this checkpoint changes none of them.

For 8/32/128 requested bodies independently crossed with 0/128/4096 unrelated
bodies, installation is exactly 28 + 3N logical visits for the frozen provider names
and one-file label profile. Repeated validate/coordinate/site queries cost exactly
4N, retain no additional storage, and do not revisit unrelated declarations.
Failed lookups preserve their work charges. Tests also cover simultaneous epochs,
exhaustion without wrap, retired catalogs, wrong remaps/kinds/source records,
foreign units, malformed ranges/labels, transactional retry and tiny quotas.
These counters describe metadata validation, not original semantic proof work.

## Qualification and provenance

Evidence is retained under `obj/parallel/pa29-origin-foundation/`:

- `first`: fmt and all-target/all-feature release cargo check passed; strict lint
  caught unused private reexports and test-style issues. Failure receipt retained.
- `second`: fmt/strict passed; architecture audit caught the seventh filename
  prefix `body`. Moving the same approved module into body_origin/mod.rs fixed
  placement without changing its typed API or ownership decisions.
- `third`: fmt, 8 new focused debug controls, strict release Clippy on all targets
  and features, architecture audit 1917 files/0 findings and registration passed.
  Debug library build took 98.79s; the 8 controls executed in 0.02s.
- `owners`: the same frozen debug binary ran 6 existing neutral owner-catalog
  controls in 0.04s, with no new Cargo invocation or rebuild.

All commands used the unchanged shared 3-job/resource/storage guard. No full Cargo,
native, performance or active-text suite was repeated for this dormant identity
foundation. Metadata codecs and all existing profile/stored-array bytes are
unchanged by source; their end-to-end tests will be exercised at actual importer
integration. `source-map.json`, `resources.json`, each checks.json and the debug
binary SHA retain the exact source, command, timing and quota receipts. No
assignment activation, generated oracle or student fixture changed.

The final production/test source-map SHA-256 is
`70566b944b77c1552c29de90f66d0cdecde82eb916a7b29dc5830e1446a3700b`.
The reused debug library-test binary SHA-256 is
`36ca4f15aeea0fec22bcc7424a4a84956d18c8d908313a2c946c37ef43d6f9d0`.
Terminal guard receipts are `3663cba561924c4286298cddebf449b4.json`
(new controls/strict/audit/registration) and `f2b5c372f75e4f7a975bf257c4db05a8.json`
(existing owner controls), both status 0/error:null. The final generated-storage
charge is 31,241,551,872 bytes, below the unchanged 50 GiB worker allowance.
