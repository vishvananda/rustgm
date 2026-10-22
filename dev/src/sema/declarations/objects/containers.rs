//! Only explicit type-parameter lifetime bounds supply generic object defaults.
use super::{Checker, DefId, Error, Kind, NodeKey, RegionId, regions};
use crate::support::hash::HashMap;
use crate::{resolve::model::Target, types::arguments::Argument};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Static,
    Parameter(DefId),
}
#[derive(Clone, Copy)]
enum Choice<T> {
    Unique(T),
    Ambiguous,
}
#[derive(Default)]
pub(super) struct Defaults {
    bounds: HashMap<DefId, Choice<Source>>,
    slots: HashMap<DefId, HashMap<DefId, usize>>,
    arguments: HashMap<NodeKey, Vec<RegionId>>,
    nodes: HashMap<NodeKey, Choice<RegionId>>,
}

impl Checker<'_> {
    /// Consume the already decoded bound while its resolved declaration
    /// identities are available; later inferred field bounds are not defaults.
    pub(in crate::sema::declarations) fn retain_object_container_bound(
        &mut self,
        parameter: DefId,
        key: NodeKey,
    ) {
        let source = if let Some(Target::Definition(id)) =
            self.facts.node(key).and_then(|facts| facts.reference)
        {
            Source::Parameter(id)
        } else if matches!(self.kind(key), Kind::Name(span)
            if self.report.package.unit(key.file).spelling(span).is_ok_and(|s| s == "'static"))
        {
            Source::Static
        } else {
            // Invalid or placeholder bounds receive their ordinary diagnostic.
            return;
        };
        self.object_syntax
            .containers
            .bounds
            .entry(parameter)
            .and_modify(|choice| {
                if matches!(choice, Choice::Unique(previous) if *previous != source) {
                    *choice = Choice::Ambiguous;
                }
            })
            .or_insert(Choice::Unique(source));
    }

    pub(in crate::sema::declarations) fn object_argument_default(
        &mut self,
        application: NodeKey,
        owner: DefId,
        parameter: DefId,
        mut node: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let Some(choice) = self
            .object_syntax
            .containers
            .bounds
            .get(&parameter)
            .copied()
        else {
            return Ok(());
        };
        loop {
            match self.kind(node) {
                Kind::Group(child) | Kind::Attributed { value: child, .. } => node.node = child,
                Kind::TraitType { opaque: false, .. } => break,
                _ => return Ok(()),
            }
        }
        let choice = match choice {
            Choice::Ambiguous => Choice::Ambiguous,
            Choice::Unique(Source::Static) => Choice::Unique(RegionId::STATIC),
            Choice::Unique(Source::Parameter(binding)) => {
                self.object_container_arguments(application, owner, context)?;
                let slot = self.object_syntax.containers.slots[&owner]
                    .get(&binding)
                    .copied()
                    .ok_or_else(|| {
                        self.error(node, "object default lifetime is not an owner input")
                    })?;
                let region = self.object_syntax.containers.arguments[&application]
                    .get(slot)
                    .copied()
                    .ok_or_else(|| self.error(node, "missing object default lifetime argument"))?;
                Choice::Unique(region)
            }
        };
        self.object_syntax.containers.nodes.insert(node, choice);
        Ok(())
    }

    fn object_container_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.object_syntax.containers.arguments.contains_key(&key) {
            return Ok(());
        }
        let parameters = &self.facts.generics.parameters[&owner];
        let count = parameters
            .iter()
            .take_while(|p| matches!(p.argument, Argument::Lifetime(_)))
            .count();
        self.object_syntax
            .containers
            .slots
            .entry(owner)
            .or_insert_with(|| {
                parameters
                    .iter()
                    .take(count)
                    .enumerate()
                    .map(|(index, p)| (p.binding, index))
                    .collect()
            });
        let (list, omitted) = self.supplied_arguments(key, owner)?;
        let nodes = list.map_or(&[][..], |list| {
            self.report.package.unit(key.file).configured.ast.list(list)
        });
        let nodes = (0..count)
            .map(|index| {
                index
                    .checked_sub(omitted)
                    .and_then(|index| nodes.get(index).copied())
            })
            .collect::<Vec<_>>();
        let mut arguments = Vec::with_capacity(count);
        for node in nodes {
            let region = self.region(key, node, context)?;
            self.regions.record(context, region);
            arguments.push(region);
        }
        self.object_syntax
            .containers
            .arguments
            .insert(key, arguments);
        Ok(())
    }

    pub(in crate::sema::declarations) fn object_application_lifetime(
        &self,
        key: NodeKey,
        index: usize,
    ) -> Option<RegionId> {
        self.object_syntax
            .containers
            .arguments
            .get(&key)
            .and_then(|arguments| arguments.get(index))
            .copied()
    }

    pub(super) fn container_object_region(&self, key: NodeKey) -> Result<Option<RegionId>, Error> {
        match self.object_syntax.containers.nodes.get(&key) {
            None => Ok(None),
            Some(Choice::Unique(region)) => Ok(Some(*region)),
            Some(Choice::Ambiguous) => Err(self.error(
                key,
                "ambiguous container object lifetime requires an explicit bound",
            )),
        }
    }
}
