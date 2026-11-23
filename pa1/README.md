# RustGM Programming Assignment 1 — Source and tokens

Implement a lexer for the Rust 1.98.0 source profile, edition 2024. It reads
source bytes and emits tokens with their original spelling and byte spans.
Later assignments interpret the tokens as Rust syntax and literal values.
The pinned Rust Reference describes [input format](../doc/rust-reference/src/input-format.md)
and [tokens](../doc/rust-reference/src/tokens.md).

## Command-line interface

```text
rustgm --emit-tokens [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Read a single source file, or read standard input when `SOURCE` is omitted or
is `-`. Write token records to standard output by default. With `-o OUTPUT`,
write them to that file.

| Result | Exit status | Diagnostic on standard error |
| --- | --- | --- |
| Success | 0 | Empty |
| Lexical error | 1 | `ERROR: OFFSET: MESSAGE` |
| Invalid UTF-8 | 1 | `ERROR: invalid UTF-8 at byte OFFSET` |
| Other input or output error | 1 | `ERROR: MESSAGE` |
| Invalid invocation | 2 | `USAGE: MESSAGE` |
| Unimplemented mode | 86 | Implementation is required |

Diagnostics end with a newline. The invalid UTF-8 offset is in the original
input bytes. Lexical errors use the normalized source offsets described below.
On a lexical error, retain the complete token records emitted before the error
and omit the EOF record.

## Source and token records

Validate UTF-8, then normalize each CRLF pair to LF in one pass. Keep lone CR
bytes. Tokenization skips a leading UTF-8 BOM and a qualifying shebang. After
the optional BOM, an initial `#!` starts a shebang if there is no next token
after whitespace and ordinary comments, or if that token differs from `[`. Skip the
shebang through the end of its text and leave its LF for tokenization.

Spans use zero-based byte offsets in the normalized source. The end is
exclusive. The normalized offsets include the skipped BOM and shebang bytes.
Do not append a final newline, splice lines, interpret trigraphs, or fall back
to a legacy encoding.

Write one line per token in the form `KIND START END "SPELLING"`. On success,
finish with `eof LENGTH`, where LENGTH is the normalized byte length. Empty
input produces `eof 0` followed by LF. `fn x()` without a final newline produces these records.

```text
identifier 0 2 "fn"
whitespace 2 3 " "
identifier 3 4 "x"
punctuation 4 5 "("
punctuation 5 6 ")"
eof 6
```

Quote spellings as ASCII bytes. Use `\"`, `\\`, `\n`, `\r`, and `\t` for
quote, backslash, LF, CR, and tab. Encode every other nonprintable byte and
every byte above ASCII as lowercase `\xNN`. Retain whitespace and comments.

## Lexical boundaries

Distinguish ordinary and documentation comments, identifiers and raw
identifiers, lifetimes and raw lifetimes, integer and float tokens, character
and byte literals, string, byte-string and C-string tokens and their raw forms,
punctuation, and unknown literal prefixes. The expected output in
[the tests](tests/manifest.json) gives the exact token kind labels.

Use the longest available compound punctuation token. Treat keywords and
`_` as identifier-like at this stage. Handle nested block comments iteratively.
Raw strings support up to 255 delimiter hashes. Keep suffix spelling inside
literal tokens. Diagnose unterminated quoted and raw literals, unterminated
block comments, excessive raw delimiter hashes, and unknown token-start
characters.

PA1 establishes token boundaries. Spellings such as `0b9`, `0x`, `'ab'`,
unknown suffixes, and lifetimes beginning with a digit can reach the next
phase as tokens. PA2 validates literal expressions. Later assignments handle
reserved prefixes and edition-specific syntax. Preserve identifier spelling
for PA6's NFC name interning.

## Implementation and testing

Use a pull lexer over immutable source with compact offsets. Store spans in
typed tokens for later phases. Avoid allocating a new string per token,
retaining the entire token stream, or building per-character records. The text
adapter formats the typed tokens. Tokenization and diagnostics depend on
source bytes and compiler options.

Use the PA1 test target from the repository root.

```sh
make -C pa1 test
```

The test runner compares each case's output, diagnostic, and exit status with
its expected files. After building with `make` in the student repository, pass a manifest path
to `--case` to run one fixture.

```sh
python3 scripts/course.py pa1 --app obj/student/release/rustgm \
    --case tests/spec/100-empty.rs
```
