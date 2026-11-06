# Reborrow source ownership and compact layout repair

Full CI42287493371 passed all13 public test targets and2773 private controls,
but eight private layout assertions found ordinary Declarations had grown from
7120 to7128 bytes. The added word was the lazy Reborrow adjustment-map pointer.
The actual operation already requires completed source language bindings, so
the existing optional source Catalog now owns that same map.

Compile-time opponents independently confirm Declarations7120, source Catalog3768,
Body456, Value24 and syntax Node36. The combined source roots remain10888 bytes.
The eight ordinary size assertions keep their previous limits; six source-only
Catalog assertions follow the moved word. Map allocation, row growth, arena and
actual role/operation checks retain their existing identity and prepayment.
Body-local pending goals and checked CFG/constant inventories keep their own
existing owners. No map copy or new registry is introduced.

Six exact public controls pass in237.86s, selected strict Clippy/format/audit
included. Five subsequent private size-expectation edits leave their production
and selected test inputs identical. One CI-driven all-target/all-feature strict
metadata batch passes in113.04s and checks the actual fixed array-size opponents;
it neither links nor executes the full private harness. One native test/two
executables pass in34.94s and preserve `reborrow 23 23 47 47 0 99 31` and
`successor 17 23 19 31 23 59 1`. Every local batch remains below five minutes.

[Machine evidence](reborrow-source-owner-repair.json) binds exact source hashes,
commands, observations and the failed CI log. Full successor qualification is
pending CI. There is no new core observation for this ownership-only change;
the latest immutable original-core frontier remains FieldRepresentingType/LIB186.
Field evidence, projections, offsets, wider Reborrow/transport and original
core/alloc/std artifacts remain required. This checkpoint completes no whole PA.
