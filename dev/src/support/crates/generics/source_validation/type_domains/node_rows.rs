//! Type recipes are traversed once; callable recursion is not a recipe cycle.
use super::{BoundUse, Builder, HeaderRef, Summary, Work, g, s};

impl Builder<'_, '_> {
    pub(super) fn dependencies(
        &mut self,
        node: usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        if node < self.result.type_count {
            match rows.types[node] {
                s::Type::Nominal { arguments, .. } => {
                    for argument in arguments.slice(&rows.arguments)?.iter().rev() {
                        if let s::Argument::Type(ty) = argument {
                            self.push_type(*ty, work)?;
                        }
                    }
                }
                s::Type::Projection(reference) => {
                    let row = rows
                        .projections
                        .get(reference.0 as usize)
                        .ok_or("unknown source projection recipe")?;
                    self.push_type(row.subject, work)?;
                    let application = rows
                        .trait_applications
                        .get(row.trait_application.0 as usize)
                        .ok_or("unknown projection trait application")?;
                    for range in [application.arguments, row.arguments] {
                        for argument in range.slice(&rows.arguments)?.iter().rev() {
                            if let s::Argument::Type(ty) = argument {
                                self.push_type(*ty, work)?;
                            }
                        }
                    }
                }
                s::Type::OpaqueFuture(reference) => {
                    let row = rows
                        .opaque_futures
                        .get(reference.0 as usize)
                        .ok_or("unknown source opaque future recipe")?;
                    self.push_type(row.output, work)?;
                    for argument in row.captures.slice(&rows.arguments)?.iter().rev() {
                        if let s::Argument::Type(ty) = argument {
                            self.push_type(*ty, work)?;
                        }
                    }
                }
                s::Type::Reference {
                    referent: child, ..
                }
                | s::Type::RawPointer { pointee: child, .. }
                | s::Type::Slice(child) => self.push_type(child, work)?,
                s::Type::Tuple(elements) => {
                    for &element in elements.slice(&rows.roots)?.iter().rev() {
                        self.push_type(element, work)?;
                    }
                }
                s::Type::Function(signature) => {
                    self.push_type(signature.result, work)?;
                    for &input in signature.parameters.slice(&rows.roots)?.iter().rev() {
                        self.push_type(input, work)?;
                    }
                }
                s::Type::FunctionItem(application) => {
                    if application.0 as usize >= self.result.function_count {
                        return Err("source function item crosses its application table namespace");
                    }
                    self.push(
                        (
                            self.result
                                .type_count
                                .saturating_add(application.0 as usize),
                            false,
                        ),
                        work,
                    )?;
                }
                s::Type::ApplicationSignature(_) => {
                    return Err(
                        "source application signature recipes require a complete wider profile",
                    );
                }
                s::Type::Scalar(_) | s::Type::Str | s::Type::Never | s::Type::Parameter(_) => {}
            }
        } else {
            let offset = node - self.result.type_count;
            let arguments = if offset < self.result.function_count {
                rows.function_applications[offset].arguments
            } else {
                let application = rows
                    .trait_applications
                    .get(offset - self.result.function_count)
                    .ok_or("source trait application is unknown")?;
                self.push_type(application.subject, work)?;
                application.arguments
            };
            // Region arguments have no recipe child; `application` admits
            // only a lifetime-generic enum constructor's leading regions.
            for argument in arguments.slice(&rows.arguments)?.iter().rev() {
                work.charge(1)?;
                if let s::Argument::Type(ty) = argument {
                    self.push_type(*ty, work)?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn finish(&self, node: usize, work: &mut Work) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        if node >= self.result.type_count {
            let offset = node - self.result.type_count;
            return if offset < self.result.function_count {
                self.application(offset, work)
            } else {
                self.trait_application(offset - self.result.function_count, work)
            };
        }
        Ok(match rows.types[node] {
            s::Type::Projection(reference) => self.projection_summary(reference, work)?,
            s::Type::OpaqueFuture(reference) => self.opaque_summary(reference, work)?,
            s::Type::Scalar(_) | s::Type::Str | s::Type::Never => Summary::default(),
            s::Type::Nominal {
                declaration,
                arguments,
            } => self.nominal_summary(declaration, arguments, work)?,
            s::Type::Parameter(slot) => {
                work.charge(1)?;
                if rows
                    .slots
                    .get(slot.0 as usize)
                    .is_none_or(|row| !matches!(row.kind, s::ParameterKind::Type { .. }))
                {
                    return Err("source type names an unknown or regional parameter slot");
                }
                Summary {
                    parameter: Some(slot),
                    ..Summary::default()
                }
            }
            s::Type::Reference {
                referent, region, ..
            } => self.merge(
                self.child(referent, work)?,
                region_summary(self.headers, self.binders, region, work)?,
                work,
            )?,
            // A pointer or slice row carries no region. A structurally unsized
            // slice element is rejected here; every consumer still re-proves
            // the element sized from its own facts.
            s::Type::RawPointer { pointee, .. } => self.child(pointee, work)?,
            s::Type::Slice(element) => {
                if matches!(
                    rows.types.get(element.0 as usize),
                    Some(s::Type::Str | s::Type::Slice(_))
                ) {
                    return Err("source slice element requires a sized type");
                }
                self.child(element, work)?
            }
            s::Type::Function(signature) => self.signature(signature, work)?,
            s::Type::Tuple(elements) => {
                let elements = elements.slice(&rows.roots)?;
                if elements.is_empty() {
                    return Err("source unit type requires its scalar recipe");
                }
                let mut result = Summary::default();
                for &element in elements {
                    result = self.merge(result, self.child(element, work)?, work)?;
                }
                result
            }
            s::Type::FunctionItem(application) => {
                let child = self
                    .result
                    .type_count
                    .saturating_add(application.0 as usize);
                if self.colours.get(child) != Some(&2) {
                    return Err("source function-item application is incomplete");
                }
                self.result.summaries[child]
            }
            s::Type::ApplicationSignature(_) => {
                return Err("source application signature recipe is unsupported");
            }
        })
    }
    fn nominal_summary(
        &self,
        declaration: s::DeclarationAddress,
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        if declaration.artifact.0 != 0 {
            return Err("foreign source nominal requires authenticated dependency transport");
        }
        let Some(HeaderRef::Nominal(index)) = self.headers.header(declaration.declaration, work)?
        else {
            return Err("source nominal has no actual nominal declaration");
        };
        let header = &rows.nominals[index as usize];
        let arguments = arguments.slice(&rows.arguments)?;
        if arguments.len() != self.headers.scopes().complete_slots(header.scope, work)? as usize {
            return Err("source nominal changed its complete generic arity");
        }
        let formals = self.headers.scopes().declared_slots(header.scope, work)?;
        let formals = formals.slice(&rows.slots)?;
        let mut result = Summary::default();
        for (argument, formal) in arguments.iter().zip(formals) {
            let child = match (argument, formal.kind) {
                (s::Argument::Type(ty), s::ParameterKind::Type { .. }) => self.child(*ty, work)?,
                (s::Argument::Region(region), s::ParameterKind::Region) => {
                    region_summary(self.headers, self.binders, *region, work)?
                }
                _ => return Err("source nominal argument changed its formal kind"),
            };
            result = self.merge(result, child, work)?;
        }
        Ok(result)
    }
    fn projection_summary(
        &self,
        reference: s::ProjectionRef,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        Ok({
            let row = rows
                .projections
                .get(reference.0 as usize)
                .ok_or("unknown source projection recipe")?;
            let application = rows
                .trait_applications
                .get(row.trait_application.0 as usize)
                .ok_or("unknown projection trait application")?;
            let mut result = self.child(row.subject, work)?;
            for range in [application.arguments, row.arguments] {
                for argument in range.slice(&rows.arguments)? {
                    let child = match argument {
                        s::Argument::Type(ty) => self.child(*ty, work)?,
                        s::Argument::Region(region) => {
                            region_summary(self.headers, self.binders, *region, work)?
                        }
                    };
                    result = self.merge(result, child, work)?;
                }
            }
            result
        })
    }
    fn opaque_summary(
        &self,
        reference: s::OpaqueFutureRef,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        Ok({
            let row = rows
                .opaque_futures
                .get(reference.0 as usize)
                .ok_or("unknown source opaque future recipe")?;
            let mut result = self.child(row.output, work)?;
            for argument in row.captures.slice(&rows.arguments)? {
                let child = match argument {
                    s::Argument::Type(ty) => self.child(*ty, work)?,
                    s::Argument::Region(region) => {
                        region_summary(self.headers, self.binders, *region, work)?
                    }
                };
                result = self.merge(result, child, work)?;
            }
            result
        })
    }
    fn application(&self, node: usize, work: &mut Work) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        let application = &rows.function_applications[node];
        work.charge(1)?;
        if application.declaration.artifact != s::ArtifactRef(0) {
            return Err(
                "source function application requires authenticated wider artifact transport",
            );
        }
        let target = application.declaration.declaration;
        let (scope, regions) = if let Some(constructor) =
            s::Constructor::of((rows.enums(), &rows.declarations), target, |row| {
                self.headers.header(row, work)
            })? {
            // A called constructor's formals are exactly its enum's,
            // version-thirteen leading region slots included.
            let scope = rows.nominals[constructor.nominal as usize].scope;
            let scopes = self.headers.scopes();
            let regions = scopes
                .complete_slots(scope, work)?
                .saturating_sub(scopes.complete_type_slots(scope, work)?);
            (scope, regions as usize)
        } else {
            let Some(HeaderRef::Function(index)) = self.headers.header(target, work)? else {
                return Err("source application target has no actual function header");
            };
            let header = &rows.functions[index as usize];
            if !header.regions.early.slice(&rows.region_slots)?.is_empty() {
                return Err(
                    "source early/captured function applications require a wider complete profile",
                );
            }
            // Version thirteen: an inherent method's captured region arguments
            // lead its type arguments; validation admitted no other capture.
            (header.scope, header.regions.captures as usize)
        };
        let arguments = application.arguments.slice(&rows.arguments)?;
        if regions == 0
            && arguments
                .iter()
                .any(|argument| matches!(argument, s::Argument::Region(_)))
        {
            return Err("source early regional applications require a wider complete profile");
        }
        if arguments.len()
            != (self.headers.scopes().complete_type_slots(scope, work)? as usize)
                .saturating_add(regions)
        {
            return Err("source application omitted or added actual unused type formals");
        }
        let mut summary = Summary::default();
        for (index, argument) in arguments.iter().enumerate() {
            work.charge(1)?;
            let child = match argument {
                s::Argument::Type(ty) if index >= regions => self.child(*ty, work)?,
                s::Argument::Region(region) if index < regions => {
                    region_summary(self.headers, self.binders, *region, work)?
                }
                _ => return Err("source application changed its actual formal kind"),
            };
            summary = self.merge(summary, child, work)?;
        }
        Ok(summary)
    }
    fn trait_application(&self, node: usize, work: &mut Work) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        let application = &rows.trait_applications[node];
        work.charge(1)?;
        if application.declaration.artifact != s::ArtifactRef(0) {
            return Err("source trait application requires authenticated wider artifact transport");
        }
        let Some(HeaderRef::Trait(index)) = self
            .headers
            .header(application.declaration.declaration, work)?
        else {
            return Err("source trait application has no actual trait header");
        };
        let header = &rows.traits[index as usize];
        let complete = self
            .headers
            .scopes()
            .complete_type_slots(header.scope, work)?;
        let inherited = self
            .headers
            .scopes()
            .complete_type_slots(header.self_definition, work)?;
        if complete != self.headers.scopes().complete_slots(header.scope, work)? {
            return Err("source regional trait arguments require a wider complete profile");
        }
        let arguments = application.arguments.slice(&rows.arguments)?;
        if complete.checked_sub(inherited).map(|count| count as usize) != Some(arguments.len()) {
            return Err("source trait application changed its complete own formal count");
        }
        let mut summary = self.child(application.subject, work)?;
        for argument in arguments {
            work.charge(1)?;
            let s::Argument::Type(ty) = argument else {
                return Err("source trait argument changed its actual formal kind");
            };
            summary = self.merge(summary, self.child(*ty, work)?, work)?;
        }
        Ok(summary)
    }
    fn signature(&self, signature: g::Signature, work: &mut Work) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        work.charge(1)?;
        let frame = rows
            .binders
            .get(signature.binder.0 as usize)
            .ok_or("source function type has an unknown actual binder")?;
        let mut summary = self.child(signature.result, work)?;
        for &input in signature.parameters.slice(&rows.roots)? {
            summary = self.merge(summary, self.child(input, work)?, work)?;
        }
        if let Some(bound) = summary.bound {
            if bound.frame != signature.binder {
                return Err("source function type roots belong to another current binder");
            }
            summary.bound = if bound.depth == 0 {
                None
            } else {
                Some(BoundUse {
                    frame: frame
                        .parent
                        .ok_or("source function type escapes its binder root")?,
                    depth: bound.depth - 1,
                })
            };
        }
        Ok(summary)
    }
}

pub(super) fn region_summary(
    headers: &super::HeaderIndex<'_>,
    binders: &super::BinderIndex<'_>,
    region: s::RegionRef,
    work: &mut Work,
) -> Result<Summary, &'static str> {
    work.charge(1)?;
    Ok(
        match headers
            .rows()
            .regions
            .get(region.0 as usize)
            .ok_or("source region reference is unknown")?
        {
            s::Region::Static => Summary::default(),
            s::Region::Bound { binder, depth, .. } => {
                if binders.target(region, work)?.is_none() {
                    return Err("source bound region has no exact resolved ancestor");
                }
                Summary {
                    bound: Some(BoundUse {
                        frame: *binder,
                        depth: *depth,
                    }),
                    ..Summary::default()
                }
            }
            s::Region::Variable { body, slot } => {
                let original = headers
                    .rows()
                    .originals
                    .get(headers.original_index(*body)?)
                    .ok_or("source free region has a foreign original owner")?;
                if *slot as usize >= original.variables.len() {
                    return Err("source free region has no exact original allocation slot");
                }
                Summary {
                    body: Some(*body),
                    ..Summary::default()
                }
            }
            // Version thirteen: a struct's or enum's own region slot, named
            // directly in its header frame. Older profiles reject the slot
            // itself (`record_contracts::require_profile`).
            s::Region::Parameter { slot, depth: 0 } if nominal_region(headers, *slot) => Summary {
                parameter: Some(*slot),
                ..Summary::default()
            },
            s::Region::Parameter { .. } => {
                return Err("source early region parameters require complete wider transport");
            }
        },
    )
}

/// A region slot whose owner is a struct or enum declaration.
fn nominal_region(headers: &super::HeaderIndex<'_>, slot: s::SlotRef) -> bool {
    let rows = headers.rows();
    rows.slots.get(slot.0 as usize).is_some_and(|row| {
        row.kind == s::ParameterKind::Region
            && rows
                .declarations
                .get(row.owner.0 as usize)
                .is_some_and(|owner| s::nominal_kind(owner.definition.kind))
    })
}
