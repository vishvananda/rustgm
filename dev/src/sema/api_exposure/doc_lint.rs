//! Documentation lints use exported declarations and retained configured attributes.
use super::{Budget, PublicBudget, Reachability};
use crate::{
    resolve::{
        expansion::Report,
        lint_policy::Level,
        model::{Access, DefId, DefKind},
        package::Error,
        unit::{FileId, NodeKey, Unit},
    },
    sema::{members::Name, model::Declarations},
    support::{diagnostics::WarningCode, hash::HashMap, retained::Growth},
    syntax::{
        ast::{Kind, NodeId},
        attribute::Input,
    },
    types::{ItemId, Kind as Type},
};

impl Reachability {
    pub(super) fn lint_missing_docs(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), Error> {
        let docs = Documentation::collect(report, &mut self.budget)?;
        for (_, key) in report.package.crate_roots() {
            docs.check(report, facts, key, &mut self.budget)?;
        }
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.budget.charge(1, 0)?;
            let id =
                DefId(u32::try_from(index).map_err(|_| {
                    Error::unlocated("documentation lint identity exceeds 32 bits")
                })?);
            if !self.exports.contains(&id)
                || definition.kind.is_local()
                || matches!(
                    definition.kind,
                    DefKind::Constructor | DefKind::Variant | DefKind::ExternCrate
                )
            {
                continue;
            }
            if let Some(key) = definition.node
                && !docs.hidden_inherent_owner(report, facts, id)
            {
                docs.check(report, facts, key, &mut self.budget)?;
                if matches!(definition.kind, DefKind::Struct | DefKind::Enum) {
                    docs.check_fields(report, facts, id, definition.kind, &mut self.budget)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct Documentation(HashMap<NodeKey, bool>);

impl Documentation {
    fn collect(report: &Report, budget: &mut Budget) -> Result<Self, Error> {
        let mut documented = Self::default();
        for (file, unit) in report.package.units.iter().enumerate() {
            if unit.include_context.is_some() {
                continue;
            }
            let file = FileId(
                u32::try_from(file)
                    .map_err(|_| Error::unlocated("too many documentation files"))?,
            );
            for owner in unit.configured.owners.iter().filter(|owner| owner.enabled) {
                budget.charge(1, 0)?;
                for (attr, forest) in unit.retained_attributes(owner.attributes.clone()) {
                    budget.charge(1, 0)?;
                    if attr.name(forest, unit.source.text()) != Some("doc") {
                        continue;
                    }
                    let hidden = if let Input::Delimited(group) = attr.input {
                        let range = forest.children(group).ok_or_else(|| {
                            Error::at(file, attr.span.start, "invalid documentation option list")
                        })?;
                        budget.charge((range.end - range.start) as usize, 0)?;
                        let mut index = range.start;
                        let mut hidden = false;
                        while index < range.end {
                            hidden |= forest.name(unit.source.text(), index) == Some("hidden");
                            index = forest.after(index);
                        }
                        hidden
                    } else {
                        false
                    };
                    if hidden || matches!(attr.input, Input::Doc(_) | Input::Equals(_)) {
                        let key = NodeKey {
                            file,
                            node: target(unit, owner.node, budget)?,
                        };
                        documented.record(key, hidden, budget)?;
                    }
                }
            }
        }
        Ok(documented)
    }

    fn record(&mut self, key: NodeKey, hidden: bool, budget: &mut Budget) -> Result<(), Error> {
        let growth = Growth::map::<NodeKey, bool>(
            self.0.len(),
            self.0.capacity(),
            usize::from(!self.0.contains_key(&key)),
        );
        budget.charge(1 + growth.work, growth.bytes)?;
        self.0
            .try_reserve(usize::from(!self.0.contains_key(&key)))
            .map_err(|_| Error::unlocated("documentation lint index allocation failed"))?;
        *self.0.entry(key).or_default() |= hidden;
        Ok(())
    }

    fn check(
        &self,
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        if facts.safety_policies.missing_docs(key) == Level::Allow {
            return Ok(());
        }
        let unit = report.package.unit(key.file);
        let node = target(unit, key.node, budget)?;
        if self.0.contains_key(&NodeKey { node, ..key }) {
            return Ok(());
        }
        facts.must_use_diagnostic(
            report,
            key,
            unit.configured.ast.node(node).span.start,
            0,
            WarningCode::MissingDocs,
            "exported declaration is missing documentation".into(),
        )
    }

    fn check_fields(
        &self,
        report: &Report,
        facts: &mut Declarations,
        id: DefId,
        kind: DefKind,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let owner = ItemId(id.0);
        for index in 0..facts.members.variants(owner).len() {
            budget.charge(1, 0)?;
            let variant = facts.members.variants(owner)[index];
            if kind == DefKind::Enum
                && let Some(key) = facts
                    .members
                    .variant_origin(variant)
                    .and_then(crate::sema::members::Origin::diagnostic)
            {
                self.check(report, facts, key, budget)?;
            }
            let size = facts.members.fields(variant).map_or(0, <[_]>::len);
            for index in 0..size {
                budget.charge(1, 0)?;
                let Some(field) = facts
                    .members
                    .fields(variant)
                    .and_then(|fields| fields.get(index))
                else {
                    return Err(Error::unlocated("documentation lint lost a checked field"));
                };
                if field.access == Access::Public
                    && matches!(field.name, Name::Named(_))
                    && let Some(key) = field.origin.diagnostic()
                {
                    self.check(report, facts, key, budget)?;
                }
            }
        }
        Ok(())
    }

    fn hidden_inherent_owner(&self, report: &Report, facts: &Declarations, member: DefId) -> bool {
        let nominal = || {
            let owner = facts.associated.get(facts.associated.owner(member)?)?;
            let schema = facts.associated.schema(owner.self_definition)?;
            let Type::Adt(item, _) = facts.types.kind(schema.self_type)? else {
                return None;
            };
            report.resolution.definitions.get(item.0 as usize)?.node
        };
        nominal()
            .and_then(|key| self.0.get(&key))
            .copied()
            .unwrap_or(false)
    }
}

fn target(unit: &Unit, mut node: NodeId, budget: &mut Budget) -> Result<NodeId, Error> {
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        budget.charge(1, 0)?;
        node = value;
    }
    Ok(node)
}
