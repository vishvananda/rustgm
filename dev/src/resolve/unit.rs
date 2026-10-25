//! A loaded source owns immutable text and movable, source-indexed syntax facts.
use crate::{
    expand::{
        config::Config,
        crate_syntax::{self, ConfiguredSyntax},
    },
    lex::token::LexError,
    literal::{
        Value,
        storage::{Storage, StoredValue},
    },
    support::{names::Names, source::SourceFile},
    syntax::{
        ast::{MacroContext, NodeId},
        attribute::Attribute,
        token_tree::{Span, error},
    },
};

pub(crate) mod contexts;
pub(crate) mod editions;
mod expansion;
mod original;
pub(crate) mod retirement;

pub use crate::support::source::FileId;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeKey {
    pub file: FileId,
    pub node: NodeId,
}

pub struct Unit {
    pub source: SourceFile,
    /// Cached include templates are provenance; only installed copies enter module traversal.
    pub include_context: Option<MacroContext>,
    pub configured: ConfiguredSyntax<StoredValue>,
    pub literals: Storage,
    pub owner_indices: Vec<Option<std::num::NonZeroU32>>,
    pub origins: Vec<crate::expand::stream::Origin>,
    original: original::Original,
    edition: crate::support::edition::Edition,
}

impl Unit {
    pub(crate) const fn edition(&self) -> crate::support::edition::Edition {
        self.edition
    }

    /// Configured attributes cross the resolution boundary as retained syntax.
    pub(crate) fn retained_attributes(
        &self,
        attributes: std::ops::Range<usize>,
    ) -> impl Iterator<
        Item = (
            crate::syntax::attribute::Attribute,
            &crate::syntax::token_tree::Forest,
        ),
    > {
        self.retained_indexed_attributes(attributes)
            .map(|(_, attribute, forest)| (attribute, forest))
    }
    pub(crate) fn retained_indexed_attributes(
        &self,
        attributes: std::ops::Range<usize>,
    ) -> impl Iterator<Item = (usize, Attribute, &crate::syntax::token_tree::Forest)> {
        attributes.filter_map(|index| {
            let effect = &self.configured.effects[index];
            (effect.attribute.effect == crate::expand::configured::Effect::Keep).then(|| {
                (
                    index,
                    effect.attribute.attribute,
                    &self.configured.ast.attributes[effect.capture as usize].forest,
                )
            })
        })
    }
    pub(crate) fn word_attribute(
        &self,
        node: Option<NodeId>,
        name: &str,
    ) -> Result<bool, LexError> {
        let Some(index) = node.and_then(|node| self.owner_indices[node.get() as usize]) else {
            return Ok(false);
        };
        let owner = &self.configured.owners[index.get() as usize - 1];
        let mut found = false;
        for (attribute, forest) in self.retained_attributes(owner.attributes.clone()) {
            if attribute.name(forest, self.source.text()) == Some(name) {
                if attribute.input != crate::syntax::attribute::Input::Empty {
                    return Err(error(
                        attribute.span.start,
                        "attribute requires no arguments",
                    ));
                }
                found = true;
            }
        }
        Ok(found)
    }
    /// Parse/configure once. Loading another file cannot invalidate these facts.
    /// Names are interned once, during parsing, into the crate's table.
    ///
    /// # Errors
    /// Propagates syntax/configuration errors and storage-range overflows.
    pub fn parse(source: SourceFile, config: &Config, names: &mut Names) -> Result<Self, LexError> {
        Self::parsed(source, config, None, names)
    }

    /// # Errors
    /// Reports malformed included fragments and demanded configuration errors.
    pub fn parse_include(
        source: SourceFile,
        config: &Config,
        context: MacroContext,
        names: &mut Names,
    ) -> Result<Self, LexError> {
        Self::parsed(source, config, Some(context), names)
    }

    fn parsed(
        source: SourceFile,
        config: &Config,
        context: Option<MacroContext>,
        names: &mut Names,
    ) -> Result<Self, LexError> {
        let ast = if let Some(context) = context {
            crate::syntax::expression::fragment::parse_include_in_edition(
                source.text(),
                context,
                Some(names),
                config.edition(),
            )?
        } else {
            crate::syntax::expression::parse_crate_in_edition(
                source.text(),
                Some(names),
                config.edition(),
            )?
        };
        let report = crate_syntax::configure(source.text(), ast, config)?;
        let mut literals = Storage::default();
        let ast = report.ast.map_literals(|value| {
            literals
                .store(source.text(), &value)
                .map_err(|message| error(0, message))
        })?;
        let configured = ConfiguredSyntax {
            ast,
            owners: report.owners,
            active: report.active,
            effects: report.effects,
            cfg_uses: report.cfg_uses,
        };
        let mut owner_indices = vec![None; configured.ast.nodes.len()];
        for (index, owner) in configured.owners.iter().enumerate() {
            let index = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(std::num::NonZeroU32::new)
                .ok_or_else(|| error(0, "too many attribute owners"))?;
            owner_indices[owner.node.get() as usize] = Some(index);
        }
        let original = original::Original::new(&configured, &literals);
        Ok(Self {
            source,
            include_context: context,
            configured,
            literals,
            owner_indices,
            origins: Vec::new(),
            original,
            edition: config.edition(),
        })
    }

    /// Whether a name's spelling is a lifetime: it begins with a quote.
    #[must_use]
    pub fn lifetime_spelling(&self, span: Span) -> bool {
        span.end > span.start
            && self.source.text().as_bytes().get(span.start as usize) == Some(&b'\'')
    }
    /// # Errors
    /// Rejects invalid manually constructed spans.
    pub fn spelling(&self, span: Span) -> Result<&str, LexError> {
        self.source
            .text()
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| error(span.start, "invalid source span"))
    }

    /// # Errors
    /// Rejects invalid manually supplied literal IDs or storage ranges.
    pub fn literal(&self, index: u32) -> Result<Value<'_>, LexError> {
        let value = self
            .configured
            .ast
            .literals
            .get(index as usize)
            .copied()
            .ok_or_else(|| error(0, "invalid literal ID"))?;
        self.literals
            .view(self.source.text(), value)
            .map_err(|message| error(0, message))
    }
}
