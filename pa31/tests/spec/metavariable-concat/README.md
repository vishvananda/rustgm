# Metavariable concatenation frontier

LIB-039 owns the pinned `${concat(...)}` template operation at core's unchanged
`fmt/num.rs:149`. PA7 owns its template representation, PA8 identifier emission
and hygiene, and PA31 the independent `macro_metavar_expr_concat` feature gate.
This feature is independent of `macro_metavar_expr`; accepting the declaration
alone does not establish generated-name resolution.

The positive source invokes the macro and resolves the generated function. The
opponent removes only its feature request. Both pinned editions accept the first
and reject the second with E0658. The historical baseline rejects both at template parsing with
`expected macro token`. The current implementation checks and resolves the
positive source and rejects the missing-feature opponent. Original predictions,
input hashes and six raw observations remain in the contract and evidence.
No runtime or original-library artifact is claimed. The complete template must
remain valid before later cfg processing; skipping unknown tokens is not a fix.

`protocol/contract.json` freezes twenty additional sources and forty independent
library-metadata observations across editions 2021/2024. Every predicted status
matches. Candidate checks now exercise these grammar and lookup boundaries;
these metadata observations do not establish native execution. Accepted controls
cover cooked string pieces, original integer spelling, repetition, forwarded
literal fragments, and matching definition-site local names. Opponents cover raw
identifiers/strings, suffixed and token-tree integers, expression fragments,
arity/separators, unknown or still-repeating variables, escaped literal symbols,
and caller-site local-name access. An unused enabled definition still requires
the feature; a cfg-removed declaration does not. Source bytes and the predeclared
expectations remain bound by hashes.


The integrated compiler at `4ec518b8f` passes six private concat groups within
80 selected macro/semantic controls, including all eleven positive sources in
both editions. Typed literals retain their original spelling, including integers
larger than u128; creating an identifier does not require evaluating that number.
Unicode results are NFC-normalized and checked as identifiers. Widths 8/32/128
and the emitted-byte budget rejection are checked. Macro Node/Kind/Piece/
Transcription/Definition/Environment remain 40/32/20/32/72/64 bytes respectively.

The three allowance sources preserve six independent observations: the pinned
parser requires the actual crate feature even for generated definitions carrying
`allow_internal_unstable`. The original two incorrect positive predictions remain
recorded. Five further literal/Unicode/scoped sources add ten matching independent
metadata observations. Other metavariable operations remain unimplemented.

Qualified CLI `884e0004` passes the original concat boundary and opens 226
unchanged core files, then stops at `slice/cmp.rs:218`, byte 8583, on
`impl for<'a> [const] FnOnce(...)`. All 722 pinned inputs verify. This is a parsing
frontier; no complete core artifact or additional PA release is claimed.
Evidence: integrator `obj/parallel/concat-implementation-v1/`, private receipt
`1793081016518908131` (80 tests, 165.50s), production receipt
`1793081227333269893` (26 tests, 28.63s). The core result takes 11.01s including
first acquisition of the immutable source tree. The ordinary macro A/A+ABBA
control retains 54 byte-equal observations: paired changes +3.17%, +5.78%, -0.48%
at widths 128/512/2048, within measured A/A spreads 16.66%/11.91%/5.96%.
No speedup or regression can be established from those noisy timing changes.
