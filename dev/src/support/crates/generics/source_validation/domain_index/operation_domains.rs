//! Every original-owned port is checked, including values outside the return path.
#[path = "body_ports/instruction_ports.rs"]
mod assembly;
#[path = "body_ports/request_ports.rs"]
mod calls;
#[path = "body_ports/cfg_storage.rs"]
mod storage;
use super::{Context, DomainIndex, HeaderRef, OriginalHeader, Work, g, s};
use crate::support::{bindings::LangItem, crates::owner_catalog::DeclarationRef};

impl DomainIndex<'_, '_> {
    pub(super) fn originals(
        &mut self,
        work: &mut Work,
        conversions: bool,
        branching: bool,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        let runtime_format_bodies = self.runtime_format_bodies(work)?;
        for original in &rows.originals {
            let input = &rows.body_inputs[self.headers.original_index(original.body)?];
            let header =
                self.original_header_for_body(original.body, original.declaration, work)?;
            let context = Context {
                owner: original.declaration,
                frame: Some(input.frame),
                body: Some(original.body),
            };
            let (frame, call_context) = match header {
                OriginalHeader::Function(function) => (
                    function.original.binder,
                    self.call_context(function, runtime_format_bodies, work)?,
                ),
                OriginalHeader::Constant { .. } => (
                    self.binders.declaration(original.declaration, work)?,
                    s::CallContext::ClosedConst,
                ),
                OriginalHeader::Closure { header, .. } => (
                    header.binder,
                    if header.constant {
                        s::CallContext::ConstDefinition
                    } else {
                        s::CallContext::Runtime
                    },
                ),
                OriginalHeader::Anonymous { header, .. } => {
                    (header.binder, s::CallContext::ClosedConst)
                }
            };
            if input.frame != frame || original.context != call_context {
                return Err("source original changed its exact defining frame or semantic context");
            }
            self.types
                .root(self.headers, context, original.result, work)?;
            for local in &original.locals {
                Self::site(original, local.site, work)?;
                self.types.root(self.headers, context, local.ty, work)?;
            }
            let mut snapshots = false;
            let mut tested_temporary = false;
            for (index, projection) in original.projections.iter().enumerate() {
                snapshots |= matches!(projection.base, s::Place::Temporary(_));
                self.place(original, projection.base, Some(index), work)?;
                self.types
                    .root(self.headers, context, projection.ty, work)?;
                self.projection_operands(context, original, projection.kind, work)?;
            }
            for (index, value) in original.values.iter().enumerate() {
                snapshots |= matches!(
                    value.kind,
                    s::ValueKind::Read(s::Place::Temporary(_))
                        | s::ValueKind::Borrow(s::Place::Temporary(_))
                        | s::ValueKind::TwoPhaseBorrow(s::Place::Temporary(_))
                        | s::ValueKind::UniqueBorrow(s::Place::Temporary(_))
                        | s::ValueKind::Address(s::Place::Temporary(_))
                        | s::ValueKind::Length(s::Place::Temporary(_))
                        | s::ValueKind::Validate(s::Place::Temporary(_))
                        | s::ValueKind::TextEquals {
                            place: s::Place::Temporary(_),
                            ..
                        }
                );
                tested_temporary |= matches!(
                    value.kind,
                    s::ValueKind::IsVariant {
                        place: s::Place::Temporary(_),
                        ..
                    }
                );
                self.value(context, original, index, value, work)?;
            }
            self.calls(context, original, work, conversions)?;
            self.storage_ports(original, work)?;
            self.assembly(context, original, header.unsafe_function(), work)?;
            self.input_ports(context, original, input, work)?;
            snapshots |= has_statement_temporary(original);
            if snapshots || tested_temporary {
                let scope = super::snapshots::Scope::for_body(
                    branching,
                    tested_temporary,
                    self.declaration_policies,
                );
                self.scratch(super::snapshots::allocation(original, scope), work)?;
                super::snapshots::validate(original, work, scope)?;
            }
        }
        Ok(())
    }
    fn runtime_format_bodies(
        &self,
        work: &mut Work,
    ) -> Result<[Option<DeclarationRef>; LangItem::RUNTIME_FORMAT_BODIES.len()], &'static str> {
        let rows = self.headers.rows();
        let mut runtime_format_bodies = [None; LangItem::RUNTIME_FORMAT_BODIES.len()];
        if self.declaration_policies {
            work.charge(rows.roles.len())?;
            for binding in &rows.roles {
                if let Some(index) = LangItem::RUNTIME_FORMAT_BODIES
                    .iter()
                    .position(|role| *role == binding.role)
                    && binding.declaration.artifact.0 == 0
                {
                    runtime_format_bodies[index] = Some(binding.declaration.declaration);
                }
            }
        }
        Ok(runtime_format_bodies)
    }
    fn call_context(
        &self,
        header: &s::FunctionHeader,
        runtime_format_bodies: [Option<DeclarationRef>; LangItem::RUNTIME_FORMAT_BODIES.len()],
        work: &mut Work,
    ) -> Result<s::CallContext, &'static str> {
        work.charge(1)?;
        if (self.declaration_policies
            && self
                .headers
                .rows()
                .is_unchecked_const_body(header.declaration))
            || runtime_format_bodies.contains(&Some(header.declaration))
        {
            return Ok(s::CallContext::Runtime);
        }
        if self.declaration_policies && self.headers.rows().is_comptime(header.declaration) {
            return Ok(s::CallContext::ClosedConst);
        }
        if (!self.declaration_policies
            || header.constness == crate::support::constness::Constness::Conditional)
            && let Some(parent) = header.parent
            && let Some(HeaderRef::Trait(index)) = self.headers.header(parent, work)?
            && self.headers.rows().traits[index as usize].flags.bits() & s::TraitFlags::CONST != 0
        {
            return Ok(s::CallContext::ConstDefinition);
        }
        Ok(match header.constness {
            crate::support::constness::Constness::Ordinary => s::CallContext::Runtime,
            crate::support::constness::Constness::Const => s::CallContext::ConstDefinition,
            crate::support::constness::Constness::Conditional => {
                return Err(
                    "source conditional declaration eligibility requires complete wider transport",
                );
            }
        })
    }
    fn value(
        &mut self,
        context: Context,
        original: &s::Original,
        index: usize,
        value: &s::Value,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        self.observed.operation_ports = self.observed.operation_ports.saturating_add(1);
        self.types.root(self.headers, context, value.ty, work)?;
        match value.kind {
            s::ValueKind::Scalar(_) => {
                value.scalar_value(&self.headers.rows().types)?;
            }
            s::ValueKind::Text(ordinal) => {
                self.text_value(original, value, ordinal, work)?;
            }
            s::ValueKind::Constant(constant) => self.constant(constant, value.ty, work)?,
            s::ValueKind::ConstArgument(slot) => {
                self.const_argument(context, slot, value.ty, work)?;
            }
            s::ValueKind::AssociatedConstant {
                declaration,
                arguments,
            } => {
                self.associated_constant(context, declaration, arguments, work)?;
            }
            s::ValueKind::Read(place) => self.read_place(original, index, place, work)?,
            s::ValueKind::Borrow(place) | s::ValueKind::TwoPhaseBorrow(place) => {
                self.value_place(original, index, place, work)?;
            }
            s::ValueKind::UniqueBorrow(place) => {
                if !self.declaration_policies {
                    return Err("source unique borrow requires the declaration policy profile");
                }
                self.value_place(original, index, place, work)?;
            }
            s::ValueKind::Constrain(operand) | s::ValueKind::EnumCast(operand) => {
                prefix_value(original, operand, index, work)?;
            }
            s::ValueKind::ScalarCast(operand) => {
                prefix_value(original, operand, index, work)?;
                value.scalar_cast_types(original, &self.headers.rows().types)?;
            }
            s::ValueKind::PointerCast(operand) => {
                prefix_value(original, operand, index, work)?;
                self.pointer_cast(original.values[operand as usize].ty, value.ty)?;
            }
            s::ValueKind::Binary {
                operator,
                left,
                right,
                ..
            } => {
                if operator.requires_v14() && !self.declaration_policies {
                    return Err("source checked division or shift requires a wider transport row");
                }
                prefix_value(original, left, index, work)?;
                prefix_value(original, right, index, work)?;
                value.binary_types(
                    original,
                    &self.headers.rows().types,
                    self.declaration_policies,
                )?;
            }
            s::ValueKind::DirectEnumCast(_)
            | s::ValueKind::AddressCast { .. }
            | s::ValueKind::Unary { .. }
            | s::ValueKind::Address(_)
            | s::ValueKind::Length(_)
            | s::ValueKind::Validate(_)
            | s::ValueKind::TextEquals { .. }
            | s::ValueKind::Union { .. }
            | s::ValueKind::Repeat { .. }
            | s::ValueKind::Unsize { .. }
            | s::ValueKind::InlineConstant(_) => {
                self.extended_value(context, original, index, value, work)?;
            }
            s::ValueKind::Function(_)
            | s::ValueKind::Reify(_)
            | s::ValueKind::Call { .. }
            | s::ValueKind::PointerCall { .. }
            | s::ValueKind::Invoke { .. } => {
                self.callable(context, original, index, value, work)?;
            }
            s::ValueKind::Aggregate { fields }
            | s::ValueKind::Assembly {
                arguments: fields, ..
            } => {
                for &argument in fields.slice(&original.arguments)? {
                    prefix_value(original, argument, index, work)?;
                }
            }
            s::ValueKind::VariantAggregate { variant, fields } => {
                self.variant(variant, work)?;
                for &argument in fields.slice(&original.arguments)? {
                    prefix_value(original, argument, index, work)?;
                }
            }
            s::ValueKind::IsVariant { place, variant } => {
                self.variant(variant, work)?;
                self.value_place(original, index, place, work)?;
            }
            // Assembly's own indexed pass checks the exact earlier effect and output type.
            s::ValueKind::Unit | s::ValueKind::AssemblyOutput { .. } => {}
        }
        Ok(())
    }
    fn read_place(
        &mut self,
        original: &s::Original,
        index: usize,
        place: s::Place,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if let s::Place::Temporary(temporary) = place {
            prefix_value(original, temporary, index, work)
        } else {
            self.place(original, place, None, work)
        }
    }
    fn value_place(
        &mut self,
        original: &s::Original,
        index: usize,
        place: s::Place,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if let s::Place::Temporary(temporary) = place
            && self.declaration_policies
        {
            prefix_value(original, temporary, index, work)
        } else {
            self.place(original, place, None, work)
        }
    }
    /// Function items, reifications, direct and pointer calls and call
    /// operators: each operand precedes its value and each signature is rooted.
    fn callable(
        &mut self,
        context: Context,
        original: &s::Original,
        index: usize,
        value: &s::Value,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match value.kind {
            s::ValueKind::Function(application) => {
                self.function_use(context, application, work)?;
                if self.headers.rows().types.get(value.ty.0 as usize)
                    != Some(&s::Type::FunctionItem(application))
                {
                    return Err("source Function value changed its exact application type");
                }
            }
            // Version thirteen: a reified operand is a preceding Function value
            // and the reification's own type is a fn pointer row.
            s::ValueKind::Reify(operand) => {
                prefix_value(original, operand, index, work)?;
                let rows = self.headers.rows();
                let operand_val = &original.values[operand as usize];
                let valid_operand = matches!(operand_val.kind, s::ValueKind::Function(_))
                    || (self.declaration_policies
                        && matches!(
                            rows.types.get(operand_val.ty.0 as usize),
                            Some(s::Type::Closure(_))
                        ));
                if !valid_operand
                    || !matches!(
                        rows.types.get(value.ty.0 as usize),
                        Some(s::Type::Function(_))
                    )
                {
                    return Err("source reification lost its function item or pointer type");
                }
            }
            s::ValueKind::Call {
                signature,
                callee,
                arguments,
            }
            | s::ValueKind::PointerCall {
                signature,
                callee,
                arguments,
                ..
            } => {
                self.types.root(self.headers, context, signature, work)?;
                prefix_value(original, callee, index, work)?;
                for &argument in arguments.slice(&original.arguments)? {
                    prefix_value(original, argument, index, work)?;
                }
            }
            s::ValueKind::Invoke {
                signature,
                arguments,
                ..
            } => {
                self.types.root(self.headers, context, signature, work)?;
                if arguments.slice(&original.arguments)?.is_empty() {
                    return Err("source call operator lost its actual receiver");
                }
                for &argument in arguments.slice(&original.arguments)? {
                    prefix_value(original, argument, index, work)?;
                }
            }
            _ => return Err("source callable value changed its operation kind"),
        }
        Ok(())
    }
    /// A constant read names a validated row of this artifact and retains
    /// the declared result type.
    fn constant(
        &self,
        constant: crate::support::crates::owner_catalog::DeclarationRef,
        ty: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let Some(HeaderRef::Constant(ordinal)) = self.headers.header(constant, work)? else {
            return Err("source constant read names no validated constant declaration");
        };
        let rows = self.headers.rows();
        let header = rows
            .constants()
            .get(ordinal as usize)
            .ok_or("source constant read names no validated constant declaration")?;
        if matches!(header.body, s::ConstantBody::Original(_)) && self.declaration_policies {
            let parent = rows.declarations[constant.0 as usize]
                .definition
                .parent
                .ok_or("source constant read lacks its actual inherent owner")?;
            let closed_owner = if rows.declarations[parent.0 as usize]
                .definition
                .kind
                .owns_free_constant()
            {
                self.headers.scopes().complete_slots(constant, work)? == 0
            } else if let Some(HeaderRef::Implementation(index)) =
                self.headers.header(parent, work)?
            {
                rows.implementations[index as usize].inherent_constant_owner()
                    && self.headers.scopes().complete_slots(parent, work)? == 0
            } else {
                false
            };
            if closed_owner && ty == header.ty {
                return Ok(());
            }
            return Err("source constant read changed its declaration's owner or type");
        }
        if matches!(header.body, s::ConstantBody::PromotedReference { .. })
            && self.declaration_policies
            && ty == header.ty
        {
            return Ok(());
        }
        if !matches!(header.body, s::ConstantBody::Evaluated(_)) {
            return Err("source constant read requires its completed value");
        }
        match (
            rows.types.get(ty.0 as usize),
            rows.types.get(header.ty.0 as usize),
        ) {
            (Some(s::Type::Scalar(read)), Some(s::Type::Scalar(declared))) if read == declared => {
                Ok(())
            }
            (Some(s::Type::F16), Some(s::Type::F16))
            | (Some(s::Type::F128), Some(s::Type::F128)) => Ok(()),
            (Some(s::Type::Tuple(_)), Some(s::Type::Tuple(_)))
                if self.declaration_policies && ty == header.ty =>
            {
                Ok(())
            }
            _ => Err("source constant read changed its declaration's type"),
        }
    }

    fn associated_constant(
        &self,
        context: Context,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if !self.declaration_policies {
            return Err(
                "source associated constant reads require the version-fourteen declaration profile",
            );
        }
        let rows = self.headers.rows();
        let Some(HeaderRef::Constant(index)) = self.headers.header(declaration, work)? else {
            return Err("source associated constant read has no actual constant declaration");
        };
        let parent = rows.declarations[declaration.0 as usize]
            .definition
            .parent
            .ok_or("source associated constant read has no actual member owner")?;
        let arity = self.headers.scopes().complete_slots(declaration, work)?;
        let valid = match self.headers.header(parent, work)? {
            Some(HeaderRef::Trait(_)) => true,
            Some(HeaderRef::Implementation(implementation)) => {
                arity != 0
                    && rows.implementations[implementation as usize].inherent_constant_owner()
                    && matches!(
                        rows.constants()[index as usize].body,
                        s::ConstantBody::Original(_)
                    )
            }
            _ => false,
        };
        if !valid {
            return Err("source associated constant read has no actual trait or inherent owner");
        }
        let arguments = arguments.slice(&rows.arguments)?;
        if arguments.len() != arity as usize {
            return Err("source associated constant read changed its full argument arity");
        }
        work.charge(arguments.len())?;
        for &argument in arguments {
            match argument {
                s::Argument::Type(ty) => self.types.root(self.headers, context, ty, work)?,
                s::Argument::Region(region) => {
                    self.types
                        .region(self.headers, &self.binders, context, region, work)?;
                }
                s::Argument::Const(term) => match rows.consts().get(term.0 as usize) {
                    Some(&s::ConstTerm::Value(value)) => {
                        self.types.root(self.headers, context, value.ty, work)?;
                        s::const_value(&rows.types, value)?;
                    }
                    Some(&s::ConstTerm::Parameter(slot))
                        if matches!(
                            rows.slots.get(slot.0 as usize).map(|row| row.kind),
                            Some(s::ParameterKind::Const { .. })
                        ) && self.headers.scopes().contains(context.owner, slot, work)? => {}
                    _ => {
                        return Err("source associated constant read has a foreign const argument");
                    }
                },
            }
        }
        Ok(())
    }

    fn const_argument(
        &self,
        context: Context,
        slot: s::SlotRef,
        ty: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if !self.declaration_policies {
            return Err(
                "source const argument reads require the version-fourteen declaration profile",
            );
        }
        if !self.headers.rows().slots.get(slot.0 as usize).is_some_and(
            |slot| matches!(slot.kind, s::ParameterKind::Const { ty: actual } if actual == ty),
        ) || !self.headers.scopes().contains(context.owner, slot, work)?
        {
            return Err("source const argument changed its actual const slot or type");
        }
        Ok(())
    }
    fn projection_operands(
        &mut self,
        context: Context,
        original: &s::Original,
        kind: s::ProjectionKind,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        let require_usize_value = |index: u32| {
            let ty = original
                .values
                .get(index as usize)
                .ok_or("source projection index value is unknown")?
                .ty;
            if rows.types.get(ty.0 as usize)
                != Some(&s::Type::Scalar(crate::support::crates::Scalar::Usize))
            {
                return Err("source projection index value is not usize");
            }
            Ok(())
        };
        match kind {
            s::ProjectionKind::Dereference { region, .. } => {
                self.types
                    .region(self.headers, &self.binders, context, region, work)?;
            }
            s::ProjectionKind::Field { .. } | s::ProjectionKind::Tuple(_) => {}
            s::ProjectionKind::CaptureBoundary { .. }
            | s::ProjectionKind::RawDereference { .. }
            | s::ProjectionKind::UnionField { .. }
            | s::ProjectionKind::ConstantIndex(_)
            | s::ProjectionKind::FromEnd { .. }
            | s::ProjectionKind::Subslice { .. }
            | s::ProjectionKind::Subarray { .. } => {
                if !self.declaration_policies {
                    return Err(
                        "source extended projection requires the declaration policy profile",
                    );
                }
            }
            s::ProjectionKind::Index { index, .. } | s::ProjectionKind::SliceIndex { index } => {
                if !self.declaration_policies {
                    return Err("source index projection requires the declaration policy profile");
                }
                require_usize_value(index)?;
            }
            s::ProjectionKind::IndexParameter { index, length } => {
                if !self.declaration_policies {
                    return Err(
                        "source parameterized index projection requires the declaration policy profile",
                    );
                }
                require_usize_value(index)?;
                let usize_slot = rows.slots.get(length.0 as usize).is_some_and(|slot| {
                    matches!(slot.kind, s::ParameterKind::Const { ty }
                        if rows.types.get(ty.0 as usize)
                            == Some(&s::Type::Scalar(crate::support::crates::Scalar::Usize)))
                });
                if !usize_slot
                    || !self
                        .headers
                        .scopes()
                        .contains(context.owner, length, work)?
                {
                    return Err("source index length slot is foreign or not usize");
                }
            }
        }
        Ok(())
    }
    fn text_value(
        &self,
        original: &s::Original,
        value: &s::Value,
        ordinal: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        if !self.declaration_policies {
            return value.text_type(&rows.types, &rows.regions);
        }
        let Some(&s::Type::Reference {
            mutable: false,
            region,
            referent,
        }) = rows.types.get(value.ty.0 as usize)
        else {
            return Err("source text value is not a shared reference");
        };
        if rows.regions.get(region.0 as usize) != Some(&s::Region::Static) {
            return Err("source text value is not a static str reference");
        }
        let bytes = original
            .texts
            .as_deref()
            .and_then(|texts| texts.rows.get(ordinal as usize))
            .ok_or("source text column differs from its text values")?;
        work.charge(bytes.len().saturating_add(1))?;
        match rows.types.get(referent.0 as usize) {
            Some(s::Type::Str) => {
                if std::str::from_utf8(bytes).is_err() {
                    return Err("source text literal is not UTF-8");
                }
                Ok(())
            }
            Some(&s::Type::Slice(element))
                if rows.types.get(element.0 as usize)
                    == Some(&s::Type::Scalar(crate::support::crates::Scalar::U8)) =>
            {
                Ok(())
            }
            Some(&s::Type::Array { element, length })
                if rows.types.get(element.0 as usize)
                    == Some(&s::Type::Scalar(crate::support::crates::Scalar::U8)) =>
            {
                let Some(s::ConstTerm::Value(constant)) =
                    rows.array_lengths().get(length.0 as usize)
                else {
                    return Err("source byte string literal length requires a concrete constant");
                };
                if constant.bits != bytes.len() as u128 {
                    return Err("source byte string literal length differs from its array type");
                }
                Ok(())
            }
            Some(&s::Type::Nominal {
                declaration,
                arguments,
            }) if arguments.start == arguments.end
                && rows
                    .roles
                    .iter()
                    .any(|role| role.role == LangItem::CStr && role.declaration == declaration) =>
            {
                if bytes.last() != Some(&0) || bytes[..bytes.len() - 1].contains(&0) {
                    return Err("source CStr literal requires a single trailing NUL byte");
                }
                Ok(())
            }
            _ => Err("source text value is not a static str reference"),
        }
    }
    fn extended_value(
        &self,
        context: Context,
        original: &s::Original,
        index: usize,
        value: &s::Value,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if !self.declaration_policies {
            return Err("source extended value operation requires the declaration policy profile");
        }
        match value.kind {
            s::ValueKind::DirectEnumCast(operand) => prefix_value(original, operand, index, work),
            s::ValueKind::AddressCast { kind, operand } => {
                prefix_value(original, operand, index, work)?;
                self.address_cast(kind, original, operand, value.ty)
            }
            s::ValueKind::Unary { operand, .. } => {
                prefix_value(original, operand, index, work)?;
                value.unary_type(original, &self.headers.rows().types)
            }
            s::ValueKind::Address(place)
            | s::ValueKind::Length(place)
            | s::ValueKind::Validate(place)
            | s::ValueKind::TextEquals { place, .. } => {
                if let s::Place::Temporary(temporary) = place {
                    prefix_value(original, temporary, index, work)
                } else {
                    self.place(original, place, None, work)
                }
            }
            s::ValueKind::Union { value: operand, .. }
            | s::ValueKind::Repeat { value: operand, .. } => {
                prefix_value(original, operand, index, work)
            }
            s::ValueKind::Unsize {
                value: operand,
                conversion,
            } => self.unsize_value(context, original, index, operand, conversion, work),
            s::ValueKind::InlineConstant(child) => {
                let rows = self.headers.rows();
                let Some((_, header)) = rows
                    .closures()
                    .and_then(|closures| closures.anonymous_for_body(child))
                else {
                    return Err("source inline constant references an unknown anonymous body");
                };
                let owner = rows
                    .body_owners
                    .get(child.0 as usize)
                    .ok_or("source inline constant body owner is unknown")?;
                if header.owner != context.owner || owner.parent != Some(original.body) {
                    return Err("source inline constant belongs to another parent or declaration");
                }
                Ok(())
            }
            _ => Err("source extended value changed its operation kind"),
        }
    }
    fn unsize_value(
        &self,
        context: Context,
        original: &s::Original,
        index: usize,
        operand: u32,
        conversion: s::UnsizeConversion,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        prefix_value(original, operand, index, work)?;
        let rows = self.headers.rows();
        match conversion {
            s::UnsizeConversion::Slice(array) => {
                self.types.root(self.headers, context, array, work)?;
                if !matches!(
                    rows.types.get(array.0 as usize),
                    Some(s::Type::Array { .. })
                ) {
                    return Err("source slice unsizing conversion requires an array type");
                }
            }
            s::UnsizeConversion::Object { concrete, target } => {
                self.types.root(self.headers, context, concrete, work)?;
                self.types.root(self.headers, context, target, work)?;
                if !matches!(rows.types.get(target.0 as usize), Some(s::Type::Dynamic(_))) {
                    return Err("source object unsizing conversion requires a trait object target");
                }
            }
            s::UnsizeConversion::Upcast { source, target } => {
                self.types.root(self.headers, context, source, work)?;
                self.types.root(self.headers, context, target, work)?;
                if !matches!(
                    (
                        rows.types.get(source.0 as usize),
                        rows.types.get(target.0 as usize)
                    ),
                    (Some(s::Type::Dynamic(_)), Some(s::Type::Dynamic(_)))
                ) {
                    return Err(
                        "source upcast unsizing conversion requires trait object endpoints",
                    );
                }
            }
            s::UnsizeConversion::Nominal { source, target } => {
                self.types.root(self.headers, context, source, work)?;
                self.types.root(self.headers, context, target, work)?;
                if !matches!(
                    (
                        rows.types.get(source.0 as usize),
                        rows.types.get(target.0 as usize)
                    ),
                    (
                        Some(s::Type::Nominal { declaration: a, .. }),
                        Some(s::Type::Nominal { declaration: b, .. })
                    ) if a == b
                ) {
                    return Err("source nominal unsizing conversion requires one nominal owner");
                }
            }
            s::UnsizeConversion::Bound { source, target } => {
                self.types.root(self.headers, context, source, work)?;
                self.types.root(self.headers, context, target, work)?;
            }
        }
        Ok(())
    }
    fn pointer_cast(&self, from: s::TypeRef, to: s::TypeRef) -> Result<(), &'static str> {
        if !self.declaration_policies {
            return Err("source pointer cast changed its pointee or access");
        }
        let types = &self.headers.rows().types;
        let strip_pattern = |mut ty: s::TypeRef| {
            if let Some(&s::Type::Pattern { base, .. }) = types.get(ty.0 as usize) {
                ty = base;
            }
            types.get(ty.0 as usize)
        };
        let valid = match (strip_pattern(from), types.get(to.0 as usize)) {
            (Some(s::Type::RawPointer { .. }), Some(s::Type::RawPointer { .. }))
            | (Some(s::Type::Function(_)), Some(s::Type::RawPointer { .. })) => true,
            (
                Some(s::Type::Reference {
                    mutable: from_mutable,
                    referent,
                    ..
                }),
                Some(s::Type::RawPointer {
                    mutable: to_mutable,
                    pointee,
                }),
            ) => referent == pointee && (!*to_mutable || *from_mutable),
            (
                Some(s::Type::Reference {
                    mutable: from_mutable,
                    ..
                }),
                Some(s::Type::Reference {
                    mutable: to_mutable,
                    ..
                }),
            ) => from == to && from_mutable == to_mutable,
            _ => false,
        };
        if !valid {
            return Err("source pointer cast changed its pointee or access");
        }
        Ok(())
    }
    fn address_cast(
        &self,
        kind: s::AddressCast,
        original: &s::Original,
        operand: u32,
        to: s::TypeRef,
    ) -> Result<(), &'static str> {
        use crate::support::crates::Scalar;
        if !self.declaration_policies {
            return Err("source address cast requires the declaration policy profile");
        }
        let types = &self.headers.rows().types;
        let from_ref = original.values[operand as usize].ty;
        let from = match types.get(from_ref.0 as usize) {
            Some(&s::Type::Pattern { base, .. }) => types.get(base.0 as usize),
            ty => ty,
        };
        let to = types.get(to.0 as usize);
        let is_int = |ty: Option<&s::Type>| {
            matches!(
                ty,
                Some(s::Type::Scalar(scalar))
                    if !matches!(
                        scalar,
                        Scalar::Unit | Scalar::Bool | Scalar::Char | Scalar::F32 | Scalar::F64
                    )
            )
        };
        let valid = match kind {
            s::AddressCast::PointerThin => {
                matches!(
                    (from, to),
                    (
                        Some(s::Type::RawPointer { .. }),
                        Some(s::Type::RawPointer { .. })
                    )
                )
            }
            s::AddressCast::PointerToInteger => {
                matches!(
                    from,
                    Some(s::Type::RawPointer { .. } | s::Type::Function(_))
                ) && is_int(to)
            }
            s::AddressCast::IntegerToPointer => {
                is_int(from) && matches!(to, Some(s::Type::RawPointer { .. }))
            }
            s::AddressCast::FunctionPointer => {
                (matches!(
                    original.values[operand as usize].kind,
                    s::ValueKind::Function(_)
                ) || matches!(from, Some(s::Type::Closure(_))))
                    && matches!(to, Some(s::Type::Function(_)))
            }
        };
        if !valid {
            return Err("source address cast changed its operand or result type");
        }
        Ok(())
    }

    /// A variant operand names a validated variant row of this artifact.
    fn variant(
        &self,
        variant: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if !matches!(
            self.headers.header(variant, work)?,
            Some(HeaderRef::Variant(_))
        ) {
            return Err("source variant operand does not name a validated enum variant");
        }
        Ok(())
    }
    pub(super) fn site(
        original: &s::Original,
        site: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if site >= original.sites {
            return Err("source operation site is outside its actual body");
        }
        Ok(())
    }
    fn place(
        &self,
        original: &s::Original,
        place: s::Place,
        prefix: Option<usize>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        match place {
            s::Place::Local(index) if (index as usize) < original.locals.len() => Ok(()),
            s::Place::Static(declaration) => {
                self.headers.static_header(declaration, work).map(|_| ())
            }
            s::Place::Temporary(index)
                if (prefix.is_some() || self.declaration_policies)
                    && (index as usize) < original.values.len() =>
            {
                Ok(())
            }
            s::Place::Projection(index)
                if (index as usize) < original.projections.len()
                    && prefix.is_none_or(|prefix| (index as usize) < prefix) =>
            {
                Ok(())
            }
            s::Place::Temporary(_) if prefix.is_none() => {
                Err("source direct temporary storage requires its wider lifetime/move profile")
            }
            _ => Err("source place has a foreign, forward or unknown actual slot"),
        }
    }
}
fn has_statement_temporary(original: &s::Original) -> bool {
    original.statements.iter().any(|statement| {
        matches!(
            statement.action,
            s::Action::Store {
                place: s::Place::Temporary(_),
                ..
            } | s::Action::CheckInitialized(s::Place::Temporary(_))
                | s::Action::Mention(s::Place::Temporary(_))
                | s::Action::BindView {
                    place: s::Place::Temporary(_),
                    ..
                }
                | s::Action::Select {
                    anchor: s::Place::Temporary(_),
                    ..
                }
        )
    }) || original.guards.as_deref().is_some_and(|guards| {
        guards
            .places
            .iter()
            .any(|place| matches!(place, s::Place::Temporary(_)))
    })
}
fn prefix_value(
    original: &s::Original,
    value: u32,
    before: usize,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(1)?;
    if value as usize >= before || value as usize >= original.values.len() {
        return Err("source value recipe is cyclic, forward or unknown");
    }
    Ok(())
}
