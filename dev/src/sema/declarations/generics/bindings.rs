//! Associated bindings keep their own syntax range and become separate typed predicates.
use super::{Checker, DefId, Error, Kind, NodeKey, Type, Work, regions};
use crate::{
    resolve::model::Namespace,
    syntax::ast::{List, NodeId},
    types::{
        GenericArgs, ItemId, TypeId,
        parameters::predicates::{Bound, Predicate},
        projection,
    },
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct Arguments {
    positional: List,
    bindings: List,
}
#[derive(Default)]
pub(in super::super) struct Pending {
    paths: HashMap<NodeKey, Arguments>,
    trait_headers: Option<Vec<HeaderBinding>>,
}
struct HeaderBinding {
    owner: DefId,
    path: NodeKey,
    predicate: Predicate,
    original: Predicate,
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

    pub(in super::super) fn complete_associated_bindings(
        &mut self,
        owner: DefId,
        path: NodeKey,
        predicate: Predicate,
        original: Predicate,
        output: &mut Vec<crate::sema::generics::Predicate>,
    ) -> Result<(), Error> {
        let Some(list) = self.associated_bindings.bindings(path) else {
            return Ok(());
        };
        if let Some(headers) = &mut self.associated_bindings.trait_headers {
            headers.push(HeaderBinding {
                owner,
                path,
                predicate,
                original,
            });
            return Ok(());
        }
        for offset in list.start..list.end {
            let node = self.report.package.unit(path.file).configured.ast.edges[offset as usize];
            let source = NodeKey { node, ..path };
            let binding = self
                .report
                .resolution
                .associated_bindings
                .get(&source)
                .ok_or_else(|| self.error(source, "associated binding requires one member name"))?;
            if binding.arguments.is_some() {
                return Err(self.error(
                    source,
                    "associated type binding arguments require advanced generic relationships",
                ));
            }
            let name = binding.name;
            let ty = self.binding_projection(source, predicate, name)?;
            let original_ty = if original == predicate {
                ty
            } else {
                self.binding_projection(source, original, name)?
            };
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
            self.facts.unreduced.retain_predicate(
                source,
                Predicate {
                    ty: original_ty,
                    bound: Bound::ProjectionEqual(original_result),
                }
                .into(),
            );
            output.push(crate::sema::generics::Predicate {
                source,
                value: value.into(),
            });
        }
        Ok(())
    }

    /// Lookup uses complete ordinary headers; equality requirements are published
    /// together before immutable trait environments or application proofs exist.
    pub(in super::super) fn complete_trait_bindings(&mut self) -> Result<(), Error> {
        let pending = self
            .associated_bindings
            .trait_headers
            .take()
            .unwrap_or_default();
        let mut additions = Vec::with_capacity(pending.len());
        for binding in pending {
            let mut values = Vec::new();
            self.complete_associated_bindings(
                binding.owner,
                binding.path,
                binding.predicate,
                binding.original,
                &mut values,
            )?;
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
    ) -> Result<TypeId, Error> {
        let root = predicate;
        let mut work = vec![predicate];
        let mut seen = HashSet::new();
        let mut selected = None;
        while let Some(predicate) = work.pop() {
            if !seen.insert(predicate) {
                continue;
            }
            if seen.len().saturating_add(work.len()) > 1_000_000 {
                return Err(self.error(source, "associated binding lookup budget exceeded"));
            }
            let Bound::Trait(trait_application) = predicate.bound else {
                continue;
            };
            let application = self
                .facts
                .types
                .trait_application(trait_application)
                .ok_or_else(|| self.error(source, "missing associated binding trait"))?;
            let owner = DefId(application.item.0);
            if let Some(scope) = self.report.resolution.definitions[owner.0 as usize].members {
                let member = self.report.resolution.scopes[scope.0 as usize]
                    .bindings
                    .get(&(name, Namespace::Type))
                    .map(|b| b.target);
                if let Some(member) = member.filter(|id| {
                    self.facts.associated.trait_owner(*id) == Some(owner)
                        && self.facts.types.projection_schema(ItemId(id.0)).is_some()
                }) {
                    let ty = self.intern(
                        source,
                        Type::Projection(projection::Application {
                            item: ItemId(member.0),
                            subject: predicate.ty,
                            trait_application,
                            arguments: GenericArgs::EMPTY,
                        }),
                    )?;
                    // A member declared by the named trait takes precedence.
                    // Otherwise every inherited declaration participates in ambiguity.
                    if predicate == root {
                        return Ok(ty);
                    }
                    if selected.replace(ty).is_some_and(|prior| prior != ty) {
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
                if matches!(implied.bound, Bound::Trait(_)) {
                    work.push(implied);
                }
            }
        }
        selected.ok_or_else(|| self.error(source, "trait bound has no such associated type"))
    }
}
