//! Demanded attribute expressions join ordinary expansion without rewriting their witnesses.
use super::{Engine, Error, FileId, Input, Kind, NodeId, NodeKey, Package, Sources, Visit, Work};
use crate::{
    expand::configured::Effect,
    lex::token::Kind as TokenKind,
    resolve::lint_policy::producer::Tail,
    syntax::{
        attribute::{Attribute, Input as AttributeInput},
        interpolation::Inputs,
    },
};
use std::collections::BTreeMap;
struct Deferred {
    visit: Visit,
    tail: Option<Tail>,
    effect: usize,
    attribute: Attribute,
    scope: super::Scope,
}

struct Value {
    owner: NodeKey,
    effect: usize,
    attribute: Attribute,
    root: NodeId,
}
#[derive(Default)]
pub(crate) struct Values {
    entries: BTreeMap<(u32, u32), Value>,
    pending: BTreeMap<(u32, u32), Vec<Deferred>>,
}
impl Values {
    /// Only the actual configured owner may consume its retained expression.
    pub(crate) fn completed(
        &self,
        package: &Package,
        owner: NodeKey,
        attribute: Attribute,
    ) -> Result<Option<NodeId>, Error> {
        let Some(value) = self.entries.get(&(owner.file.0, attribute.span.start)) else {
            return Ok(None);
        };
        let error = || {
            Error::at(
                owner.file,
                attribute.span.start,
                "attribute value lost its actual source owner",
            )
        };
        let unit = package.unit(owner.file);
        let configured = unit
            .owner_indices
            .get(value.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .ok_or_else(error)?;
        let effect = unit
            .configured
            .effects
            .get(value.effect)
            .ok_or_else(error)?;
        if value.owner.file != owner.file
            || configured.node != value.owner.node
            || !configured.enabled
            || !configured.attributes.contains(&value.effect)
            || effect.attribute.effect != Effect::Keep
            || effect.attribute.attribute != attribute
            || value.attribute != attribute
        {
            return Err(error());
        }
        let mut node = configured.node;
        while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            node = value;
        }
        if node != owner.node {
            return Err(error());
        }
        let Kind::Expansion {
            children,
            tail: None,
        } = unit.configured.ast.node(value.root).kind
        else {
            return Err(error());
        };
        let [root] = unit.configured.ast.list(children) else {
            return Err(error());
        };
        Ok(Some(*root))
    }
}

impl<S: Sources> Engine<'_, S> {
    /// Language and diagnostic identities share actual scoped expression expansion.
    /// Other attributes retain their owning validators and expression contracts.
    pub(super) fn expand_attribute_values(
        &mut self,
        visit: Visit,
        tail: Option<Tail>,
    ) -> Result<(), Error> {
        let key = Self::visit_key(visit);
        let unit = self.loader.package.unit(key.file);
        let mut node = visit.node;
        while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            node = value;
        }
        let deferred = matches!(unit.configured.ast.node(node).kind, Kind::Item(index)
            if matches!(unit.configured.ast.items[index as usize].kind, crate::syntax::item::ItemKind::Module { .. }))
            && self.loader.package.modules[visit.module.0 as usize].declaration
                != Some(NodeKey {
                    file: key.file,
                    node,
                });
        let index = unit.owner_indices[visit.node.get() as usize]
            .ok_or_else(|| Error::at(key.file, 0, "attribute value has no configured owner"))?;
        let attributes = unit.configured.owners[index.get() as usize - 1]
            .attributes
            .clone();
        for effect in attributes {
            let unit = self.loader.package.unit(key.file);
            let configured = &unit.configured.effects[effect];
            let attribute = configured.attribute.attribute;
            let forest = &unit.configured.ast.attributes[configured.capture as usize].forest;
            if configured.attribute.effect != Effect::Keep
                || !matches!(
                    attribute.name(forest, unit.source.text()),
                    Some("rustc_diagnostic_item" | "lang")
                )
            {
                continue;
            }
            let AttributeInput::Equals(range) = attribute.input else {
                continue;
            };
            if range.end == range.start + 1
                && forest
                    .token(range.start)
                    .is_some_and(|token| !matches!(token.kind, TokenKind::Interpolated(_)))
            {
                continue;
            }
            if self
                .attribute_effects
                .as_deref()
                .and_then(|effects| effects.values.as_deref())
                .is_some_and(|values| {
                    values
                        .entries
                        .contains_key(&(key.file.0, attribute.span.start))
                })
            {
                continue;
            }
            if deferred {
                self.defer_module_value(visit, node, effect, attribute, tail)?;
            } else {
                self.retain_attribute_value(visit, effect, attribute, tail, self.head)?;
            }
        }
        Ok(())
    }
    fn retain_attribute_value(
        &mut self,
        visit: Visit,
        effect: usize,
        attribute: Attribute,
        tail: Option<Tail>,
        scope: super::Scope,
    ) -> Result<(), Error> {
        let key = Self::visit_key(visit);
        let error = |message| Error::at(key.file, attribute.span.start, message);
        let AttributeInput::Equals(range) = attribute.input else {
            return Err(error("missing attribute expression"));
        };
        let width = attribute.span.end.saturating_sub(attribute.span.start) as usize;
        self.ensure_attribute_policy(key)?;
        let producer = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .ok_or_else(|| error("missing attribute expression payer"))?;
        producer.diagnostic_payload(key, width + 1, width.saturating_mul(64))?;
        producer.reserve_diagnostic(key, 4 * size_of::<Value>() + size_of::<Values>())?;
        let unit = self.loader.package.unit(key.file);
        let forest = &unit.configured.ast.attributes
            [unit.configured.effects[effect].capture as usize]
            .forest;
        let input = Input::from_forest(forest, range)
            .map_err(|e| Error::at(key.file, e.offset, e.message))?;
        self.pay_attribute_opaque(key, &input)?;
        let root = self.parse_attribute_value(key.file, &input)?;
        let Kind::Expansion {
            children,
            tail: None,
        } = self
            .loader
            .package
            .unit(key.file)
            .configured
            .ast
            .node(root)
            .kind
        else {
            return Err(error("missing attribute expression root"));
        };
        if self
            .loader
            .package
            .unit(key.file)
            .configured
            .ast
            .list(children)
            .len()
            != 1
        {
            return Err(error("attribute value requires one expression"));
        }
        self.attribute_effects
            .get_or_insert_default()
            .values
            .get_or_insert_default()
            .entries
            .insert(
                (key.file.0, attribute.span.start),
                Value {
                    owner: key,
                    effect,
                    attribute,
                    root,
                },
            );
        let visit = Visit {
            node: root,
            owner: None,
            literal_value: true,
            ..visit
        };
        let frame = self.frame_mark()?;
        self.lint_set_frame(frame, visit, tail)?;
        self.work.push(Work::Frames(frame));
        self.frames.push(visit);
        self.work.push(Work::AttributeValue {
            node: root,
            frame,
            scope,
        });
        Ok(())
    }
    fn defer_module_value(
        &mut self,
        visit: Visit,
        module: NodeId,
        effect: usize,
        attribute: Attribute,
        tail: Option<Tail>,
    ) -> Result<(), Error> {
        let key = Self::visit_key(visit);
        self.ensure_attribute_policy(key)?;
        self.attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .ok_or_else(|| {
                Error::at(
                    key.file,
                    attribute.span.start,
                    "missing deferred attribute payer",
                )
            })?
            .reserve_diagnostic(key, 4 * size_of::<Deferred>() + size_of::<Values>())?;
        self.attribute_effects
            .get_or_insert_default()
            .values
            .get_or_insert_default()
            .pending
            .entry((key.file.0, module.get()))
            .or_default()
            .push(Deferred {
                visit,
                tail,
                effect,
                attribute,
                scope: self.head,
            });
        Ok(())
    }
    pub(super) fn expand_module_values(
        &mut self,
        visit: Visit,
        module: super::ModuleId,
        path_scope: super::ScopeId,
    ) -> Result<(), Error> {
        let queued = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.values.as_deref_mut())
            .and_then(|values| values.pending.remove(&(visit.file.0, visit.node.get())));
        for value in queued.into_iter().flatten() {
            self.retain_attribute_value(
                Visit {
                    module,
                    path_scope,
                    ..value.visit
                },
                value.effect,
                value.attribute,
                value.tail,
                value.scope,
            )?;
        }
        Ok(())
    }
    fn pay_attribute_opaque(&self, key: NodeKey, input: &Input) -> Result<(), Error> {
        let producer = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.policy.as_ref())
            .ok_or_else(|| Error::at(key.file, 0, "missing attribute expression payer"))?;
        let view = self.captures.view(|id| {
            self.loader
                .package
                .units
                .get(id.0 as usize)
                .map(|unit| unit.source.text())
        });
        let charge = |steps, bytes| {
            producer.diagnostic_payload(key, steps, bytes).map_err(|_| {
                crate::syntax::token_tree::error(
                    0,
                    "attribute expression exceeds work/storage budget",
                )
            })
        };
        let counted = crate::expand::builtin::choice::Counted {
            inputs: &view,
            charge: &charge,
        };
        for &token in &input.tokens {
            counted
                .get(token)
                .map_err(|e| Error::at(key.file, e.offset, e.message))?;
        }
        Ok(())
    }
    fn parse_attribute_value(&mut self, file: FileId, input: &Input) -> Result<NodeId, Error> {
        let (before, tail) = self.loader.package.units.split_at_mut(file.0 as usize);
        let (unit, after) = tail
            .split_first_mut()
            .ok_or_else(|| Error::at(file, 0, "missing attribute value source"))?;
        unit.retain_arguments(
            file,
            &input.tokens,
            &self.captures,
            |id| {
                if id.0 < file.0 {
                    before.get(id.0 as usize)
                } else {
                    after.get((id.0 - file.0 - 1) as usize)
                }
            },
            self.config,
            crate::syntax::expression::fragment::ArgumentsGrammar::Expressions,
        )
        .map_err(|e| Error::at(file, e.offset, e.message))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/metavalue_witnesses.rs"]
mod tests;
