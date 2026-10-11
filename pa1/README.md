# RustGM Programming Assignment 1: source and tokens

Status: **implemented draft**. Build the initial lexer and its text adapter.
This is a lexical boundary exercise, not full Rust syntax/semantic acceptance.
The target source profile is Rust 1.98.0, edition 2024.

## Interface

```sh
rustgm --emit-tokens [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Read one source file, or stdin if omitted. Success exits 0. Lexical/input failure
exits 1 and writes `ERROR: OFFSET: MESSAGE` on stderr (encoding failures state
the invalid raw byte offset). Invocation errors exit 2. An unimplemented future
mode exits 86 and never counts as a valid rejection.

## Source and representation

Reject invalid UTF-8. Normalize CRLF once, remove a leading BOM from tokenization
and ignore a qualifying shebang. Do not append a newline, splice lines, process
trigraphs or permit a legacy encoding fallback. Offsets are zero-based bytes in
the normalized file; BOM and shebang bytes occupy offsets even though no tokens
are emitted for them. The end is exclusive. Empty input emits `eof 0`.

One line per token:

```text
identifier 0 2 "fn"
whitespace 2 3 " "
identifier 3 4 "x"
punctuation 4 5 "("
punctuation 5 6 ")"
eof 6
```

The spelling is ASCII quoted bytes: escape quote/backslash as `\"`/`\\`,
newline/carriage-return/tab as `\n`/`\r`/`\t`; every other non-printable or
non-ASCII UTF-8 byte is lower-case `\xNN`. Retain whitespace and comments. Text
rendering is a tool adapter; later compiler phases consume typed tokens.

## Owned behavior

The vocabulary in `dev/src/lex/token.rs` distinguishes ordinary/doc comments,
identifiers/raw identifiers, lifetimes/raw lifetimes, integer/float tokens,
character/byte/string/byte-string/C-string and their raw forms, punctuation and
unknown literal prefixes. Compound punctuation uses maximal munch. Underscore
and keywords are identifier-like at this stage. Nest block comments iteratively.
Raw strings support at most 255 delimiter hashes. Keep suffix spelling inside
literal tokens. Return already emitted complete tokens before a lexical error.

Literals here are **boundaries**, not interpreted values: `0b9`, `0x`, `'ab'`,
unknown suffixes and a lifetime beginning with a digit may tokenize successfully
but must fail the later assignment owning validation. Identifier spelling is
retained; NFC semantic identity is introduced with interning at PA6. Reserved
prefix validation and edition-specific token rejection must be specified before
promoting the corresponding grammar/edition assignments. The current lexer is
the first buildout slice, not a substitute for the complete Rust token grammar.

Unterminated quoted/raw literals or block comments, invalid UTF-8, raw strings
with too many hashes and unknown token-start characters are diagnosed here.
No reference output may depend on filenames or contain fixture recognition.

## Implementation and tests

Use a pull lexer over immutable source with compact offsets. Do not allocate a
String per token, retain the whole token stream, or make per-character records.
`make -C pa1 test` runs every manifest-listed fixture. `make ref-test-pa1`
regenerates text only through the maintainer tool after outcome validation.
Keep `make lint audit-rust test-harness test-upstream test-rustc` green.
`make perf` measures the same lexer without token rendering; retain full-dump
correctness checks separately so throughput optimization cannot omit work.

Before declaring PA1 release-ready, expand the grammar-edge matrix, reconcile
the deferred-invalid-token boundary with PA2/PA3, and independently audit the
reference. The initial checked-in suite establishes the development loop.
