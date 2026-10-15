//! Decode resolved bounds once; subjects and trait arguments share the declaration worklist.
use super::{Checker, DefId, Error, Kind, NodeKey, Type, Work, regions};
use crate::{
    resolve::model::{BuiltinTrait, Target},
    syntax::{
        ast::{List, TypeFlavor},
        item::GenericFlavor,
    },
    types::parameters::predicates::{Bound, Predicate},
};
#[derive(Clone, Copy)]
pub(in super::super) enum Subject {
    Parameter(DefId),
    Type(NodeKey),
}
#[derive(Clone, Copy)]
pub(in super::super) struct Syntax {
    source: NodeKey,
    subject: Subject,
    bound: Written,
}
#[derive(Clone, Copy)]
pub(in super::super) enum Written {
    Trait(Bound),
    User { path: NodeKey, definition: DefId },
    Outlives(NodeKey),
}
impl Checker<'_> {
    pub(in super::super) fn prepare_bounds(
        &mut self,
        owner: DefId,
        key: NodeKey,
        parameters: &[NodeKey],
    ) -> Result<(), Error> {
        let item = self.item(key)?;
        let mut syntax = Vec::new();
        let mut type_parameters = std::collections::HashSet::new();
        for key in parameters {
            if self.generic_syntax(*key)?.flavor == GenericFlavor::Type {
                type_parameters.insert(self.generic_binding(*key)?);
            }
        }
        for key in parameters {
            let parameter = self.generic_syntax(*key)?;
            if parameter.flavor == GenericFlavor::Lifetime {
                continue;
            }
            self.bound_list(
                *key,
                Subject::Parameter(self.generic_binding(*key)?),
                parameter.bounds,
                &type_parameters,
                &mut syntax,
            )?;
        }
        if let crate::syntax::item::ItemKind::Trait { bounds, .. } = item.kind {
            let self_definition = self
                .facts
                .associated
                .owner(owner)
                .and_then(|id| self.facts.associated.get(id))
                .ok_or_else(|| self.error(key, "missing trait Self declaration"))?
                .self_definition;
            self.bound_list(
                key,
                Subject::Parameter(self_definition),
                bounds,
                &type_parameters,
                &mut syntax,
            )?;
        }
        let clauses = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(item.where_clause)
            .to_vec();
        for node in clauses {
            let key = NodeKey { node, ..key };
            let Kind::Where {
                binders,
                ty,
                bounds,
            } = self.kind(key)
            else {
                return Err(self.error(key, "invalid where-clause"));
            };
            let subject = NodeKey { node: ty, ..key };
            if self.lifetime_argument(subject) {
                continue;
            }
            if binders.start != binders.end {
                return Err(self.error(
                    key,
                    "higher-ranked where-clauses require advanced generic relationships",
                ));
            }
            self.bound_list(
                key,
                Subject::Type(subject),
                bounds,
                &type_parameters,
                &mut syntax,
            )?;
        }
        if !syntax.is_empty() {
            self.bound_syntax.insert(owner, syntax);
        }
        Ok(())
    }
    fn bound_list(
        &mut self,
        key: NodeKey,
        subject: Subject,
        list: List,
        parameters: &std::collections::HashSet<DefId>,
        output: &mut Vec<Syntax>,
    ) -> Result<(), Error> {
        let nodes = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(list)
            .to_vec();
        for node in nodes {
            let key = NodeKey { node, ..key };
            if self.lifetime_argument(key) {
                output.push(Syntax {
                    source: key,
                    subject,
                    bound: Written::Outlives(key),
                });
                continue;
            }
            let Kind::Bound {
                optional,
                binders,
                value,
            } = self.kind(key)
            else {
                return Err(self.error(
                    key,
                    "generic type-outlives bounds require symbolic region proofs",
                ));
            };
            if binders.start != binders.end {
                return Err(self.error(
                    key,
                    "higher-ranked trait bounds require advanced generic relationships",
                ));
            }
            let value = NodeKey { node: value, ..key };
            if self.lifetime_argument(value) {
                if optional {
                    return Err(self.error(key, "lifetime bounds cannot be relaxed"));
                }
                output.push(Syntax {
                    source: key,
                    subject,
                    bound: Written::Outlives(value),
                });
                continue;
            }
            let bound = self.trait_bound(value)?;
            if optional {
                if !matches!(bound, Written::Trait(Bound::Sized)) {
                    return Err(self.error(key, "only Sized bounds can be relaxed"));
                }
                let binding = match subject {
                    Subject::Parameter(id) => Some(id),
                    Subject::Type(key) => self.direct_bound_parameter(key),
                };
                let binding = binding
                    .filter(|id| parameters.contains(id))
                    .ok_or_else(|| {
                        self.error(
                            key,
                            "relaxed Sized bounds require a declared type parameter",
                        )
                    })?;
                if !self.relaxed.insert(binding) {
                    return Err(self.error(key, "type parameter has repeated relaxed Sized bounds"));
                }
            } else {
                output.push(Syntax {
                    source: key,
                    subject,
                    bound,
                });
            }
        }
        Ok(())
    }
    fn direct_bound_parameter(&self, mut key: NodeKey) -> Option<DefId> {
        loop {
            if let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference) {
                return Some(id);
            }
            key.node = match self.kind(key) {
                Kind::Type {
                    flavor: TypeFlavor::Path,
                    elements,
                } => *self
                    .report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(elements)
                    .first()?,
                _ => return None,
            };
        }
    }
    pub(in super::super) fn trait_bound(&self, mut key: NodeKey) -> Result<Written, Error> {
        loop {
            if self.lifetime_argument(key) {
                return Err(self.error(
                    key,
                    "generic type-outlives bounds require symbolic region proofs",
                ));
            }
            if let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference) {
                let builtin = self.report.resolution.builtin_traits.identify(id);
                if builtin.is_some()
                    && crate::sema::lifetimes::arguments(self.report, key, false)?
                        .is_some_and(|list| list.start != list.end)
                {
                    return Err(self.error(key, "builtin traits do not accept generic arguments"));
                }
                return match builtin {
                    Some(BuiltinTrait::Copy) => Ok(Written::Trait(Bound::Copy)),
                    Some(BuiltinTrait::Clone) => Ok(Written::Trait(Bound::Clone)),
                    Some(BuiltinTrait::Sized) => Ok(Written::Trait(Bound::Sized)),
                    None if self.report.resolution.definitions[id.0 as usize].kind
                        == crate::resolve::model::DefKind::Trait =>
                    {
                        Ok(Written::User {
                            path: key,
                            definition: id,
                        })
                    }
                    None => Err(self.error(key, "bound requires a trait declaration")),
                };
            }
            key.node = match self.kind(key) {
                Kind::Group(node) => node,
                Kind::Type {
                    flavor: TypeFlavor::Path,
                    elements,
                } => *self
                    .report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(elements)
                    .first()
                    .ok_or_else(|| self.error(key, "empty trait bound"))?,
                _ => return Err(self.error(key, "bound requires a resolved trait")),
            };
        }
    }
    pub(in super::super) fn schedule_bounds(
        &mut self,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let Some(bounds) = self.bound_syntax.remove(&owner) else {
            return Ok(());
        };
        self.work.push(Work::Bounds(owner, context));
        for bound in bounds.iter().rev() {
            if let Written::User { path, definition } = bound.bound {
                self.bound_subject_lifetimes(path, owner)?;
                self.work
                    .push(Work::PathArguments(path, definition, context));
            }
            if let Subject::Type(key) = bound.subject {
                self.bound_subject_lifetimes(key, owner)?;
                self.work.push(Work::Node(key, false, context));
            }
        }
        self.bound_syntax.insert(owner, bounds);
        Ok(())
    }
    fn bound_subject_lifetimes(&mut self, key: NodeKey, owner: DefId) -> Result<(), Error> {
        let ast = &self.report.package.unit(key.file).configured.ast;
        let raw = self.report.resolution.definitions[owner.0 as usize].kind
            == crate::resolve::model::DefKind::Function;
        let mut work = vec![key.node];
        let mut seen = std::collections::HashSet::new();
        while let Some(node) = work.pop() {
            if !seen.insert(node) {
                continue;
            }
            if seen.len() + work.len() > 1_000_000 {
                return Err(self.error(key, "bound syntax work budget exceeded"));
            }
            let source = NodeKey { node, ..key };
            if raw {
                self.defer_bound_type(source);
            }
            match ast.node(node).kind {
                Kind::FunctionType(_) => continue,
                Kind::Type {
                    flavor: TypeFlavor::Ref | TypeFlavor::RefMut,
                    elements,
                } => {
                    let children = ast.list(elements);
                    let valid = children.len() == 2
                        && matches!(ast.node(children[0]).kind,Kind::Name(span) if self.report.package.unit(key.file).spelling(span).is_ok_and(|s|s!="'_"));
                    if !valid {
                        return Err(self.error(
                            NodeKey { node, ..key },
                            "bound references require explicit named lifetimes",
                        ));
                    }
                }
                _ => {}
            }
            ast.push_children(node, &mut work);
        }
        Ok(())
    }
    pub(in super::super) fn complete_bounds(
        &mut self,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let bounds = self.bound_syntax.remove(&owner).ok_or_else(|| {
            Error::at(crate::resolve::unit::FileId(0), 0, "missing pending bounds")
        })?;
        let mut completed = Vec::with_capacity(bounds.len());
        for &bound in &bounds {
            let ty = match bound.subject {
                Subject::Parameter(id) => self.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(bound.source, "missing bounded parameter type"))?,
                Subject::Type(key) => self.ty(key)?,
            };
            let value = match bound.bound {
                Written::Trait(bound) => bound,
                Written::User { path, definition } => {
                    Bound::Trait(self.trait_path(path, definition, ty, context)?)
                }
                Written::Outlives(key) => {
                    let Kind::Name(span) = self.kind(key) else {
                        return Err(self.error(key, "outlives bound requires a named lifetime"));
                    };
                    if self
                        .report
                        .package
                        .unit(key.file)
                        .spelling(span)
                        .is_ok_and(|name| name == "'_")
                    {
                        return Err(self.error(key, "outlives bound requires a named lifetime"));
                    }
                    Bound::Outlives(self.region(key, Some(key.node), context)?)
                }
            };
            if matches!(bound.subject, Subject::Parameter(_))
                && !matches!(self.facts.types.kind(ty), Some(Type::Parameter(_)))
            {
                return Err(self.error(bound.source, "trait bounds require a type parameter"));
            }
            let value = Predicate { ty, bound: value };
            if value
                .contains_projections(&self.facts.types)
                .map_err(|e| self.error(bound.source, e))?
            {
                self.facts.unreduced.retain_predicate(bound.source, value);
            }
            completed.push(crate::sema::generics::Predicate {
                source: bound.source,
                value,
            });
        }
        self.facts.generics.predicates.insert(owner, completed);
        self.retain_raw_bounds(owner, bounds)?;
        Ok(())
    }
}

impl Syntax {
    pub(in super::super) fn trait_path(self) -> Option<(NodeKey, DefId)> {
        match self.bound {
            Written::User { path, definition } => Some((path, definition)),
            Written::Trait(_) | Written::Outlives(_) => None,
        }
    }
    pub(in super::super) fn subject(self) -> Option<NodeKey> {
        match self.subject {
            Subject::Type(key) => Some(key),
            Subject::Parameter(_) => None,
        }
    }
}
