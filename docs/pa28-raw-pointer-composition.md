# Actual source raw pointer composition

PA28 supplies the operation and validity semantics; PA31 supplies the actual
source binding. This checkpoint does not activate either assignment or qualify
original core/alloc/std. Preserve NOTICE and the pinned Rust source attribution.

The appended `SourceIntrinsic::AggregateRawPtr` recognizes the actual safe const
bodyless Rust declaration with its three owned Sized type parameters and exact
`(D, M) -> P` signature. Ordinary source `BuiltinDeref` and Pointee associated
bounds remain ordinary proof obligations. There is no BuiltinDeref language
item, library factory or replacement slice/UB body. A transient 16-byte selection
retains the exact instance, declaration, signature and result. Local source
incarnation, actual caller Code and completed header/Pointee owners authenticate
execution. Imported source operations and reified intrinsic calls remain closed.
Existing SourceIntrinsic ordinals and metadata profiles are unchanged.

Both argument values are evaluated once, in data/metadata order, including unit
metadata. Data must have an actual thin raw-pointer ABI. Output supports actual
Thin/unit or Length/usize metadata through the existing ABI metadata owner,
Constant::with_metadata and the existing 16-byte runtime wide-pointer producer.
Neither pointer width nor a source-associated-type claim can turn VTable into
Length. Dynamic/unknown metadata still requires its real typed value producer.
Thin data may point to Foreign; public ptr::from_raw_parts nevertheless retains
its independent implicit Sized bound on impl Thin. Direct intrinsic Foreign data
and public-helper Foreign output controls preserve that distinction.

Raw construction and extraction do not inspect initialization, alignment,
liveness, physical bounds or UTF-8. Null, numeric, expired, out-of-bounds and
uninitialized raw pointers retain their metadata without acquiring access
permission. Raw Slice reference formation now shares the existing sequence-word
reconstruction and validity owner with Str. Actual reads/publication and mutable
access keep their own authority. The valid native companion observes argument
order and erased-unit effects through external C callbacks; it does not execute
invalid access or establish the full original slice wrapper graph.

Associated result defaults reuse decoded bounds and the actual source sizing
roles. Direct MetaSized/PointeeSized suppress implicit Sized; explicit Sized,
aliases, ordinary same-name traits and where clauses retain their distinctions.
`?Sized` supplies the existing actual MetaSized default when appropriate. There
is no pointer-specific sizing exception, new cache or ordinary body field.

## Independent observations and scope

Inputs and predictions were committed before observation in
`pa31/tests/spec/pointer-composition`. There are 48 independent commands: the
initial 22 metadata cases plus object/link/execute, four followups, two original
source-profile cases, two corrected profile cases and 15 associated-sizing
cases. No observation was an infrastructure failure. The following original
prediction disagreements remain retained rather than rewritten:

- The public helper rejects Foreign data because its impl Thin is implicitly
  Sized. Direct-intrinsic Foreign data and Foreign output followups pass.
- Rust accepts a discarded reference to uninitialized u8 storage; this remains a
  required capability control, not a language rejection invented by this work.
- Rust accepts the published byte255 Str reference in the frozen profile. The
  candidate accepts that reference; this change adds no UTF-8 read or rejection.
- The original source-profile positive had invalid explicit tuple Copy setup.
  The corrected profile removes that setup; original diagnostics remain raw.

The corrected source profile and all 15 sizing opponents match their independent
predictions. The valid original runtime observer prints `3 12 7 34` followed by a
newline, with empty stderr and status zero. Candidate native evidence uses actual
RustGM typed source/LowIR, the pinned CPPGM object backend and the same observer;
no backend source or contract is changed.

## Qualification and required followups

Final private11 at fast/1793133802937245605 passes in 127.40 seconds, including
private strict Clippy, format and global architecture checks. Earlier private15
at fast/1793132301106360713 qualified the associated sizing, generic/alias defaults,
existing pointer-metadata and raw-slice guards. Later exact access/incarnation/
reification/scaling groups also pass. Production/CLI strict Clippy and 21 exact lexer/sizing/alias tests pass in
122.38 seconds at fast/1793133978226852878, with an identical source fingerprint.
The final observer reuses the exact qualified private binary, verifies all 24
pinned fixtures and 126 Reference files, and repeats object/link/native execution
successfully after ABI-kind hardening. Native stdout is exactly `3 12 7 34\n`,
with empty stderr. Inputs, tool hashes and actual generated object/executable
hashes are retained; the disposable generated native files are pruned.
Failed audit, lint and compilation receipts remain retained.

The printed negative controls diagnose null provenance, physical bounds, expired
storage and static-write authority. The uninitialized-read and packed/unaligned
opponents currently stop at `constant range requires array or slice storage`.
They establish rejection at an unsupported backing profile, not completed
initialization/alignment validation of that original scalar/union/packed input.
Those exact backing and validity capabilities remain required.

The source ledger is cumulative charged bytes/work, not current retained memory.
Kind/Node/Operation remain 24/36/16 bytes; Body remains 456, SourceIntrinsic 1 and
the transient selection 16. For repeated mandatory calls:

| Constants | CTFE steps | Started frames | Peak slots | Code bytes | Template bytes | Cumulative source ledger bytes / steps |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 228 | 6 | 25 | 2,632 | 10,460 | 53,028 / 3,394 |
| 8 | 1,740 | 48 | 25 | 8,036 | 30,648 | 54,428 / 4,619 |
| 32 | 6,924 | 192 | 25 | 26,564 | 99,864 | 59,228 / 8,819 |

These counters bound changed-path work; they make no allocator or wall-time
speedup claim. The ordinary control retains no source catalog. The existing ABI
metadata owner retains completed shared tails; successful actual footprint
queries prepare it before the paid composition consumer. A zero query budget
fails, and a VTable result cannot use either supported metadata representation.

These are language-level raw-reference reducers. The complete unchanged original
slice::from_raw_parts and UB-check macro bodies are retained as source data.
Executing that full wrapper graph still requires B's genuine Arguments::from_str,
panic_nounwind_fmt, const_eval_select, PanicInfo, Location and handler closure.
The checkpoint supplies no stripped UB path or panic stub. Broader backing,
discarded uninitialized-reference and byte-read parity, scalar/union/packed raw
sequence backing, dynamic metadata values, imported/wire
operation transport and original core/alloc/std remain required followups.
