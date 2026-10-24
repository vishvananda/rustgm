# Source-declared `cfg_select`

These PA31 hypotheses reproduce LIB-015, the next observed original-core
expansion blocker. `contract.json` freezes outcomes before observation; a
failed hypothesis must remain visible in a separate review record. No text
oracle or assignment activation is implied.

The fixtures use genuine `no_core` sizing declarations and an attributed
builtin macro. The authored same-spelling control has no builtin attribute.
The original-provider case contains a byte-exact slice of the pinned Rust core
declaration; its source and slice coordinates, hashes and Rust project
MIT/Apache-2.0 attribution are recorded in the contract. See the repository
NOTICE and `third_party/sources.json`.

The contracts cover first-match and fallback selection, complete predicate
validation, braced/unbraced branches, alias and qualified calls, source-token
provenance, configuration inputs and every expansion fragment category.
Unselected branches still require syntax in the invocation category, including
branches after a wildcard. Their names and nested macros are not resolved or
expanded. Width and depth inputs vary parsed branch work and actual selected
expansion separately.

Fragment context comes from the actual invocation. Empty branches are valid
statement fragments, including a brace-delimited block tail; forced expressions
reject them. Opaque captured expressions keep their original category. Raw
tokens forwarded through a parenthesized tail invocation still enter expression
grammar, while a brace-delimited tail or explicit semicolon permits statements.
The additive context contracts and `context-review.json` retain these observations
and the original falsified forwarding hypothesis. A separate native control
observes the selected tail value directly at the C ABI boundary.

Independent qualification uses the pinned Rust compiler for metadata and object
generation. Positive objects link to a C consumer and execute their declared
answer. RustGM must subsequently consume these same source bytes and explicit
provider/configuration inputs; an earlier unsupported-feature diagnostic cannot
qualify a semantic negative. Scope and function-type privacy remain distinct
requirements of general scoped macro execution.
