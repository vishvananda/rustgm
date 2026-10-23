These are exact frozen inputs from the PA27 string storage investigation. The
four historical original/check forms are unchanged. The mutable conversion
reduction follows the real cast/reborrow body of Rust 1.98.0 (commit
88d9e12ae178fab0fb5cc050a94da85685d449ea),
`library/core/src/str/converts.rs:208-213` (MIT OR Apache-2.0).
The shared cast reduction isolates the same storage identity without pretending
to implement the original `transmute` source binding.

`string-word-outcomes.json` records declared outcomes and source hashes. Bare
raw Str storage has initialized U8 validity, separate from library UTF8 contracts;
see `docs/pa27-string-byte-words.md` for the normative source links and the retained
falsified eager-UTF8 hypotheses. Only explicitly native-valid rows make execution
claims. The original empty E0745 input and the distinct corrected local-cell
input remain separate sources.

The two original panic-macro sources remain required successful inputs. Their
current unresolved ordinary source-library macro dependency is not a negative
language contract; helper reductions do not count as original-source acceptance.
