# Testing and references

This checkout owns the maintainer solution. `dev/rustgm-ref` invokes the built
solution, just as reference wrappers do in cppgm-extended's solution checkout.
Default tests compare candidates with **committed text**, not a live solution.

## Oracles

Each manifest-listed `x.rs` has `x.rs.ref` (stdout), `x.rs.ref.stderr` and
`x.rs.ref.exit_status`. Compare bytes exactly; do not strip whitespace, reorder
nodes or normalize diagnostics. PA-specific handouts own their deterministic
format. Errors initially use source byte offsets, not checkout-specific paths.

`tests/manifest.json` independently declares expected success (0) or diagnosed
rejection (1). Exit 86, other statuses, signals and timeouts cannot be accepted
as rejection. There is no keep-going success loophole: collect all failures but
return failure if any fixture fails. Empty suites and missing oracles fail.

```sh
make test-pa1
make -C pa1 test
make test-report
make test RUSTGM_TEST_APP=/absolute/path/to/candidate
make ref-test-pa1
```

Only the explicit `ref-test` targets regenerate sidecars. The generator first
checks every declared outcome and writes no reference if any differs. Review
the complete generated diff. `reference.json` records source and input hashes;
it is provenance, not an alternate expected answer. Never auto-bless in CI.

## Independent evidence

Text generation alone cannot prove the solution correct. Maintain hand-checked
small token vectors and invariant tests; compare full supported programs with
the pinned rustc for acceptance/runtime behavior. `make test-rustc` checks the
imported full-program fixtures independently. Raw lexer/parser fragments are
not automatically valid whole Rust programs and must not be classified that way.
PA2 adds `scripts/check_literals.py`: rustc evaluates numeric bits and textual
bytes and checks expression rejection independently of the decoder. Its output
is compared with both the candidate and committed oracles. Overflow linting is
explicitly allowed for this decoding stage; inference and contextual lints are
later assignments. Stream fragments and upstream programs are classified in the
manifest rather than silently treated as runnable value expressions.

PA3's `check_syntax.py pa3` checks complete programs and evaluates configuration
predicates with the pinned target/profile. Cases may declare `args` as explicit
`--cfg`/value pairs; the harness preserves argument boundaries and records them
in evidence. Reference receipts hash the manifest as well as source fixtures,
so changing configuration or expected outcomes requires an explicit review.

Rustc has no LowIR output. From PA11, maintainer-generated LowIR is the course
text oracle, with independent run-pass checks through CPPGM versus rustc. Do
not grade LLVM IR or assume Rust's unspecified layout matches rustc. Test exact
layouts where guaranteed or where the course contract deliberately specifies
them. Keep runtime evidence separate from the raw LowIR text comparison.

## Placement and provenance

Every `.rs` fixture under an active assignment's tests directory must be in its
manifest. The earliest assignment owning all required features owns the test.
Use feature clusters in `tests/spec/`; unchanged upstream inputs live in
`tests/upstream/PROJECT/`. Never reformat test inputs. `make test-harness` checks
discovery and oracle presence; `make test-upstream` verifies original bytes,
including the pinned Reference under `doc/rust-reference/`.

`third_party/sources.json` pins repository, commit, source path, local path,
checksum, adaptation and applicability. Its license files and notices ship
with an eventual student export. Fetching imports is explicit and is not part
of ordinary tests. Do not copy upstream `.stderr`, `.rast` or `.txt` outputs as
our oracle: their representation and diagnostic contracts belong to upstream.

## Release discipline

PA1–PA3 are implemented drafts, not a claim of full Rust frontend conformance.
Planned PA4–PA38 fail on invocation and are excluded from pass counts. Promote
a stage only after handout/reference/fixtures are coherent, independent checks
pass, the source-to-feature ledger is reviewed and all prior stages remain green.
Keep maintainer measurements and logs in ignored `obj/`; keep student handout
contracts and required fixtures tracked. Student export is not implemented yet.
