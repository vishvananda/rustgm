//! Crate, module, associated and local item grammar over the shared body parser.
use super::{
    Ast, BlockFlavor, Kind, LexError, List, NORMAL, NodeId, P, Parser, Span, TokenKind, error,
};
use crate::syntax::item::{FunctionFlags, Item, ItemKind, Safety, Shape, Visibility};

pub(super) use crate::syntax::item::ItemContext as Scope;

/// # Errors
/// Rejects unsupported/malformed crate syntax and excessive recursive nesting.
pub fn parse_crate(source: &str) -> Result<Ast<'_>, LexError> {
    let mut parser = Parser::new(source, true)?;
    let attrs = parser.attributes(true)?;
    let items = parser.item_list(false, Scope::Module)?;
    let root = parser.node(0, Kind::Crate(items))?;
    parser.ast.root = parser.attach(attrs, root)?;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(parser.ast)
}

use super::fragment::FragmentKind as F;

impl Parser<'_, '_> {
    pub(super) fn item_list(&mut self, braced: bool, scope: Scope) -> Result<List, LexError> {
        let mark = self.scratch.len();
        loop {
            if braced && self.eat(P::CloseBrace)? {
                break;
            }
            if self.cursor.peek(0)?.is_none() {
                if braced {
                    return Err(self.error("unclosed item body")?);
                }
                break;
            }
            let item = self.item_in(scope)?;
            self.scratch.push(item);
        }
        self.list(mark)
    }

    pub(super) fn starts_item(&mut self) -> Result<bool, LexError> {
        let Some(token) = self.cursor.peek(0)? else {
            return Ok(false);
        };
        if matches!(token.kind, TokenKind::Interpolated(_)) {
            return Ok(self.splice()?.is_some_and(|s| {
                s.syntax_kind() == Some(F::Item)
                    || matches!(
                        s.fragment,
                        crate::syntax::interpolation::Nonterminal::Visibility { .. }
                    )
            }));
        }
        if token.kind != TokenKind::Identifier {
            return Ok(false);
        }
        let spelling = self.cursor.spelling(token);
        if spelling == "union" {
            return Ok(self.cursor.peek(1)?.is_some_and(|t| {
                t.kind == TokenKind::Identifier || t.kind == TokenKind::RawIdentifier
            }));
        }
        if spelling == "macro_rules" {
            return Ok(self
                .cursor
                .peek(1)?
                .is_some_and(|t| t.kind == TokenKind::Punctuation(P::Bang))
                && self.cursor.peek(2)?.is_some_and(|t| {
                    t.kind == TokenKind::Identifier || t.kind == TokenKind::RawIdentifier
                }));
        }
        if matches!(
            spelling,
            "fn" | "struct"
                | "enum"
                | "union"
                | "trait"
                | "impl"
                | "mod"
                | "use"
                | "type"
                | "static"
                | "extern"
                | "pub"
                | "macro_rules"
        ) {
            return Ok(true);
        }
        if matches!(spelling, "unsafe" | "async" | "const" | "safe") {
            return Ok(!self
                .cursor
                .peek(1)?
                .is_some_and(|next| next.kind == TokenKind::Punctuation(P::OpenBrace))
                && (spelling == "const"
                    || self.cursor.peek(1)?.is_some_and(|next| {
                        next.kind == TokenKind::Identifier
                            && matches!(
                                self.cursor.spelling(next),
                                "fn" | "trait" | "impl" | "extern" | "unsafe"
                            )
                    })));
        }
        Ok(false)
    }

    pub(super) fn item_in(&mut self, scope: Scope) -> Result<NodeId, LexError> {
        let previous = self.item_scope;
        self.item_scope = scope;
        let result = self.item();
        self.item_scope = previous;
        result
    }

    fn item(&mut self) -> Result<NodeId, LexError> {
        if self.depth >= self.nesting_limit() {
            return Err(self.error(if self.extended {
                "crate syntax nesting exceeds 128"
            } else {
                "syntax nesting exceeds 256"
            })?);
        }
        self.depth += 1;
        let result = self.item_inner();
        self.depth -= 1;
        result
    }

    #[expect(
        clippy::too_many_lines,
        reason = "item introducer dispatch shares visibility and attribute ownership and stays within the file audit limit"
    )]
    fn item_inner(&mut self) -> Result<NodeId, LexError> {
        let attrs = self.attributes(false)?;
        if let Some(node) = self.interpolated(&[F::Item])? {
            let mut owner = node;
            while let Kind::Attributed { value, .. } = self.ast.node(owner).kind {
                owner = value;
            }
            let Kind::Item(index) = self.ast.node(owner).kind else {
                return Err(self.error("opaque item has no item root")?);
            };
            let item = self.ast.items[index as usize];
            self.validate_item(&item)?;
            return self.attach(attrs, node);
        }
        let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
        let visibility = self.visibility()?;
        let mut item = Item {
            name: None,
            visibility,
            generics: Self::empty_list(),
            where_clause: Self::empty_list(),
            kind: ItemKind::Module { items: None },
        };
        let mut inner = crate::syntax::item::AttributeRange::default();
        let introducer = self
            .cursor
            .peek(0)?
            .ok_or_else(|| error(self.cursor.end(), "expected item"))?;
        item.kind = if self.word("struct")?
            || (self.cursor.word("union")?
                && self.cursor.peek(1)?.is_some_and(|t| {
                    matches!(t.kind, TokenKind::Identifier | TokenKind::RawIdentifier)
                })
                && self.word("union")?)
            || self.word("enum")?
        {
            self.aggregate_item(self.cursor.spelling(introducer), &mut item)?
        } else if self.word("type")? {
            item.name = Some(self.name()?);
            item.generics = self.generics()?;
            let bounds = if self.eat(P::Colon)? {
                self.bounds()?
            } else {
                Self::empty_list()
            };
            item.where_clause = self.where_clause()?;
            let ty = if self.eat(P::Equal)? {
                Some(self.ty()?)
            } else {
                None
            };
            if self.cursor.word("where")? {
                item.where_clause = self.where_clause()?;
            }
            self.expect(P::Semicolon)?;
            ItemKind::TypeAlias { bounds, ty }
        } else if self.word("mod")? {
            item.name = Some(self.name()?);
            let items = if self.eat(P::Semicolon)? {
                None
            } else {
                self.expect(P::OpenBrace)?;
                inner = self.attributes(true)?;
                Some(self.item_list(true, Scope::Module)?)
            };
            ItemKind::Module { items }
        } else if self.word("use")? {
            let tree = self.use_tree()?;
            self.expect(P::Semicolon)?;
            ItemKind::Use(tree)
        } else if self.word("macro_rules")? {
            self.expect(P::Bang)?;
            item.name = Some(self.name()?);
            let first = self.required()?;
            if !matches!(
                first.kind,
                TokenKind::Punctuation(P::OpenBrace | P::OpenParen | P::OpenBracket)
            ) {
                return Err(error(first.start, "expected macro definition delimiter"));
            }
            let forest = self.cursor.capture(first)?;
            if first.kind != TokenKind::Punctuation(P::OpenBrace) {
                self.expect(P::Semicolon)?;
            }
            let capture = u32::try_from(self.ast.captures.len())
                .map_err(|_| error(start, "too many captures"))?;
            self.ast.captures.push(forest);
            ItemKind::MacroRules { capture }
        } else if self.cursor.word("static")?
            || (self.cursor.word("const")? && !self.function_after_const()?)
        {
            self.value_item(&mut item)?
        } else {
            let constant = self.word("const")?;
            let asynchronous = self.word("async")?;
            let unsafe_ = self.word("unsafe")?;
            let safe = self.cursor.word("safe")?
                && self
                    .cursor
                    .peek(1)?
                    .is_some_and(|t| self.cursor.spelling(t) == "fn")
                && self.word("safe")?;
            if self.word("trait")? {
                if constant || asynchronous || safe {
                    return Err(error(start, "invalid trait qualifier"));
                }
                item.name = Some(self.name()?);
                item.generics = self.generics()?;
                let bounds = if self.eat(P::Colon)? {
                    self.bounds()?
                } else {
                    Self::empty_list()
                };
                item.where_clause = self.where_clause()?;
                self.expect(P::OpenBrace)?;
                inner = self.attributes(true)?;
                ItemKind::Trait {
                    unsafe_,
                    bounds,
                    items: self.item_list(true, Scope::Trait)?,
                }
            } else if self.word("impl")? {
                if constant || asynchronous || safe {
                    return Err(error(start, "invalid impl qualifier"));
                }
                item.generics = self.generics()?;
                let negative = self.eat(P::Bang)?;
                let first = self.ty()?;
                let (trait_path, ty) = if self.word("for")? {
                    (Some(first), self.ty()?)
                } else {
                    (None, first)
                };
                item.where_clause = self.where_clause()?;
                self.expect(P::OpenBrace)?;
                inner = self.attributes(true)?;
                ItemKind::Impl {
                    unsafe_,
                    negative,
                    trait_path,
                    ty,
                    items: self.item_list(true, Scope::Impl)?,
                }
            } else {
                let external = self.word("extern")?;
                if external && self.word("crate")? {
                    item.name = Some(if self.cursor.word("self")? {
                        let token = self.required()?;
                        Span {
                            start: token.start,
                            end: token.end,
                        }
                    } else {
                        self.name()?
                    });
                    let rename = if self.word("as")? {
                        Some(self.alias_name()?)
                    } else {
                        None
                    };
                    self.expect(P::Semicolon)?;
                    ItemKind::ExternCrate { rename }
                } else {
                    let abi = if external { self.abi()? } else { None };
                    if external && self.eat(P::OpenBrace)? {
                        inner = self.attributes(true)?;
                        ItemKind::ExternBlock {
                            unsafe_,
                            abi,
                            items: self.item_list(true, Scope::External)?,
                        }
                    } else if self.word("fn")? {
                        item.name = Some(self.name()?);
                        item.generics = self.generics()?;
                        let parameters = self.function_parameters(false)?;
                        let result = if self.eat(P::Arrow)? {
                            Some(self.ty()?)
                        } else {
                            None
                        };
                        item.where_clause = self.where_clause()?;
                        let body = if self.eat(P::Semicolon)? {
                            None
                        } else {
                            Some(self.block(BlockFlavor::Plain)?)
                        };
                        ItemKind::Function {
                            flags: FunctionFlags {
                                constant,
                                asynchronous,
                                safety: if unsafe_ {
                                    Safety::Unsafe
                                } else if safe {
                                    Safety::Safe
                                } else {
                                    Safety::Default
                                },
                            },
                            abi,
                            parameters,
                            result,
                            body,
                        }
                    } else if constant || asynchronous || unsafe_ || safe || external {
                        return Err(self.error("expected item after qualifiers")?);
                    } else {
                        let path = self.rich_path(false)?;
                        self.expect(P::Bang)?;
                        let value = self.macro_capture(
                            path,
                            crate::syntax::ast::MacroContext::Items(self.item_scope),
                        )?;
                        if !self.with_block(value) {
                            self.expect(P::Semicolon)?;
                        }
                        ItemKind::MacroInvocation(value)
                    }
                }
            }
        };
        self.validate_item(&item)?;
        let id = u32::try_from(self.ast.items.len()).map_err(|_| error(start, "too many items"))?;
        self.ast.items.push(item);
        let node = self.node(start, Kind::Item(id))?;
        let node = self.attach(inner, node)?;
        self.attach(attrs, node)
    }

    fn validate_item(&mut self, item: &Item) -> Result<(), LexError> {
        let valid = match self.item_scope {
            Scope::Module => true,
            Scope::Trait | Scope::Impl => matches!(
                item.kind,
                ItemKind::Function { .. }
                    | ItemKind::Const { .. }
                    | ItemKind::TypeAlias { .. }
                    | ItemKind::MacroInvocation(_)
            ),
            Scope::External => matches!(
                item.kind,
                ItemKind::Function { .. } | ItemKind::Static { .. } | ItemKind::MacroInvocation(_)
            ),
        };
        if !valid {
            return Err(self.error("item is not permitted in this owner")?);
        }
        let missing = match item.kind {
            ItemKind::Function { body, .. } => body.is_none(),
            ItemKind::Const { value, .. } | ItemKind::Static { value, .. } => value.is_none(),
            ItemKind::TypeAlias { ty, .. } => ty.is_none(),
            _ => false,
        };
        if missing && matches!(self.item_scope, Scope::Module | Scope::Impl) {
            return Err(self.error("item requires a definition")?);
        }
        if self.item_scope == Scope::External
            && !missing
            && matches!(
                item.kind,
                ItemKind::Function { .. } | ItemKind::Static { .. }
            )
        {
            return Err(self.error("external item cannot have a definition")?);
        }
        Ok(())
    }

    pub(super) fn visibility(&mut self) -> Result<Visibility, LexError> {
        if let Some(splice) = self.splice()?
            && matches!(
                splice.fragment,
                crate::syntax::interpolation::Nonterminal::Visibility { .. }
            )
        {
            self.required()?;
            return splice.visibility(self.cursor.source, &mut self.ast);
        }
        if !self.word("pub")? {
            return Ok(Visibility::Private);
        }
        if !self.eat(P::OpenParen)? {
            return Ok(Visibility::Public);
        }
        let explicit = self.word("in")?;
        if !(explicit
            || self.cursor.word("self")?
            || self.cursor.word("super")?
            || self.cursor.word("crate")?)
        {
            return Err(self.error("expected visibility restriction")?);
        }
        let path = self.simple_path()?;
        if !explicit
            && matches!(self.ast.node(path).kind, Kind::Path { segments, .. } if self.ast.list(segments).len() != 1)
        {
            return Err(self.error("nested visibility path requires in")?);
        }
        self.expect(P::CloseParen)?;
        Ok(Visibility::Restricted(path))
    }

    fn function_after_const(&mut self) -> Result<bool, LexError> {
        Ok(self.cursor.peek(1)?.is_some_and(|t| {
            t.kind == TokenKind::Identifier
                && matches!(
                    self.cursor.spelling(t),
                    "fn" | "async" | "unsafe" | "extern"
                )
        }))
    }

    fn value_item(&mut self, item: &mut Item) -> Result<ItemKind, LexError> {
        let static_ = self.word("static")?;
        if !static_ {
            self.required()?;
        }
        let mutable = static_ && self.word("mut")?;
        item.name = Some(self.alias_name()?);
        self.expect(P::Colon)?;
        let ty = self.ty()?;
        let value = if self.eat(P::Equal)? {
            Some(self.expr(0, NORMAL)?)
        } else {
            None
        };
        self.expect(P::Semicolon)?;
        Ok(if static_ {
            ItemKind::Static { mutable, ty, value }
        } else {
            ItemKind::Const { ty, value }
        })
    }

    pub(super) fn abi(&mut self) -> Result<Option<NodeId>, LexError> {
        if self
            .cursor
            .peek(0)?
            .is_some_and(|t| matches!(t.kind, TokenKind::String | TokenKind::RawString))
        {
            Ok(Some(self.prefix(NORMAL)?))
        } else {
            Ok(None)
        }
    }

    pub(super) fn function_parameters(&mut self, pointer: bool) -> Result<List, LexError> {
        self.expect(P::OpenParen)?;
        let mark = self.scratch.len();
        while !self.eat(P::CloseParen)? {
            let attrs = self.attributes(false)?;
            let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
            let parameter = if self.eat(P::DotDotDot)? {
                self.node(start, Kind::Variadic)?
            } else if pointer {
                let name = if self
                    .cursor
                    .peek(1)?
                    .is_some_and(|t| t.kind == TokenKind::Punctuation(P::Colon))
                {
                    let name = self.alias_name()?;
                    self.expect(P::Colon)?;
                    Some(name)
                } else {
                    None
                };
                let ty = self.ty()?;
                self.node(
                    start,
                    Kind::FieldDefinition {
                        name,
                        visibility: Visibility::Private,
                        ty,
                    },
                )?
            } else {
                let pattern = if self.cursor.at(P::And)?
                    && self.cursor.peek(1)?.is_some_and(|t| {
                        matches!(t.kind, TokenKind::Lifetime | TokenKind::RawLifetime)
                    }) {
                    self.required()?;
                    let lifetime = self.required()?;
                    let mutable = self.word("mut")?;
                    let name = self
                        .cursor
                        .peek(0)?
                        .ok_or_else(|| error(self.cursor.end(), "expected receiver"))?;
                    if !self.word("self")? {
                        return Err(self.error("reference lifetime in a parameter requires self")?);
                    }
                    self.node(
                        start,
                        Kind::Receiver {
                            name: Span {
                                start: name.start,
                                end: name.end,
                            },
                            lifetime: Span {
                                start: lifetime.start,
                                end: lifetime.end,
                            },
                            mutable,
                        },
                    )?
                } else {
                    self.pattern(false)?
                };
                let ty = if self.eat(P::Colon)? {
                    Some(if self.eat(P::DotDotDot)? {
                        self.node(start, Kind::Variadic)?
                    } else {
                        self.ty()?
                    })
                } else if self.self_pattern(pattern) {
                    None
                } else {
                    return Err(self.error("function parameter requires a type")?);
                };
                self.node(start, Kind::Parameter { pattern, ty })?
            };
            let variadic = match self.ast.node(parameter).kind {
                Kind::Variadic => true,
                Kind::Parameter { ty: Some(ty), .. } => {
                    matches!(self.ast.node(ty).kind, Kind::Variadic)
                }
                _ => false,
            };
            let parameter = self.attach(attrs, parameter)?;
            self.scratch.push(parameter);
            if self.eat(P::CloseParen)? {
                break;
            }
            self.expect(P::Comma)?;
            if variadic {
                self.expect(P::CloseParen)?;
                break;
            }
        }
        self.list(mark)
    }

    fn self_pattern(&self, pattern: NodeId) -> bool {
        match self.ast.node(pattern).kind {
            Kind::Receiver { .. } => true,
            Kind::Binding { name, .. } => {
                &self.cursor.source[name.start as usize..name.end as usize] == "self"
            }
            Kind::ReferencePattern { pattern, .. } => self.self_pattern(pattern),
            _ => false,
        }
    }

    fn aggregate_item(&mut self, category: &str, item: &mut Item) -> Result<ItemKind, LexError> {
        item.name = Some(self.name()?);
        item.generics = self.generics()?;
        if category == "struct" && self.eat(P::OpenParen)? {
            let fields = self.fields(Shape::Tuple)?;
            item.where_clause = self.where_clause()?;
            self.expect(P::Semicolon)?;
            return Ok(ItemKind::Struct {
                shape: Shape::Tuple,
                fields,
            });
        }
        item.where_clause = self.where_clause()?;
        if category == "struct" && self.eat(P::Semicolon)? {
            return Ok(ItemKind::Struct {
                shape: Shape::Unit,
                fields: Self::empty_list(),
            });
        }
        self.expect(P::OpenBrace)?;
        if category == "enum" {
            let mark = self.scratch.len();
            while !self.eat(P::CloseBrace)? {
                let attrs = self.attributes(false)?;
                let name = self.name()?;
                let (shape, fields) = if self.eat(P::OpenParen)? {
                    (Shape::Tuple, self.fields(Shape::Tuple)?)
                } else if self.eat(P::OpenBrace)? {
                    (Shape::Record, self.fields(Shape::Record)?)
                } else {
                    (Shape::Unit, Self::empty_list())
                };
                let discriminant = if self.eat(P::Equal)? {
                    Some(self.expr(0, NORMAL)?)
                } else {
                    None
                };
                let index = u32::try_from(self.ast.variants.len())
                    .map_err(|_| error(name.start, "too many enum variants"))?;
                self.ast.variants.push(crate::syntax::item::Variant {
                    name,
                    shape,
                    fields,
                    discriminant,
                });
                let variant = self.node(name.start, Kind::Variant(index))?;
                let variant = self.attach(attrs, variant)?;
                self.scratch.push(variant);
                if self.eat(P::CloseBrace)? {
                    break;
                }
                self.expect(P::Comma)?;
            }
            Ok(ItemKind::Enum {
                variants: self.list(mark)?,
            })
        } else {
            let fields = self.fields(Shape::Record)?;
            Ok(if category == "union" {
                ItemKind::Union { fields }
            } else {
                ItemKind::Struct {
                    shape: Shape::Record,
                    fields,
                }
            })
        }
    }

    fn fields(&mut self, shape: Shape) -> Result<List, LexError> {
        let mark = self.scratch.len();
        let close = if shape == Shape::Tuple {
            P::CloseParen
        } else {
            P::CloseBrace
        };
        while !self.eat(close)? {
            let attrs = self.attributes(false)?;
            let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
            let visibility = self.visibility()?;
            let name = if shape == Shape::Record {
                let name = self.name()?;
                self.expect(P::Colon)?;
                Some(name)
            } else {
                None
            };
            let ty = self.ty()?;
            let field = self.node(
                start,
                Kind::FieldDefinition {
                    name,
                    visibility,
                    ty,
                },
            )?;
            let field = self.attach(attrs, field)?;
            self.scratch.push(field);
            if self.eat(close)? {
                break;
            }
            self.expect(P::Comma)?;
        }
        self.list(mark)
    }

    fn alias_name(&mut self) -> Result<Span, LexError> {
        if self.cursor.word("_")? {
            let token = self.required()?;
            Ok(Span {
                start: token.start,
                end: token.end,
            })
        } else {
            self.name()
        }
    }

    fn use_tree(&mut self) -> Result<NodeId, LexError> {
        if self.depth >= self.nesting_limit() {
            return Err(self.error(if self.extended {
                "crate syntax nesting exceeds 128"
            } else {
                "syntax nesting exceeds 256"
            })?);
        }
        self.depth += 1;
        let result = self.use_tree_inner();
        self.depth -= 1;
        result
    }
    fn use_tree_inner(&mut self) -> Result<NodeId, LexError> {
        let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
        let mark = self.scratch.len();
        let absolute = self.eat(P::ColonColon)?;
        let mut prefix = None;
        let mut separator = true;
        if !self.cursor.at(P::Star)? && !self.cursor.at(P::OpenBrace)? {
            loop {
                let token = self.required()?;
                let text = self.cursor.spelling(token);
                if !(super::keyword::identifier(
                    token.kind,
                    text.strip_prefix("r#").unwrap_or(text),
                ) || matches!(text, "self" | "super" | "crate"))
                {
                    return Err(error(token.start, "expected use path"));
                }
                let name = self.node(
                    token.start,
                    Kind::Name(Span {
                        start: token.start,
                        end: token.end,
                    }),
                )?;
                self.scratch.push(name);
                separator = self.eat(P::ColonColon)?;
                if !separator || self.cursor.at(P::Star)? || self.cursor.at(P::OpenBrace)? {
                    break;
                }
            }
            let segments = self.list(mark)?;
            prefix = Some(self.node(start, Kind::Path { absolute, segments })?);
        }
        if absolute && prefix.is_none() {
            prefix = Some(self.node(
                start,
                Kind::Path {
                    absolute: true,
                    segments: Self::empty_list(),
                },
            )?);
        }
        if !separator && (self.cursor.at(P::Star)? || self.cursor.at(P::OpenBrace)?) {
            return Err(self.error("use group or glob requires a path separator")?);
        }
        let glob = self.eat(P::Star)?;
        let grouped = self.eat(P::OpenBrace)?;
        let children = if grouped {
            let mark = self.scratch.len();
            while !self.eat(P::CloseBrace)? {
                let child = self.use_tree()?;
                self.scratch.push(child);
                if self.eat(P::CloseBrace)? {
                    break;
                }
                self.expect(P::Comma)?;
            }
            self.list(mark)?
        } else {
            Self::empty_list()
        };
        let rename = if self.word("as")? {
            if glob || grouped {
                return Err(self.error("use group or glob cannot be renamed")?);
            }
            Some({
                let name = self.alias_name()?;
                self.node(name.start, Kind::Name(name))?
            })
        } else {
            None
        };
        let index = u32::try_from(self.ast.use_trees.len())
            .map_err(|_| error(start, "too many use trees"))?;
        self.ast.use_trees.push(crate::syntax::item::UseTree {
            prefix,
            children,
            rename,
            glob,
        });
        self.node(start, Kind::UseTree(index))
    }

    pub(super) fn macro_capture(
        &mut self,
        path: NodeId,
        context: crate::syntax::ast::MacroContext,
    ) -> Result<NodeId, LexError> {
        let first = self.required()?;
        if !matches!(
            first.kind,
            TokenKind::Punctuation(P::OpenBrace | P::OpenParen | P::OpenBracket)
        ) {
            return Err(error(first.start, "expected macro delimiter"));
        }
        let forest = self.cursor.capture(first)?;
        let capture = u32::try_from(self.ast.captures.len())
            .map_err(|_| error(first.start, "too many captures"))?;
        self.ast.captures.push(forest);
        self.node(
            self.ast.node(path).span.start,
            Kind::Macro {
                path,
                capture,
                context,
            },
        )
    }
}
