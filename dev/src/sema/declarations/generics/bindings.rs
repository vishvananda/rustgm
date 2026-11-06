//! Associated bindings keep their own syntax range and become separate typed predicates.
mod constraint;
mod nest;
mod proj;
mod selectors;
use super::bounds::lexical::Scope;
use super::{Checker, DefId, Error, Kind, NodeKey, Type, Work, regions};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::model::Namespace,
    syntax::ast::{List, NodeId},
    types::{
        GenericArgs, ItemId, TypeId,
        parameters::predicates::{Bound, Predicate},
        projection,
    },
};

#[derive(Clone, Copy)]
struct Arguments {
    positional: List,
    bindings: List,
}
#[derive(Default)]
pub(in super::super) struct Pending {
    paths: HashMap<NodeKey, Arguments>,
    trait_headers: Option<Vec<Binding>>,
    prepared_arguments: HashSet<NodeKey>,
    projected: Option<Box<proj::Pending>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::sema::declarations) struct Binding {
    pub owner: DefId,
    pub path: NodeKey,
    pub predicate: Predicate,
    pub original: Predicate,
    pub scope: Option<Scope>,
    pub context: regions::Context,
}
impl Pending {
    pub(in super::super) fn begin_trait_headers(&mut self) {
        self.trait_headers = Some(Vec::new());
    }
    pub(in super::super) fn positional(&self, key: NodeKey) -> Option<List> {
        self.paths.get(&key).map(|p| p.positional)
    }
    pub(in super::super) fn bindings(&self, key: NodeKey) -> Option<List> {
        self.paths.get(&key).map(|p| p.bindings)
    }
}

impl Checker<'_> {
    pub(in super::super) fn schedule_associated_bindings(
        &mut self,
        path: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        if crate::sema::lifetimes::call_arguments(self.report, path)?.is_some() {
            return Ok(());
        }
        let Some(list) = self.prepare_associated_bindings(path)? else {
            return Ok(());
        };
        for offset in (list.start..list.end).rev() {
            let node = self.report.package.unit(path.file).configured.ast.edges[offset as usize];
            let key = NodeKey { node, ..path };
            let Some(value) = self.associated_binding_rhs(key)? else {
                continue;
            };
            self.work.push(Work::Node(
                NodeKey {
                    node: value,
                    ..path
                },
                false,
                context,
            ));
        }
        Ok(())
    }

    pub(in super::super) fn prepare_associated_bindings(
        &mut self,
        path: NodeKey,
    ) -> Result<Option<List>, Error> {
        if crate::sema::lifetimes::call_arguments(self.report, path)?.is_some() {
            return Ok(None);
        }
        if !self.associated_bindings.paths.contains_key(&path) {
            let Some(list) = crate::sema::lifetimes::arguments(self.report, path, false)? else {
                return Ok(None);
            };
            let ast = &self.report.package.unit(path.file).configured.ast;
            let Some(first) = ast
                .list(list)
                .iter()
                .position(|&node| matches!(ast.node(node).kind, Kind::Constraint { .. }))
            else {
                return Ok(None);
            };
            let middle = list.start
                + u32::try_from(first).map_err(|_| self.error(path, "too many bound arguments"))?;
            self.associated_bindings.paths.insert(
                path,
                Arguments {
                    positional: List {
                        start: list.start,
                        end: middle,
                    },
                    bindings: List {
                        start: middle,
                        end: list.end,
                    },
                },
            );
        }
        Ok(Some(self.associated_bindings.paths[&path].bindings))
    }

    pub(in super::super) fn associated_binding_value(&self, key: NodeKey) -> Result<NodeId, Error> {
        self.associated_binding_rhs(key)?
            .ok_or_else(|| self.error(key, "this associated binding requires an equality"))
    }
    pub(in super::super) fn associated_binding_rhs(
        &self,
        key: NodeKey,
    ) -> Result<Option<NodeId>, Error> {
        match self.kind(key) {
            Kind::Constraint {
                value: Some(value),
                bounds,
                ..
            } if bounds.start == bounds.end => Ok(Some(value)),
            Kind::Constraint {
                value: None,
                bounds,
                ..
            } if bounds.start != bounds.end => Ok(None),
            Kind::Constraint { .. } => Err(self.error(
                key,
                "associated binding requires either an equality or bounds",
            )),
            _ => Err(self.error(key, "positional arguments must precede associated bindings")),
        }
    }

    pub(in crate::sema::declarations) fn complete_associated_bindings(
        &mut self,
        binding: Binding,
        output: &mut Vec<crate::sema::generics::Predicate>,
        dependencies: &mut Vec<super::super::aliases::Edge>,
    ) -> Result<bool, Error> {
        if crate::sema::lifetimes::call_arguments(self.report, binding.path)?.is_some() {
            self.call_output_binding(
                binding.path,
                binding.predicate,
                binding.original,
                binding.scope,
                output,
            )?;
            return Ok(false);
        }
        let path = binding.path;
        let Some(list) = self.associated_bindings.bindings(path) else {
            return Ok(false);
        };
        if self.associated_bindings.trait_headers.is_some() {
            self.stage_header_binding(binding, list)?;
            return Ok(false);
        }
        let mut pending = false;
        for offset in list.start..list.end {
            let node = self.report.package.unit(path.file).configured.ast.edges[offset as usize];
            let source = NodeKey { node, ..path };
            match self.complete_binding_constraint(source, binding)? {
                constraint::Outcome::Pending => pending = true,
                constraint::Outcome::Equality(value) => output.push(value),
                constraint::Outcome::Bounds(subject) => {
                    pending |= self.complete_projected_bounds(
                        source,
                        binding,
                        subject,
                        output,
                        dependencies,
                    )?;
                }
            }
        }
        Ok(pending)
    }

    fn stage_header_binding(&mut self, binding: Binding, list: List) -> Result<(), Error> {
        let path = binding.path;
        for offset in list.start..list.end {
            let node = self.report.package.unit(path.file).configured.ast.edges[offset as usize];
            if self
                .associated_binding_rhs(NodeKey { node, ..path })?
                .is_none()
            {
                self.prepare_projected_owner(binding.owner)?;
                self.bound_subject_lifetimes(path, binding.owner)?;
                break;
            }
        }
        self.associated_bindings
            .trait_headers
            .as_mut()
            .ok_or_else(|| Error::unlocated("missing trait header staging"))?
            .push(binding);
        Ok(())
    }

    /// Lookup uses complete ordinary headers; equality requirements are published
    /// together before immutable trait environments or application proofs exist.
    pub(in super::super) fn complete_trait_bindings(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let pending = self
            .associated_bindings
            .trait_headers
            .take()
            .unwrap_or_default();
        let mut additions = Vec::with_capacity(pending.len());
        for binding in pending {
            let mut values = Vec::new();
            let mut dependencies = Vec::new();
            loop {
                values.clear();
                dependencies.clear();
                *steps = steps.saturating_add(1);
                if *steps > limit {
                    return Err(self.error(
                        binding.path,
                        "associated bindings exceed declaration work budget",
                    ));
                }
                if !self.complete_associated_bindings(binding, &mut values, &mut dependencies)? {
                    break;
                }
                self.drain(steps, limit)?;
            }
            self.retain_alias_edges(binding.owner, dependencies)?;
            additions.push((ItemId(binding.owner.0), values));
        }
        for (owner, predicates) in &additions {
            self.facts
                .generics
                .predicates
                .entry(DefId(owner.0))
                .or_default()
                .extend(predicates.iter().copied());
        }
        if !additions.is_empty() {
            self.facts
                .generics
                .traits
                .as_mut()
                .ok_or_else(|| {
                    Error::at(crate::resolve::unit::FileId(0), 0, "missing trait headers")
                })?
                .complete_bindings(&self.facts.types, additions)
                .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        }
        Ok(())
    }

    fn binding_projection(
        &mut self,
        source: NodeKey,
        predicate: Predicate,
        name: crate::resolve::names::NameId,
    ) -> Result<projection::Application, Error> {
        let root = predicate;
        let mut work = vec![predicate];
        let mut seen = HashSet::default();
        let mut selected = None;
        while let Some(predicate) = work.pop() {
            if !seen.insert(predicate) {
                continue;
            }
            if seen.len().saturating_add(work.len()) > 1_000_000 {
                return Err(self.error(source, "associated binding lookup budget exceeded"));
            }
            let Some((trait_application, _)) = predicate.bound.trait_parts() else {
                continue;
            };
            let application = self
                .facts
                .types
                .trait_application(trait_application)
                .ok_or_else(|| self.error(source, "missing associated binding trait"))?;
            let owner = DefId(application.item.0);
            if let Some(scope) = self.report.resolution.definitions[owner.0 as usize].members {
                let member = self
                    .report
                    .resolution
                    .bindings
                    .get(scope, name, Namespace::Type)
                    .map(|b| b.target);
                if let Some(member) = member.filter(|id| {
                    self.facts.associated.trait_owner(*id) == Some(owner)
                        && self.facts.types.projection_schema(ItemId(id.0)).is_some()
                }) {
                    let application = projection::Application {
                        item: ItemId(member.0),
                        subject: predicate.ty,
                        trait_application,
                        arguments: GenericArgs::EMPTY,
                    };
                    // A member declared by the named trait takes precedence.
                    // Otherwise every inherited declaration participates in ambiguity.
                    if predicate == root {
                        return Ok(application);
                    }
                    if selected
                        .replace(application)
                        .is_some_and(|prior| prior != application)
                    {
                        return Err(
                            self.error(source, "ambiguous inherited associated type binding")
                        );
                    }
                }
            }
            let requirements = self
                .facts
                .generics
                .traits
                .as_mut()
                .ok_or_else(|| {
                    Error::at(
                        source.file,
                        0,
                        "associated binding requires completed trait contracts",
                    )
                })?
                .requirements(&mut self.facts.types, predicate)
                .map_err(|e| self.error(source, e))?;
            for clause in &requirements.implied {
                let implied = clause.ordinary().map_err(|e| self.error(source, e))?;
                if matches!(
                    implied.bound,
                    Bound::Trait(_) | Bound::ConstTrait(_) | Bound::MaybeConstTrait(_)
                ) {
                    work.push(implied);
                }
            }
        }
        selected.ok_or_else(|| self.error(source, "trait bound has no such associated type"))
    }
}
