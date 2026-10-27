# Source lookup and target-policy integration

The combined compiler retains the compact lint storage repair and integrates
both authenticated worker handoffs: real source string/slice methods plus
ordinary primitive-member lookup, and typed target-feature call policy. The
source merges are6e1f10a3c/d45dc83ce. No backend operation or source body export
permission was added by this integration.

Original core opens372 files with all722 pinned inputs unchanged, then rejects
actual formatting-argument construction in write!/format_args! (file55). The
combined observation takes9.25s and has no infrastructure failure. Its stderr
is byte-identical to the preceding compact-witness observation. Neither run
produces a core artifact or establishes an A/B timing comparison.

## Combined qualification

- Nineteen exact private controls pass with private strict Clippy, formatting
  and architecture in264.29s. They cover source methods/lookup, all six target
  policy controls, SIMD/copy, assembly safety/proof, source comparison fences
  and complete compact witnesses.
- Twenty-six exact production/integration/private controls pass in196.15s,
  including production/CLI strict Clippy, four numeric-frontier controls,
  ownership/lexer and the independent projection selection suite. All237 PA6
  and120 PA8 text cases pass in that same gate.
- The separate finishing batch passes all1,386 PA20 cases in8.79s, all236
  harness tests/383 registrations/44-stage layout in11.99s and upstream pins.
- Both focused gates and the immutable CLI bind the same2,966 source fingerprints.
  Existing isolated native source-length and SIMD-copy results remain their
  original observations; this checkpoint does not claim a new native sweep.

The immutable combined CLI SHA-256 is
`ebd0d4d09f53b2bde0d216be5e5d041ae7f4039ee97786eee3219bb520c781ff`.
Its binding SHA-256 is
`ba6d9244bbaffdb6824821c00ff8d493b4c3f7e34910848956d957ffc8cf6753`.
Root evidence lives in `obj/parallel/source-lookup-target-integration-v1/`.

## Reviewed CI diagnostic references

Full CI40559487420 failed six PA20 stderr comparisons; all1,942 private tests
and the reported integration suites passed, including the independent owning
projection suite. The three source cases, each in semantics and LowIR modes,
still reject as required. Two errors now use the actual parenthesized Group
start rather than its inner Field (178→177 and173→172); the third detects
unresolved implementation choice at the P annotation144 rather than field169.
Associated output types still cannot choose an implementation header.

The source-level explanation is the qualified numeric scheduling/publication
path, not a replay-proved introducing commit. The current-source independent
owner suite and four numeric controls pass before generation. The integrator
predeclared and checked the six exact new diagnostics, then used the existing
scoped writer. Only those six stderr sidecars change: all other4,152 prior
PA20 sidecars, fixture bytes, outcomes and stdout remain unchanged. Provenance:
`pa20/reference-batches/1793127580938715181-578d434e3fdd.json`.
The old CI failure remains a failure; fresh full qualification is still required.

## Next exits

B now owns genuine two-argument Alignment formatting, exact source Arguments
constructors and temporary lifetimes, then PanicInfo/Location/handler→Abort.
C owns the real128-bit numeric SIMD/XMM operand consumer of the existing type
and body/target-feature owners, retaining backend and metadata fences. Root owns
integration and finite component publication; unsafe-places27.1–27.3 is the
first candidate, requiring unit-aware shared manifest/provenance selection and
current release checks. No whole PA25–PA30 is marked complete by these merges.

## Receipt bindings

- private19: `6694ed3bd3c4402818f4b209eb05a3ede52cb89c939f372c437a2dd9f74749db`.
- production26: `4a6668f157f90f99a1a0b21ad1f3e6bb6a0322e81c60d3ab2c5201b6d382e9a8`.
- finishing checks: `61535f1099623c627f882f7895beca456d2130af9eea8ab27a4f9c006c944bef`.
- unchanged core: `aa7a023f5144b4ca8308b6e8909c355cfa0c86f4d26e4b5b7145c85540a4926a`.
