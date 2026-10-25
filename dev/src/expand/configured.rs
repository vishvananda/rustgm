//! Attribute configuration preserves syntax ownership and original tree ranges.
use super::config::{self, Config};
use crate::{
    lex::token::LexError,
    syntax::{
        attribute::{self, Attribute, Input},
        token_tree::{Delimiter, Forest, TreeKind, TreeRange, error},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Keep,
    Cfg(bool),
    CfgAttr(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfiguredAttribute {
    pub attribute: Attribute,
    pub effect: Effect,
    /// The nearest `cfg_attr` that produced this attribute, if any.
    pub parent: Option<u32>,
}

#[derive(Default, Debug)]
pub struct Attributes {
    pub entries: Vec<ConfiguredAttribute>,
    disabled: bool,
    cfg_uses: Option<Box<ConfiguredUses>>,
}

/// An existing effect index binds a demanded predicate use to its real owner/capture.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ConfiguredUse {
    pub effect: u32,
    pub predicate: config::CfgUse,
}

/// One cold owner keeps demanded uses ordered by their existing effect identities.
#[derive(Clone, Debug, Default)]
pub(crate) struct ConfiguredUses {
    entries: Vec<ConfiguredUse>,
}
impl ConfiguredUses {
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, ConfiguredUse> {
        self.entries.iter()
    }
    #[cfg(test)]
    pub(crate) fn as_slice(&self) -> &[ConfiguredUse] {
        &self.entries
    }
    pub(crate) fn relocate(
        &mut self,
        base: u32,
        effects: usize,
        mut span: impl FnMut(
            crate::syntax::token_tree::Span,
        ) -> Result<crate::syntax::token_tree::Span, LexError>,
    ) -> Result<(), LexError> {
        for used in &mut self.entries {
            if used.effect as usize >= effects {
                return Err(error(0, "generated cfg use has no actual effect"));
            }
            used.effect = used
                .effect
                .checked_add(base)
                .ok_or_else(|| error(0, "cfg effect overflow"))?;
            used.predicate.span = span(used.predicate.span)?;
        }
        Ok(())
    }

    pub(crate) fn reserve(&mut self, count: usize) -> Result<(), LexError> {
        if self.entries.len().saturating_add(count) > 1_000_000 {
            return Err(error(0, "configured feature uses exceed one million"));
        }
        self.entries
            .try_reserve(count)
            .map_err(|_| error(0, "cfg use allocation failed"))
    }
    /// Commit after checked reserve, without an allocation or failure after publication.
    pub(crate) fn append_reserved(&mut self, incoming: &mut Self) {
        self.entries.append(&mut incoming.entries);
    }
    fn record(&mut self, effect: u32, uses: Vec<config::CfgUse>) -> Result<(), LexError> {
        self.reserve(uses.len())?;
        self.entries.extend(
            uses.into_iter()
                .map(|predicate| ConfiguredUse { effect, predicate }),
        );
        Ok(())
    }
    pub(crate) fn original_prefix(&self, effects: usize) -> Option<Box<Self>> {
        let end = self
            .entries
            .partition_point(|used| (used.effect as usize) < effects);
        (end != 0).then(|| {
            Box::new(Self {
                entries: self.entries[..end].to_vec(),
            })
        })
    }
}

pub(crate) fn retain_cfg_uses(
    target: &mut Option<Box<ConfiguredUses>>,
    incoming: Option<Box<ConfiguredUses>>,
) -> Result<(), LexError> {
    let Some(mut incoming) = incoming else {
        return Ok(());
    };
    let Some(target) = target else {
        *target = Some(incoming);
        return Ok(());
    };
    target.reserve(incoming.len())?;
    target.append_reserved(&mut incoming);
    Ok(())
}

#[derive(Clone, Copy)]
struct Expansion<'a> {
    inputs: &'a dyn crate::syntax::interpolation::Inputs,
    parent: Option<u32>,
    depth: u32,
    owned: bool,
}

impl Attributes {
    /// Configure one syntax-owned attribute and append any demanded `cfg_attr`
    /// expansion. The caller supplies attachment/placement, not this phase.
    ///
    /// # Errors
    /// Rejects malformed cfg/`cfg_attr` syntax or excessive `cfg_attr` nesting.
    pub fn push(
        &mut self,
        forest: &Forest,
        source: &str,
        attribute: Attribute,
        config: &Config,
    ) -> Result<(), LexError> {
        self.expand(
            forest,
            source,
            attribute,
            config,
            Expansion {
                inputs: &config.edition(),
                parent: None,
                depth: 0,
                owned: false,
            },
        )
    }

    /// Configure a real syntax owner. Once disabled, later cfg predicates are
    /// not demanded; the diagnostic PA3 stream keeps its exhaustive behavior.
    ///
    /// # Errors
    /// Returns demanded configuration failures.
    pub fn push_owned(
        &mut self,
        forest: &Forest,
        source: &str,
        attribute: Attribute,
        config: &Config,
    ) -> Result<(), LexError> {
        self.push_owned_with_inputs(forest, source, attribute, config, &config.edition())
    }

    /// Preserve generated tokens' editions while configuring a real syntax owner.
    /// # Errors
    /// Returns demanded configuration and provenance failures.
    pub fn push_owned_with_inputs(
        &mut self,
        forest: &Forest,
        source: &str,
        attribute: Attribute,
        config: &Config,
        inputs: &dyn crate::syntax::interpolation::Inputs,
    ) -> Result<(), LexError> {
        if self.disabled {
            return Ok(());
        }
        self.expand(
            forest,
            source,
            attribute,
            config,
            Expansion {
                inputs,
                parent: None,
                depth: 0,
                owned: true,
            },
        )
    }

    /// Reuse owner scratch capacity without retaining prior configuration facts.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.disabled = false;
        self.cfg_uses = None;
    }

    pub(crate) fn take_cfg_uses(&mut self) -> Option<Box<ConfiguredUses>> {
        self.cfg_uses.take()
    }

    fn predicate(
        &mut self,
        forest: &Forest,
        source: &str,
        range: TreeRange,
        config: &Config,
        inputs: &dyn crate::syntax::interpolation::Inputs,
    ) -> Result<bool, LexError> {
        let answer = config::predicate_checked(forest, source, range, config, inputs)?;
        if !answer.uses.is_empty() {
            let effect =
                u32::try_from(self.entries.len()).map_err(|_| error(0, "cfg effect overflow"))?;
            self.cfg_uses
                .get_or_insert_with(Box::default)
                .record(effect, answer.uses)?;
        }
        Ok(answer.value)
    }

    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.disabled
    }

    fn expand(
        &mut self,
        forest: &Forest,
        source: &str,
        attribute: Attribute,
        config: &Config,
        expansion: Expansion<'_>,
    ) -> Result<(), LexError> {
        if expansion.depth >= 128 {
            return Err(error(attribute.span.start, "cfg_attr nesting exceeds 128"));
        }
        let name = attribute.name(forest, source);
        let effect = match name {
            Some("cfg") => {
                let range = arguments(forest, attribute)?;
                let (predicate, next) = config::comma_item(forest, range);
                if next != range.end {
                    return Err(error(
                        attribute.span.start,
                        "cfg requires exactly one predicate",
                    ));
                }
                Effect::Cfg(self.predicate(forest, source, predicate, config, expansion.inputs)?)
            }
            Some("cfg_attr") => {
                let arguments = arguments(forest, attribute)?;
                let (predicate, next) = config::comma_item(forest, arguments);
                if next == predicate.end {
                    return Err(error(
                        attribute.span.start,
                        "cfg_attr requires a comma after its predicate",
                    ));
                }
                Effect::CfgAttr(self.predicate(
                    forest,
                    source,
                    predicate,
                    config,
                    expansion.inputs,
                )?)
            }
            _ => Effect::Keep,
        };
        let index = u32::try_from(self.entries.len())
            .map_err(|_| error(attribute.span.start, "too many configured attributes"))?;
        self.entries.push(ConfiguredAttribute {
            attribute,
            effect,
            parent: expansion.parent,
        });
        self.disabled |= effect == Effect::Cfg(false);
        if let Effect::CfgAttr(selected) = effect {
            let range = arguments(forest, attribute)?;
            let (_, mut next) = config::comma_item(forest, range);
            while next < range.end {
                let (item, following) = config::comma_item(
                    forest,
                    TreeRange {
                        start: next,
                        end: range.end,
                    },
                );
                let nested = attribute::parse_with_inputs(
                    forest,
                    source,
                    item,
                    attribute.style,
                    attribute.span,
                    expansion.inputs,
                )?;
                if selected && !(expansion.owned && self.disabled) {
                    if matches!(
                        nested.name(forest, source),
                        Some("crate_name" | "crate_type")
                    ) {
                        return Err(error(
                            attribute.span.start,
                            "crate_name and crate_type are forbidden in cfg_attr",
                        ));
                    }
                    self.expand(
                        forest,
                        source,
                        nested,
                        config,
                        Expansion {
                            parent: Some(index),
                            depth: expansion.depth + 1,
                            ..expansion
                        },
                    )?;
                }
                next = following;
            }
        }
        Ok(())
    }
}

fn arguments(forest: &Forest, attribute: Attribute) -> Result<TreeRange, LexError> {
    if let Input::Delimited(index) = attribute.input
        && matches!(
            forest.tree(index).map(|t| t.kind),
            Some(TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            })
        )
    {
        return forest
            .children(index)
            .ok_or_else(|| error(attribute.span.start, "missing cfg arguments"));
    }
    Err(error(
        attribute.span.start,
        "cfg and cfg_attr require parentheses",
    ))
}
