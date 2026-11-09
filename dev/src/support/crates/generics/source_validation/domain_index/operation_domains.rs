//! Every original-owned port is checked, including values outside the return path.
#[path = "body_ports/instruction_ports.rs"]
mod assembly;
#[path = "body_ports/request_ports.rs"]
mod calls;
#[path = "body_ports/cfg_storage.rs"]
mod storage;
use super::{Context, DomainIndex, HeaderRef, Types, Work, g, s};

impl DomainIndex<'_, '_> {
    pub(super) fn originals(
        &mut self,
        work: &mut Work,
        conversions: bool,
        branching: bool,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        for original in &rows.originals {
            let input = &rows.body_inputs[self.headers.original_index(original.body)?];
            let header = self.function(original.declaration, work)?;
            let context = Context {
                owner: original.declaration,
                frame: Some(input.frame),
                body: Some(original.body),
            };
            if input.frame != header.original.binder
                || original.context != self.call_context(header, work)?
            {
                return Err("source original changed its exact defining frame or semantic context");
            }
            let unsafe_ = header.callable.unsafe_;
            self.types
                .root(self.headers, context, original.result, work)?;
            for local in &original.locals {
                Self::site(original, local.site, work)?;
                self.types.root(self.headers, context, local.ty, work)?;
            }
            let mut snapshots = false;
            for (index, projection) in original.projections.iter().enumerate() {
                snapshots |= matches!(projection.base, s::Place::Temporary(_));
                Self::place(original, projection.base, Some(index), work)?;
                self.types
                    .root(self.headers, context, projection.ty, work)?;
                if let s::ProjectionKind::Dereference { region, .. } = projection.kind {
                    Types::region(self.headers, &self.binders, context, region, work)?;
                }
            }
            for (index, value) in original.values.iter().enumerate() {
                snapshots |= matches!(value.kind, s::ValueKind::Read(s::Place::Temporary(_)));
                self.value(context, original, index, value, work)?;
            }
            self.calls(context, original, work, conversions)?;
            self.storage_ports(original, work)?;
            self.assembly(context, original, unsafe_, work)?;
            self.input_ports(context, original, input, work)?;
            if snapshots {
                self.scratch(super::snapshots::allocation(original), work)?;
                super::snapshots::validate(original, work, branching)?;
            }
        }
        Ok(())
    }
    fn call_context(
        &self,
        header: &s::FunctionHeader,
        work: &mut Work,
    ) -> Result<s::CallContext, &'static str> {
        work.charge(1)?;
        if let Some(parent) = header.parent
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
            s::ValueKind::Text(_) => {
                let rows = self.headers.rows();
                value.text_type(&rows.types, &rows.regions)?;
            }
            s::ValueKind::Constant(constant) => self.constant(constant, value.ty, work)?,
            s::ValueKind::Read(s::Place::Temporary(value)) => {
                prefix_value(original, value, index, work)?;
            }
            s::ValueKind::Read(place)
            | s::ValueKind::Borrow(place)
            | s::ValueKind::TwoPhaseBorrow(place) => {
                Self::place(original, place, None, work)?;
            }
            s::ValueKind::Constrain(operand) | s::ValueKind::EnumCast(operand) => {
                prefix_value(original, operand, index, work)?;
            }
            s::ValueKind::ScalarCast(operand) => {
                prefix_value(original, operand, index, work)?;
                value.scalar_cast_types(original, &self.headers.rows().types)?;
            }
            s::ValueKind::Binary { left, right, .. } => {
                prefix_value(original, left, index, work)?;
                prefix_value(original, right, index, work)?;
                value.binary_types(original, &self.headers.rows().types)?;
            }
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
                if !matches!(
                    original.values[operand as usize].kind,
                    s::ValueKind::Function(_)
                ) || !matches!(
                    rows.types.get(value.ty.0 as usize),
                    Some(s::Type::Function(_))
                ) {
                    return Err("source reification lost its function item or pointer type");
                }
            }
            s::ValueKind::Call {
                signature,
                callee,
                arguments,
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
                Self::place(original, place, None, work)?;
            }
            s::ValueKind::Unit => {}
        }
        Ok(())
    }
    /// Version thirteen: a constant read names a validated constant row of
    /// this artifact and has exactly its declared scalar type.
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
        match (
            rows.types.get(ty.0 as usize),
            rows.types.get(header.ty.0 as usize),
        ) {
            (Some(s::Type::Scalar(read)), Some(s::Type::Scalar(declared))) if read == declared => {
                Ok(())
            }
            _ => Err("source constant read changed its declaration's type"),
        }
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
        original: &s::Original,
        place: s::Place,
        prefix: Option<usize>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        match place {
            s::Place::Local(index) if (index as usize) < original.locals.len() => Ok(()),
            s::Place::Temporary(index)
                if prefix.is_some() && (index as usize) < original.values.len() =>
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
