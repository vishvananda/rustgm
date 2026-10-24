# Source formatting builtin contracts

These are preimplementation hypotheses for LIB-017, not observations or candidate
support. `declaration-contract.json` and `literal-contract.json` freeze separate
milestones. Both use actual source providers and independently check metadata,
object generation, then link and execute the actual object with a C consumer.
The admitted physical editions are 2021 and 2024.

The declaration group retains the pinned original `format_args`,
`const_format_args`, and `format_args_nl` header slices, plus explicit role,
attribute, target and diagnostic opponents. Three Rust-positive identity/scope
cases are deliberately outside the initial candidate subset. Declaration-only
roles must fail before invocation/capture/mark work and cannot authenticate a
Report allowance frame or obtain foreign export permission.

The literal group declares a genuine `format_arguments` language owner and its
ordinary source `from_str` function. RustGM must select and check that actual
call, its visibility, lifetime and const eligibility. C observes the returned
string's bytes through a thin pointer; the provider does not simulate formatting
or return an invented compiler layout. Raw/escaped/newline/empty values and a
captured expression are distinct cases. No `no_core` string-library methods,
opaque runtime shim, FFI fat aggregate, disabled checks or fake panic symbol are
needed. Full nonempty formatting/argument construction remains required later.

The pinned original source is Rust commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, MIT OR Apache-2.0; inherited
attribution and provenance remain in `third_party/sources.json`. Source/slice
hashes and exact starting lines are in the declaration contract. Never edit a
frozen hypothesis in response to an observation: retain disagreements and add a
separately declared correction when a provider is incomplete.

The first frozen independent run executed 140 of 172 planned rows and 18 of 34
native programs. All declaration-language outcomes matched; unknown-role text
uses `built-in`, so its four textual predicates differ. Eight literal positives
stop at missing Freeze in their const source constructor, and their 16 native
programs remain unrun. The original sources/expectations/results are unchanged.
`independent-review.json` binds this disagreement; `literal-provider-contract.json`
predeclares distinct variants with the exact already-qualified Destruct/Freeze
source declarations and required feature requests. Their outcomes are unobserved.

The additive complete-provider run has all 48 language checks matched, genuine
unmasked constructor negatives and 14 native successes. The raw source's actual
newline disagreed with its initial four-byte consumer; both failures remain.
A separate consumer-only continuation links the same two object files and
successfully compares the actual three bytes `61 0a 62`. It repeats no language
checks. `observations.json` records the independently observed milestone inputs
and the corrected native-byte contract, while preserving all original hypotheses.
