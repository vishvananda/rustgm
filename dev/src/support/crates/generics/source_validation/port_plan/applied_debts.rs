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
        let header = domains.function(applied.declaration.declaration, work)?;
        // DomainIndex already checked exact full arity and the actual caller's
        // scope. Visiting every argument retains unused formal formation debts.
        for argument in applied.arguments.slice(&rows.arguments)? {
            work.charge(1)?;
            let s::Argument::Type(root) = *argument else {
                return Err("source regional application debts require a complete wider profile");
            };
            self.root(
                domains,
                uses::key(domains, root, Rewrite::Original, caller, work)?,
                work,
            )?;
        }
        self.applied_clauses(domains, application, caller, header.inputs, work)?;
        if let Some(parent) = header.parent {
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
            match clause.requirement {
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
                s::Requirement::TypeOutlives { .. } | s::Requirement::Outlives(_) => {
                    return Err(
                        "source applied lifetime debts require a complete wider opening profile",
                    );
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
