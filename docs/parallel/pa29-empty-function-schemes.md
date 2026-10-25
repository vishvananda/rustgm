# Completed empty source function schemes

Source-only checkpoint on public `7a40e005`; qualification is pending.

Declaration completion now retains an actual zero lifetime scheme for Core and
Client sessions selected by the typed `Package::uses_source_bindings()` policy.
LocalMacros and legacy sessions keep their existing empty elision. The producer
checks the actual definition/source key, binder owner/count, original and
normalized function types, captures and bounds before returning a private owned
completion record. `SignatureFinish` inserts it before publishing the function.
It installs no new parent/body binder and does not invent a defining environment.

The existing declaration work counter pays the insertion attempt and existing
entry rehash work only when actual map capacity must grow. A byte preflight
includes the current map and conservative replacement-table storage before
allocation, against the existing metadata byte cap. There is no new persistent
ledger or hot field. Zero schemes own empty vectors; their cold map entries and
table growth still have real storage costs, bounded by completed function count.

The exporter still requires the retained scheme; a removed entry fails rather
than becoming checked empty. Source headers preserve the actual presence or
absence of their defining environment. Existing nonempty and unused lifetime
domains retain the full scheme producer. This record is declaration data, not a
body, region, loan, const-call, attribute-absence or imported-default permission.
All current Functions guards, profiles, wire formats and parser limits remain.

Four proposed controls check actual Core zero domains, Client/codec/fresh proof,
removed/foreign-owner completion and failed-work attempts, plus 1/8/32 entry
growth with 1000 warm reads. They compare the legacy and LocalMacros map extent
with the ordinary baseline. The existing Functions fresh-client and nonempty
source-header scheme controls complete the six-test selection. Provider/client
test helpers are shared with the existing codec control without changing its
proof path. No tests have run on this checkpoint yet.
