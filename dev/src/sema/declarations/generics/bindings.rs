//! Associated bindings keep their own syntax range and become separate typed predicates.
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
}
#[derive(Clone, Copy)]
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
            let value = self.associated_binding_value(key)?;
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
        match self.kind(key) {
            Kind::Constraint {
                value: Some(value),
                bounds,
                ..
            } if bounds.start == bounds.end => Ok(value),
            Kind::Constraint { .. } => Err(self.error(
                key,
                "associated type bounds require projected bound obligations",
            )),
            _ => Err(self.error(key, "positional arguments must precede associated bindings")),
        }
    }

    pub(in crate::sema::declarations) fn complete_associated_bindings(
        &mut self,
        binding: Binding,
        output: &mut Vec<crate::sema::generics::Predicate>,
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
        let Binding {
            path,
            predicate,
            original,
            scope,
            context,
            ..
        } = binding;
        let Some(list) = self.associated_bindings.bindings(path) else {
            return Ok(false);
        };
        if let Some(headers) = &mut self.associated_bindings.trait_headers {
            headers.push(binding);
            return Ok(false);
        }
        let mut pending = false;
        for offset in list.start..list.end {
            let node = self.report.package.unit(path.file).configured.ast.edges[offset as usize];
            let source = NodeKey { node, ..path };
            let binding = self
                .report
                .resolution
                .associated_bindings
                .get(&source)
                .ok_or_else(|| self.error(source, "associated binding requires one member name"))?;
            let name = binding.name;
            let application = self.binding_projection(source, predicate, name)?;
            if self.prepare_binding_arguments(source, application, context)? {
                pending = true;
                continue;
            }
            let ty = self.applied_binding(source, application, context, false)?;
            let original_ty = if original == predicate {
                self.applied_binding(source, application, context, true)?
            } else {
                let original = self.binding_projection(source, original, name)?;
                self.applied_binding(source, original, context, true)?
            };
            if self
                .facts
                .associated
                .families
                .contains_key(&DefId(application.item.0))
            {
                // The equality's result cannot erase the family selector's
                // input requirements. Retain that typed root at its own source.
                self.store(source, original_ty)?;
            }
            let value = NodeKey {
                node: self.associated_binding_value(source)?,
                ..path
            };
            let result = self.ty(value)?;
            let original_result = self.facts.unreduced.node(value).unwrap_or(result);
            let value = Predicate {
                ty,
                bound: Bound::ProjectionEqual(result),
            };
            let original = self.bound_clause(
                source,
                scope,
                Predicate {
                    ty: original_ty,
                    bound: Bound::ProjectionEqual(original_result),
                },
            )?;
            self.facts.unreduced.retain_predicate(source, original);
            output.push(crate::sema::generics::Predicate {
                source,
                value: self.bound_clause(source, scope, value)?,
            });
        }
        Ok(pending)
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
            if self.complete_associated_bindings(binding, &mut values)? {
                self.drain(steps, limit)?;
                values.clear();
                if self.complete_associated_bindings(binding, &mut values)? {
                    return Err(self.error(binding.path, "unfinished associated binding arguments"));
                }
            }
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
