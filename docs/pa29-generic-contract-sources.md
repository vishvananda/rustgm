# Executable generic crate transport source contract

The independently declared matrix is `pa29/tests/generic-contract-outcomes.json`.
It contains 18 compile scenarios over 17 unchanged source files: two distinct
provider artifacts from identical bytes, a transitive relay, seven executable
clients and eight rejecting programs/libraries. The required family includes
complete unused generic slots, private helpers, shared reborrows, const calls,
const-generic arrays, real consumer trait implementations and cross-artifact
identity collisions. No RustGM acceptance, metadata tag or execution permission
is claimed by this source inventory.

Pinned Rust 1.98.0 compiled all 18 scenarios with their declared outcomes and
error codes; all seven positive clients executed successfully. Commands use
edition 2024, x86_64-unknown-linux-gnu, panic abort and checked overflow, with
real rlibs and the complete declared dependency graph. The two provider builds
have the same crate name and different metadata disambiguators.

Retained independent receipts:
`obj/parallel/pa29-executable-originals/rust-matrix/receipts.json`, SHA-256
`ed75c5b10589442b69941f71782d356fe1502e2039d6779343d5d9d17f1a36c2`.
The receipts bind the pre-probe outcome manifest SHA-256
`7daff1303cf81676525d8f3916e6c53cbe89a8970f7b1d312005d5e6efe74056`,
every original source hash, exact compiler command, status and diagnostic hash.
The bounded Rust-only guard completed successfully; it ran no RustGM compiler.

The implementation must import typed originals and reprove their operation,
trait, lifetime, ownership and const obligations before demanded instantiation.
An artifact cannot transport Checked, Permit, Proved, caller handles or success
flags. Existing scalar/record/storage metadata profiles keep their bytes. The
source-free ownership prerequisite and consumer migration are documented in
`docs/pa29-source-free-originals.md`; the executable matrix remains pending until
actual cross-crate candidate and native execution qualify it.
