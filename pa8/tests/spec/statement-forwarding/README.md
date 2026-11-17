# Statement expansion forwarding

This authored reduction isolates the first std dependency-graph expansion
failure after guest editions: `std_detect/src/detect/macros.rs` line 23 emits
semicolon-terminated `cfg!` calls through an unsemicolonized recursive macro
call. The outer invocation is a statement. Its generated raw final invocation
retains that statement category.

The manifest declares six acceptances and four syntax rejections before running
either compiler. Forced initializers, parenthesized expressions, ordinary
parenthesized block tails and opaque captured expressions keep expression
grammar. A brace-delimited block tail retains its existing statement category.
The stable target-feature query uses PA8's existing builtin configuration path;
no unstable target query or source-provider feature is needed.

`outcomes.json` is the independent source contract. The assignment manifest owns
all ten complete programs. New expanded-text references must be generated and
reviewed only after the independent source outcomes and focused implementation
checks pass. This packet establishes no library artifact or timing claim.
