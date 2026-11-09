//! Whole declaration debts are checked once per applied template, never paid here.
use super::{
    Context, DomainIndex, Work, g,
    nodes::{Arena, Rewrite},
    s, uses,
};

impl Arena {
    pub(super) fn application_inputs(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        application: s::FunctionApplicationRef,
        caller: Option<Context>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = domains.headers.rows();
        let applied = &rows.function_applications[application.0 as usize];
        let target = domains.target(applied.declaration.declaration, work)?;
        // DomainIndex already checked exact full arity and the actual caller's
        // scope. Visiting every argument retains unused formal formation debts.
        // A captured method's leading region arguments are its impl's lifetimes.
        let captures = match target {
            super::super::Target::Function(header) => header.regions.captures as usize,
            super::super::Target::Constructor { .. } => 0,
        };
        for (index, argument) in applied.arguments.slice(&rows.arguments)?.iter().enumerate() {
            work.charge(1)?;
            let root = match *argument {
                s::Argument::Type(root) => root,
                // A lifetime-generic enum constructor's or captured method's
                // leading region has no formation root; its domain was
                // checked with the caller.
                s::Argument::Region(_)
                    if matches!(target, super::super::Target::Constructor { .. })
                        || index < captures =>
                {
                    continue;
                }
                s::Argument::Region(_) => {
                    return Err(
                        "source regional application debts require a complete wider profile",
                    );
                }
                // A const term's type is a primitive row with no formation debt.
                s::Argument::Const(_) => continue,
            };
            self.root(
                domains,
                uses::key(domains, root, Rewrite::Original, caller, work)?,
                work,
            )?;
        }
        self.applied_clauses(domains, application, caller, target.inputs(), work)?;
        if let Some(parent) = target.parent() {
            let range = match domains.headers.header(parent, work)? {
                Some(super::super::HeaderRef::Trait(index)) => rows.traits[index as usize].inputs,
                Some(super::super::HeaderRef::Implementation(index)) => {
                    rows.implementations[index as usize].inputs
                }
                _ => {
                    return Err("source applied debt has an unsupported actual declaration parent");
                }
            };
            self.applied_clauses(domains, application, caller, range, work)?;
        }
        Ok(())
    }
    fn applied_clauses(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        application: s::FunctionApplicationRef,
        caller: Option<Context>,
        range: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = domains.headers.rows();
        for clause in range.slice(&rows.clauses)? {
            work.charge(1)?;
            if clause
                .binder
                .is_some_and(|binder| rows.binders[binder.0 as usize].count != 0)
            {
                return Err(
                    "source quantified applied debts require their complete binder-opening profile",
                );
            }
            match &clause.requirement {
                s::Requirement::Trait {
                    application: goal, ..
                } => {
                    let goal = &rows.trait_applications[goal.0 as usize];
                    self.applied_root(domains, application, caller, goal.subject, work)?;
                    for argument in goal.arguments.slice(&rows.arguments)? {
                        work.charge(1)?;
                        let s::Argument::Type(root) = *argument else {
                            return Err(
                                "source regional trait debt requires a complete wider profile",
                            );
                        };
                        self.applied_root(domains, application, caller, root, work)?;
                    }
                }
                s::Requirement::ProjectionEqual { projection, value } => {
                    self.applied_root(domains, application, caller, *projection, work)?;
                    self.applied_root(domains, application, caller, *value, work)?;
                }
                s::Requirement::TypeOutlives { .. } | s::Requirement::Outlives(_) => {
                    return Err(
                        "source applied lifetime debts require a complete wider opening profile",
                    );
                }
                // RequiredKind::Call retains this exact target header debt.
                // It has no type/region root to rewrite in this allocation plan.
                s::Requirement::UnstableFeature(name) => {
                    work.charge(name.len().saturating_add(1))?;
                }
            }
        }
        Ok(())
    }
    fn applied_root(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        application: s::FunctionApplicationRef,
        caller: Option<Context>,
        root: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let key = uses::key(
            domains,
            root,
            Rewrite::Application(application),
            caller,
            work,
        )?;
        self.root(domains, key, work)?;
        Ok(())
    }
}
