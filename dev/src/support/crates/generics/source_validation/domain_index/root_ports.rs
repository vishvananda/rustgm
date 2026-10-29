//! Header and clause roots use their real declaration scope and binder frames.
use super::{Context, DomainIndex, HeaderRef, Types, Work, g, s};

impl DomainIndex<'_, '_> {
    pub(super) fn header_roots(&mut self, work: &mut Work) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        for header in &rows.traits {
            let context = Context {
                owner: header.scope,
                frame: Some(self.binders.declaration(header.declaration, work)?),
                body: None,
            };
            self.types
                .root(self.headers, context, header.self_type, work)?;
            let Some(s::Type::Parameter(slot)) = rows.types.get(header.self_type.0 as usize) else {
                return Err("source trait Self has no actual parameter slot");
            };
            let actual = rows
                .slots
                .get(slot.0 as usize)
                .ok_or("source trait Self slot is unknown")?;
            if actual.owner != header.self_definition
                || actual.definition != Some(header.self_definition)
                || self
                    .headers
                    .scopes()
                    .complete_type_slots(header.self_definition, work)?
                    != 1
            {
                return Err("source trait Self is not its exact declared binding");
            }
            self.requirements(context, header.inputs, work)?;
            self.bounds(context, header.region_bounds, work)?;
        }
        for header in &rows.implementations {
            let context = Context {
                owner: header.scope,
                frame: Some(self.binders.declaration(header.self_definition, work)?),
                body: None,
            };
            self.types
                .root(self.headers, context, header.self_type, work)?;
            if let Some(application) = header.trait_application {
                self.trait_use(context, application, work)?;
            }
            self.requirements(context, header.inputs, work)?;
            self.bounds(context, header.region_bounds, work)?;
        }
        for header in &rows.functions {
            let context = Context {
                owner: header.scope,
                frame: Some(header.original.binder),
                body: None,
            };
            self.signature(context, header.original, work)?;
            self.signature(
                Context {
                    frame: Some(header.callable.binder),
                    ..context
                },
                header.callable,
                work,
            )?;
            self.requirements(context, header.inputs, work)?;
            self.bounds(context, header.regions.bounds, work)?;
            if let Some(receiver) = header.receiver {
                self.types
                    .root(self.headers, context, receiver.base, work)?;
                if receiver.formal as usize >= header.formals.slice(&rows.formals)?.len() {
                    return Err("source receiver is outside its actual formal inputs");
                }
            }
        }
        Ok(())
    }
    pub(super) fn signature(
        &self,
        context: Context,
        signature: g::Signature,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if context.frame != Some(signature.binder) {
            return Err("source signature root lost its actual binder context");
        }
        self.types
            .root(self.headers, context, signature.result, work)?;
        for &input in signature.parameters.slice(&self.headers.rows().roots)? {
            self.types.root(self.headers, context, input, work)?;
        }
        Ok(())
    }
    fn requirements(
        &mut self,
        context: Context,
        range: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        for input in range.slice(&rows.clauses)? {
            work.charge(1)?;
            let inner = if let Some(binder) = input.binder {
                let frame = &rows.binders[binder.0 as usize];
                if frame.parent != context.frame || frame.owner != context.owner {
                    return Err("source clause binder changed its exact declaration context");
                }
                Context {
                    frame: Some(binder),
                    ..context
                }
            } else {
                context
            };
            self.requirement(inner, &input.requirement, work)?;
            if let Some(slot) = input.self_subject {
                let subject = match &input.requirement {
                    s::Requirement::Trait { application, .. } => {
                        rows.trait_applications[application.0 as usize].subject
                    }
                    s::Requirement::TypeOutlives { subject, .. } => *subject,
                    s::Requirement::Outlives(_) | s::Requirement::UnstableFeature(_) => {
                        return Err("source regional clause cannot claim a typed Self subject");
                    }
                };
                if rows.types.get(subject.0 as usize) != Some(&s::Type::Parameter(slot)) {
                    return Err("source canonical Self classification has a foreign actual slot");
                }
                self.headers
                    .scopes()
                    .type_argument_position(context.owner, slot, work)?;
                let Some(HeaderRef::Trait(index)) = self.headers.header(context.owner, work)?
                else {
                    return Err("source Self clause belongs to a non-trait header");
                };
                if rows.traits[index as usize].self_type != subject {
                    return Err(
                        "source canonical Self classification names another actual parameter",
                    );
                }
            }
        }
        Ok(())
    }
    pub(super) fn requirement(
        &mut self,
        context: Context,
        requirement: &s::Requirement,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match requirement {
            s::Requirement::Trait { application, .. } => {
                self.trait_use(context, *application, work)
            }
            s::Requirement::TypeOutlives { subject, shorter } => {
                self.types.root(self.headers, context, *subject, work)?;
                Types::region(self.headers, &self.binders, context, *shorter, work)
            }
            s::Requirement::Outlives(bound) => self.bound(context, *bound, work),
            // The whole declaration/call input remains in the semantic request plan;
            // this non-type row contributes no fictitious type root.
            s::Requirement::UnstableFeature(name) => work.charge(name.len().saturating_add(1)),
        }
    }
    pub(super) fn bounds(
        &self,
        context: Context,
        range: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        for &bound in range.slice(&self.headers.rows().region_bounds)? {
            self.bound(context, bound, work)?;
        }
        Ok(())
    }
    pub(super) fn bound(
        &self,
        context: Context,
        bound: s::RegionBound,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        Types::region(self.headers, &self.binders, context, bound.longer, work)?;
        Types::region(self.headers, &self.binders, context, bound.shorter, work)
    }
}
