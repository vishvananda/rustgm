//! Crate, module, associated and local item grammar over the shared body parser.
use super::{
    Ast, BlockFlavor, Kind, LexError, List, NORMAL, NodeId, P, Parser, Span, TokenKind, Word, error,
};
use crate::syntax::item::{FunctionFlags, Item, ItemKind, Safety, Shape, Visibility};
mod declarative_forms;

pub(super) use crate::syntax::item::ItemContext as Scope;

/// # Errors
/// Rejects unsupported/malformed crate syntax and excessive recursive nesting.
pub fn parse_crate(source: &str) -> Result<Ast<'_>, LexError> {
    parse_crate_with(source, None)
}

/// Parse a crate, interning each name once into the crate's shared table.
///
/// # Errors
/// Rejects malformed crate syntax, excessive nesting and name-table overflow.
pub fn parse_crate_with<'s>(
    source: &'s str,
    names: Option<&mut crate::support::names::Names>,
) -> Result<Ast<'s>, LexError> {
    parse_crate_in_edition(source, names, crate::support::edition::Edition::Rust2024)
}

/// Parse source with an explicit token edition. This syntax entrypoint does not
/// enable a complete semantic or driver profile for that edition.
///
/// # Errors
/// Rejects malformed syntax, excessive nesting and name-table overflow.
pub fn parse_crate_in_edition<'s>(
    source: &'s str,
    names: Option<&mut crate::support::names::Names>,
    edition: crate::support::edition::Edition,
) -> Result<Ast<'s>, LexError> {
    let mut parser = Parser::new(source, true)?;
    parser.names = names;
    parser.inputs = &edition;
    let attrs = parser.attributes(true)?;
    parser.item_attributes(attrs);
    let items = parser.item_list(false, Scope::Module)?;
    let root = parser.node(0, Kind::Crate(items))?;
    parser.ast.root = parser.attach(attrs, root)?;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(parser.ast)
}

use super::fragment::FragmentKind as F;

impl Parser<'_, '_> {
    /// A standalone `!` is an inherent Self type, before its body or where clause.
    fn negative_impl(&mut self) -> Result<bool, LexError> {
        if !self.cursor.at(P::Bang)? {
            return Ok(false);
        }
        if self
            .cursor
            .peek(1)?
            .is_some_and(|token| token.kind == TokenKind::Punctuation(P::OpenBrace))
            || self.cursor.peek_word(1)? == Word::Where
        {
            return Ok(false);
        }
        self.eat(P::Bang)
    }

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
        let Some((token, word)) = self.cursor.current()? else {
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
        Ok(match word {
            Word::Auto => self.cursor.peek_word(1)? == Word::Trait,
            Word::Union => self.cursor.peek(1)?.is_some_and(|t| {
                t.kind == TokenKind::Identifier || t.kind == TokenKind::RawIdentifier
            }),
            Word::MacroRules => {
                self.cursor
                    .peek(1)?
                    .is_some_and(|t| t.kind == TokenKind::Punctuation(P::Bang))
                    && self.cursor.peek(2)?.is_some_and(|t| {
                        t.kind == TokenKind::Identifier || t.kind == TokenKind::RawIdentifier
                    })
            }
            Word::Fn
            | Word::Macro
            | Word::Struct
            | Word::Enum
            | Word::Trait
            | Word::Impl
            | Word::Mod
            | Word::Use
            | Word::Type
            | Word::Static
            | Word::Extern
            | Word::Pub => true,
            Word::Unsafe | Word::Async | Word::Const | Word::Safe => {
                !self
                    .cursor
                    .peek(1)?
                    .is_some_and(|next| next.kind == TokenKind::Punctuation(P::OpenBrace))
                    && (word == Word::Const
                        || matches!(
                            self.cursor.peek_word(1)?,
                            Word::Fn | Word::Trait | Word::Impl | Word::Extern | Word::Unsafe
                        ))
            }
            _ => false,
        })
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
        let configured = self.configured_items;
        let result = self.item_inner();
        self.configured_items = configured;
        self.depth -= 1;
        result
    }

    #[expect(
        clippy::too_many_lines,
        reason = "item introducer dispatch shares visibility and attribute ownership and stays within the file audit limit"
    )]
    fn item_inner(&mut self) -> Result<NodeId, LexError> {
        let attrs = self.attributes(false)?;
        let intrinsic = self.item_attributes(attrs);
        if let Some(node) = self.interpolated(&[F::Item])? {
            self.validate_spliced_item(node, intrinsic)?;
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
        if self.cursor.peek(0)?.is_none() {
            return Err(error(self.cursor.end(), "expected item"));
        }
        item.kind = if self.eat_word(Word::Struct)?
            || (self.cursor.at_word(Word::Union)?
                && self.cursor.peek(1)?.is_some_and(|t| {
                    matches!(t.kind, TokenKind::Identifier | TokenKind::RawIdentifier)
                })
                && self.eat_word(Word::Union)?)
            || self.eat_word(Word::Enum)?
        {
            self.aggregate_item(self.cursor.last_word(), &mut item)?
        } else if self.eat_word(Word::Type)? {
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
            if self.cursor.at_word(Word::Where)? {
                item.where_clause = self.where_clause()?;
            }
            self.expect(P::Semicolon)?;
            ItemKind::TypeAlias { bounds, ty }
        } else if self.eat_word(Word::Mod)? {
            item.name = Some(self.name()?);
            let items = if self.eat(P::Semicolon)? {
                None
            } else {
                self.expect(P::OpenBrace)?;
                inner = self.attributes(true)?;
                self.item_attributes(inner);
                Some(self.item_list(true, Scope::Module)?)
            };
            ItemKind::Module { items }
        } else if self.eat_word(Word::Use)? {
            let tree = self.use_tree()?;
            self.expect(P::Semicolon)?;
            ItemKind::Use(tree)
        } else if self.eat_word(Word::MacroRules)? || self.eat_word(Word::Macro)? {
            let scoped = self.cursor.last_word() == Word::Macro;
            if !scoped {
                self.expect(P::Bang)?;
            }
            item.name = Some(self.name()?);
            self.macro_definition(scoped)?
        } else if self.cursor.at_word(Word::Static)?
            || (self.cursor.at_word(Word::Const)? && !self.function_after_const()?)
        {
            self.value_item(&mut item, Safety::Default)?
        } else {
            let restriction = self.impl_restriction()?;
            let constant = self.eat_word(Word::Const)?;
            let asynchronous = self.eat_word(Word::Async)?;
            let unsafe_ = self.eat_word(Word::Unsafe)?;
            let safe = self.cursor.at_word(Word::Safe)?
                && self
                    .cursor
                    .peek(1)?
                    .is_some_and(|t| matches!(self.cursor.spelling(t), "fn" | "static"))
                && self.eat_word(Word::Safe)?;
            let auto_ = self.auto_qualifier(start, (constant, asynchronous, safe))?;
            if restriction.is_some() && !self.cursor.at_word(Word::Trait)? {
                return Err(error(start, "implementation restriction requires a trait"));
            }
            let safety = if unsafe_ {
                Safety::Unsafe
            } else if safe {
                Safety::Safe
            } else {
                Safety::Default
            };
            if self.cursor.at_word(Word::Static)? && (unsafe_ || safe) {
                if constant || asynchronous {
                    return Err(error(start, "invalid static qualifier"));
                }
                self.value_item(&mut item, safety)?
            } else if self.eat_word(Word::Trait)? {
                if asynchronous || safe {
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
                if restriction.is_some() && self.cursor.at(P::Eq)? {
                    return Err(error(
                        start,
                        "trait aliases cannot have implementation restrictions",
                    ));
                }
                self.expect(P::OpenBrace)?;
                inner = self.attributes(true)?;
                self.item_attributes(inner);
                ItemKind::Trait {
                    restriction,
                    auto_,
                    unsafe_,
                    constant,
                    bounds,
                    items: self.item_list(true, Scope::Trait)?,
                }
            } else if self.eat_word(Word::Impl)? {
                if asynchronous || safe {
                    return Err(error(start, "invalid impl qualifier"));
                }
                item.generics = self.generics()?;
                let negative = self.negative_impl()?;
                let first = self.ty()?;
                let (trait_path, ty) = if self.eat_word(Word::For)? {
                    (Some(first), self.ty()?)
                } else {
                    (None, first)
                };
                item.where_clause = self.where_clause()?;
                self.expect(P::OpenBrace)?;
                inner = self.attributes(true)?;
                self.item_attributes(inner);
                ItemKind::Impl {
                    unsafe_,
                    constant,
                    negative,
                    trait_path,
                    ty,
                    items: self.item_list(true, Scope::Impl)?,
                }
            } else {
                let external = self.eat_word(Word::Extern)?;
                if external && self.eat_word(Word::Crate)? {
                    item.name = Some(if self.cursor.at_word(Word::SelfValue)? {
                        let token = self.required()?;
                        Span {
                            start: token.start,
                            end: token.end,
                        }
                    } else {
                        self.name()?
                    });
                    let rename = if self.eat_word(Word::As)? {
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
                        self.item_attributes(inner);
                        ItemKind::ExternBlock {
                            unsafe_,
                            abi,
                            items: self.item_list(true, Scope::External)?,
                        }
                    } else if self.eat_word(Word::Fn)? {
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
                                safety,
                            },
                            abi: if external {
                                crate::syntax::item::Abi::Extern(abi)
                            } else {
                                crate::syntax::item::Abi::Rust
                            },
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
        self.validate_item(&item, intrinsic)?;
        let id = u32::try_from(self.ast.items.len()).map_err(|_| error(start, "too many items"))?;
        self.ast.items.push(item);
        let node = self.node(start, Kind::Item(id))?;
        let node = self.attach(inner, node)?;
        self.attach(attrs, node)
    }

    fn auto_qualifier(&mut self, start: u32, flags: (bool, bool, bool)) -> Result<bool, LexError> {
        let auto_ = self.eat_word(Word::Auto)?;
        if auto_ && (!self.cursor.at_word(Word::Trait)? || flags.0 || flags.1 || flags.2) {
            return Err(error(
                start,
                "auto qualifier requires an ordinary trait declaration",
            ));
        }
        Ok(auto_)
    }

    fn validate_spliced_item(
        &mut self,
        mut owner: NodeId,
        mut intrinsic: bool,
    ) -> Result<(), LexError> {
        while let Kind::Attributed { value, attributes } = self.ast.node(owner).kind {
            intrinsic |= self.item_attributes(attributes);
            owner = value;
        }
        let Kind::Item(index) = self.ast.node(owner).kind else {
            return Err(self.error("opaque item has no item root")?);
        };
        let item = self.ast.items[index as usize];
        self.validate_item(&item, intrinsic)
    }
    fn validate_item(&mut self, item: &Item, intrinsic: bool) -> Result<(), LexError> {
        if self.item_scope != Scope::External
            && matches!(item.kind, ItemKind::Static { safety, .. } if safety != Safety::Default)
        {
            return Err(self.error("static safety qualifier requires a foreign declaration")?);
        }
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
        let provisional_function = matches!(item.kind, ItemKind::Function { .. })
            && (self.configured_items || self.item_scope == Scope::Module && intrinsic);
        if missing
            && matches!(self.item_scope, Scope::Module | Scope::Impl)
            && !provisional_function
        {
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
        if !self.eat_word(Word::Pub)? {
            return Ok(Visibility::Private);
        }
        if !self.eat(P::OpenParen)? {
            return Ok(Visibility::Public);
        }
        let explicit = self.eat_word(Word::In)?;
        if !(explicit
            || self.cursor.at_word(Word::SelfValue)?
            || self.cursor.at_word(Word::Super)?
            || self.cursor.at_word(Word::Crate)?)
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
        Ok(matches!(
            self.cursor.peek_word(1)?,
            Word::Fn | Word::Async | Word::Unsafe | Word::Extern | Word::Trait | Word::Impl
        ))
    }

    fn value_item(&mut self, item: &mut Item, safety: Safety) -> Result<ItemKind, LexError> {
        let static_ = self.eat_word(Word::Static)?;
        if !static_ {
            self.required()?;
        }
        let mutable = static_ && self.eat_word(Word::Mut)?;
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
            ItemKind::Static {
                safety,
                mutable,
                ty,
                value,
            }
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
                    let mutable = self.eat_word(Word::Mut)?;
                    let name = self
                        .cursor
                        .peek(0)?
                        .ok_or_else(|| error(self.cursor.end(), "expected receiver"))?;
                    if !self.eat_word(Word::SelfValue)? {
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

    /// `category` is the consumed `struct`, `union` or `enum` introducer.
    fn aggregate_item(&mut self, category: Word, item: &mut Item) -> Result<ItemKind, LexError> {
        item.name = Some(self.name()?);
        item.generics = self.generics()?;
        if category == Word::Struct && self.eat(P::OpenParen)? {
            let fields = self.fields(Shape::Tuple)?;
            item.where_clause = self.where_clause()?;
            self.expect(P::Semicolon)?;
            return Ok(ItemKind::Struct {
                shape: Shape::Tuple,
                fields,
            });
        }
        item.where_clause = self.where_clause()?;
        if category == Word::Struct && self.eat(P::Semicolon)? {
            return Ok(ItemKind::Struct {
                shape: Shape::Unit,
                fields: Self::empty_list(),
            });
        }
        self.expect(P::OpenBrace)?;
        if category == Word::Enum {
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
            Ok(if category == Word::Union {
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
        if self.cursor.at_word(Word::Underscore)? {
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
    fn use_path_segment(&mut self, first: bool) -> Result<NodeId, LexError> {
        let token = self.required()?;
        let text = self.cursor.spelling(token);
        if !(self.identifier(token, self.cursor.last_word())?
            || matches!(text, "self" | "super" | "crate")
            || matches!(token.kind, TokenKind::CrateRoot(_)))
        {
            return Err(error(token.start, "expected use path"));
        }
        if let TokenKind::CrateRoot(id) = token.kind {
            if !first {
                return Err(error(token.start, "crate marker must begin a use path"));
            }
            return self.node(token.start, Kind::CrateRoot(id.get()));
        }
        self.name_node(token)
    }
    fn opaque_use_path(&mut self) -> Result<(NodeId, bool), LexError> {
        let path = self.rich_path(true)?;
        let node = self.ast.node(path);
        let Some((_, segments)) = self.ast.path_segments(path) else {
            return Err(error(node.span.start, "use requires a simple path"));
        };
        if segments.iter().any(|node| {
            !matches!(
                self.ast.node(*node).kind,
                Kind::Name(..) | Kind::CrateRoot(_)
            )
        }) {
            return Err(error(node.span.start, "use requires simple path segments"));
        }
        let separator = self.eat(P::ColonColon)?;
        if separator && !self.cursor.at(P::Star)? && !self.cursor.at(P::OpenBrace)? {
            return Err(self.error("opaque use path requires a group or glob after separator")?);
        }
        Ok((path, separator))
    }
    fn use_tree_inner(&mut self) -> Result<NodeId, LexError> {
        let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
        let mark = self.scratch.len();
        let opaque = self
            .splice()?
            .is_some_and(|s| matches!(s.syntax_kind(), Some(F::Path | F::Type)));
        let absolute = !opaque && self.eat(P::ColonColon)?;
        let mut prefix = None;
        let mut separator = true;
        if opaque {
            let (path, trailing_separator) = self.opaque_use_path()?;
            prefix = Some(path);
            separator = trailing_separator;
        } else if !self.cursor.at(P::Star)? && !self.cursor.at(P::OpenBrace)? {
            loop {
                let name = self.use_path_segment(!absolute && self.scratch.len() == mark)?;
                self.scratch.push(name);
                separator = self.eat(P::ColonColon)?;
                if !separator || self.cursor.at(P::Star)? || self.cursor.at(P::OpenBrace)? {
                    break;
                }
            }
            prefix = Some(
                if !absolute
                    && self.scratch.len() == mark + 1
                    && matches!(self.ast.node(self.scratch[mark]).kind, Kind::Name(..))
                {
                    // A relative path of one plain name is that name alone.
                    let name = self.scratch[mark];
                    self.scratch.truncate(mark);
                    name
                } else {
                    let segments = self.list(mark)?;
                    self.node(start, Kind::Path { absolute, segments })?
                },
            );
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
        let glob = if self.cursor.at(P::Star)? {
            let token = self.required()?;
            Some(Span {
                start: token.start,
                end: token.end,
            })
        } else {
            None
        };
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
        let rename = if self.eat_word(Word::As)? {
            if glob.is_some() || grouped {
                return Err(self.error("use group or glob cannot be renamed")?);
            }
            Some({
                let name = self.alias_name()?;
                self.name_span(name)?
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
