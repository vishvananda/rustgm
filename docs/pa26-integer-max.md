# PA26 integer primitive MAX checkpoint

This bounded checkpoint implements the twelve integer primitives' inherent `MAX`
values on the Linux x86-64 target. It does not activate PA26. Float MAX and other primitive constants and legacy numeric-module constants remain required
library work, alongside the remaining structural comparisons and language hooks.
The later [char checkpoint](pa26-char-constants.md) qualifies MIN/MAX; historical
float/char boundary observations below remain retained.

## Typed value ownership

`sema::primitive_constants::Uses` is a separate lazy owner. Each source use retains
its typed prefix, a private primitive/member descriptor and the canonical
`ConstId`. The producer accepts the actual normalized primitive type, never a
source spelling or a nominal with the same name. A repeated selection must match
before publication; failed publication does not intern a value or mutate the map.
Readonly consumers independently check the prefix and canonical typed bits.
Integer widths come from the existing target primitive model, including 64-bit
usize/isize. No host integer size, initializer, synthetic function, source AST,
DefId or public method permission is fabricated.

Associated prefixes use the ordinary declaration scheduler. Applied aliases
finish their existing scoped normalization and retain its proof before selecting
the value. Alias arguments keep ordinary arity/type checks; the constant itself
has no arguments. The `core::primitive` and `std::primitive` modules bind aliases
to existing primitive identities, preserving lexical shadowing. Expansion can
poll imports before primitive indexing: the module remains pending then and is
published only when those identities exist. It is installed once in the actual
canonical library scope, in deterministic primitive order.

Expression inference, constant queries and scalar patterns use the same selected
value. Runtime IR reuses `Rvalue::ConstArgument`; direct/range patterns use the
existing typed query/coverage worklist. Borrowed constants enter the ordinary
temporary-place and promotion paths. A mutable borrow creates independent
storage, without changing subsequent MAX uses. No constant-call or function
pointer CTFE eligibility changes.

## Lookup precedence and independent evidence

The frozen hypotheses and pinned Rust 1.98.0/2024 observations live under
`obj/parallel/pa26/primitive-constants/declared-v{1,2,3,4}`. The committed draft
`pa26/tests/primitive-constant-outcomes.json` retains exact source hashes, mode,
independent outcomes and explicit native metadata. Its 62 sources cover all
integer widths; aliases and qualified uses; primitive modules and shadowing;
borrow/promotion; patterns, const generics and inline constants; independent
Self/trait precedence; and rejected type/argument/call/place combinations.
The two Rust-positive float/char sources remain explicit required gaps.

A preexisting wrong-code case is preserved unchanged:
`impl-self-constant-precedence.rs` has SHA-256
`e622c759e559c4dd2267f74bff9686b26b7318b52bf53f9efd860cf7d1b9dcc7`.
Rust chooses the inherent primitive MAX for concrete `Self::MAX` even when the
local trait implementation declares its own `MAX=7`. The immutable pre-feature
binary accepted the program but returned 7; the locked backend executable aborts
134 against the source's value check, while Rust executes 0. Explicit
`<Self as Trait>::MAX` and a generic trait default's `Self::MAX` retain value 7.
The original v2 baseline-status prediction was wrong; its original receipt is
retained with a separate corrected receipt and actual native evidence in
`fourth/`. Float/char inherent MAX also must not fall through to a user trait
constant while their actual values remain unsupported.

## Layout, scaling and qualification

The private descriptor is one byte, the selected record 16 bytes and its lazy
HashMap owner 48 bytes. The optional owner adds one eight-byte pointer: matched debug type information
measures Declarations at 8,408 bytes before and 8,416 after. NodeFacts remains 80 bytes; no IR value, operand, temporary, call
argument or callable registry is widened. Ordinary sources allocate no Uses
owner. At widths 8/32/128, the private source controls retain exactly 8/32/128
records, one canonical MAX value and 31 definition slots; inference work is
29/101/389 steps. No primitive method bodies are demanded by these controls.

The frozen `performance-inputs-v1` matrix separates shared ordinary/literal
controls from candidate-only MAX and unrelated-source scaling at 8/32/128.
Independent qualification and output comparisons are retained by the closing
batch. These logical work bounds and preflights are not timing claims.

The source matrix passes 48 supported positives and 12 independently rejected
cases, retaining two explicit float/char gaps. Forty-six native harness programs
pass paired Rust/backend value checks; final exact-source execution additionally
covers all 48 positives, including both no-prelude forms. Four private invariants,
strict all-target/all-feature release Clippy, architecture audit (1,620 files,
zero findings), layout, grouped discovery and upstream checks pass.

All 8,360 active PA1–24 outcomes match, with 8,357 exact output triplets. The only
three differences are byte-identical to the frozen ordering base: its PA6 names
append and two PA21 hidden-equality diagnostics. MAX adds no active text delta.
The six shared scaling outputs are exact; six candidate-only inputs qualify.
Detailed results and hashes are recorded in the final handoff receipt. Full Cargo
publication and immutable performance integration remain the integrator's work;
no references are regenerated by this checkpoint. Earlier failed build, private
layout expectation, test formatting and filename-ownership checks remain in the
numbered preparation receipts rather than being overwritten.
