# Writes through unique references of immutable bindings

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/impl-region-methods-20261107`, after the captured-method
checkpoint `5b8a645d0`, based on main `0311a54ea`. It is a transport slice,
not a library milestone, and makes no claim about an original-core artifact
or a whole assignment.

It fixes gap 6 under version 13, as integrator decision 2 requires. rustc
admits `*c.n = v` and `&mut *c.n` when an immutable binding `c` owns a
`&mut` field. Provider validation instead required every dereference's parent
projection to be writable, so a provider stopped at "source Store writes
through an immutable dereference". The captured-method checkpoint's
`Cell::set` had to take `mut self` to avoid it.

## Wire rule (version 13)

There is no new tag, column or row shape. The rule is in spec.md's
version-13 unique-path paragraph.

- A path is unique when it is a local, a temporary, a field of a unique
  path or a mutable dereference of a unique path.
- A place is writable when it is a mutable local, a field of a writable
  place or a mutable dereference of a unique path.
- Older profiles keep main's rule, a writable parent projection under every
  dereference, and its diagnostics.

No write proof travels. Each consumer recomputes write access over the
rebuilt places, and its own borrow checker proves every write and loan.

## Implementation

`port_plan::evaluation_ports` keeps one `Access { writable, unique }` per
projection instead of one writability flag. A field inherits both flags from
its base. A dereference is writable and unique when it is mutable and its
base path is unique (version 13) or its parent projection is writable (older
profiles). Stores and mutable Borrows read the same table. The scratch
charge counts the two flags per projection.

## Forgeries

`forged_unique_write_rows_reject_in_fresh_consumers` uses a generated minimal
provider: the lifetime-nominal provider, `Count<'a> { z: u8, n: &'a mut u8
}` and `poke(c: Count<'_>) { *c.n = 42; }`. The leading `z` field makes
`poke`'s projection rows unique; that amendment was recorded before the
control first ran.

| Mutation | Fresh consumer |
| --- | --- |
| the Store retargeted from `*c.n` to the immutable owning field `c.n` | "source Store writes through an immutable dereference" |
| the dereference row claims a shared dereference | "source dereference changed its referent, lifetime or mutability" |
| relabelled version 12 | "unknown source declaration kind" |
| every strict prefix | fails to decode |

## Evidence

The fixtures are in `pa29/tests/spec/unique-writes/`. Their outcomes were
declared at 2026-11-07T23:59:04Z, before the oracle and the candidate ran.
They are in `pa29/tests/unique-write-outcomes.json`. The integration tests
are in `crate_linking::unique_writes`.

- **Oracle.** In the single rustc 1.98.0 round, the provider compiles and all
  six accepted clients exit 42: bump, after, lend, set, put and deep.
  shared-path and field-write fail with E0594. All as declared.
- **Candidate.** All eight outcomes are as declared. shared-path fails with
  "cannot assign through a shared reference" and field-write with "cannot
  assign through an immutable local".
- **Native.** All six accepted clients exit 42 natively, as rustc's do.
- **Older profiles.** `Own { r: &'static mut u8 }` with `poke(o: Own) { *o.r
  = 42; }` on the source-nominals provider is rejected under versions 12 and
  10 with main's "source operation omitted a required nontrivial formation
  input". It exports under version 13, which the baseline rejects with
  "source Store writes through an immutable dereference".
- **Scaling.** Packets are 43,118, 59,012 and 122,588 bytes for 6, 24 and 96
  writing functions. That is 883 bytes per function, exactly linear. The
  client LowIR is 860 bytes at every width.
- **Byte identity.** The baseline is main `0311a54ea` (pin sha `0f5f95ee`).
  The candidate is sha `9e2a78be`, which includes the captured-method
  checkpoint. They were compared over every pa29 spec provider under
  versions 10, 12 and 13, each with every fresh consumer in its directory.
  That covers 3,266 files, including all 21 earlier version-13 packets. The
  3,185 files outside the two new packets' version-13 runs are identical.
  The 67 that differ are those runs: 45 for the captured-method packet and
  22 for this one, whose provider the baseline rejects at `Cell::set` with
  "source function early/captured callable frame transport is not
  implemented". Under versions 10 and 12 both binaries reject both new
  providers with the same bytes.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `unique_writes` integration (4), `lifetime_methods` (4), `lifetime_transport` (4) and the neighbouring transport tests listed in the captured-method checkpoint | 69/69 | `1794096308637045189` |
| native `--ignored`: this packet, captured methods, lifetime nominals, inherent methods, source nominals, constructors, enum match, str-never, normalized results | 9/9 | `1794096662297467348` |
| native `--ignored`: body nodes, cold, guards, branches, fn roles, spelled roles, direct calls, instantiable | 8/8 | `1794096699730959997` |
| private selection (39), as in the captured-method checkpoint, including the input-plan tests over `port_plan` | 39/39 | `1794096464064517719` |
| PA1-PA12 | pass | `1794096817979323964` |
| PA13-PA19, PA21-PA24 | pass | `1794096875430469766` |
| PA20 | 1,451/1,451 | `1794096945537918803` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/impl-regions/t3-test-upstream.log` |
| `make test-harness` | 304 script tests; 464 roots in 13 binaries, 44 stages | `obj/parallel/impl-regions/t3-test-harness.log` |

All rows ran on the final code. `Ports` is per-original validation scratch;
its projection table grows from one to two bytes per projection and is
charged through `size_of`. No layout invariant covers it.

The rustc 1.98.0 receipt is `obj/parallel/impl-regions/t3-oracle1.log`. The
identity receipt is `obj/parallel/impl-regions/identity3/receipt.txt`. The
pinned binaries are `obj/parallel/impl-regions/{baseline,candidate3}/`.
