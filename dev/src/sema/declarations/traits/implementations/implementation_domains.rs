//! Implementation permission consumes the exact retained module reference.
use super::super::super::{Checker, DefId, Error};
use crate::{
    resolve::{
        impl_restrictions,
        model::{DefKind, Target},
    },
    sema::associated::SourceOwner,
    syntax::ast::Kind,
};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn validate_implementation_domain(
        &mut self,
        owner: SourceOwner,
        definition: DefId,
    ) -> Result<(), Error> {
        let Some(clause) = impl_restrictions::clause(self.report, definition)? else {
            if self.report.resolution.definitions[definition.0 as usize]
                .node
                .is_none()
                && self
                    .report
                    .resolution
                    .builtin_traits
                    .identify(definition)
                    .is_none()
            {
                self.facts.body_work.canonical_steps =
                    self.facts.body_work.canonical_steps.saturating_add(1);
                let domain = self
                    .facts
                    .associated
                    .imported_implementation_domain(definition)
                    .map_err(|message| self.error(owner.source, message))?;
                if let crate::support::crates::generics::source::ImplementationDomain::RestrictedTo{..}=domain {
                    let client=self.report.package.source_client().ok_or_else(||self.error(owner.source,"imported implementation domain requires its actual source client"))?;
                    let catalog=self.report.resolution.crates.as_ref()
                        .ok_or_else(||self.error(owner.source,"imported implementation catalog is missing"))?
                        .origin_catalog().map_err(|e|self.error(owner.source,e))?
                        .ok_or_else(||self.error(owner.source,"imported implementation catalog is incomplete"))?;
                    let id=catalog.declaration(definition).map_err(|e|self.error(owner.source,e))?;
                    let (provider,_,_)=catalog.declaration_coordinate(id).map_err(|e|self.error(owner.source,e))?;
                    if client.provider()!=provider ||client.current()==provider {
                        return Err(self.error(owner.source,"imported trait domain changed its actual provider/current identity"));
                    }
                    return Err(self.error(owner.source,"trait cannot be implemented outside its restricted module"));
                }
            }
            return Ok(());
        };
        self.facts.body_work.canonical_steps =
            self.facts.body_work.canonical_steps.saturating_add(1);
        let Kind::ImplRestriction { path } = self.kind(clause) else {
            return Err(self.error(clause, "trait implementation domain clause changed"));
        };
        let target = self
            .facts
            .node(crate::resolve::unit::NodeKey {
                file: clause.file,
                node: path,
            })
            .and_then(|facts| facts.reference)
            .ok_or_else(|| self.error(clause, "trait implementation domain is unresolved"))?;
        let Target::Definition(target) = target else {
            return Err(self.error(clause, "trait implementation domain is not a module"));
        };
        let module =
            impl_restrictions::module(&self.report.package, &self.report.resolution, target)
                .map_err(|message| self.error(clause, message))?;
        let declaration = self
            .report
            .resolution
            .definitions
            .get(owner.self_definition.0 as usize)
            .filter(|declaration| {
                declaration.kind == DefKind::SelfType && declaration.node == Some(owner.source)
            })
            .ok_or_else(|| {
                self.error(
                    owner.source,
                    "implementation domain lost its actual Self owner",
                )
            })?;
        let current = self
            .report
            .resolution
            .scopes
            .get(declaration.scope.0 as usize)
            .map(|scope| scope.module)
            .ok_or_else(|| {
                self.error(
                    owner.source,
                    "implementation domain owner module is missing",
                )
            })?;
        let allowed = impl_restrictions::within(&self.report.package, current, module, || {
            self.facts.body_work.canonical_steps =
                self.facts.body_work.canonical_steps.saturating_add(1);
            Ok(())
        })
        .map_err(|message| self.error(owner.source, message))?;
        if !allowed {
            return Err(self.error(
                owner.source,
                "trait cannot be implemented outside its restricted module",
            ));
        }
        Ok(())
    }
}
