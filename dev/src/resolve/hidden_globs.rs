//! Completed import candidates prove private winners hide exported glob names.
use super::{
    expansion::{Report, attribute_effects::Effects},
    imports::Work,
    lint_policy::producer::Producer,
    model::{Access, Binding, Namespace, Origin, Resolution},
    package::{Error, Package},
    table::BindingKey,
    unit::NodeKey,
};
use crate::{
    support::{retained::Growth, source::OriginalImage},
    syntax::{ast::Node, token_tree::Span},
};
use std::sync::Arc;

struct Witness {
    key: NodeKey,
    image: Arc<OriginalImage>,
    node: Node,
}
pub(crate) struct Cause {
    key: BindingKey,
    winner: Binding,
    counterpart: (BindingKey, Binding),
    declaration: Option<Witness>,
    glob: u32,
    owner: Witness,
    reexport: Witness,
}
impl Witness {
    fn new(package: &Package, key: NodeKey) -> Self {
        let unit = package.unit(key.file);
        Self {
            key,
            image: Arc::clone(unit.source.original_image()),
            node: unit.configured.ast.node(key.node),
        }
    }
    fn validate(&self, package: &Package) -> Result<(), Error> {
        let unit = package
            .units
            .get(self.key.file.0 as usize)
            .ok_or_else(|| Error::at(self.key.file, 0, "hidden glob source file changed"))?;
        if !Arc::ptr_eq(&self.image, unit.source.original_image())
            || unit.configured.ast.nodes.get(self.key.node.get() as usize) != Some(&self.node)
            || unit.configured.active.get(self.key.node.get() as usize) != Some(&true)
        {
            return Err(Error::at(
                self.key.file,
                0,
                "hidden glob source owner changed",
            ));
        }
        Ok(())
    }
}
impl Cause {
    pub(crate) fn site(
        &self,
        report: &Report,
        producer: &Producer,
    ) -> Result<(NodeKey, Span, Namespace), Error> {
        producer.attribute_read(self.owner.key, 5)?;
        self.owner.validate(&report.package)?;
        self.reexport.validate(&report.package)?;
        if let Some(declaration) = &self.declaration {
            declaration.validate(&report.package)?;
        }
        if report.resolution.bindings.get_key(self.key) != Some(&self.winner)
            || report.resolution.bindings.get_key(self.counterpart.0) != Some(&self.counterpart.1)
            || report
                .resolution
                .definitions
                .get(self.counterpart.1.target.0 as usize)
                .and_then(|definition| definition.node)
                != self.declaration.as_ref().map(|witness| witness.key)
            || report
                .resolution
                .imports
                .get(self.glob as usize)
                .is_none_or(|import| {
                    !import.glob
                        || import.access != Access::Public
                        || import.node != self.reexport.key
                })
        {
            return Err(Error::at(
                self.owner.key.file,
                0,
                "hidden glob resolution identity changed",
            ));
        }
        Ok((self.owner.key, self.owner.node.span, self.key.2))
    }
}
impl Work {
    fn private_owner(facts: &Resolution, binding: Binding) -> Option<NodeKey> {
        match binding.origin {
            Origin::Direct => facts.definitions.get(binding.target.0 as usize)?.node,
            Origin::Import(index) => facts.imports.get(index as usize).map(|import| import.node),
            Origin::Glob(_) => None,
        }
    }
    fn hidden_candidates(
        &mut self,
        facts: &Resolution,
        owner: NodeKey,
    ) -> Result<Vec<(BindingKey, Binding)>, Error> {
        let mut result = Vec::new();
        for (&key, candidates) in &self.candidates {
            self.lineage_budget.charge(owner, 1 + candidates.len(), 0)?;
            if facts.bindings.get_key(key).is_none_or(|winner| {
                winner.access == Access::Public
                    || winner.ambiguous
                    || matches!(winner.origin, Origin::Glob(_))
            }) {
                continue;
            }
            // Only candidate facts survive here; effective export is checked later.
            for candidate in candidates {
                if candidate.access == Access::Public && !candidate.ambiguous {
                    let growth =
                        Growth::vector::<(BindingKey, Binding)>(result.len(), result.capacity(), 1);
                    self.lineage_budget
                        .charge(owner, 1 + growth.work, growth.bytes)?;
                    result.push((key, *candidate));
                }
            }
        }
        // Stable key/first import order, independent of randomized candidate buckets.
        self.lineage_budget.charge(
            owner,
            result
                .len()
                .saturating_mul(1 + result.len().checked_ilog2().unwrap_or(0) as usize),
            0,
        )?;
        result.sort_unstable_by_key(|(key, candidate)| {
            (
                key.0.0,
                key.1.0,
                key.2.name(),
                key.3.key(),
                match candidate.origin {
                    Origin::Glob(index) => index,
                    _ => u32::MAX,
                },
            )
        });
        Ok(result)
    }
}
impl Report {
    pub(super) fn complete_hidden_globs(&mut self, mut work: Work) -> Result<(), Error> {
        let root = &self.package.modules[0];
        let owner = NodeKey {
            file: root.file,
            node: root.root,
        };
        let candidates = work.hidden_candidates(&self.resolution, owner)?;
        if candidates.is_empty() {
            return Ok(());
        }
        let exported = work.exported_globs(&self.resolution, owner)?;
        let mut causes = Vec::new();
        let mut last = None;
        for (key, candidate) in candidates {
            work.lineage_budget.charge(owner, 1, 0)?;
            if last == Some(key) {
                continue;
            }
            let Some(winner) = self.resolution.bindings.get_key(key).copied() else {
                continue;
            };
            if winner.access == Access::Public || winner.ambiguous {
                continue;
            }
            let Origin::Glob(index) = candidate.origin else {
                continue;
            };
            if !exported.contains(&index) {
                continue;
            }
            let Some(site) = Work::private_owner(&self.resolution, winner) else {
                continue;
            };
            let glob = &self.resolution.imports[index as usize];
            if glob.access != Access::Public {
                continue;
            }
            let counterpart = work.counterpart(site, key, index)?;
            let declaration = self.resolution.definitions[counterpart.1.target.0 as usize]
                .node
                .map(|source| Witness::new(&self.package, source));
            let growth = Growth::vector::<Cause>(causes.len(), causes.capacity(), 1);
            work.lineage_budget
                .charge(site, 1 + growth.work, growth.bytes)?;
            causes.push(Cause {
                key,
                winner,
                counterpart,
                declaration,
                glob: index,
                owner: Witness::new(&self.package, site),
                reexport: Witness::new(&self.package, glob.node),
            });
            last = Some(key);
        }
        if causes.is_empty() {
            return Ok(());
        }
        if self.attribute_effects.is_none() {
            let producer = Producer::new(0, 0);
            producer.diagnostic_read(owner, size_of::<Effects>())?;
            let mut effects = Effects::default();
            effects.policy = Some(producer);
            self.attribute_effects = Some(Box::new(effects));
        }
        let effects = self
            .attribute_effects
            .as_deref_mut()
            .ok_or_else(|| Error::at(owner.file, 0, "hidden glob source producer is required"))?;
        let producer = effects.policy.get_or_insert_with(|| Producer::new(0, 0));
        producer.diagnostic_payload(
            owner,
            work.lineage_budget.steps,
            work.lineage_budget.bytes + size_of::<Vec<Cause>>(),
        )?;
        producer.hidden_globs = Some(Box::new(causes));
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/reexport_shadows.rs"]
mod tests;

impl Producer {
    pub(crate) fn hidden_glob_events(&self) -> &[Cause] {
        self.hidden_globs.as_deref().map_or(&[], Vec::as_slice)
    }
}
