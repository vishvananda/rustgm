//! Verify a demanded fragment's span transfer without allocating copied arenas.
use super::{Bases, Syntax, attribute, error};
use crate::{
    lex::token::LexError,
    syntax::token_tree::{Forest, Span, TreeKind},
};

pub(crate) fn validate<L>(
    source: &Syntax<L>,
    mut span: impl FnMut(Span) -> Result<Span, LexError>,
) -> Result<(), LexError> {
    // Zero bases exercise the same exhaustive field mapping as the real copy.
    let bases = Bases {
        nodes: 0,
        edges: 0,
        literals: 0,
        captures: 0,
        items: 0,
        gate_sites: crate::syntax::gate_sites::Prefix::default(),
        variants: 0,
        use_trees: 0,
        generics: 0,
        function_types: 0,
        attributes: 0,
    };
    if source.root.get() as usize >= source.nodes.len() {
        return Err(error(0, "syntax span preflight has no actual root"));
    }
    for node in &source.nodes {
        span(node.span)?;
        let mut kind = node.kind;
        bases.kind(&mut kind, &mut span)?;
    }
    for item in &source.items {
        if let Some(name) = item.name {
            span(name)?;
        }
        let mut kind = item.kind;
        bases.item(&mut kind, &mut span)?;
    }
    if let Some(entries) = &source.gate_sites {
        for entry in entries.entries() {
            entry.validate(source)?;
            span(entry.span)?;
        }
        for (ordinal, entry) in entries.expressions().iter().enumerate() {
            entry.validate(
                source,
                u32::try_from(ordinal).map_err(|_| error(0, "expression-gate ordinal overflow"))?,
            )?;
            span(entry.keyword)?;
        }
    }
    for variant in &source.variants {
        span(variant.name)?;
    }
    for tree in &source.use_trees {
        if let Some(glob) = tree.glob {
            span(glob)?;
        }
    }
    for generic in &source.generics {
        span(generic.name)?;
    }
    for capture in &source.captures {
        forest(capture, &mut span)?;
    }
    for capture in &source.attributes {
        forest(&capture.forest, &mut span)?;
        span(capture.attribute.span)?;
        if let attribute::Input::Doc(body) = capture.attribute.input {
            span(body)?;
        }
    }
    Ok(())
}
fn forest(
    forest: &Forest,
    span: &mut impl FnMut(Span) -> Result<Span, LexError>,
) -> Result<(), LexError> {
    for tree in forest.nodes() {
        span(tree.span)?;
        if let TreeKind::Doc { body, .. } = tree.kind {
            span(body)?;
        }
    }
    let end = forest.offset(
        u32::try_from(forest.nodes().len())
            .map_err(|_| error(0, "forest span count exceeds 32 bits"))?,
    );
    span(Span { start: end, end })?;
    Ok(())
}
