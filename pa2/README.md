# RustGM Programming Assignment 2 — Literals

Build on the [PA1 lexer](../pa1/README.md) to decode literal expressions and
convert numeric values in safe Rust. Use Rust 1.98.0, edition 2024, with a fixed
x86-64 target and 64-bit `usize` and `isize`. The pinned Rust Reference describes
[tokens](../doc/rust-reference/src/tokens.md) and
[literal expressions](../doc/rust-reference/src/expressions/literal-expr.md).

## Interface and scope

```text
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

Follow PA1's source normalization, offsets, input, and output conventions.
Success exits 0. A literal error exits 1 and writes `ERROR: OFFSET: MESSAGE`
and LF to standard error. OFFSET is the start of the invalid literal. Lexical
and input errors follow PA1's diagnostic format. Keep the complete output
lines preceding the error. Omit the failed literal and EOF records. Usage
errors exit 2. An unimplemented mode exits 86. The test cases provide the
expected diagnostic wording.

## Typed representation and conversion

Keep exact unsigned integer magnitudes up to `u128::MAX` and an optional typed
suffix. Validate radix digits, required digits and suffixes; ignore separators
when accumulating with checked arithmetic. A decimal integer spelling with
a floating-point suffix (`f16`, `f32`, `f64`, or `f128`) is a float. Hex
digits in `0x01_f32` belong to the integer,
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

Keep validated decimal spelling without its suffix or separators, and
retain the optional type. Convert directly to the requested IEEE width with
round-to-nearest, ties-to-even. For `f32` and `f64` you can parse directly into
the destination type using `from_str`. Convert `f16` and `f128` from the
retained decimal with bounded integer arithmetic. Preserve unsuffixed
spellings until inference selects a width, since an intermediate rounding can
change the result. Support subnormals, underflow to zero, and overflow to
infinity. Later phases apply the overflow lint and enforce source-level
feature gates for `f16` and `f128`. The PA2 inspection command decodes those
suffixes directly. `NaN` and `inf` are identifiers. Unary negative zero belongs
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

## Output format

All fields are ASCII, with one space between fields and LF after each record.
START and END are PA1's decimal normalized byte offsets (exclusive end).

| Value | Record |
| --- | --- |
| Integer | `integer START END TYPE MAGNITUDE` |
| Unsuffixed float | `float START END infer "DECIMAL"` |
| Suffixed float | `float START END TYPE 0xBITS` (4, 8, 16, or 32 hex digits for `f16`, `f32`, `f64`, or `f128`) |
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

## Run the tests

From the repository root:

```sh
make -C pa2 test
make -C pa1 test
```

[PA2's manifest](tests/manifest.json) covers numeric boundaries, IEEE rounding,
escapes, suffixes, raw forms, and output written before an error. Run PA1 again
after changing the lexer.
