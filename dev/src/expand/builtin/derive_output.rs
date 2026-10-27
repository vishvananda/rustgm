//! Source Copy/paired Clone produce ordinary impls with real field WF assertions.
mod active_members;
mod comparisons;
mod debug_format;
mod derivation_contexts;
use super::{choice::Charge, delegation::Delegated, derive_types};
use crate::{
    expand::{
        config::Config,
        crate_syntax,
        marks::{ExpansionId, SyntaxContextId},
        stream::{Location, Stream},
    },
    lex::{
        Lexer,
        token::{Kind as TokenKind, LexError, Token},
    },
    literal::storage::StoredValue,
    syntax::{
        ast::{Ast, Kind, MacroContext, NodeId, Syntax},
        expression::fragment::{self, FragmentKind},
        interpolation::{Nonterminal, ParsedSyntax, Splice},
        item::{GenericFlavor, Item, ItemContext, ItemKind},
        token_tree::{Span, error},
    },
};
pub(crate) use derivation_contexts::{MEMBERS, MemberContext};

pub(crate) struct Site<'a> {
    pub file: crate::support::source::FileId,
    pub source: &'a str,
    pub ast: &'a Syntax<StoredValue>,
    pub active: &'a [bool],
    pub item: Item,
    pub original: Span,
    pub members: [Option<MemberContext>; MEMBERS.len()],
    pub const_: bool,
    pub prior_copy: bool,
}
struct Field {
    parsed: ParsedSyntax,
    original: Span,
    destination: Span,
}
struct Output<'a> {
    site: &'a Site<'a>,
    stream: Stream,
    fields: Vec<Field>,
    charge: &'a Charge<'a>,
    context: (SyntaxContextId, ExpansionId),
    edition: crate::support::edition::Edition,
}

pub(crate) fn build(
    site: &Site<'_>,
    config: &Config,
    charge: &Charge<'_>,
) -> Result<Delegated, LexError> {
    let fields = active_members::collect(site, charge)?;
    let copy = site.members[0].is_some();
    let clone = site.members[1].is_some();
    if clone && !(copy || site.prior_copy)
        || site.item.where_clause.start != site.item.where_clause.end
    {
        return Err(error(
            site.original.start,
            "fieldwise or constrained source Clone requires a wider derive profile",
        ));
    }
    let mut output = Output {
        site,
        stream: Stream::default(),
        fields: Vec::new(),
        charge,
        context: (SyntaxContextId::ROOT, ExpansionId(0)),
        edition: crate::support::edition::Edition::Rust2024,
    };
    if output.select(0) {
        output.header("crate::marker::Copy", false)?;
        output.text("{}")?;
    }
    if clone {
        output.select(1);
        output.header("crate::clone::TrivialClone", true)?;
        output.text("{}")?;
        output.header("crate::clone::Clone", false)?;
        output.text("{ #[inline] fn clone(&self) -> Self {")?;
        for &field in &fields {
            if site.active.get(field.get() as usize) != Some(&true) {
                continue;
            }
            let mut field = field;
            while let Kind::Attributed { value, .. } = site.ast.node(field).kind {
                field = value;
            }
            let Kind::FieldDefinition { ty, .. } = site.ast.node(field).kind else {
                return Err(error(
                    site.original.start,
                    "derived Clone lost an actual field declaration",
                ));
            };
            output.text("let _: crate::clone::AssertParamIsClone<")?;
            output.field(ty)?;
            output.text(">;")?;
        }
        output.text("*self } }")?;
    }
    if output.select(2) {
        output.debug(&fields)?;
    }
    output.comparison_members(&fields)?;
    output.finish(config)
}

impl Output<'_> {
    fn finish(self, config: &Config) -> Result<Delegated, LexError> {
        self.validate_origins()?;
        let splices: Vec<_> = self
            .fields
            .iter()
            .map(|field| Splice {
                fragment: Nonterminal::Syntax {
                    kind: FragmentKind::Type,
                    parsed: &field.parsed,
                },
                original_source: self.site.source,
                original: field.original,
                destination: field.destination,
            })
            .collect();
        let ast = fragment::parse_expansion(
            &self.stream.source,
            &self.stream.tokens,
            MacroContext::Items(ItemContext::Module),
            &splices,
        )?;
        (self.charge)(
            ast.nodes.len().saturating_add(ast.edges.len()),
            super::choice::syntax_payload(&ast).saturating_mul(4),
        )?;
        let configured = crate_syntax::configure(&self.stream.source, ast, config)?;
        // Configuration interprets demanded literals once in their generated image.
        // Only the new Debug labels are strings; copied field types retain splices.
        let ast: Ast<'static> = configured.ast.map_literals(|value| match value {
            crate::literal::Value::String(bytes) => {
                (self.charge)(bytes.len().saturating_add(1), bytes.len().saturating_mul(4))?;
                Ok(crate::literal::Value::String(bytes.into_owned().into()))
            }
            crate::literal::Value::Boolean(value) => Ok(crate::literal::Value::Boolean(value)),
            _ => Err(error(
                self.site.original.start,
                "source derive introduced an unsupported literal",
            )),
        })?;
        let syntax = crate_syntax::ConfiguredSyntax {
            ast,
            owners: configured.owners,
            active: configured.active,
            effects: configured.effects,
            cfg_uses: configured.cfg_uses,
        };
        Ok(Delegated {
            stream: self.stream,
            syntax,
        })
    }
    fn validate_origins(&self) -> Result<(), LexError> {
        use crate::expand::stream::OriginKind;
        (self.charge)(self.stream.origins.len(), 0)?;
        for origin in &self.stream.origins {
            let generated = self
                .stream
                .source
                .get(origin.generated.start as usize..origin.generated.end as usize)
                .ok_or_else(|| {
                    error(
                        self.site.original.start,
                        "derived origin exceeds its produced image",
                    )
                })?;
            match origin.kind {
                OriginKind::Copied => {
                    (self.charge)(generated.len(), 0)?;
                    if origin.context.is_some()
                        || origin.introduction.0 != 0
                        || origin.original.file != self.site.file
                        || self.site.source.get(
                            origin.original.span.start as usize..origin.original.span.end as usize,
                        ) != Some(generated)
                    {
                        return Err(error(
                            self.site.original.start,
                            "derived copied origin changed its actual bytes",
                        ));
                    }
                }
                OriginKind::Synthetic(edition) => {
                    if origin.original
                        != (Location {
                            file: self.site.file,
                            span: self.site.original,
                        })
                        || !self.site.members.iter().flatten().any(|member| {
                            origin.context == Some(member.context.0)
                                && origin.introduction == member.context.1
                                && edition == member.edition
                        })
                    {
                        return Err(error(
                            self.site.original.start,
                            "derived vocabulary changed its selected source introduction",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    fn text(&mut self, text: &str) -> Result<(), LexError> {
        (self.charge)(
            text.len(),
            text.len().saturating_mul(4 * (size_of::<Token>() + 64)),
        )?;
        let span = self.stream.synthetic_image(
            text,
            Location {
                file: self.site.file,
                span: self.site.original,
            },
            self.context.0,
            self.context.1,
            self.edition,
        )?;
        let mut lexer = Lexer::new(text)?;
        while let Some(mut token) = lexer.next_token()? {
            if matches!(
                token.kind,
                TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment
            ) {
                continue;
            }
            token.start += span.start;
            token.end += span.start;
            self.stream.tokens.push(token);
        }
        Ok(())
    }
    fn spelling(&mut self, original: Span, lifetime: bool) -> Result<(), LexError> {
        let text = self
            .site
            .source
            .get(original.start as usize..original.end as usize)
            .ok_or_else(|| error(original.start, "derived name is outside its source"))?;
        (self.charge)(
            text.len(),
            text.len().saturating_mul(4) + 4 * size_of::<Token>(),
        )?;
        let span = self.stream.image(
            text,
            Location {
                file: self.site.file,
                span: original,
            },
            None,
            ExpansionId(0),
        )?;
        let kind = match (
            lifetime,
            text.starts_with(if lifetime { "'r#" } else { "r#" }),
        ) {
            (true, true) => TokenKind::RawLifetime,
            (true, false) => TokenKind::Lifetime,
            (false, true) => TokenKind::RawIdentifier,
            (false, false) => TokenKind::Identifier,
        };
        self.stream.tokens.push(Token {
            kind,
            start: span.start,
            end: span.end,
        });
        Ok(())
    }
    fn header(&mut self, path: &str, unsafe_: bool) -> Result<(), LexError> {
        self.text("#[automatically_derived]")?;
        if self.site.const_ && path != "crate::marker::StructuralPartialEq" {
            self.text("#[rustc_const_unstable(feature=\"derive_const\",issue=\"118304\")] const")?;
        }
        if unsafe_ {
            self.text("unsafe")?;
        }
        self.text("impl")?;
        self.generics()?;
        self.text(path)?;
        self.text("for")?;
        let name = self
            .site
            .item
            .name
            .ok_or_else(|| error(self.site.original.start, "derived struct has no name"))?;
        self.spelling(name, false)?;
        self.generics()
    }
    fn generics(&mut self) -> Result<(), LexError> {
        let parameters = self.site.ast.list(self.site.item.generics);
        if parameters.is_empty() {
            return Ok(());
        }
        self.text("<")?;
        for &node in parameters {
            (self.charge)(1, 0)?;
            let Kind::Generic(index) = self.site.ast.node(node).kind else {
                return Err(error(
                    self.site.original.start,
                    "derive lost an actual generic parameter",
                ));
            };
            let parameter = self.site.ast.generics[index as usize];
            if parameter.flavor != GenericFlavor::Lifetime
                || parameter.bounds.start != parameter.bounds.end
            {
                return Err(error(
                    parameter.name.start,
                    "this paired source derive requires lifetime-only unbounded formals",
                ));
            }
            self.spelling(parameter.name, true)?;
            self.text(",")?;
        }
        self.text(">")
    }
    fn field(&mut self, ty: NodeId) -> Result<(), LexError> {
        (self.charge)(1, 4 * size_of::<Field>())?;
        let original = self.site.ast.node(ty).span;
        let parsed = derive_types::field(self.site.ast, ty, self.charge)?;
        crate::syntax::transfer::validate_spans(&parsed.ast, |span| {
            if span.start < original.start
                || span.end > original.end
                || self
                    .site
                    .source
                    .get(span.start as usize..span.end as usize)
                    .is_none()
            {
                return Err(error(
                    span.start,
                    "derived type span is outside its actual field type",
                ));
            }
            Ok(span)
        })?;
        let spelling = self
            .site
            .source
            .get(original.start as usize..original.end as usize)
            .ok_or_else(|| error(original.start, "derived field type is outside its source"))?;
        (self.charge)(spelling.len(), spelling.len().saturating_mul(4))?;
        let destination = self.stream.image(
            spelling,
            Location {
                file: self.site.file,
                span: original,
            },
            None,
            ExpansionId(0),
        )?;
        let slot = u32::try_from(self.fields.len())
            .map_err(|_| error(original.start, "too many derived field types"))?;
        self.stream.tokens.push(Token {
            kind: TokenKind::interpolated(slot)?,
            start: destination.start,
            end: destination.end,
        });
        self.fields.push(Field {
            parsed,
            original,
            destination,
        });
        Ok(())
    }
}
