# Associated initializers and call inference checkpoint

Compiler cutoff `17955ead6db1203a312ac28fb77da929bb4526ef` preserves selected
caller aliases in projected constant patterns, uses exact prefix trait context
when coercing call arguments, retains returned function-item targets, and preserves
canonical static initializer results. Optional call hints do not force unresolved
projected inputs. Original declaration, caller, ownership and lifetime proofs
remain required. This is a development checkpoint; PA25–PA30 remain incomplete.

## Qualification and references

The frozen compiler passed 2,429 regular maintainer functions and 242 explicitly
run native functions, strict release all-target/all-feature Clippy, formatting,
the architecture/file audit (1,598 files, no findings), 90 harness tests, registration
of 337 roots in 13 binaries, layout and upstream verification. The default-feature
production executable reused the qualified build with an identical hash.

Of 8,305 existing text cases, 8,302 matched exactly. Three stdout differences
were reviewed individually: PA16's consuming-call two-phase-borrow case renumbers
three regions; PA19's explicit bound-constant case corrects a universal-alias
source offset from 93 to 90; PA21's named-lifetime opaque-result case exchanges
two region IDs. Region row ordering changes with those identities. Acceptance,
stderr and all other existing sidecars are unchanged.

Publication adds 55 active cases: 21 in PA18, 27 in PA19, four in PA21 and three
in PA24, for 8,360 active cases. Nine additional sources remain declared drafts
in planned assignments. All 64 new sources were independently qualified with
pinned Rust. The affected PA16/18/19/21/24 independent source and native suites
passed before reference generation. The multiple-call-inputs positive retains
its explicit code-generation-only contract; it is not reported as a runtime
value comparison. Review preserves 9,273 old sidecars and changes only the three
existing stdout files described above. New oracles match the exact frontend
output used by the independent native checks. The installed tree passes all
8,360 text cases exactly, the 90-test harness, registration, layout and upstream
verification.

Candidate SHA-256:
`e7ff3ee3c7d6882d83e646cc24e31a208fb1d1639052147fe94a94388d1df615`.
Full qualification lives in
`rustgm-validation-work/obj/parallel/associated-call-checkpoint-v2/`.
Generation, review, installation and final installed-tree checks live in
`obj/parallel/associated-call-publication-v2/`. Failed preliminary runs remain
retained separately. Successful validation cleanup reclaimed 17.36 GiB and
losslessly archived three large diagnostics with their original-byte hashes.

## Performance attribution

The immutable public baseline has SHA-256
`1d7fe2a2af3a8f956dc1e928130cb6a22eb6c496a3fdef123464dc1cb921a3e9`.
The frozen study independently qualified 45 inputs: 27 succeed under both
compilers with byte-identical output; 18 newly supported inputs have no baseline
speedup claim. Each shared case retains A/A calibration, five paired ABBA blocks,
three executions per observation, eight RSS samples and both Callgrind profiles.
Measurements held an exclusive RustGM resource lease, which cannot exclude
unrelated host workloads.

Shared instruction deltas range from -0.303% to +0.817%. No shared case crosses
the predeclared instruction or RSS attribution thresholds. The ordinary512
control measures 43,147,499 baseline instructions and 43,016,841 candidate
instructions; candidate median time is 19.39 ms. Its paired wall change is
-0.54%, with 5.78% A/A spread. Other cases have substantially noisier wall
observations; these data do not establish tight wall-time neutrality.

Two wide-argument series trigger the scaling rule. For 8/32/128 arguments,
shared baseline instructions are 3,199,005 / 7,993,804 / 49,557,283 and candidate
instructions are 3,206,766 / 8,022,424 / 49,694,657. The newly supported series
measures 2,859,071 / 6,592,671 / 43,933,618 candidate instructions. These are
superlinear curves, not evidence of whole-compiler linearity.

The single declared attribution pass reads every original triggered profile and
the ordinary512 A/B control; it does not repeat measurements. Self-cost sums
match every profile's instruction total. At every size, both shared binaries and
the newly supported variant make exactly `2*n*n + 4*n` destructor-component
queries and `2*n*n + 3*n` region-point insertions. At width 128 those are 33,280
and 33,152 calls respectively. The cleanup recorder itself runs only `7*n + 15`
times. The recorder's component-query edge costs exactly 17,338,496 instructions
in both shared binaries at width 128.

`borrow/liveness/cleanup.rs` enumerates each live entity at each point and emits
its symbolic destructor components. Repeated parameters of the same generic
type repeat the same component; `borrow/loans.rs` then maps every seed and
`borrow/regions/points.rs` receives the duplicates. These files and
`types/destruction/regions.rs` are byte-identical to the public baseline. This
attributes the scaling to existing entity-by-point work rather than an added
prefix-context algorithm. The existing cost remains a required performance
follow-up; publishing these correctness changes does not claim to repair it.

The algorithmic follow-up should share immutable destructor components and
track live component multiplicities across CFG transfers, emitting each distinct
component/point once. Merely changing hashers or deduplicating after quadratic
enumeration does not remove the cause. Preserve ordered cleanup, storage/move
kills, opaque parameter obligations and the full loan proof. Opposing controls
must include distinct parameters, branches, loops and ordinary512, alongside
the retained 8/32/128 series.

All plans, observations, binary/input hashes, analysis and attribution are in
`obj/parallel/associated-call-performance-v2/`. Results SHA-256:
`2b9393d8960ab4b185573950f40d64300bca7b8a1bfdaca7ce7e6af55e7fd2e4`.
Plan SHA-256:
`9753082bbb5b48e606c3419914a7ddea443cc7a1361bc001fb551e04554abace`.
