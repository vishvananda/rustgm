//! Decode format pieces from a literal value, binding only actual trait formals.
use super::{Error, Piece, Template, WarningCode, attribute_directives::Parser};

pub(super) fn parse(parser: &mut Parser<'_, '_>, value: &str) -> Result<Option<Template>, Error> {
    parser.charge(value.len(), value.len().saturating_mul(3))?;
    let mut pieces = Vec::new();
    let mut literal = String::new();
    let mut cursor = 0;
    while cursor < value.len() {
        let Some(ch) = value[cursor..].chars().next() else {
            break;
        };
        cursor += ch.len_utf8();
        if ch != '{' && ch != '}' {
            literal.push(ch);
            continue;
        }
        if value[cursor..].starts_with(ch) {
            literal.push(ch);
            cursor += 1;
            continue;
        }
        if ch == '}' {
            parser.warn(
                WarningCode::MalformedDiagnosticFormatLiterals,
                "malformed on_unimplemented format literal",
            )?;
            return Ok(None);
        }
        let Some(end) = value[cursor..].find('}').map(|length| cursor + length) else {
            parser.warn(
                WarningCode::MalformedDiagnosticFormatLiterals,
                "malformed on_unimplemented format literal",
            )?;
            return Ok(None);
        };
        parser.charge(parser.parameters.len(), 0)?;
        let Some(slot) = parser.slot(&value[cursor..end]) else {
            parser.warn(
                WarningCode::MalformedDiagnosticFormatLiterals,
                "on_unimplemented template names an unknown trait parameter",
            )?;
            return Ok(None);
        };
        append_literal(parser, &mut pieces, &mut literal)?;
        parser.charge(1, 4 * size_of::<Piece>())?;
        pieces.push(Piece::Argument(slot));
        cursor = end + 1;
    }
    append_literal(parser, &mut pieces, &mut literal)?;
    Ok(Some(Template(pieces)))
}
fn append_literal(
    parser: &mut Parser<'_, '_>,
    pieces: &mut Vec<Piece>,
    literal: &mut String,
) -> Result<(), Error> {
    if literal.is_empty() {
        return Ok(());
    }
    parser.charge(1, 4 * size_of::<Piece>())?;
    pieces.push(Piece::Literal(std::mem::take(literal).into_boxed_str()));
    Ok(())
}
