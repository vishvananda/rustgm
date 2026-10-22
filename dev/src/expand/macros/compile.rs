//! Compile retained macro trees once, preserving typed tokens and source provenance.
use super::model::{
    Atom, Definition, Fragment, Kind, List, Node, NodeId, Repetition, Rule, Variable, VariableId,
};
use crate::support::hash::HashMap;
use crate::{
    lex::token::{Kind as TokenKind, LexError, Punctuation as P, Token},
    support::names::{NameId, Names},
    syntax::{
        expression::fragment::FragmentKind,
        token_tree::{Delimiter, Forest, Span, TreeKind, TreeRange, error},
    },
};
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Matcher,
    Template,
}
struct Compiler<'a> {
    source: &'a str,
    forest: &'a Forest,
    names: &'a mut Names,
    definition: Definition,
    variables: Vec<Variable>,
    by_name: HashMap<NameId, VariableId>,
    repetitions: Vec<NodeId>,
    scratch: Vec<NodeId>,
    depth: u32,
}
/// # Errors
/// Reports malformed rules, metavariables, fragment names and repetition syntax.
pub fn compile(source: &str, forest: &Forest, names: &mut Names) -> Result<Definition, LexError> {
    let mut compiler = Compiler {
        source,
        forest,
        names,
        definition: Definition::default(),
        variables: Vec::new(),
        by_name: HashMap::default(),
        repetitions: Vec::new(),
        scratch: Vec::new(),
        depth: 0,
    };
    let body = forest
        .children(forest.roots().start)
        .ok_or_else(|| error(0, "macro definition requires a delimited body"))?;
    let mut next = body.start;
    while next < body.end {
        compiler.by_name.clear();
        let matcher = compiler.group_sequence(next, Mode::Matcher)?;
        next = forest.after(next);
        if !forest.punctuation(next, P::FatArrow) {
            return Err(error(forest.offset(next), "macro rule requires =>"));
        }
        next += 1;
        let template = compiler.group_sequence(next, Mode::Template)?;
        next = forest.after(next);
        compiler.definition.rules.push(Rule {
            matcher,
            template,
            variables: std::mem::take(&mut compiler.variables),
        });
        if next < body.end {
            if !forest.punctuation(next, P::Semicolon) {
                return Err(error(
                    forest.offset(next),
                    "macro rules require semicolon separation",
                ));
            }
            next += 1;
        }
    }
    if compiler.definition.rules.is_empty() {
        return Err(error(0, "macro definition requires at least one rule"));
    }
    super::follow::check(&compiler.definition, compiler.names)?;
    Ok(compiler.definition)
}
impl Compiler<'_> {
    fn group_sequence(&mut self, index: u32, mode: Mode) -> Result<List, LexError> {
        let range = self
            .forest
            .children(index)
            .ok_or_else(|| error(self.forest.offset(index), "expected macro delimiter group"))?;
        self.sequence(range, mode)
    }
    fn sequence(&mut self, range: TreeRange, mode: Mode) -> Result<List, LexError> {
        if self.depth >= 128 {
            return Err(error(
                self.forest.offset(range.start),
                "macro grammar nesting exceeds 128",
            ));
        }
        self.depth += 1;
        let mark = self.scratch.len();
        let mut next = range.start;
        while next < range.end {
            let (node, after) = self.piece(next, range.end, mode)?;
            self.scratch.push(node);
            next = after;
        }
        self.depth -= 1;
        let start = u32::try_from(self.definition.edges.len())
            .map_err(|_| error(0, "too many macro edges"))?;
        self.definition.edges.extend(self.scratch.drain(mark..));
        let end = u32::try_from(self.definition.edges.len())
            .map_err(|_| error(0, "too many macro edges"))?;
        Ok(List { start, end })
    }
    fn piece(&mut self, index: u32, end: u32, mode: Mode) -> Result<(NodeId, u32), LexError> {
        let tree = self
            .forest
            .tree(index)
            .ok_or_else(|| error(self.forest.offset(index), "missing macro tree"))?;
        if let TreeKind::Group { delimiter, .. } = tree.kind {
            let children = self.group_sequence(index, mode)?;
            return Ok((
                self.node(
                    tree.span,
                    Kind::Group {
                        delimiter,
                        children,
                    },
                )?,
                self.forest.after(index),
            ));
        }
        if let TreeKind::Doc { inner, body } = tree.kind {
            return Ok((self.node(tree.span, Kind::Doc { inner, body })?, index + 1));
        }
        if self.forest.punctuation(index, P::Dollar) {
            return self.dollar(index, end, mode);
        }
        let atom = self.atom(index)?;
        Ok((self.node(tree.span, Kind::Atom(atom))?, index + 1))
    }
    fn dollar(&mut self, index: u32, end: u32, mode: Mode) -> Result<(NodeId, u32), LexError> {
        let dollar = self.token(index)?;
        if index + 1 >= end {
            let atom = self.atom(index)?;
            return Ok((
                self.node(
                    Span {
                        start: dollar.start,
                        end: dollar.end,
                    },
                    Kind::Atom(atom),
                )?,
                index + 1,
            ));
        }
        if matches!(
            self.forest.tree(index + 1).map(|t| t.kind),
            Some(TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            })
        ) {
            return self.repetition(dollar, index + 1, end, mode);
        }
        let identifier = self.token(index + 1)?;
        let name = self.name(index + 1)?;
        if self.names.get(name) == "crate" {
            return Ok((
                self.node(
                    Span {
                        start: dollar.start,
                        end: identifier.end,
                    },
                    Kind::Crate,
                )?,
                index + 2,
            ));
        }
        if mode == Mode::Template {
            let span = Span {
                start: dollar.start,
                end: identifier.end,
            };
            return Ok((
                self.node(
                    span,
                    Kind::Substitute {
                        name,
                        dollar,
                        identifier,
                    },
                )?,
                index + 2,
            ));
        }
        if index + 3 >= end {
            return Err(error(
                identifier.start,
                "matcher metavariable requires a fragment specifier",
            ));
        }
        if !self.forest.punctuation(index + 2, P::Colon) {
            return Err(error(
                identifier.start,
                "matcher metavariable requires a fragment specifier",
            ));
        }
        let fragment_name = self
            .forest
            .name(self.source, index + 3)
            .ok_or_else(|| error(self.forest.offset(index + 3), "expected fragment specifier"))?;
        let fragment = fragment(fragment_name).ok_or_else(|| {
            error(
                self.forest.offset(index + 3),
                "unknown macro fragment specifier",
            )
        })?;
        let variable = VariableId(
            u32::try_from(self.variables.len())
                .map_err(|_| error(dollar.start, "too many macro variables"))?,
        );
        if self.by_name.insert(name, variable).is_some() {
            return Err(error(identifier.start, "duplicate macro matcher variable"));
        }
        let start = u32::try_from(self.definition.edges.len())
            .map_err(|_| error(dollar.start, "too many repetition edges"))?;
        self.definition.edges.extend_from_slice(&self.repetitions);
        let end = u32::try_from(self.definition.edges.len())
            .map_err(|_| error(dollar.start, "too many repetition edges"))?;
        self.variables.push(Variable {
            name,
            fragment,
            repetitions: List { start, end },
        });
        let span = Span {
            start: dollar.start,
            end: self.token(index + 3)?.end,
        };
        Ok((self.node(span, Kind::Capture(variable))?, index + 4))
    }
    fn repetition(
        &mut self,
        dollar: Token,
        group: u32,
        end: u32,
        mode: Mode,
    ) -> Result<(NodeId, u32), LexError> {
        let body = self
            .forest
            .children(group)
            .ok_or_else(|| error(dollar.start, "missing repetition body"))?;
        if body.start == body.end {
            return Err(error(dollar.start, "empty macro repetition"));
        }
        let mut next = self.forest.after(group);
        if next >= end {
            return Err(error(dollar.start, "expected macro repetition operator"));
        }
        let mut separator = None;
        let mut operator = self.operator(next);
        if operator.is_none() {
            separator = Some(self.atom(next)?);
            next += 1;
            operator = self.operator(next);
        }
        let operator = operator.filter(|_| next < end).ok_or_else(|| {
            error(
                self.forest.offset(next),
                "expected macro repetition operator",
            )
        })?;
        if operator == Repetition::Optional && separator.is_some() {
            return Err(error(
                dollar.start,
                "optional macro repetition cannot have a separator",
            ));
        }
        let span = Span {
            start: dollar.start,
            end: self.token(next)?.end,
        };
        let node = self.node(
            span,
            Kind::Repeat {
                children: List { start: 0, end: 0 },
                separator,
                operator,
            },
        )?;
        self.repetitions.push(node);
        let children = self.sequence(body, mode)?;
        self.repetitions.pop();
        self.definition.nodes[node.0 as usize].kind = Kind::Repeat {
            children,
            separator,
            operator,
        };
        Ok((node, next + 1))
    }
    fn operator(&self, index: u32) -> Option<Repetition> {
        match self.forest.token(index)?.kind {
            TokenKind::Punctuation(P::Star) => Some(Repetition::ZeroOrMore),
            TokenKind::Punctuation(P::Plus) => Some(Repetition::OneOrMore),
            TokenKind::Punctuation(P::Question) => Some(Repetition::Optional),
            _ => None,
        }
    }
    fn atom(&mut self, index: u32) -> Result<Atom, LexError> {
        let token = self.token(index)?;
        let name = if matches!(token.kind, TokenKind::Identifier | TokenKind::RawIdentifier) {
            Some(self.name(index)?)
        } else {
            None
        };
        Ok(Atom { token, name })
    }
    fn name(&mut self, index: u32) -> Result<NameId, LexError> {
        let name = self
            .forest
            .name(self.source, index)
            .ok_or_else(|| error(self.forest.offset(index), "expected macro variable name"))?;
        self.names
            .intern(name)
            .map_err(|m| error(self.forest.offset(index), m))
    }
    fn token(&self, index: u32) -> Result<Token, LexError> {
        self.forest
            .token(index)
            .ok_or_else(|| error(self.forest.offset(index), "expected macro token"))
    }
    fn node(&mut self, span: Span, kind: Kind) -> Result<NodeId, LexError> {
        let id = NodeId(
            u32::try_from(self.definition.nodes.len())
                .map_err(|_| error(span.start, "too many macro nodes"))?,
        );
        self.definition.nodes.push(Node { span, kind });
        Ok(id)
    }
}
fn fragment(name: &str) -> Option<Fragment> {
    Some(match name {
        "ident" => Fragment::Identifier,
        "lifetime" => Fragment::Lifetime,
        "tt" => Fragment::Tree,
        "meta" => Fragment::Metadata,
        "vis" => Fragment::Visibility,
        "block" => Fragment::Syntax(FragmentKind::Block),
        "expr" => Fragment::Syntax(FragmentKind::Expression),
        "expr_2021" => Fragment::Syntax(FragmentKind::Expression2021),
        "item" => Fragment::Syntax(FragmentKind::Item),
        "literal" => Fragment::Syntax(FragmentKind::Literal),
        "pat" => Fragment::Syntax(FragmentKind::Pattern),
        "pat_param" => Fragment::Syntax(FragmentKind::PatternParameter),
        "path" => Fragment::Syntax(FragmentKind::Path),
        "stmt" => Fragment::Syntax(FragmentKind::Statement),
        "ty" => Fragment::Syntax(FragmentKind::Type),
        _ => return None,
    })
}
