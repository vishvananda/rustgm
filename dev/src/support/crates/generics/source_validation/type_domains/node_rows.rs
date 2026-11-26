//! Type recipes are traversed once; callable recursion is not a recipe cycle.
use super::{BoundUse, Builder, HeaderRef, Summary, Work, g, s};
use crate::support::crates::{Scalar, owner_catalog::DeclarationRef};

/// Own projection and opaque arguments require their complete const domain.
const OUTSIDE_APPLICATIONS: &str =
    "source const arguments outside nominal and function applications require a wider profile";

impl Builder<'_, '_> {
    fn region(&self, region: s::RegionRef, work: &mut Work) -> Result<Summary, &'static str> {
        region_summary(
            self.headers,
            self.binders,
            region,
            self.profile.declaration_policies(),
            work,
        )
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
            s::Type::OpaqueResult(reference) => self.opaque_result_summary(reference, work)?,
            s::Type::Dynamic(reference) => self.dynamic_summary(reference, work)?,
            s::Type::Closure(reference) => self.closure_summary(reference, work)?,
            s::Type::Scalar(_) | s::Type::Str | s::Type::Never => Summary::default(),
            s::Type::F16 | s::Type::F128 => {
                if !self.profile.declaration_policies() {
                    return Err("source float type requires the version-fourteen profile");
                }
                Summary::default()
            }
            s::Type::Foreign(declaration) => {
                if !self.profile.declaration_policies() {
                    return Err("source foreign type requires the version-fourteen profile");
                }
                super::super::declaration(rows, declaration, s::DefinitionKind::ForeignType, work)?;
                Summary::default()
            }
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
                self.region(region, work)?,
                work,
            )?,
            // A pointer or slice row carries no region. A structurally unsized
            // slice element is rejected here; every consumer still re-proves
            // the element sized from its own facts.
            s::Type::RawPointer { pointee, .. } => self.child(pointee, work)?,
            s::Type::Slice(element) => {
                if matches!(
                    rows.types.get(element.0 as usize),
                    Some(
                        s::Type::Str
                            | s::Type::Slice(_)
                            | s::Type::Foreign(_)
                            | s::Type::Dynamic(_)
                    )
                ) {
                    return Err("source slice element requires a sized type");
                }
                self.child(element, work)?
            }
            s::Type::Pattern { base, constraint } => {
                self.pattern_summary(base, constraint, work)?
            }
            s::Type::Array { element, length } => self.array_summary(element, length, work)?,
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
    /// A pattern row carries no region. Only its structural form is checked
    /// here; every consumer re-proves the constraint against its own base.
    fn pattern_summary(
        &self,
        base: g::TypeRef,
        constraint: s::PatternConstraint,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        let base_row = rows.types.get(base.0 as usize);
        match constraint {
            s::PatternConstraint::NonNull => {
                let allowed = matches!(base_row, Some(s::Type::RawPointer { .. }))
                    || (self.profile.declaration_policies()
                        && matches!(base_row, Some(s::Type::Parameter(_))));
                if !allowed {
                    return Err("imported non-null pattern requires a raw pointer base");
                }
            }
            s::PatternConstraint::Range(range) => {
                if range.0 as usize >= rows.pattern_ranges().len() {
                    return Err("source pattern has no actual range row");
                }
                let integer_or_char = matches!(
                    base_row,
                    Some(s::Type::Scalar(scalar)) if !matches!(
                        scalar,
                        Scalar::Unit | Scalar::Bool | Scalar::F32 | Scalar::F64
                    )
                );
                if !integer_or_char {
                    return Err("scalar pattern requires an integer or char base");
                }
            }
        }
        self.child(base, work)
    }
    /// An array row carries no region. A structurally unsized element is
    /// rejected here, and every consumer still re-proves the element sized
    /// from its own facts. The length is a canonical usize value or a usize
    /// const slot.
    fn array_summary(
        &self,
        element: g::TypeRef,
        length: s::ArrayLengthRef,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        work.charge(1)?;
        let rows = self.headers.rows();
        if matches!(
            rows.types.get(element.0 as usize),
            Some(s::Type::Str | s::Type::Slice(_) | s::Type::Foreign(_) | s::Type::Dynamic(_))
        ) {
            return Err("source array element requires a sized type");
        }
        let child = self.child(element, work)?;
        let usize_row = |ty: g::TypeRef| {
            matches!(
                rows.types.get(ty.0 as usize),
                Some(s::Type::Scalar(Scalar::Usize))
            )
        };
        let length = match rows.array_lengths().get(length.0 as usize) {
            Some(&s::ConstTerm::Value(value)) => {
                self.child(value.ty, work)?;
                if !usize_row(value.ty) {
                    return Err("source array length term is not usize");
                }
                crate::support::crates::generics::source::const_value(&rows.types, value)?;
                Summary::default()
            }
            Some(&s::ConstTerm::Parameter(slot)) => match rows.slots.get(slot.0 as usize) {
                Some(row) if matches!(row.kind, s::ParameterKind::Const { ty } if usize_row(ty)) => {
                    Summary {
                        parameter: Some(slot),
                        ..Summary::default()
                    }
                }
                _ => return Err("source array length slot is not a usize const parameter"),
            },
            Some(&s::ConstTerm::Variant(_)) => return Err("source array length is not usize"),
            None => return Err("source array type has no length row"),
        };
        self.merge(child, length, work)
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
                    self.region(*region, work)?
                }
                (s::Argument::Const(term), s::ParameterKind::Const { ty }) => {
                    self.const_summary(*term, ty, work)?
                }
                _ => return Err("source nominal argument changed its formal kind"),
            };
            result = self.merge(result, child, work)?;
        }
        Ok(result)
    }
    /// A const argument: an evaluated value of an earlier primitive row with
    /// canonical bits, or a const slot, of its formal's type.
    fn const_summary(
        &self,
        term: s::ConstRef,
        formal: g::TypeRef,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let (actual, summary) = self.const_argument_summary(term, work)?;
        if s::const_formal_type(&self.headers.rows().types, formal)? != actual {
            return Err("source const argument changed its parameter type");
        }
        Ok(summary)
    }
    fn const_argument_summary(
        &self,
        term: s::ConstRef,
        work: &mut Work,
    ) -> Result<(s::ConstFormalType, Summary), &'static str> {
        let (ty, parameter) = const_argument_domain(
            self.headers,
            self.profile.declaration_policies(),
            term,
            work,
        )?;
        if parameter.is_none() {
            self.child(ty, work)?;
        }
        Ok((
            s::const_formal_type(&self.headers.rows().types, ty)?,
            Summary {
                parameter,
                ..Summary::default()
            },
        ))
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
                        s::Argument::Region(region) => self.region(*region, work)?,
                        s::Argument::Const(_) => return Err(OUTSIDE_APPLICATIONS),
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
                    s::Argument::Region(region) => self.region(*region, work)?,
                    s::Argument::Const(_) => return Err(OUTSIDE_APPLICATIONS),
                };
                result = self.merge(result, child, work)?;
            }
            result
        })
    }
    fn opaque_result_summary(
        &self,
        reference: s::OpaqueResultRef,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        if !self.profile.declaration_policies() {
            return Err("source return-position opaque transport requires version fourteen");
        }
        let rows = self.headers.rows();
        let opaques = &rows.cold().opaques;
        let app = opaques
            .applications
            .get(reference.0 as usize)
            .ok_or("unknown source opaque result recipe")?;
        work.charge(opaques.declarations.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
        let decl_index = opaques
            .declarations
            .binary_search_by_key(&(app.declaration.0, app.ordinal), |row| {
                (row.declaration.0, row.ordinal)
            })
            .map_err(|_| "source opaque result application lacks its declaration")?;
        let decl = &opaques.declarations[decl_index];
        let Some(&s::Type::OpaqueResult(canon_ref)) = rows.types.get(decl.application.0 as usize)
        else {
            return Err("source opaque declaration lacks its canonical application recipe");
        };
        let canon = opaques
            .applications
            .get(canon_ref.0 as usize)
            .ok_or("unknown source canonical opaque application")?;
        let captures = app.captures.slice(&rows.arguments)?;
        let formals = canon.captures.slice(&rows.arguments)?;
        if captures.len() != formals.len() {
            return Err("source opaque application changed its capture arity");
        }
        let mut result = Summary::default();
        for (argument, formal) in captures.iter().zip(formals) {
            let child = match (*argument, *formal) {
                (s::Argument::Type(ty), s::Argument::Type(_)) => self.child(ty, work)?,
                (s::Argument::Region(region), s::Argument::Region(_)) => {
                    self.region(region, work)?
                }
                (s::Argument::Const(term), s::Argument::Const(formal_term)) => {
                    let Some(s::ConstTerm::Parameter(slot)) =
                        rows.consts().get(formal_term.0 as usize)
                    else {
                        return Err("source opaque formal const capture is not a const slot");
                    };
                    let Some(s::ParameterKind::Const { ty }) =
                        rows.slots.get(slot.0 as usize).map(|row| row.kind)
                    else {
                        return Err("source opaque formal const slot has an invalid kind");
                    };
                    self.const_summary(term, ty, work)?
                }
                _ => return Err("source opaque application changed its capture kind"),
            };
            result = self.merge(result, child, work)?;
        }
        Ok(result)
    }
    fn closure_summary(
        &self,
        reference: s::ClosureTypeRef,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        if !self.profile.declaration_policies() {
            return Err("source closure transport requires version fourteen");
        }
        let rows = self.headers.rows();
        let closures = rows
            .closures()
            .ok_or("source closure type lacks its closure columns")?;
        let row = closures
            .types
            .get(reference.0 as usize)
            .ok_or("unknown source closure type recipe")?;
        let header = closures
            .header(row.closure)
            .ok_or("source closure type names an unknown closure header")?;
        if !matches!(
            rows.types.get(row.signature.0 as usize),
            Some(s::Type::Function(_))
        ) || !matches!(
            rows.types.get(row.environment.0 as usize),
            Some(s::Type::Scalar(Scalar::Unit) | s::Type::Tuple(_))
        ) {
            return Err("source closure type lost its function signature or capture tuple");
        }
        let formals = match self.headers.header(header.owner, work)? {
            Some(HeaderRef::Function(index)) => {
                self.type_space_formals(rows.functions[index as usize].scope, work)?
            }
            Some(HeaderRef::Constant(_)) => {
                if self
                    .headers
                    .scopes()
                    .scope_row(header.owner, work)?
                    .is_some()
                {
                    self.type_space_formals(header.owner, work)?
                } else {
                    Vec::new()
                }
            }
            _ => return Err("source closure owner has no actual function or constant header"),
        };
        let arguments = row.arguments.slice(&rows.arguments)?;
        let inherited = header.inherited as usize;
        if arguments.len() != inherited.saturating_add(formals.len()) {
            return Err("source closure type changed its inherited or generic argument arity");
        }
        let mut result = Summary::default();
        for (index, &argument) in arguments.iter().enumerate() {
            work.charge(1)?;
            let formal = index
                .checked_sub(inherited)
                .and_then(|position| formals.get(position));
            let child = match (argument, formal) {
                (s::Argument::Region(region), None) if index < inherited => {
                    self.region(region, work)?
                }
                (s::Argument::Type(ty), Some(s::ParameterKind::Type { .. })) => {
                    self.child(ty, work)?
                }
                (s::Argument::Const(term), Some(&s::ParameterKind::Const { ty })) => {
                    self.const_summary(term, ty, work)?
                }
                _ => return Err("source closure type changed an argument kind"),
            };
            result = self.merge(result, child, work)?;
        }
        result = self.merge(result, self.child(row.signature, work)?, work)?;
        self.merge(result, self.child(row.environment, work)?, work)
    }
    fn application(&self, node: usize, work: &mut Work) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        let application = &rows.function_applications[node];
        let mut summary = Summary {
            has_item: true,
            ..Summary::default()
        };
        work.charge(1)?;
        if application.declaration.artifact != s::ArtifactRef(0) {
            if self.profile.bootstrap_bindings() {
                return self.dependency_arguments(application.arguments, summary, work);
            }
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
            let early = header.regions.early.slice(&rows.region_slots)?;
            if !early.is_empty() && !self.profile.declaration_policies() {
                return Err(
                    "source early/captured function applications require a wider complete profile",
                );
            }
            // Captured and early region arguments lead type/const arguments.
            (
                header.scope,
                (header.regions.captures as usize).saturating_add(early.len()),
            )
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
        let formals = self.type_space_formals(scope, work)?;
        for (index, argument) in arguments.iter().enumerate() {
            work.charge(1)?;
            let formal = index
                .checked_sub(regions)
                .and_then(|position| formals.get(position));
            let child = match (argument, formal) {
                (s::Argument::Type(ty), Some(s::ParameterKind::Type { .. })) => {
                    self.child(*ty, work)?
                }
                (s::Argument::Const(term), Some(&s::ParameterKind::Const { ty })) => {
                    self.const_summary(*term, ty, work)?
                }
                (s::Argument::Region(region), None) if index < regions => {
                    self.region(*region, work)?
                }
                _ => return Err("source application changed its actual formal kind"),
            };
            summary = self.merge(summary, child, work)?;
        }
        Ok(summary)
    }
    /// The kinds of a callee's complete type-space formals, inherited first and
    /// each scope in declared order, since const slots share that space.
    fn type_space_formals(
        &self,
        scope: DeclarationRef,
        work: &mut Work,
    ) -> Result<Vec<s::ParameterKind>, &'static str> {
        let scopes = self.headers.scopes();
        let rows = self.headers.rows();
        let mut chain = Vec::new();
        let mut owner = Some(scope);
        while let Some(current) = owner {
            work.charge(1)?;
            chain.push(scopes.declared_slots(current, work)?);
            owner = scopes.parent(current, work)?;
        }
        let mut formals = Vec::with_capacity(scopes.complete_type_slots(scope, work)? as usize);
        for declared in chain.iter().rev() {
            for slot in declared.slice(&rows.slots)? {
                work.charge(1)?;
                if matches!(
                    slot.kind,
                    s::ParameterKind::Type { .. } | s::ParameterKind::Const { .. }
                ) {
                    formals.push(slot.kind);
                }
            }
        }
        Ok(formals)
    }
    fn trait_application(&self, node: usize, work: &mut Work) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        let application = &rows.trait_applications[node];
        work.charge(1)?;
        let subject = self.child(application.subject, work)?;
        if application.declaration.artifact != s::ArtifactRef(0) {
            if self.profile.bootstrap_bindings() {
                return self.dependency_arguments(application.arguments, subject, work);
            }
            return Err("source trait application requires authenticated wider artifact transport");
        }
        let Some(HeaderRef::Trait(index)) = self
            .headers
            .header(application.declaration.declaration, work)?
        else {
            return Err("source trait application has no actual trait header");
        };
        let header = &rows.traits[index as usize];
        let arguments = application.arguments.slice(&rows.arguments)?;
        self.trait_arguments(header.scope, arguments, subject, work)
    }
    /// Summarize the caller's roots before the dependency graph supplies the
    /// defining header and its formal parameters.
    fn dependency_arguments(
        &self,
        arguments: g::Range,
        mut summary: Summary,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        for argument in arguments.slice(&rows.arguments)? {
            work.charge(1)?;
            let child = match *argument {
                s::Argument::Type(ty) => self.child(ty, work)?,
                s::Argument::Region(region) => self.region(region, work)?,
                s::Argument::Const(term) => self.const_argument_summary(term, work)?.1,
            };
            summary = self.merge(summary, child, work)?;
        }
        Ok(summary)
    }
    pub(super) fn trait_arguments(
        &self,
        scope: DeclarationRef,
        arguments: &[s::Argument],
        mut summary: Summary,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        let rows = self.headers.rows();
        let declared = self
            .headers
            .scopes()
            .declared_slots(scope, work)?
            .slice(&rows.slots)?;
        if declared.len() != arguments.len() {
            return Err("source trait application changed its complete own formal count");
        }
        for (argument, formal) in arguments.iter().zip(declared) {
            work.charge(1)?;
            let child = match (*argument, formal.kind) {
                (s::Argument::Type(ty), s::ParameterKind::Type { .. }) => self.child(ty, work)?,
                (s::Argument::Region(region), s::ParameterKind::Region) => {
                    self.region(region, work)?
                }
                (s::Argument::Const(term), s::ParameterKind::Const { ty })
                    if self.profile.declaration_policies() =>
                {
                    self.const_summary(term, ty, work)?
                }
                _ => return Err("source trait argument changed its actual formal kind"),
            };
            summary = self.merge(summary, child, work)?;
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
        super::super::reject_foreign_value(rows, signature.result)?;
        let mut summary = self.child(signature.result, work)?;
        for &input in signature.parameters.slice(&rows.roots)? {
            super::super::reject_foreign_value(rows, input)?;
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
        if let Some(depth) = summary.nominal_depth {
            if frame.parent.is_none() {
                return Err("source function type escapes its binder root");
            }
            summary.nominal_depth = Some(depth.checked_sub(1).ok_or(
                "source function type captured a nominal region at an invalid binder depth",
            )?);
        }
        summary.has_function = true;
        Ok(summary)
    }
}

pub(super) fn region_summary(
    headers: &super::HeaderIndex<'_>,
    binders: &super::BinderIndex<'_>,
    region: s::RegionRef,
    declaration_policies: bool,
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
            // directly in its header frame. Version fourteen also preserves its
            // nested-binder depth inside `fn` and `dyn` types.
            s::Region::Parameter { slot, depth }
                if (*depth == 0 || declaration_policies) && nominal_region(headers, *slot) =>
            {
                Summary {
                    parameter: Some(*slot),
                    nominal_depth: Some(*depth),
                    ..Summary::default()
                }
            }
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

pub(super) fn const_argument_domain(
    headers: &super::HeaderIndex<'_>,
    declaration_policies: bool,
    term: s::ConstRef,
    work: &mut Work,
) -> Result<(g::TypeRef, Option<s::SlotRef>), &'static str> {
    work.charge(1)?;
    let rows = headers.rows();
    match rows.consts().get(term.0 as usize) {
        Some(&s::ConstTerm::Value(value)) => {
            s::const_value(&rows.types, value)?;
            Ok((value.ty, None))
        }
        Some(&s::ConstTerm::Parameter(slot)) => {
            let Some(s::ParameterKind::Const { ty }) =
                rows.slots.get(slot.0 as usize).map(|row| row.kind)
            else {
                return Err("source const term names a non-const slot");
            };
            Ok((ty, Some(slot)))
        }
        Some(&s::ConstTerm::Variant(value)) => {
            check_variant_const(headers, declaration_policies, value, work)?;
            Ok((value.ty, None))
        }
        None => Err("source const argument has no term row"),
    }
}

fn check_variant_const(
    headers: &super::HeaderIndex<'_>,
    declaration_policies: bool,
    value: s::ConstVariant,
    work: &mut Work,
) -> Result<(), &'static str> {
    if !declaration_policies {
        return Err("source enum const argument requires body operations");
    }
    let rows = headers.rows();
    let s::ConstFormalType::Nominal(owner) = s::const_formal_type(&rows.types, value.ty)? else {
        return Err("source enum const argument has a non-nominal type");
    };
    let Some(HeaderRef::Variant(index)) = headers.header(value.declaration, work)? else {
        return Err("source enum const argument has no actual variant");
    };
    let enums = rows.enums();
    work.charge(enums.headers.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
    let header = enums.owner(index)?;
    let variant = &enums.variants[index as usize];
    if header.declaration != owner
        || variant.shape != s::VariantShape::Unit
        || variant.fields.start != variant.fields.end
    {
        return Err("source enum const argument changed its owner or has a payload");
    }
    Ok(())
}
