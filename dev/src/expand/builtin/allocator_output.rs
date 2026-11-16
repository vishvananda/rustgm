//! `#[global_allocator]` and `#[alloc_error_handler]` keep their item and add
//! the anonymous constant of internal-symbol functions that rustc 1.98.0's
//! `rustc_builtin_macros` writes beside it.
use super::{choice::Charge, delegation::Delegated};
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
    support::{edition::Edition, source::FileId},
    syntax::{
        ast::{Ast, MacroContext},
        expression::fragment,
        item::ItemContext,
        token_tree::{Span, error},
    },
};

/// The item that a selected allocator attribute registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    /// `#[global_allocator]` on a static whose type implements `GlobalAlloc`.
    GlobalAllocator,
    /// `#[alloc_error_handler]` on a function taking a `Layout`.
    ErrorHandler,
}
impl Role {
    /// rustc's expansion, with `@` where the registered item's own name is
    /// copied. Its `crate` roots name the macro's defining crate, as
    /// `ExtCtxt::std_path` does. A global allocator's functions follow
    /// `ALLOCATOR_METHODS`: a `Layout` argument becomes `size: usize` and
    /// `align: Alignment` rebuilt by `Layout::from_size_alignment_unchecked`.
    /// The handler rebuilds its `Layout` from two `usize`s.
    const fn template(self) -> &'static str {
        match self {
            Self::GlobalAllocator => GLOBAL_ALLOCATOR,
            Self::ErrorHandler => ERROR_HANDLER,
        }
    }
}

const GLOBAL_ALLOCATOR: &str = "const _: () = {
#[rustc_std_internal_symbol] #[rustc_allocator]
unsafe fn __rust_alloc(size: usize, align: crate::mem::Alignment) -> *mut u8 {
crate::alloc::GlobalAlloc::alloc(&@,
crate::alloc::Layout::from_size_alignment_unchecked(size, align)) }
#[rustc_std_internal_symbol] #[rustc_deallocator]
unsafe fn __rust_dealloc(ptr: *mut u8, size: usize, align: crate::mem::Alignment) -> () {
crate::alloc::GlobalAlloc::dealloc(&@, ptr,
crate::alloc::Layout::from_size_alignment_unchecked(size, align)) }
#[rustc_std_internal_symbol] #[rustc_reallocator]
unsafe fn __rust_realloc(ptr: *mut u8, size: usize, align: crate::mem::Alignment,
new_size: usize) -> *mut u8 {
crate::alloc::GlobalAlloc::realloc(&@, ptr,
crate::alloc::Layout::from_size_alignment_unchecked(size, align), new_size) }
#[rustc_std_internal_symbol] #[rustc_allocator_zeroed]
unsafe fn __rust_alloc_zeroed(size: usize, align: crate::mem::Alignment) -> *mut u8 {
crate::alloc::GlobalAlloc::alloc_zeroed(&@,
crate::alloc::Layout::from_size_alignment_unchecked(size, align)) }
};";

const ERROR_HANDLER: &str = "const _: () = {
#[rustc_std_internal_symbol]
unsafe fn __rust_alloc_error_handler(size: usize, align: usize) -> ! {
@(crate::alloc::Layout::from_size_align_unchecked(size, align)) }
};";

pub(crate) struct Site<'a> {
    pub file: FileId,
    pub source: &'a str,
    /// The attribute: every synthesized token's diagnostic anchor.
    pub original: Span,
    /// The registered item's identifier, copied with its own hygiene.
    pub name: Span,
    /// The crate that defines the selected macro: 0 is this crate.
    pub crate_id: u32,
    /// The attribute's opaque definition-site mark and its invocation.
    pub context: (SyntaxContextId, ExpansionId),
    /// The selected definition's edition.
    pub edition: Edition,
    pub role: Role,
}

/// Write and parse one expansion as module items.
/// # Errors
/// Rejects a name outside its source, exhausted work and invalid syntax.
pub(crate) fn build(
    site: &Site<'_>,
    config: &Config,
    charge: &Charge<'_>,
) -> Result<Delegated, LexError> {
    let name = site
        .source
        .get(site.name.start as usize..site.name.end as usize)
        .ok_or_else(|| {
            error(
                site.original.start,
                "allocator item name is outside its source",
            )
        })?;
    let template = site.role.template();
    charge(
        template.len().saturating_mul(2),
        template.len().saturating_mul(8),
    )?;
    let mut stream = Stream::default();
    for (index, piece) in template.split('@').enumerate() {
        if index != 0 {
            let span = stream.image(
                name,
                Location {
                    file: site.file,
                    span: site.name,
                },
                None,
                ExpansionId(0),
            )?;
            stream.tokens.push(Token {
                kind: if name.starts_with("r#") {
                    TokenKind::RawIdentifier
                } else {
                    TokenKind::Identifier
                },
                start: span.start,
                end: span.end,
            });
        }
        synthesize(&mut stream, site, piece)?;
    }
    let ast = fragment::parse_expansion(
        &stream.source,
        &stream.tokens,
        MacroContext::Items(ItemContext::Module),
        &[],
    )?;
    charge(
        ast.nodes.len().saturating_add(ast.edges.len()),
        super::choice::syntax_payload(&ast).saturating_mul(4),
    )?;
    let configured = crate_syntax::configure(&stream.source, ast, config)?;
    let ast: Ast<'static> = configured.ast.map_literals(|_| {
        Err(error(
            site.original.start,
            "allocator expansion introduced a literal",
        ))
    })?;
    let syntax = crate_syntax::ConfiguredSyntax {
        ast,
        owners: configured.owners,
        active: configured.active,
        effects: configured.effects,
        cfg_uses: configured.cfg_uses,
    };
    Ok(Delegated { stream, syntax })
}

/// Append `text` as definition-site vocabulary of the attribute's expansion.
fn synthesize(stream: &mut Stream, site: &Site<'_>, text: &str) -> Result<(), LexError> {
    let span = stream.synthetic_image(
        text,
        Location {
            file: site.file,
            span: site.original,
        },
        site.context.0,
        site.context.1,
        site.edition,
    )?;
    let mut lexer = Lexer::new(text)?;
    while let Some(mut token) = lexer.next_token()? {
        match token.kind {
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment => continue,
            TokenKind::Identifier
                if text.get(token.start as usize..token.end as usize) == Some("crate") =>
            {
                token.kind = TokenKind::crate_root(site.crate_id)?;
            }
            _ => {}
        }
        token.start += span.start;
        token.end += span.start;
        stream.tokens.push(token);
    }
    Ok(())
}
