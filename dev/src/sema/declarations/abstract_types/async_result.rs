//! Async outputs use the existing opaque identity and associated equality machinery.
use super::super::{BinderId, Checker, DefId, Error, ItemId, NodeKey, Type, TypeId, regions};
use crate::{
    sema::opaque::Declaration,
    syntax::item::ItemKind,
    types::{
        GenericArgs, copy, opaque,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        projection, traits,
    },
};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn async_result(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
        output: TypeId,
    ) -> Result<TypeId, Error> {
        let ItemKind::Function { flags, .. } = self.item(key)?.kind else {
            return Err(self.error(key, "async result requires its actual function declaration"));
        };
        if !flags.asynchronous {
            return Ok(output);
        }
        if flags.constant {
            return Err(self.error(key, "async functions cannot be const"));
        }
        let (future, member) =
            crate::sema::compiler_items::futures::signature_roles(self.report, &self.facts)?;
        let captures = self.opaque_captures(
            key,
            owner,
            regions::Context {
                binder: Some(binder),
                position: regions::Position::Output,
            },
            None,
            None,
        )?;
        let captures = self
            .facts
            .types
            .intern_arguments(&captures)
            .map_err(|e| self.error(key, e))?;
        let id = self
            .facts
            .types
            .declare_opaque(opaque::Declaration {
                trait_result: self.facts.associated.trait_owner(owner).is_some(),
                sized: true,
                copies: copy::Properties::default(),
            })
            .map_err(|e| self.error(key, e))?;
        let application = self.intern(key, Type::Opaque(id, captures))?;
        let trait_application = self
            .facts
            .types
            .intern_trait_application(traits::Application {
                item: ItemId(future.0),
                arguments: GenericArgs::EMPTY,
            })
            .map_err(|e| self.error(key, e))?;
        let projection = self.intern(
            key,
            Type::Projection(projection::Application {
                item: ItemId(member.0),
                subject: application,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        let bounds = vec![
            Clause::Predicate(Predicate {
                ty: application,
                bound: Bound::Trait(trait_application),
            }),
            Clause::Predicate(Predicate {
                ty: projection,
                bound: Bound::ProjectionEqual(output),
            }),
        ];
        self.facts.opaques.owners.entry(owner).or_default().push(id);
        self.facts.opaques.declarations.insert(
            id,
            Declaration {
                owner,
                origin: key.into(),
                application,
                bounds,
                hidden: None,
                inferred: None,
                free_regions: Vec::new(),
            },
        );
        // This is the actual authored body's result, not a fabricated future representation.
        self.facts.opaques.body_results.insert(owner, output);
        Ok(application)
    }
}
