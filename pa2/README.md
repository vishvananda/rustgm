# RustGM Programming Assignment 2: literals

Status: **implemented draft**. Prerequisite: PA1. Implement literal-expression
decoding and target numeric conversion in safe Rust. The language profile is
Rust 1.98.0, edition 2024, with a fixed x86-64 target (64-bit `usize`/`isize`).
The full [pinned Reference](../doc/README.md) is available offline; the owning
rules are [tokens](../doc/rust-reference/src/tokens.md) and
[literal expressions](../doc/rust-reference/src/expressions/literal-expr.md).

## Interface and scope

```sh
rustgm --emit-literals [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Consume PA1 tokens in source order and interpret numeric, character, byte,
string, byte-string, C-string and boolean literals. Omit other tokens, including
whitespace and comments. `true`/`false` are booleans; raw identifiers `r#true`
and `r#false` are not. Unary signs remain punctuation and are omitted. No
parsing, evaluation of surrounding expressions, type inference, macro expansion
or Rust-to-LowIR compilation is claimed by this view.

Every literal encountered by this diagnostic view is treated as a literal
expression. Arbitrary suffixes may be legal **macro tokens** at PA1, but are
not valid literal expressions. The future parser/expander must call the typed
decoder only when interpreting a literal; do not validate all macro input
eagerly. Reserved identifier prefixes and lifetime/name validation remain with
their grammar assignments. PA2 checks non-decimal numeric forms, including
reserved period continuations such as `0x80.0`; `0x80..1` remains two integers
separated by range punctuation. This diagnostic stream is not a syntax checker.

Source normalization, offsets, stdin/files and `-o` follow PA1. Success exits
0. The first error exits 1 with `ERROR: OFFSET: MESSAGE` and LF on stderr, where
OFFSET is the start of the invalid literal (lexical/input errors retain PA1's
location). Keep complete lines preceding the failed token; emit no failed
literal line and no EOF line. Usage errors exit 2; unimplemented modes exit 86.
The committed negative oracles freeze diagnostic wording.

## Typed representation and conversion

Keep exact unsigned integer magnitudes up to `u128::MAX` and an optional typed
suffix. Validate radix digits, required digits and suffixes; ignore separators
when accumulating with checked arithmetic. A decimal integer token with `f32`
or `f64` suffix is a float. Hex digits in `0x01_f32` belong to the integer,
whose value is 7986. Non-decimal floats are rejected.

Retain unsuffixed integer magnitudes for inference. A suffix alone does not
determine the unary-negation context: `128i8` is the magnitude needed in
`-128i8`. Decoding therefore accepts narrow-type overflow. The typed target
conversion takes an integer type and a negative-context flag, returns masked
two's-complement bits and an overflow fact, and rejects unsigned negation.
For signed negatives the allowed magnitude includes the minimum value.
`usize` and `isize` use 64 bits regardless of the compiler host. Type agreement,
the default-deny `overflowing_literals` lint, and arithmetic on expressions
belong to later semantic assignments. The conversion must expose overflow,
not silently lose it or use host signed overflow. This stage does not implement
the full constant evaluator.

Floats retain validated decimal spelling without suffix/separators and their
optional type. Convert directly to the demanded `f32` or `f64`, as the Reference
specifies with `from_str`; do not round through f64 before f32. Preserve the
spelling until inference, since eagerly choosing f64 can double-round. Suffixed
float dumps contain IEEE bits. Overflow produces infinity bits; later semantics
owns the overflow lint. Subnormal rounding and underflow to zero are supported.
`NaN` and `inf` are identifiers, not literal tokens. Unary negative zero belongs
to expression evaluation.

Character literals yield exactly one Unicode scalar; byte literals exactly one
byte. Decode simple, hex and Unicode escapes left to right. Unicode escapes
require braces, one to six hex digits, no leading underscore, and a valid scalar;
underscores after digits do not count toward the limit. Unicode text permits
only ASCII `\xNN` escapes. Byte literals/strings allow arbitrary hex bytes but
forbid bare non-ASCII and Unicode escapes. Bare CR is rejected, as are bare tab
and LF in character literals. CRLF has already been normalized by PA1.

String continuations discard the escaped LF and subsequent ASCII space, tab,
CR and LF. Other Unicode whitespace is retained. Raw strings do not decode
escapes. Non-numeric suffixes are rejected on every textual literal form.
C strings accept UTF-8 scalars and arbitrary escaped bytes, reject NUL in any
form and include one implicit trailing NUL. Ordinary strings may contain NUL.
Decode plain/raw contents as borrowed source slices; allocate only when escapes
change bytes. The typed C-string content excludes its implicit terminator so a
raw C string does not require an allocation merely to append zero.

## Text oracle

All fields are ASCII, with one space between fields and LF after each record.
START and END are PA1's decimal normalized byte offsets (exclusive end).

| Value | Record |
| --- | --- |
| Integer | `integer START END TYPE MAGNITUDE` |
| Unsuffixed float | `float START END infer "DECIMAL"` |
| Suffixed float | `float START END f32 0xXXXXXXXX` or `f64 0xXXXXXXXXXXXXXXXX` |
| Character | `character START END U+XXXXXX` (six uppercase hex digits) |
| Byte | `byte START END 0xXX` (two lowercase hex digits) |
| Boolean | `boolean START END true` or `false` |
| String | `string START END BYTE_COUNT "BYTES"` |
| Byte string | `byte-string START END BYTE_COUNT "BYTES"` |
| C string | `c-string START END BYTE_COUNT "BYTES_WITH_NUL"` |

Integer TYPE is the suffix or `infer`; MAGNITUDE is unsigned decimal. Float
bits are lowercase hex of the stated fixed width, independent of host byte
order. Unsuffixed DECIMAL preserves digit/exponent spelling after removing
underscores, without forcing a default type. Raw and cooked strings with equal
values use the same record kind. String lengths count encoded bytes, not
characters; C-string length includes the terminator. Byte quoting follows PA1:
`\"`, `\\`, `\n`, `\r`, `\t`, printable ASCII, otherwise lowercase `\xNN`.

For the input `42 0.1f32 "é" c"x"` with no trailing newline:

```text
integer 0 2 infer 42
float 3 9 f32 0x3dcccccd
string 10 14 2 "\xc3\xa9"
c-string 15 19 2 "x\x00"
eof 19
```

Finish successful input with `eof NORMALIZED_BYTE_LENGTH` and LF, including
empty input or input containing no literals. Never serialize then reparse this
dump between frontend phases. Use typed tokens and literal values instead.

## Validation and efficiency

`make -C pa2 test` runs every manifest case against committed stdout/stderr/status
sidecars. Cases cover numeric boundaries, rounding, escapes, suffixes, raw forms
and complete-prefix failures. `make test-rustc` independently compiles selected
fixture expressions and compares evaluated bytes/bits, and checks rejection
cases. Overflow-only cases explicitly allow rustc's overflow lint because this
stage decodes magnitude before contextual linting. Imported full-program tests
retain their upstream bytes, revision and licenses.

Rust integration tests check typed target conversion, inference preservation,
borrowing, independent fixed vectors and large malformed inputs. Run all active
earlier suites, `make lint audit-rust test-harness test-upstream`, and scaling
measurements before updating the accepted report. `--count-literals` exercises
the same interpretation and explicit float conversions without rendering.
The reference implementation is in `dev/src/literal/`, with no frontend fallback.
