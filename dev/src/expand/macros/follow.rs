//! Bounded FIRST sets validate all nullable repetition paths without enumerating them.
use super::model::{Atom, Definition, Fragment, Kind, List, NodeId, Repetition, Rule};
use crate::{
    lex::token::{Kind as TokenKind, LexError, Punctuation as P},
    support::names::Names,
    syntax::{
        expression::fragment::FragmentKind as F,
        token_tree::{Delimiter, error},
    },
};
const END: u64 = 1;
const ARROW: u64 = 1 << 1;
const COMMA: u64 = 1 << 2;
const SEMICOLON: u64 = 1 << 3;
const EQUAL: u64 = 1 << 4;
const OR: u64 = 1 << 5;
const COLON: u64 = 1 << 6;
const GREATER: u64 = 1 << 7;
const SHIFT_RIGHT: u64 = 1 << 8;
const BRACKET: u64 = 1 << 9;
const BRACE: u64 = 1 << 10;
const IF: u64 = 1 << 11;
const IN: u64 = 1 << 12;
const AS: u64 = 1 << 13;
const WHERE: u64 = 1 << 14;
const PRIV: u64 = 1 << 15;
const IDENTIFIER: u64 = 1 << 16;
const TYPE_START: u64 = 1 << 17;
const OTHER: u64 = 1 << 18;
const PAREN: u64 = 1 << 19;
const VARIABLE_BASE: u32 = 24;
#[derive(Clone, Copy)]
struct First {
    tokens: u64,
    nullable: bool,
}
impl First {
    const fn terminal(tokens: u64) -> Self {
        Self {
            tokens,
            nullable: false,
        }
    }
    fn union(self, other: Self) -> Self {
        Self {
            tokens: self.tokens | other.tokens,
            nullable: self.nullable && other.nullable,
        }
    }
}
struct Check<'a> {
    definition: &'a Definition,
    names: &'a Names,
    rule: &'a Rule,
    facts: &'a mut [Option<First>],
}
/// # Errors
/// Rejects forbidden fragment followers and repetitions that can consume nothing.
pub fn check(definition: &Definition, names: &Names) -> Result<(), LexError> {
    let mut facts = vec![None; definition.nodes.len()];
    for rule in &definition.rules {
        let mut check = Check {
            definition,
            names,
            rule,
            facts: &mut facts,
        };
        check.sequence(rule.matcher, First::terminal(END))?;
    }
    Ok(())
}
impl Check<'_> {
    fn first(&mut self, id: NodeId) -> Result<First, LexError> {
        if let Some(value) = self.facts[id.0 as usize] {
            return Ok(value);
        }
        let node = self.definition.node(id);
        let value = match node.kind {
            Kind::Crate => First::terminal(IDENTIFIER),
            Kind::Doc { .. } => First::terminal(OTHER),
            Kind::Atom(atom) => First {
                tokens: atom_start(atom, self.names),
                nullable: false,
            },
            Kind::Group { delimiter, .. } => First {
                tokens: match delimiter {
                    Delimiter::Parenthesis => PAREN,
                    Delimiter::Bracket => BRACKET,
                    Delimiter::Brace => BRACE,
                },
                nullable: false,
            },
            Kind::Capture(variable) => {
                let fragment = self.rule.variables[variable.0 as usize].fragment;
                First {
                    tokens: fragment_bit(fragment),
                    nullable: fragment == Fragment::Visibility,
                }
            }
            Kind::Repeat {
                children, operator, ..
            } => {
                let first = self.first_list(children)?;
                if first.nullable {
                    return Err(error(
                        node.span.start,
                        "macro repetition can match empty input",
                    ));
                }
                First {
                    nullable: operator != Repetition::OneOrMore,
                    ..first
                }
            }
            Kind::Substitute { .. }
            | Kind::Concat(_)
            | Kind::Expression(_)
            | Kind::EscapedDollar(_) => {
                return Err(error(node.span.start, "substitution in a matcher"));
            }
        };
        self.facts[id.0 as usize] = Some(value);
        Ok(value)
    }
    fn first_list(&mut self, list: List) -> Result<First, LexError> {
        let mut result = First {
            tokens: 0,
            nullable: true,
        };
        for offset in list.start..list.end {
            let first = self.first(self.definition.edges[offset as usize])?;
            result = result.union(first);
            if !result.nullable {
                break;
            }
        }
        Ok(result)
    }
    fn sequence(&mut self, list: List, mut following: First) -> Result<(), LexError> {
        for offset in (list.start..list.end).rev() {
            let id = self.definition.edges[offset as usize];
            let node = self.definition.node(id);
            let first = self.first(id)?;
            match node.kind {
                Kind::Capture(variable) => {
                    let fragment = self.rule.variables[variable.0 as usize].fragment;
                    if following.tokens & !allowed(fragment) != 0 {
                        return Err(error(
                            node.span.start,
                            "token is forbidden after this macro fragment",
                        ));
                    }
                }
                Kind::Group { children, .. } => self.sequence(children, First::terminal(END))?,
                Kind::Repeat {
                    children,
                    separator,
                    operator,
                } => {
                    let mut next = following;
                    if operator != Repetition::Optional
                        && let Some(atom) = separator
                    {
                        next = next.union(First::terminal(atom_start(atom, self.names)));
                    }
                    self.sequence(children, next)?;
                }
                _ => {}
            }
            following = if first.nullable {
                first.union(following)
            } else {
                first
            };
        }
        Ok(())
    }
}
fn fragment_bit(fragment: Fragment) -> u64 {
    let index = match fragment {
        Fragment::Identifier => 0,
        Fragment::Lifetime => 1,
        Fragment::Tree => 2,
        Fragment::Metadata => 3,
        Fragment::Visibility => 4,
        Fragment::Syntax(kind) => match kind {
            F::Block => 5,
            F::Expression => 6,
            F::Expression2021 => 7,
            F::Item => 8,
            F::Literal => 9,
            F::Pattern => 10,
            F::PatternParameter => 11,
            F::Path => 12,
            F::Statement => 13,
            F::Type => 14,
        },
    };
    1_u64 << (VARIABLE_BASE + index)
}
fn allowed(fragment: Fragment) -> u64 {
    END | match fragment {
        Fragment::Syntax(F::Expression | F::Expression2021 | F::Statement) => {
            ARROW | COMMA | SEMICOLON
        }
        Fragment::Syntax(F::Pattern) => ARROW | COMMA | EQUAL | IF | IN,
        Fragment::Syntax(F::PatternParameter) => ARROW | COMMA | EQUAL | OR | IF | IN,
        Fragment::Syntax(F::Path | F::Type) => {
            ARROW
                | COMMA
                | EQUAL
                | OR
                | SEMICOLON
                | COLON
                | GREATER
                | SHIFT_RIGHT
                | BRACKET
                | BRACE
                | AS
                | WHERE
                | fragment_bit(Fragment::Syntax(F::Block))
        }
        Fragment::Visibility => {
            COMMA
                | IDENTIFIER
                | IF
                | IN
                | AS
                | WHERE
                | TYPE_START
                | PAREN
                | BRACKET
                | fragment_bit(Fragment::Identifier)
                | fragment_bit(Fragment::Syntax(F::Path))
                | fragment_bit(Fragment::Syntax(F::Type))
        }
        _ => u64::MAX,
    }
}
fn atom_start(atom: Atom, names: &Names) -> u64 {
    if atom.token.kind == TokenKind::RawIdentifier {
        return IDENTIFIER;
    }
    if let Some(name) = atom.name {
        return match names.get(name) {
            "if" => IF,
            "in" => IN,
            "as" => AS,
            "where" => WHERE,
            "priv" => PRIV,
            _ => IDENTIFIER,
        };
    }
    match atom.token.kind {
        TokenKind::Lifetime | TokenKind::RawLifetime => TYPE_START,
        TokenKind::Punctuation(p) => match p {
            P::FatArrow => ARROW,
            P::Comma => COMMA,
            P::Semicolon => SEMICOLON,
            P::Equal => EQUAL,
            P::Or => OR,
            P::Colon => COLON,
            P::Greater => GREATER,
            P::ShiftRight => SHIFT_RIGHT,
            P::OpenBracket => BRACKET,
            P::OpenBrace => BRACE,
            P::OpenParen => PAREN,
            P::Star
            | P::And
            | P::AndAnd
            | P::Bang
            | P::Less
            | P::ShiftLeft
            | P::Underscore
            | P::ColonColon => TYPE_START,
            _ => OTHER,
        },
        _ => OTHER,
    }
}
