//! A flat, preorder token forest: iterative construction, traversal and drop.
use crate::lex::{
    Lexer,
    token::{Kind, LexError, Punctuation, Token},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delimiter {
    Parenthesis,
    Bracket,
    Brace,
}

impl Delimiter {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Parenthesis => "paren",
            Self::Bracket => "bracket",
            Self::Brace => "brace",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeKind {
    Token(Kind),
    Group {
        delimiter: Delimiter,
        after: u32,
    },
    /// Typed desugaring to #[doc = body] or #![doc = body], without stringifying.
    Doc {
        inner: bool,
        body: Span,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tree {
    pub span: Span,
    pub kind: TreeKind,
}

#[derive(Debug)]
pub struct Forest {
    nodes: Vec<Tree>,
    end: u32,
}

impl Forest {
    /// Capture a diagnostic file or a retained macro fragment. The ordinary
    /// source parser must not use a whole-file forest as its token buffer.
    ///
    /// # Errors
    /// Rejects lexical failures, delimiter mismatch and bare CR in doc comments.
    pub fn parse(source: &str) -> Result<Self, LexError> {
        let mut lexer = Lexer::new(source)?;
        let mut forest = Self {
            nodes: Vec::new(),
            end: 0,
        };
        let mut open: Vec<u32> = Vec::new();
        while let Some(token) = lexer.next_token()? {
            if matches!(
                token.kind,
                Kind::Whitespace | Kind::LineComment | Kind::BlockComment
            ) {
                continue;
            }
            let index = u32::try_from(forest.nodes.len())
                .map_err(|_| error(token.start, "too many token trees"))?;
            let span = Span {
                start: token.start,
                end: token.end,
            };
            let kind = if let Some((delimiter, closing)) = delimiter(token.kind) {
                if closing {
                    let opening = open
                        .pop()
                        .ok_or_else(|| error(token.start, "unexpected closing delimiter"))?;
                    let node = &mut forest.nodes[opening as usize];
                    let TreeKind::Group {
                        delimiter: expected,
                        ..
                    } = node.kind
                    else {
                        return Err(error(token.start, "invalid delimiter stack"));
                    };
                    if expected != delimiter {
                        return Err(error(token.start, "mismatched closing delimiter"));
                    }
                    node.span.end = token.end;
                    node.kind = TreeKind::Group {
                        delimiter,
                        after: index,
                    };
                    continue;
                }
                open.push(index);
                TreeKind::Group {
                    delimiter,
                    after: index + 1,
                }
            } else if let Some(inner) = doc_style(token.kind) {
                let body = Span {
                    start: token.start + 3,
                    end: token.end
                        - u32::from(matches!(
                            token.kind,
                            Kind::InnerBlockDoc | Kind::OuterBlockDoc
                        )) * 2,
                };
                if source[body.start as usize..body.end as usize].contains('\r') {
                    return Err(error(
                        token.start,
                        "bare carriage return in documentation comment",
                    ));
                }
                TreeKind::Doc { inner, body }
            } else {
                TreeKind::Token(token.kind)
            };
            forest.nodes.push(Tree { span, kind });
        }
        if let Some(index) = open.last() {
            return Err(error(
                forest.nodes[*index as usize].span.start,
                "unclosed delimiter",
            ));
        }
        forest.end = lexer.offset();
        Ok(forest)
    }

    #[must_use]
    pub fn roots(&self) -> TreeRange {
        TreeRange {
            start: 0,
            end: self.nodes.len().try_into().unwrap_or(u32::MAX),
        }
    }

    #[must_use]
    pub fn nodes(&self) -> &[Tree] {
        &self.nodes
    }

    #[must_use]
    pub const fn source_end(&self) -> u32 {
        self.end
    }

    #[must_use]
    pub fn tree(&self, index: u32) -> Option<Tree> {
        self.nodes.get(index as usize).copied()
    }

    #[must_use]
    pub fn after(&self, index: u32) -> u32 {
        match self.tree(index).map(|tree| tree.kind) {
            Some(TreeKind::Group { after, .. }) => after,
            _ => index.saturating_add(1),
        }
    }

    #[must_use]
    pub fn children(&self, index: u32) -> Option<TreeRange> {
        match self.tree(index)?.kind {
            TreeKind::Group { after, .. } => Some(TreeRange {
                start: index + 1,
                end: after,
            }),
            _ => None,
        }
    }

    #[must_use]
    pub fn token(&self, index: u32) -> Option<Token> {
        let Tree {
            span,
            kind: TreeKind::Token(kind),
        } = self.tree(index)?
        else {
            return None;
        };
        Some(Token {
            kind,
            start: span.start,
            end: span.end,
        })
    }

    #[must_use]
    pub fn punctuation(&self, index: u32, punctuation: Punctuation) -> bool {
        self.token(index)
            .is_some_and(|token| token.kind == Kind::Punctuation(punctuation))
    }

    #[must_use]
    pub fn name<'source>(&self, source: &'source str, index: u32) -> Option<&'source str> {
        let token = self.token(index)?;
        if !matches!(token.kind, Kind::Identifier | Kind::RawIdentifier) {
            return None;
        }
        let spelling = source.get(token.start as usize..token.end as usize)?;
        Some(spelling.strip_prefix("r#").unwrap_or(spelling))
    }

    #[must_use]
    pub fn offset(&self, index: u32) -> u32 {
        self.tree(index).map_or(self.end, |node| node.span.start)
    }
}

#[must_use]
pub const fn error(offset: u32, message: &'static str) -> LexError {
    LexError { offset, message }
}

fn doc_style(kind: Kind) -> Option<bool> {
    match kind {
        Kind::InnerLineDoc | Kind::InnerBlockDoc => Some(true),
        Kind::OuterLineDoc | Kind::OuterBlockDoc => Some(false),
        _ => None,
    }
}

fn delimiter(kind: Kind) -> Option<(Delimiter, bool)> {
    Some(match kind {
        Kind::Punctuation(Punctuation::OpenParen) => (Delimiter::Parenthesis, false),
        Kind::Punctuation(Punctuation::CloseParen) => (Delimiter::Parenthesis, true),
        Kind::Punctuation(Punctuation::OpenBracket) => (Delimiter::Bracket, false),
        Kind::Punctuation(Punctuation::CloseBracket) => (Delimiter::Bracket, true),
        Kind::Punctuation(Punctuation::OpenBrace) => (Delimiter::Brace, false),
        Kind::Punctuation(Punctuation::CloseBrace) => (Delimiter::Brace, true),
        _ => return None,
    })
}
