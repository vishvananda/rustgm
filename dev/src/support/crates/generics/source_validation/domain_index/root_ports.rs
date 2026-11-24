//! Header and clause roots use their real declaration scope and binder frames.
use super::{Context, DomainIndex, HeaderRef, Types, Work, g, s};

impl DomainIndex<'_, '_> {
    fn definition_roots(&mut self, work: &mut Work) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        for alias in rows.type_aliases() {
            if self
                .headers
                .scopes()
                .scope_row(alias.declaration, work)?
                .is_some()
            {
                self.types.alias_root(self.headers, *alias, work)?;
            } else if !self.types.closed_root(alias.target, work)? {
                return Err("source alias target requires a foreign parameter, frame or body");
            }
        }
        self.alias_normalization_roots(work)?;
        // An impl's result can name only the formals of its own frame.
        for row in rows.associated_definitions() {
            let parent = rows
                .declarations
                .get(row.declaration.0 as usize)
                .and_then(|raw| raw.definition.parent)
                .ok_or("source associated definition has no actual impl parent")?;
            let Some(HeaderRef::Implementation(index)) = self.headers.header(parent, work)? else {
                return Err("source associated definition has no actual impl header");
            };
            let header = &rows.implementations[index as usize];
            let context = Context {
                owner: header.scope,
                frame: Some(self.binders.declaration(header.self_definition, work)?),
                body: None,
            };
            self.types.root(self.headers, context, row.value, work)?;
        }
        Ok(())
    }

    fn alias_normalization_roots(&self, work: &mut Work) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        let normalizations = &rows.cold().alias_normalizations;
        work.charge(normalizations.len())?;
        let mut previous = None;
        for group in normalizations.chunk_by(|left, right| left.declaration == right.declaration) {
            let owner = group[0].declaration;
            if previous.is_some_and(|previous| previous >= owner.0)
                || self.headers.scopes().scope_row(owner, work)?.is_none()
            {
                return Err("source alias normalizations lack an ordered actual scope");
            }
            previous = Some(owner.0);
            let Some(HeaderRef::TypeAlias(index)) = self.headers.header(owner, work)? else {
                return Err("source alias normalizations lack an actual alias header");
            };
            let target = rows.type_aliases()[index as usize].target;
            let context = Context {
                owner,
                frame: None,
                body: None,
            };
            let mut covers_target = false;
            for row in group {
                if !matches!(
                    rows.types.get(row.target.0 as usize),
                    Some(s::Type::Projection(_))
                ) {
                    return Err("source alias normalization requires an actual projection");
                }
                self.types.root(self.headers, context, row.target, work)?;
                covers_target |= row.target == target;
            }
            if matches!(
                rows.types.get(target.0 as usize),
                Some(s::Type::Projection(_))
            ) && !covers_target
            {
                return Err("source alias normalizations omit the projected target");
            }
        }
        Ok(())
    }

    pub(super) fn header_roots(&mut self, work: &mut Work) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        self.trait_alias_roots(work)?;
        for header in &rows.nominals {
            let context = Context {
                owner: header.scope,
                frame: Some(self.binders.declaration(header.declaration, work)?),
                body: None,
            };
            self.types
                .root(self.headers, context, header.self_type, work)?;
            for field in header.fields.slice(&rows.fields)? {
                self.types.root(self.headers, context, field.ty, work)?;
            }
            self.requirements(context, header.inputs, work)?;
        }
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
        for header in &rows.associated_types {
            let context = Context {
                owner: header.scope,
                frame: Some(self.binders.declaration(header.declaration, work)?),
                body: None,
            };
            self.types
                .root(self.headers, context, header.projection, work)?;
            self.requirements(context, header.inputs, work)?;
            self.requirements(context, header.bounds, work)?;
        }
        self.definition_roots(work)?;
        self.constant_roots(work)?;
        for projection in &rows.projections {
            self.used_traits[projection.trait_application.0 as usize] = true;
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

    fn trait_alias_roots(&mut self, work: &mut Work) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        for alias in rows.trait_aliases() {
            let context = Context {
                owner: alias.declaration,
                frame: None,
                body: None,
            };
            self.types
                .root(self.headers, context, alias.self_type, work)?;
            let Some(s::Type::Parameter(slot)) = rows.types.get(alias.self_type.0 as usize) else {
                return Err("source trait alias Self has no actual parameter slot");
            };
            let actual = rows
                .slots
                .get(slot.0 as usize)
                .ok_or("source trait alias Self slot is unknown")?;
            if actual.owner != alias.self_definition
                || actual.definition != Some(alias.self_definition)
                || actual.kind != (s::ParameterKind::Type { sized: None })
                || actual.default.is_some()
                || self
                    .headers
                    .scopes()
                    .scope_row(alias.declaration, work)?
                    .is_none_or(|scope| scope.captures != 1)
            {
                return Err("source trait alias Self changed its lexical unsized binding");
            }
            self.requirements(context, alias.inputs, work)?;
        }
        Ok(())
    }
    fn constant_roots(&mut self, work: &mut Work) -> Result<(), &'static str> {
        if !self.declaration_policies {
            return Ok(());
        }
        let rows = self.headers.rows();
        work.charge(rows.constants().len())?;
        if !rows.constants().iter().any(|row| !row.body.evaluated()) {
            return Ok(());
        }
        work.charge(rows.declarations.len())?;
        for (owner, constant) in rows.constant_rows() {
            if constant.body.evaluated() {
                continue;
            }
            let frame = self.binders.declaration_frame(owner, work)?;
            let context = if let Some(frame) = frame {
                Context {
                    owner,
                    frame: Some(frame),
                    body: None,
                }
            } else {
                let parent = rows.declarations[owner.0 as usize]
                    .definition
                    .parent
                    .ok_or("source required constant lost its actual parent")?;
                Context {
                    owner: parent,
                    frame: Some(self.binders.declaration(parent, work)?),
                    body: None,
                }
            };
            self.types.root(self.headers, context, constant.ty, work)?;
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
        let rows = self.headers.rows();
        super::super::reject_foreign_value(rows, signature.result)?;
        self.types
            .root(self.headers, context, signature.result, work)?;
        for &input in signature.parameters.slice(&rows.roots)? {
            super::super::reject_foreign_value(rows, input)?;
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
                    s::Requirement::Outlives(_)
                    | s::Requirement::UnstableFeature(_)
                    | s::Requirement::ProjectionEqual { .. } => {
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
            s::Requirement::ProjectionEqual { projection, value } => {
                if !matches!(
                    self.headers.rows().types.get(projection.0 as usize),
                    Some(s::Type::Projection(_))
                ) {
                    return Err("source result binding does not name a declared-result projection");
                }
                self.types.root(self.headers, context, *projection, work)?;
                self.types.root(self.headers, context, *value, work)
            }
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
