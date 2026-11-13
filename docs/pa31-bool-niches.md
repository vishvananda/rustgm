# Bool niches in source enums

Core's `slice/cmp.rs:52` transmutes `ControlFlow<bool>` to `u8`. Release
candidate11 rejects it because its nullable carriers all exclude zero.
`Continue(())` is instead encoded as byte2 by pinned Rust; `Break(false)` and
`Break(true)` use bytes0 and1. The zero-sized companion rule alone cannot
provide that representation.

The existing physical carrier now also describes bool, including its unused
value2. An explicit source-policy flag admits that carrier for source enums and
validated imported source field groups. The historical course policy does not
admit it, and standalone transparent-wrapper ABI queries keep their existing
profile. The physical cache remains independent of those permissions; both
query orders are checked.

Every consumer reads the carrier's actual empty value: constant byte encoding
and decoding, typed transmutability, runtime construction and variant tests,
destructor selection, static initialization and zero-data classification.
The enum travels as an `i8`; loading an `i1` would discard its companion value.
The ordinary bool payload retains its validity checks. Const reads of byte3
still fail as invalid bool values. A static companion is initialized to2 rather
than being mistaken for zero-filled storage.

This reuses the paid nullable layout transaction and carrier cache. It adds no
backend vocabulary, alternate frontend, resource-policy change or compiler-limit
increase. The policy flag is in the existing sparse enum policy; canonical type,
body and layout record sizes stay unchanged.

## Declared contracts

The [PA31 packet](../pa31/tests/spec/bool-niches/README.md) freezes every Rust/C
input hash and prior expected outcome. Twenty-two independent Rust outcomes
include21 active cases (17accept/4reject) and one retained positive for a second
niche in an already niche-encoded payload. That broader scalar-niche contract
remains required in `pending.json`; it is not reclassified as a rejection.

| Family | Contract |
| --- | --- |
| ControlFlow | size1, byte0/1/2 encoding, byte0/1/2 decoding |
| Option | size1, Some(false)=0, Some(true)=1, None=2 |
| Invalid values | byte3 rejects in both enum families with invalid bool representation |
| Size opponents | an incorrect size2 assertion and transmute to u16 reject |
| Shapes | reverse variant order and transparent bool payload qualify; repr(u8) stays explicitly tagged |
| Runtime | construction/matching, initialized globals and actual non-Copy payload destruction |

Rust metadata agrees with the frozen declarations. The three independently
linked Rust/C programs print `0 1 2 0 1 2`, `2 0 2 0`, and
`0:0 1:0 2:1 2:1`: the last program separately counts destruction of false and
true payloads and skips the destructor for both companion values. Its trace
also checks the bool passed to the real callback. The source `drop_glue`
declaration is the genuine compiler
hook from pinned `core/ptr/mod.rs:821`, whose actual body is replaced by compiler
glue in both implementations; the user destructor retains its real callback.

Raw declarations and malformed setup attempts remain under
`obj/parallel/zst-niche/`. Batch1 lacked bool Clone/Copy and produced four delayed
rustc ICEs. The added cleanup control in batch3 lacked mutable LegacyReceiver;
its prefix failed with E0658. The static getter's metadata accepted, but object
emission required the genuine drop_glue declaration, added in batch5. None of
those setup failures qualifies a language rejection. Unchanged successful
independent checks are retained rather than rerun.

## Qualification

- Focused integration receipt1794561838421695979 passes11 exact tests over two
  targets in110.70s, with formatting, architecture and strict selected
  production/CLI/test Clippy. All21 active packet outcomes type as declared;
  accepted cases also lower. The Option, enum-byte, zero-sized companion and
  UnsafeCell controls pass.
- Native receipt1794562065413043594 passes two exact ignored groups in37.37s:
  all three new programs match independent Rust/C outputs, and four existing
  cell/pointer/cleanup programs still execute correctly through pinned CPPGM.
- Private receipt1794562152284043406 passes18 exact layout, byte transport,
  reader, scalar-range, ABI and cleanup controls in156.71s. Strict private Clippy
  links the harness in the same Cargo build. The new bool-cache test preserves
  the course layout whether source or course is queried first.

Before publication, the destructor observer was strengthened to include a false
payload, which a zero-niche test could otherwise skip. Batch6 declares its exact
new Rust/C bytes and stdout before execution. Independent Rust/C passes. Fresh
receipt1794562814860019382 rechecks all21 API/CLI cases, and native
receipt1794562983528985835 passes all three current executable observers in35.63s.
The four unchanged cell programs and private checks retain their earlier scoped
passing evidence. Production hashes are identical; only the integration test's
expected stdout and the declared cleanup inputs changed. Managed CLIf042c27d
still matches the full text candidate byte for byte.

The source-specific integration root now lives beside `cell_sources` in the
frontend suite so it can reuse the existing FFI helper. It is registered once;
no helper or test root is duplicated in the memory suite.

Earlier compile/format/registration failures are retained as failed receipts.
Frozen exploratory dev-fast CLIf042c27d binds3945 source hashes and its tracked
patch before that test-only change. All9307 active course cases pass in73.58s
with identical existing references and no generation. Upstream verification,
315 Python tests,485 integration roots and44-stage assignment layout pass;
upstream was repeated after strengthening the authored fixture. The retained
nested-niche positive still rejects with incompatible types under this CLI;
its Rust-positive contract remains required.

Clean source8950ad99c freezes release candidate12, CLI
`bca8b9e1a83122a0441f95d6a4b7cca3fa95eb93ef418122234316132cdf466b`, with3945
Rust hashes and an empty patch. The unchanged120s original observer runs
exclusively after load admission and finishes its body pass in23.29s. Actual
`as_underlying`7631 clears. All42310 declarations pass;35120 item attempts check
25852 bodies, with two failures and four blocked bodies remaining, all in the
borrowed-closure pattern methods. No previously passing body fails. Postcheck
verifies722 source hashes,375 opened sources, six configuration bindings, all
compiler sources and unchanged profile/limits. The completion pass is not run.

Forty-eight exclusive observations compare immutable release11/12 on frozen
accepting widths8/32/128 and an ordinary control, using A/A calibration and two
ABBA blocks. Complete outputs agree. Wall ratios are1.009/1.030/.937/.918,
against A/A.956/1.012/1.102/.914; CPU ratios are1.009/1.034/.937/.900. Short
process timings make no speedup, neutrality or whole-core throughput claim.
Raw observations remain under `obj/parallel/zst-niche/measure-b/` and
`obj/parallel/capture-match/original-candidate12*`.
No whole-library artifact or full assignment release is inferred.
