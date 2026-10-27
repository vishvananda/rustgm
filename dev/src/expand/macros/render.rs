//! PA7's diagnostic view serializes typed captures and the one-step token result.
use super::{
    capture::Payload,
    model::Fragment,
    session::{self, Report},
    transcribe::{Origin, Piece},
};
use crate::{
    expand::config::Config,
    lex::render::RenderError,
    support::quoted,
    syntax::{cursor::TokenPosition, expression::fragment::FragmentKind as F},
};
use std::io::{self, Write};
/// # Errors
/// Writes nothing on a source error; otherwise reports output failures.
pub fn expanded(source: &str, config: &Config, output: &mut impl Write) -> Result<(), RenderError> {
    let report = session::analyze(source, config).map_err(RenderError::Lex)?;
    write_report(source, &report, output).map_err(RenderError::Io)
}
/// # Errors
/// Reports failures writing the deterministic report.
pub fn write_report(source: &str, report: &Report, output: &mut impl Write) -> io::Result<()> {
    writeln!(output, "macro-matching 1")?;
    for (index, definition) in report.definitions.iter().enumerate() {
        write!(
            output,
            "definition {index} {} {} ",
            definition.span.start, definition.span.end
        )?;
        quoted::bytes(output, report.names.get(definition.name).as_bytes(), false)?;
        writeln!(
            output,
            " rules {}",
            definition.compiled.definition.rules.len()
        )?;
    }
    for (index, invocation) in report.invocations.iter().enumerate() {
        let input_source = invocation
            .prepared
            .as_ref()
            .map_or(source, |p| p.source.as_str());
        writeln!(
            output,
            "invocation {index} {} {} definition {} rule {}",
            invocation.span.start,
            invocation.span.end,
            invocation.definition,
            invocation.matched.rule
        )?;
        write_captures(input_source, invocation, output)?;
        for piece in &invocation.transcription.pieces {
            match piece {
                Piece::Token { token, origin } => {
                    let (label, text) = match origin {
                        Origin::Definition => ("definition", source),
                        Origin::Invocation => ("invocation", input_source),
                    };
                    write!(output, "token {label} {} ", token.kind.name())?;
                    quoted::bytes(
                        output,
                        &text.as_bytes()[token.start as usize..token.end as usize],
                        false,
                    )?;
                    writeln!(output)?;
                }
                Piece::Opaque(id) => writeln!(output, "opaque {}", id.0)?,
                Piece::Identifier { name, .. } => {
                    write!(output, "token definition identifier ")?;
                    quoted::bytes(output, report.names.get(*name).as_bytes(), false)?;
                    writeln!(output)?;
                }
                Piece::Integer { value, .. } => {
                    writeln!(output, "token definition integer \"{value}\"")?;
                }
                Piece::Crate(_) => writeln!(output, "crate definition {}", invocation.definition)?,
                Piece::Doc { inner, body, .. } => {
                    write!(output, "doc definition inner={inner} ")?;
                    quoted::bytes(
                        output,
                        &source.as_bytes()[body.start as usize..body.end as usize],
                        false,
                    )?;
                    writeln!(output)?;
                }
            }
        }
        writeln!(
            output,
            "end-invocation parser-calls {}",
            invocation.matched.captures.parser_calls
        )?;
    }
    writeln!(
        output,
        "end-macro-matching definitions {} invocations {}",
        report.definitions.len(),
        report.invocations.len()
    )
}
fn write_captures(
    source: &str,
    invocation: &super::session::Invocation,
    output: &mut impl Write,
) -> io::Result<()> {
    for (capture_id, capture) in invocation.matched.captures.values.iter().enumerate() {
        let offset = |position: TokenPosition| {
            invocation.input.tokens.get(position.index).map_or_else(
                || invocation.input.tokens.last().map_or(0, |t| t.end),
                |t| t.start + position.offset,
            )
        };
        let start = offset(capture.start);
        let end = if capture.end.offset == 0 && capture.end.index > capture.start.index {
            invocation.input.tokens[capture.end.index - 1].end
        } else {
            offset(capture.end)
        };
        write!(
            output,
            "capture {capture_id} {} {}:{} {}:{} ",
            fragment_name(capture.fragment),
            capture.start.index,
            capture.start.offset,
            capture.end.index,
            capture.end.offset
        )?;
        quoted::bytes(
            output,
            &source.as_bytes()[start as usize..end as usize],
            false,
        )?;
        let nodes = match &capture.payload {
            Payload::Syntax(parsed) => parsed.ast.nodes.len(),
            Payload::Metadata { expression, .. } => {
                expression.as_ref().map_or(0, |p| p.ast.nodes.len())
            }
            Payload::Visibility { path, .. } => path.as_ref().map_or(0, |p| p.ast.nodes.len()),
            Payload::Transparent => 0,
        };
        writeln!(output, " nodes {nodes}")?;
    }
    Ok(())
}
#[must_use]
pub const fn fragment_name(fragment: Fragment) -> &'static str {
    match fragment {
        Fragment::Identifier => "ident",
        Fragment::Lifetime => "lifetime",
        Fragment::Tree => "tt",
        Fragment::Metadata => "meta",
        Fragment::Visibility => "vis",
        Fragment::Syntax(kind) => match kind {
            F::Block => "block",
            F::Expression => "expr",
            F::Expression2021 => "expr_2021",
            F::Item => "item",
            F::Literal => "literal",
            F::Pattern => "pat",
            F::PatternParameter => "pat_param",
            F::Path => "path",
            F::Statement => "stmt",
            F::Type => "ty",
        },
    }
}
