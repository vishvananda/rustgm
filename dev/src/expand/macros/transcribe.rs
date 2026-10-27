//! Typed substitution keeps opaque syntax identities and explicit spelling origins.
mod concatenation;
mod metavariables;
use super::{
    bindings::{Binding, Bindings, Cardinalities, Count, PathIndex, Template},
    capture::{CaptureId, Payload},
    input::{Input, delimiters},
    matcher::Matched,
    model::{Atom, Definition, Fragment, Kind, List, Repetition, Rule, VariableId},
};
use crate::support::hash::HashMap;
use crate::{
    lex::token::{Kind as TokenKind, LexError, Token},
    support::names::{NameId, Names},
    syntax::{
        interpolation::{EMPTY_INPUTS, Inputs},
        token_tree::{Span, error},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Definition,
    Invocation,
}
#[derive(Clone, Copy, Debug)]
pub enum Piece {
    Token {
        token: Token,
        origin: Origin,
    },
    /// Only the later syntax consumer may clone/splice this parsed fragment.
    Opaque(CaptureId),
    /// Definition-site crate identity is attached by the expansion session.
    Crate(Span),
    /// Canonical generated spelling, anchored in the actual definition context.
    Identifier {
        name: NameId,
        span: Span,
    },
    Integer {
        value: u32,
        span: Span,
    },
    Doc {
        inner: bool,
        body: Span,
        span: Span,
    },
}
pub struct Transcription {
    pub pieces: Vec<Piece>,
    pub work: usize,
}
struct Writer<'a> {
    definition: &'a Definition,
    rule: &'a Rule,
    matched: &'a Matched,
    bindings: Bindings,
    variables: HashMap<NameId, VariableId>,
    input: &'a Input,
    source: &'a str,
    names: &'a mut Names,
    inputs: &'a dyn Inputs,
    indices: Vec<PathIndex>,
    cardinalities: Option<Box<Cardinalities>>,
    output: Transcription,
}
/// # Errors
/// Reports repetition cardinality/depth errors, corrupt capture facts and output limits.
pub fn transcribe(
    definition: &Definition,
    matched: &Matched,
    source: &str,
    input: &Input,
    names: &mut Names,
) -> Result<Transcription, LexError> {
    transcribe_with_inputs(definition, matched, source, input, names, &EMPTY_INPUTS)
}
/// Forwarded literals retain their actual parsed fragment and original source.
/// # Errors
/// Rejects invalid captures, concatenation operands and transcription budgets.
pub fn transcribe_with_inputs(
    definition: &Definition,
    matched: &Matched,
    source: &str,
    input: &Input,
    names: &mut Names,
    inputs: &dyn Inputs,
) -> Result<Transcription, LexError> {
    let rule = definition
        .rules
        .get(matched.rule)
        .ok_or_else(|| error(0, "invalid matched macro rule"))?;
    let mut variables = HashMap::default();
    for (index, variable) in rule.variables.iter().enumerate() {
        variables.insert(
            variable.name,
            VariableId(u32::try_from(index).map_err(|_| error(0, "too many macro variables"))?),
        );
    }
    let mut writer = Writer {
        definition,
        rule,
        matched,
        bindings: Bindings::from_match(matched)?,
        variables,
        input,
        source,
        names,
        inputs,
        indices: Vec::new(),
        cardinalities: None,
        output: Transcription {
            pieces: Vec::new(),
            work: 0,
        },
    };
    writer.sequence(rule.template)?;
    Ok(writer.output)
}
impl Writer<'_> {
    fn tick(&mut self, offset: u32) -> Result<(), LexError> {
        self.charge(offset, 1)
    }
    fn charge(&mut self, offset: u32, work: usize) -> Result<(), LexError> {
        if work > 1_000_000_usize.saturating_sub(self.output.work) {
            return Err(error(offset, "macro transcription work budget exceeded"));
        }
        self.output.work += work;
        Ok(())
    }
    fn token(&mut self, token: Token, origin: Origin) -> Result<(), LexError> {
        self.tick(token.start)?;
        self.output.pieces.push(Piece::Token { token, origin });
        Ok(())
    }
    fn sequence(&mut self, list: List) -> Result<(), LexError> {
        for id in self.definition.list(list) {
            let node = self.definition.node(*id);
            self.tick(node.span.start)?;
            match node.kind {
                Kind::Crate => self.output.pieces.push(Piece::Crate(node.span)),
                Kind::Concat(operands) => self.concat(operands, node.span)?,
                Kind::Expression(expression) => self.metavariable(expression, node.span)?,
                Kind::EscapedDollar(token) => self.token(token, Origin::Definition)?,
                Kind::Doc { inner, body } => self.output.pieces.push(Piece::Doc {
                    inner,
                    body,
                    span: node.span,
                }),
                Kind::Atom(atom) => self.token(atom.token, Origin::Definition)?,
                Kind::Group {
                    delimiter,
                    children,
                } => {
                    let (open, close) = delimiters(delimiter);
                    self.token(
                        Token {
                            kind: TokenKind::Punctuation(open),
                            start: node.span.start,
                            end: node.span.start + 1,
                        },
                        Origin::Definition,
                    )?;
                    self.sequence(children)?;
                    self.token(
                        Token {
                            kind: TokenKind::Punctuation(close),
                            start: node.span.end - 1,
                            end: node.span.end,
                        },
                        Origin::Definition,
                    )?;
                }
                Kind::Substitute {
                    name,
                    dollar,
                    identifier,
                } => {
                    if let Some(variable) = self.variables.get(&name).copied() {
                        let Binding::Capture(capture) = self.bindings.at_indices(
                            self.definition,
                            self.rule,
                            variable,
                            self.indices.iter().map(|frame| frame.index as usize),
                        )?
                        else {
                            return Err(error(
                                dollar.start,
                                "metavariable is still repeating at this depth",
                            ));
                        };
                        self.capture(capture)?;
                    } else {
                        // Unknown substitutions can be bindings of a generated nested macro.
                        self.token(dollar, Origin::Definition)?;
                        self.token(identifier, Origin::Definition)?;
                    }
                }
                Kind::Repeat {
                    children,
                    separator,
                    operator,
                } => {
                    self.repeat(children, separator, operator, node.span)?;
                }
                Kind::Capture(_) => {
                    return Err(error(node.span.start, "capture declaration in template"));
                }
            }
        }
        Ok(())
    }
    fn repeat(
        &mut self,
        children: List,
        separator: Option<Atom>,
        operator: Repetition,
        span: Span,
    ) -> Result<(), LexError> {
        let count = self.count(children, span.start)?;
        if count == 0 && operator == Repetition::OneOrMore {
            return Err(error(
                span.start,
                "macro transcription must repeat at least once",
            ));
        }
        let length =
            u32::try_from(count).map_err(|_| error(span.start, "too many macro iterations"))?;
        for index in 0..length {
            self.tick(span.start)?;
            if index != 0
                && let Some(atom) = separator
            {
                self.token(atom.token, Origin::Definition)?;
            }
            let key = u32::try_from(self.output.work)
                .map_err(|_| error(span.start, "macro frame identity overflow"))?;
            self.indices.push(PathIndex { index, length, key });
            self.sequence(children)?;
            self.indices.pop();
        }
        Ok(())
    }
    fn capture(&mut self, id: CaptureId) -> Result<(), LexError> {
        let capture = &self.matched.captures.values[id.0 as usize];
        if !matches!(capture.payload, Payload::Transparent) {
            self.output.pieces.push(Piece::Opaque(id));
            return Ok(());
        }
        let mut position = capture.start;
        let end = capture.end;
        while position != end {
            let token = self
                .input
                .token(self.source, position)?
                .ok_or_else(|| error(0, "capture extends beyond macro input"))?;
            self.token(token, Origin::Invocation)?;
            position = self.input.advance(self.source, position)?;
        }
        Ok(())
    }
    fn count(&mut self, list: List, offset: u32) -> Result<usize, LexError> {
        let template = Template {
            definition: self.definition,
            rule: self.rule,
            variables: &self.variables,
            path: &self.indices,
        };
        self.cardinalities
            .get_or_insert_with(|| Box::new(Cardinalities::default()))
            .repetition_count(&self.bindings, &template, list, &mut self.output.work)
            .map_err(|failure| error(offset, failure.message))
    }
}
