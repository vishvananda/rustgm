//! Diagnostic summaries share typed structure, never executable or trait evidence.
use super::{Declarations, Error, Report};
use crate::{
    resolve::{model::DefId, unit::FileId},
    types::{Kind, TypeId, parameters::clauses::Clause},
};
#[derive(Clone, Copy)]
pub(in crate::sema) enum Requirement {
    Declaration(DefId),
    Closure,
}
#[derive(Clone, Copy)]
pub(in crate::sema) enum TypeResult {
    Absent,
    Direct(Requirement),
    Components,
}
impl TypeResult {
    fn present(self) -> bool {
        !matches!(self, Self::Absent)
    }
}
impl Declarations {
    pub(in crate::sema) fn must_use_type(
        &mut self,
        report: &Report,
        root: TypeId,
    ) -> Result<TypeResult, Error> {
        if self.must_use_contracts.is_none()
            && !matches!(
                self.types.kind(root),
                Some(Kind::Tuple(_) | Kind::Array { .. } | Kind::Opaque(..) | Kind::Closure { .. })
            )
        {
            return Ok(TypeResult::Absent);
        }
        if let Some(result) = self
            .must_use_contracts
            .as_deref()
            .and_then(|table| table.type_requirements.get(&root))
            .copied()
        {
            return Ok(result);
        }
        let mut pending = vec![(root, false)];
        let mut steps = 0_usize;
        while let Some((ty, complete)) = pending.pop() {
            steps += 1;
            if steps > 1_000_000 {
                return Err(error("must_use type work budget exceeded"));
            }
            if self
                .must_use_contracts
                .as_deref()
                .is_some_and(|table| table.type_requirements.contains_key(&ty))
            {
                continue;
            }
            let kind = self
                .types
                .kind(ty)
                .ok_or_else(|| error("must_use result type is unknown"))?;
            if !complete {
                match kind {
                    Kind::Tuple(list) => {
                        let children = self
                            .types
                            .list(list)
                            .ok_or_else(|| error("invalid must_use tuple type"))?;
                        pending.push((ty, true));
                        pending.extend(children.iter().rev().map(|&child| (child, false)));
                        continue;
                    }
                    Kind::Array { element, length } if length != 0 => {
                        pending.push((ty, true));
                        pending.push((element, false));
                        continue;
                    }
                    _ => {}
                }
            }
            let requirement = self.observed_type_requirement(report, kind)?;
            let table = self.must_use_contracts.get_or_insert_default();
            if table.type_requirements.len() >= 1_000_000 {
                return Err(error("must_use type storage budget exceeded"));
            }
            table.type_requirements.insert(ty, requirement);
            table.work.type_steps += 1;
        }
        Ok(self
            .must_use_contracts
            .as_deref()
            .and_then(|table| table.type_requirements.get(&root).copied())
            .unwrap_or(TypeResult::Absent))
    }
    fn observed_type_requirement(&self, report: &Report, kind: Kind) -> Result<TypeResult, Error> {
        let direct = match kind {
            Kind::Adt(item, _) => self
                .must_use_contract(report, DefId(item.0))?
                .map(|_| Requirement::Declaration(DefId(item.0))),
            Kind::Closure { .. } => Some(Requirement::Closure),
            Kind::Opaque(id, _) => {
                let declaration = self
                    .opaques
                    .declarations
                    .get(&id)
                    .ok_or_else(|| error("must_use opaque declaration is missing"))?;
                self.must_use_traits(report, &declaration.bounds)?
            }
            Kind::Projection(application) => {
                let declaration = self
                    .associated
                    .type_contract(DefId(application.item.0))
                    .ok_or_else(|| error("must_use projected result contract is missing"))?;
                self.must_use_traits(report, &declaration.bounds)?
            }
            Kind::Dynamic { shape, .. } => {
                let item = self
                    .types
                    .dynamic_shape(shape)
                    .ok_or_else(|| error("must_use object shape is missing"))?
                    .principal;
                item.map(|item| {
                    self.must_use_contract(report, DefId(item.0))
                        .map(|contract| contract.map(|_| Requirement::Declaration(DefId(item.0))))
                })
                .transpose()?
                .flatten()
            }
            Kind::Tuple(list) => {
                let children = self
                    .types
                    .list(list)
                    .ok_or_else(|| error("invalid must_use tuple type"))?;
                return Ok(
                    if children.iter().any(|child| {
                        self.must_use_contracts
                            .as_deref()
                            .and_then(|table| table.type_requirements.get(child))
                            .is_some_and(|result| result.present())
                    }) {
                        TypeResult::Components
                    } else {
                        TypeResult::Absent
                    },
                );
            }
            Kind::Array { element, length } if length != 0 => {
                return Ok(
                    if self
                        .must_use_contracts
                        .as_deref()
                        .and_then(|table| table.type_requirements.get(&element))
                        .is_some_and(|result| result.present())
                    {
                        TypeResult::Components
                    } else {
                        TypeResult::Absent
                    },
                );
            }
            _ => None,
        };
        Ok(direct.map_or(TypeResult::Absent, TypeResult::Direct))
    }
    fn must_use_traits(
        &self,
        report: &Report,
        bounds: &[Clause],
    ) -> Result<Option<Requirement>, Error> {
        for &clause in bounds.iter().rev() {
            let predicate = match clause {
                Clause::Predicate(predicate) => predicate,
                Clause::ForAll(id) => {
                    self.types
                        .quantified(id)
                        .ok_or_else(|| error("must_use quantified bound is missing"))?
                        .predicate
                }
            };
            if let Some((application, _)) = predicate.bound.trait_parts() {
                let item = self
                    .types
                    .trait_application(application)
                    .ok_or_else(|| error("must_use trait bound is missing"))?
                    .item;
                if let Some(requirement) = self.must_use_trait(report, item)? {
                    return Ok(Some(requirement));
                }
            }
        }
        Ok(None)
    }
}
fn error(message: &'static str) -> Error {
    Error::at(FileId(0), 0, message)
}
