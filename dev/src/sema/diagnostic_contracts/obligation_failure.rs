//! Select declaration diagnostics only from the actual failed typed trait goal.
use super::{
    Condition, Contract, Declarations, DefId, DefKind, Mode, Piece, Report, Slot, Template,
    typed_format,
};
use crate::types::{Kind, TypeId, arguments::Argument, parameters::predicates::Predicate};
use typed_format::{Formatter, append};

impl Declarations {
    pub(in crate::sema) fn has_trait_diagnostic(&self, item: crate::types::ItemId) -> bool {
        self.must_use_contracts.as_deref().is_some_and(|table| {
            table.unknown.contains(&DefId(item.0))
                || table
                    .diagnostics
                    .as_deref()
                    .is_some_and(|contracts| contracts.declarations.contains_key(&DefId(item.0)))
        })
    }

    pub(in crate::sema) fn custom_trait_failure(
        &self,
        report: &Report,
        predicate: Predicate,
        fallback: &str,
    ) -> Result<Option<String>, &'static str> {
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(None);
        };
        let application = self
            .types
            .trait_application(application)
            .ok_or("diagnostic trait application is unknown")?;
        let definition = DefId(application.item.0);
        if self
            .must_use_contracts
            .as_deref()
            .is_some_and(|table| table.unknown.contains(&definition))
        {
            return Err("imported diagnostic contract transport is not implemented");
        }
        let Some(table) = self
            .must_use_contracts
            .as_deref()
            .and_then(|table| table.diagnostics.as_deref())
        else {
            return Ok(None);
        };
        table.charge(1, 0)?;
        let Some(contract) = table.declarations.get(&definition) else {
            return Ok(None);
        };
        contract.validate(report)?;
        let inputs = Inputs {
            facts: self,
            report,
            contract,
            subject: predicate.ty,
            arguments: application.arguments,
        };
        let mut formatter = Formatter::new(self, report, table);
        let mut message = None;
        let mut label = None;
        let mut notes = Vec::new();
        for directive in &contract.directives {
            table.charge(1, 0)?;
            if !inputs.matches(directive.condition)? {
                continue;
            }
            if message.is_none()
                && let Some(template) = &directive.message
            {
                message = Some(inputs.render(&mut formatter, template)?);
            }
            if label.is_none()
                && let Some(template) = &directive.label
            {
                label = Some(inputs.render(&mut formatter, template)?);
            }
            for template in &directive.notes {
                table.charge(1, 4 * size_of::<String>())?;
                notes.push(inputs.render(&mut formatter, template)?);
            }
        }
        if message.is_none() && label.is_none() && notes.is_empty() {
            return Ok(None);
        }
        let mut text = String::new();
        append(table, &mut text, message.as_deref().unwrap_or(fallback))?;
        if let Some(label) = label {
            append(table, &mut text, "\nlabel: ")?;
            append(table, &mut text, &label)?;
        }
        for note in notes {
            append(table, &mut text, "\nnote: ")?;
            append(table, &mut text, &note)?;
        }
        Ok(Some(text))
    }
}
impl Contract {
    fn validate(&self, report: &Report) -> Result<(), &'static str> {
        let unit = report
            .package
            .units
            .get(self.owner.file.0 as usize)
            .ok_or("diagnostic owner file is missing")?;
        let owner = unit
            .owner_indices
            .get(self.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
        if report
            .resolution
            .definitions
            .get(self.definition.0 as usize)
            .is_none_or(|definition| {
                definition.kind != DefKind::Trait || definition.node != Some(self.declaration)
            })
            || owner.is_none_or(|owner| {
                !owner.enabled
                    || owner.node != self.owner.node
                    || !owner.attributes.contains(&self.effect)
            })
            || unit
                .retained_attributes(self.effect..self.effect + 1)
                .next()
                .is_none_or(|(attribute, forest)| {
                    attribute.span != self.span
                        || Mode::attribute(attribute, forest, unit.source.text()) != Some(self.mode)
                })
        {
            return Err("diagnostic contract actual declaration owner changed");
        }
        Ok(())
    }
}
struct Inputs<'a> {
    facts: &'a Declarations,
    report: &'a Report,
    contract: &'a Contract,
    subject: TypeId,
    arguments: crate::types::GenericArgs,
}
impl Inputs<'_> {
    fn ty(&self, slot: Slot) -> Result<TypeId, &'static str> {
        match slot {
            Slot::Subject => Ok(self.subject),
            Slot::Parameter { binding, ordinal } => {
                if self
                    .facts
                    .generics
                    .parameters
                    .get(&self.contract.definition)
                    .and_then(|parameters| parameters.get(ordinal))
                    .is_none_or(|parameter| parameter.binding != binding)
                {
                    return Err("diagnostic formal binding or ordinal changed");
                }
                let argument = self
                    .facts
                    .types
                    .arguments(self.arguments)
                    .ok_or("diagnostic arguments are missing")?
                    .nth(ordinal);
                let Some(Argument::Type(ty)) = argument else {
                    return Err("diagnostic non-type generic formatting is not implemented");
                };
                Ok(ty)
            }
            Slot::Trait => Err("trait identity is not a diagnostic subject type"),
        }
    }
    fn matches(&self, condition: Option<Condition>) -> Result<bool, &'static str> {
        Ok(match condition {
            None => true,
            Some(Condition::Unit(slot)) => {
                matches!(self.facts.types.kind(self.ty(slot)?), Some(Kind::Tuple(list)) if self.facts.types.list(list).is_some_and(<[_]>::is_empty))
            }
            Some(Condition::UnsafeFunction(slot)) => {
                let mut kind = self.facts.types.kind(self.ty(slot)?);
                if let Some(Kind::FunctionItem { signature, .. }) = kind {
                    kind = self.facts.types.kind(signature);
                }
                matches!(kind, Some(Kind::Function { unsafe_: true, .. }))
            }
            Some(Condition::Unsupported) => {
                return Err("this diagnostic condition selector is not implemented");
            }
        })
    }
    fn append_type(
        &self,
        formatter: &mut Formatter<'_>,
        text: &mut String,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        let value = formatter.ty(ty)?;
        let table = self
            .facts
            .must_use_contracts
            .as_deref()
            .and_then(|table| table.diagnostics.as_deref())
            .ok_or("diagnostic budget owner is missing")?;
        append(table, text, value)
    }
    fn trait_text(
        &self,
        formatter: &mut Formatter<'_>,
        text: &mut String,
    ) -> Result<(), &'static str> {
        let definition = self
            .report
            .resolution
            .definitions
            .get(self.contract.definition.0 as usize)
            .ok_or("diagnostic selected trait is missing")?;
        if definition.scope.0 != 0 {
            return Err("diagnostic nested trait path formatting is not implemented");
        }
        formatter.append(text, self.report.package.names.get(definition.name))?;
        let mut arguments = self
            .facts
            .types
            .arguments(self.arguments)
            .ok_or("diagnostic selected trait inputs are missing")?;
        let parenthesized = self
            .facts
            .associated
            .callable(self.contract.definition)
            .is_some()
            || self
                .facts
                .parenthesis_marker(self.report, self.contract.definition)
                .map_err(|_| "diagnostic parenthesis marker is unavailable")?
                .is_some();
        if parenthesized {
            let (Some(Argument::Type(ty)), None) = (arguments.next(), arguments.next()) else {
                return Err("diagnostic callable trait inputs are not one tuple");
            };
            let Some(Kind::Tuple(list)) = self.facts.types.kind(ty) else {
                return Err("diagnostic callable trait input is not a tuple");
            };
            formatter.append(text, "(")?;
            for (index, &child) in self
                .facts
                .types
                .list(list)
                .ok_or("diagnostic callable tuple is missing")?
                .iter()
                .enumerate()
            {
                if index != 0 {
                    formatter.append(text, ", ")?;
                }
                self.append_type(formatter, text, child)?;
            }
            return formatter.append(text, ")");
        }
        if arguments.len() == 0 {
            return Ok(());
        }
        formatter.append(text, "<")?;
        for (index, argument) in arguments.enumerate() {
            let Argument::Type(ty) = argument else {
                return Err("diagnostic non-type trait formatting is not implemented");
            };
            if index != 0 {
                formatter.append(text, ", ")?;
            }
            self.append_type(formatter, text, ty)?;
        }
        formatter.append(text, ">")
    }
    fn render(
        &self,
        formatter: &mut Formatter<'_>,
        template: &Template,
    ) -> Result<String, &'static str> {
        let mut text = String::new();
        for piece in &template.0 {
            match piece {
                Piece::Literal(value) => formatter.append(&mut text, value)?,
                Piece::Argument(Slot::Trait) => self.trait_text(formatter, &mut text)?,
                Piece::Argument(slot) => self.append_type(formatter, &mut text, self.ty(*slot)?)?,
            }
        }
        Ok(text)
    }
}
