//! Private completion certifies real named type formals before their flags retire.
use super::{Declarations, DefId, Report, Stage, s};
use crate::{
    resolve::{model::DefKind, package::Error, unit::NodeKey},
    syntax::{
        ast::Kind,
        item::{GenericFlavor, ItemKind},
    },
    types::{self, arguments::Argument, parameters::ParameterKind},
};

#[cfg(test)]
#[path = "../../../../tests/invariants/collected_formal_controls.rs"]
mod tests;

#[derive(Clone, Copy)]
struct Formal<'a> {
    definition: DefId,
    source: NodeKey,
    owner: super::DeclarationRef,
    ordinal: usize,
    parameter: &'a crate::sema::generics::Parameter,
}

impl Stage {
    pub(super) fn selected_type_slots(&mut self) -> Result<usize, &'static str> {
        self.charge(self.rows.slots.len())?;
        let mut count = 0usize;
        for slot in &self.rows.slots {
            if !matches!(slot.kind, s::ParameterKind::Type { .. }) {
                continue;
            }
            let owner = self
                .rows
                .declarations
                .get(slot.owner.0 as usize)
                .ok_or("collected formal has an unknown actual owner")?;
            let selected = match owner.definition.kind {
                s::DefinitionKind::Function | s::DefinitionKind::Trait => true,
                s::DefinitionKind::SelfType => {
                    let parent = owner
                        .definition
                        .parent
                        .map(|parent| {
                            self.rows
                                .declarations
                                .get(parent.0 as usize)
                                .ok_or("collected Self formal has an unknown actual parent")
                        })
                        .transpose()?;
                    !parent.is_some_and(|parent| parent.definition.kind == s::DefinitionKind::Trait)
                }
                _ => return Err("collected type formal has no supported written scope"),
            };
            count = count.saturating_add(usize::from(selected));
        }
        Ok(count)
    }

    pub(super) fn require_named_parameters(
        &mut self,
        report: &Report,
        facts: &Declarations,
        definition: DefId,
        source: NodeKey,
    ) -> Result<usize, Error> {
        let unit = report
            .package
            .units
            .get(source.file.0 as usize)
            .ok_or_else(|| Error::unlocated("collected formal source unit is absent"))?;
        let ast = &unit.configured.ast;
        let node = ast
            .nodes
            .get(source.node.get() as usize)
            .ok_or_else(|| Error::unlocated("collected formal source item is absent"))?;
        let error = |message| Error::at(source.file, node.span.start, message);
        let Kind::Item(index) = node.kind else {
            return Err(error("collected formal owner has no actual item"));
        };
        let item = ast
            .items
            .get(index as usize)
            .ok_or_else(|| error("collected formal owner item is absent"))?;
        let kind = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| error("collected formal definition is absent"))?
            .kind;
        let function = match (kind, item.kind) {
            (DefKind::Function, ItemKind::Function { .. }) => true,
            (DefKind::Trait, ItemKind::Trait { .. })
            | (DefKind::SelfType, ItemKind::Impl { .. }) => false,
            _ => return Err(error("collected formal owner has an unsupported item kind")),
        };
        let parameters = facts
            .generics
            .parameters
            .get(&definition)
            .map_or(&[][..], Vec::as_slice);
        let written = ast.list(item.generics);
        self.charge(
            written
                .len()
                .saturating_add(parameters.len().saturating_mul(12))
                .saturating_add(1),
        )
        .map_err(Error::unlocated)?;
        let owner = self.declaration(definition).map_err(Error::unlocated)?;
        let mut count = 0usize;
        for &node in written {
            let Kind::Generic(index) = ast.node(node).kind else {
                return Err(error("collected type formal lacks its actual named syntax"));
            };
            let syntax = ast
                .generics
                .get(index as usize)
                .ok_or_else(|| error("collected type formal syntax is absent"))?;
            if syntax.flavor == GenericFlavor::Lifetime && function {
                continue;
            }
            if syntax.flavor != GenericFlavor::Type || syntax.default.is_some() {
                return Err(error(
                    "collected const/lifetime/default formal transport is not implemented",
                ));
            }
            let parameter = parameters
                .get(count)
                .ok_or_else(|| error("collected formal omitted its actual semantic parameter"))?;
            self.require_parameter_slot(
                report,
                facts,
                Formal {
                    definition,
                    source: NodeKey {
                        file: source.file,
                        node,
                    },
                    owner,
                    ordinal: count,
                    parameter,
                },
            )?;
            count = count.saturating_add(1);
        }
        if count != parameters.len() {
            return Err(error(
                "collected anonymous or missing named formal transport is not implemented",
            ));
        }
        Ok(count)
    }

    fn require_parameter_slot(
        &mut self,
        report: &Report,
        facts: &Declarations,
        formal: Formal<'_>,
    ) -> Result<(), Error> {
        let Formal {
            definition,
            source,
            owner,
            ordinal,
            parameter,
        } = formal;
        let ast = &report.package.unit(source.file).configured.ast;
        let error = |message| Error::at(source.file, ast.node(source.node).span.start, message);
        if parameter.anonymous || parameter.default.is_some() {
            return Err(error(
                "collected anonymous/default formal transport is not implemented",
            ));
        }
        let Argument::Type(ty) = parameter.argument else {
            return Err(error(
                "collected non-type formal transport is not implemented",
            ));
        };
        let Some(types::Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err(error("collected formal lost its actual rigid parameter"));
        };
        let actual = facts
            .types
            .parameter(id)
            .ok_or_else(|| error("collected rigid formal is absent"))?;
        let ParameterKind::Type { sized } = actual.kind else {
            return Err(error("collected formal has an unsupported canonical kind"));
        };
        let reference = self
            .graph
            .parameters
            .get(&id)
            .copied()
            .ok_or_else(|| error("collected formal has no actual staged slot"))?;
        let slot = self
            .rows
            .slots
            .get(reference.0 as usize)
            .copied()
            .ok_or_else(|| error("collected formal slot is absent"))?;
        let binding = report
            .resolution
            .definitions
            .get(parameter.binding.0 as usize)
            .ok_or_else(|| error("collected formal binding is absent"))?;
        if actual.owner != types::ItemId(definition.0)
            || actual.index as usize != ordinal
            || slot.owner != owner
            || slot.ordinal as usize != ordinal
            || slot.index != actual.index
            || !matches!(slot.kind, s::ParameterKind::Type { sized: expected } if expected.is_some() == sized)
            || slot.definition
                != Some(
                    self.declaration(parameter.binding)
                        .map_err(Error::unlocated)?,
                )
            || binding.kind != DefKind::Generic
            || binding.node != Some(source)
        {
            return Err(error(
                "collected formal changed its exact syntax/owner/slot/binding",
            ));
        }
        Ok(())
    }
}
