//! Selected method goals are complete substitutions of the actual target formals.
use super::{
    Context, DomainIndex, Work, g,
    matching::Opening,
    nodes::{Arena, Rewrite},
    s, uses,
};

impl Arena {
    pub(super) fn method_goal(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        request: &s::CallRequest,
        caller: Context,
        opening: Opening,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let Some(method) = request.method else {
            return Ok(());
        };
        let rows = domains.headers.rows();
        let application = rows
            .function_applications
            .get(request.target.0 as usize)
            .ok_or("source method goal lost its actual target application")?;
        let target = domains.function(application.declaration.declaration, work)?;
        let Some(parent) = target.parent else {
            return Err("source method goal has no actual associated owner");
        };
        let Some(s::HeaderRef::Implementation(index)) = domains.headers.header(parent, work)?
        else {
            return self.trait_goal(domains, request, caller, opening, work);
        };
        let declared = rows.implementations[index as usize]
            .trait_application
            .ok_or("source selected method has no actual implementation predicate")?;
        if declared == method {
            // The old profiles retain their exact raw-header relationship.
            return Ok(());
        }
        let expected = rows
            .trait_applications
            .get(declared.0 as usize)
            .ok_or("source selected method predicate is absent")?;
        let actual = rows
            .trait_applications
            .get(method.0 as usize)
            .ok_or("source selected method caller goal is absent")?;
        let left = expected.arguments.slice(&rows.arguments)?;
        let right = actual.arguments.slice(&rows.arguments)?;
        if expected.declaration != actual.declaration || left.len() != right.len() {
            return Err("source selected method changed its complete trait application");
        }
        let roots = std::iter::once(Ok((expected.subject, actual.subject))).chain(
            left.iter().zip(right).map(|(&a, &b)| match (a, b) {
                (s::Argument::Type(a), s::Argument::Type(b)) => Ok((a, b)),
                _ => Err("source selected method requires actual type-only trait arguments"),
            }),
        );
        for roots in roots {
            work.charge(1)?;
            let (left, right) = roots?;
            self.goal_root(
                domains,
                (left, right),
                Rewrite::Application(request.target),
                caller,
                opening,
                work,
            )?;
        }
        Ok(())
    }
    fn trait_goal(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        request: &s::CallRequest,
        caller: Context,
        opening: Opening,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let rows = domains.headers.rows();
        let application = &rows.function_applications[request.target.0 as usize];
        let target = domains.function(application.declaration.declaration, work)?;
        let Some(s::HeaderRef::Trait(index)) = target
            .parent
            .map(|parent| domains.headers.header(parent, work))
            .transpose()?
            .flatten()
        else {
            return Err("source method goal has no actual trait declaration");
        };
        let trait_ = &rows.traits[index as usize];
        let method = request.method.ok_or("source trait method goal is absent")?;
        let actual = &rows.trait_applications[method.0 as usize];
        let arguments = actual.arguments.slice(&rows.arguments)?;
        let applied = application.arguments.slice(&rows.arguments)?;
        let range = domains
            .headers
            .scopes()
            .declared_slots(trait_.scope, work)?;
        let mut argument = 0;
        for (offset, slot) in range.slice(&rows.slots)?.iter().enumerate() {
            work.charge(1)?;
            let reference = s::SlotRef(
                range.start
                    + u32::try_from(offset)
                        .map_err(|_| "source method trait slot ordinal overflow")?,
            );
            let position =
                domains
                    .headers
                    .scopes()
                    .type_argument_position(target.scope, reference, work)?;
            let Some(&s::Argument::Type(expected)) = applied.get(position as usize) else {
                return Err("source method target lost its actual trait type formal");
            };
            let actual = if slot.definition == Some(trait_.self_definition) {
                actual.subject
            } else {
                let Some(&s::Argument::Type(root)) = arguments.get(argument) else {
                    return Err("source method goal omitted its actual trait type argument");
                };
                argument += 1;
                root
            };
            self.goal_root(
                domains,
                (expected, actual),
                Rewrite::Original,
                caller,
                opening,
                work,
            )?;
        }
        if argument != arguments.len() {
            return Err("source method goal fabricated an extra trait argument");
        }
        Ok(())
    }
    fn goal_root(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        roots: (g::TypeRef, g::TypeRef),
        rewrite: Rewrite,
        caller: Context,
        opening: Opening,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let left = uses::key(domains, roots.0, rewrite, Some(caller), work)?;
        let right = uses::key(domains, roots.1, Rewrite::Original, Some(caller), work)?;
        let left = self.root(domains, left, work)?;
        let right = self.root(domains, right, work)?;
        self.same(domains, left, right, opening, work)
    }
}
