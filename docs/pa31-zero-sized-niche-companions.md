# Zero-sized niche companions

Core's `io/error/repr_bitpacked.rs:350` asserts that `Result<(), Repr>` is eight
bytes. `Repr` is a transparent `NonNull<()>` wrapper with a `PhantomData` field.
Rust places `Ok(())` in the null niche. Release candidate10 instead uses a tagged
layout: its source nullable policy requires a fieldless variant beside one
single-field variant. A unit field does not satisfy that policy.

The source policy now also admits two single-field variants. For each concrete
instantiation, layout tries the declared orientation, then the reverse. One field
must be zero-sized, with alignment no greater than the other field's alignment;
the other must supply an existing pointer or zero-excluded unsigned carrier.
The retained niche records the chosen empty and payload variants. Constant reads,
construction and static initialization use those actual identities and preserve
the companion's zero-sized field. The typed-header importer re-derives the same
eligibility from validated field groups; no policy bytes are transported.

This extends the existing carrier cache and paid layout transaction. It adds no
frontend, backend contract, resource-policy change or compiler-limit increase.
Course-mode nullable eligibility is unchanged.

Tradeoffs and limits:

- A two-field policy that finds no niche keeps the historical `I64` fallback
  tag. `Result<usize, Repr>` remains a tagged aggregate with an out-pointer ABI.
- A companion aligned more strictly than its payload remains tagged. The
  eight-byte-aligned `[u64; 0]` companion beside `NonNull<u8>` does qualify.
- A wider scalar-range payload retains the existing explicit unsupported-profile
  error, as it does for `Option`.
- The declared orientation wins when it qualifies; only two single-field variants
  permit the reverse orientation. `Result<Repr, ()>` therefore shares this rule.
- `bool`'s nonzero niche is separate work. General scalar/multifield niches,
  smaller default enum tags and completed library artifacts remain required.

## Declared contract

PA31 owns these source-core inputs. The
[packet](../pa31/tests/spec/zero-sized-niche-companions/README.md) preserves exact
batch5 bytes and hashes. Each outcome was declared before either compiler ran.
Rust 1.98.0 agrees with all eleven declarations.

| Case | Contract | Rust | Candidate10 | Current |
| --- | --- | --- | --- | --- |
| `result-unit-repr` | unit beside transparent pointer wrapper, size 8 | accept | reject | accept |
| `result-unit-nonnull` | unit beside NonNull, size 8 | accept | reject | accept |
| `result-repr-unit` | reverse orientation, size 8 | accept | reject | accept |
| `result-phantom-nonnull` | PhantomData companion, size 8 | accept | reject | accept |
| `result-unit-reference` | reference carrier, size 8 | accept | reject | accept |
| `result-aligned-zst` | equally aligned zero-sized array, size 8 | accept | reject | accept |
| `result-unit-match` | const matching chooses both actual variants | accept | accept | accept |
| `result-unit-transmute` | empty variant encodes as a zero word | accept | reject | accept |
| `option-repr` | existing fieldless Option carrier | accept | accept | accept |
| `result-usize-repr` | nonzero-sized companion keeps size 16 | accept | accept | accept |
| `result-unit-repr-wrong` | incorrect size 16 assertion | E0308 | wrongly accepts | rejects |

Batches1–4 retain their malformed-prefix evidence: unknown `pattern_type_macro`,
missing `legacy_receiver`/`clone_fn`/`destruct`, and the pinned source requirement
`Destruct: PointeeSized`. They are not language rejections of the final inputs.

## Qualification

The resumed combined fastcheck `1794558954797138354` passed formatting,
architecture, selected strict Clippy, private strict Clippy and all fifteen exact
private tests. Its private build took 200.54s; the complete run exhausted its
five-minute budget in the integration lane and remains a failed receipt.

Integration receipt `1794559238481312566` then exposed a test setup error: the ABI
test looked up unmangled export names without declaring `no_mangle`. The fixture
layout test passed. The test now explicitly names those four exports; production
code and packet bytes did not change, so the fifteen private results remain
applicable. Fresh receipt `1794559299273190847` passes ten exact integration tests
across two targets, with formatting, architecture and strict selected Clippy.

Every accepted packet case types and lowers. Separate ABI assertions check that
construction in either orientation returns `Ptr`, matching consumes `Ptr`, and
the ordinary tagged control still uses an out-pointer. These are typed LowIR
checks; no new native execution is claimed. The selected Option, enum-byte and
UnsafeCell controls also pass. Raw evidence is in `obj/parallel/zst-niche/`.

Frozen dev-fast CLI `52e35fa7` passes every active PA1–PA24 text case:
9,307/9,307 in 74.46s, with no oracle generation or reference edits. The receipt
also preserves unchanged hashes of all 18,614 stderr/exit sidecars. Upstream
verification, all 315 Python harness tests, 485 integration-root registrations and
the 44-stage assignment layout check pass.

Clean source `34a28702e` freezes release candidate11, CLI
`58d7312a59bcceeecccd65fdcef163b535b0e94c5658d0cabbf31bb80fb45e51`,
with 3,945 Rust source hashes and an empty patch. The unchanged 120s original
observer runs exclusively after load admission and finishes its body pass in
24.56s. Initializer25396, the actual `repr_bitpacked.rs:350` assertion, clears.
All 42,310 declarations pass; 35,120 item attempts check 25,851 bodies, with three
failures and four blocked bodies remaining. No previously passing body fails.
The completion pass is not run and no original library artifact exists.
Postcheck verifies all 722 input hashes, 375 opened sources, six configuration
bindings, all compiler source hashes and unchanged limits/profile. Raw observation,
comparison and postcheck are under `obj/parallel/capture-match/original-candidate11*`.

Forty-eight exclusive accepting observations compare immutable release10/11:
A/A calibration plus two ABBA blocks for widths8/32/128 and an ordinary control.
Complete outputs agree. Wall ratios are 1.083/1.003/1.001/1.015, against A/A
0.924/1.024/1.171/1.036; CPU ratios are 1.085/1.003/1.000/1.016. These short
observations make no speedup, neutrality or whole-core throughput claim.
Raw data and frozen inputs are in `obj/parallel/zst-niche/measure-a2/`. The first
attempt used library emission and stopped on candidate10's existing source-header
transport fence, before any candidate timing. It is retained separately; the
qualified inputs instead have an empty main and identical executable output.
