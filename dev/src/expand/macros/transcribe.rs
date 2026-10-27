//! Typed substitution keeps opaque syntax identities and explicit spelling origins.
mod concatenation;
use super::{
    bindings::{Binding, Bindings},
    capture::{CaptureId, Payload},
    input::{Input, delimiters},
    matcher::Matched,
    model::{Definition, Fragment, Kind, List, Repetition, Rule, VariableId},
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
    indices: Vec<usize>,
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
                        let Binding::Capture(capture) = self.bindings.at_path(
                            self.definition,
                            self.rule,
                            variable,
                            &self.indices,
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
                    let count = self.count(children, node.span.start)?;
                    if count == 0 && operator == Repetition::OneOrMore {
                        return Err(error(
                            node.span.start,
                            "macro transcription must repeat at least once",
                        ));
                    }
                    for index in 0..count {
                        self.tick(node.span.start)?;
                        if index != 0
                            && let Some(atom) = separator
                        {
                            self.token(atom.token, Origin::Definition)?;
                        }
                        self.indices.push(index);
                        self.sequence(children)?;
                        self.indices.pop();
                    }
                }
                Kind::Capture(_) => {
                    return Err(error(node.span.start, "capture declaration in template"));
                }
            }
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
        let mut work = vec![list];
        let mut count = None;
        while let Some(list) = work.pop() {
            for id in self.definition.list(list) {
                self.tick(offset)?;
                match self.definition.node(*id).kind {
                    Kind::Substitute { name, .. } => {
                        if let Some(variable) = self.variables.get(&name)
                            && let Binding::Sequence(values) = self.bindings.at_path(
                                self.definition,
                                self.rule,
                                *variable,
                                &self.indices,
                            )?
                        {
                            if count.is_some_and(|old| old != values.len()) {
                                return Err(error(
                                    offset,
                                    "metavariables repeat different numbers of times",
                                ));
                            }
                            count = Some(values.len());
                        }
                    }
                    Kind::Group { children, .. }
                    | Kind::Repeat { children, .. }
                    | Kind::Concat(children) => {
                        work.push(children);
                    }
                    Kind::Atom(_) | Kind::Capture(_) | Kind::Crate | Kind::Doc { .. } => {}
                }
            }
        }
        count.ok_or_else(|| error(offset, "template repetition has no repeating metavariable"))
    }
}
