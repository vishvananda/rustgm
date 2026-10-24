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
                s::Type::Reference { referent, .. } => self.push_type(referent, work)?,
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
                s::Type::Scalar(_) | s::Type::Parameter(_) => {}
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
            for argument in arguments.slice(&rows.arguments)?.iter().rev() {
                work.charge(1)?;
                let s::Argument::Type(ty) = argument else {
                    return Err(
                        "source early regional applications require a wider complete profile",
                    );
                };
                self.push_type(*ty, work)?;
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
            s::Type::Scalar(_) => Summary::default(),
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
            s::Type::Function(signature) => self.signature(signature, work)?,
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
    fn application(&self, node: usize, work: &mut Work) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        let application = &rows.function_applications[node];
        work.charge(1)?;
        if application.declaration.artifact != s::ArtifactRef(0) {
            return Err(
                "source function application requires authenticated wider artifact transport",
            );
        }
        let Some(HeaderRef::Function(index)) = self
            .headers
            .header(application.declaration.declaration, work)?
        else {
            return Err("source application target has no actual function header");
        };
        let header = &rows.functions[index as usize];
        if header.regions.captures != 0
            || !header.regions.early.slice(&rows.region_slots)?.is_empty()
        {
            return Err(
                "source early/captured function applications require a wider complete profile",
            );
        }
        let arguments = application.arguments.slice(&rows.arguments)?;
        if arguments.len()
            != self
                .headers
                .scopes()
                .complete_type_slots(header.scope, work)? as usize
        {
            return Err("source application omitted or added actual unused type formals");
        }
        let mut summary = Summary::default();
        for argument in arguments {
            work.charge(1)?;
            let s::Argument::Type(ty) = argument else {
                return Err("source application changed its actual formal kind");
            };
            summary = self.merge(summary, self.child(*ty, work)?, work)?;
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
                    .get(body.0 as usize)
                    .ok_or("source free region has a foreign original owner")?;
                if *slot as usize >= original.variables.len() {
                    return Err("source free region has no exact original allocation slot");
                }
                Summary {
                    body: Some(*body),
                    ..Summary::default()
                }
            }
            s::Region::Parameter { .. } => {
                return Err("source early region parameters require complete wider transport");
            }
        },
    )
}
