//! Compiler-created caller locations use the actual source nominal and immutable storage.
use super::{Constant, Evaluator, Failure, LocationId, Metadata};
use crate::{resolve::unit::NodeKey, sema::model::Scalar, support::bindings::LangItem,
    types::{ItemId, Kind, RegionId, arguments::Argument}};

impl Evaluator<'_> {
    pub(crate) fn prepare_source_location(&mut self, source: NodeKey) -> Result<LocationId, Failure> {
        let report = self.queries.report;
        let unit = report.package.units.get(source.file.0 as usize).ok_or("caller location source file disappeared")?;
        let node = unit.configured.ast.nodes.get(source.node.get() as usize).ok_or("caller location source node disappeared")?;
        if unit.configured.active.get(source.node.get() as usize) != Some(&true) {
            return Err("caller location source node is inactive".into());
        }
        self.facts().charge_caller_inputs(1, 0)?;
        let mut origin_steps = 0;
        let physical = report.physical_call_location(crate::expand::stream::Location { file: source.file, span: node.span }, &mut origin_steps);
        self.facts().charge_caller_inputs(origin_steps, 0)?;
        let physical = physical.map_err(|_| "caller location has invalid expansion provenance")?;
        let position = report.package.unit(physical.file).source.position(physical.span.start)?;
        let filename = report.package.file_paths.get(physical.file.0 as usize).and_then(|path| path.to_str())
            .ok_or("caller location filename is not representable as UTF-8")?;
        let facts = &mut *self.queries.typing.facts;
        let catalog = facts.source_bindings.as_deref().ok_or("caller location lacks its actual source catalog")?;
        catalog.require_provider_report(report)?;
        let location = catalog.validated_language(LangItem::PanicLocation)?
            .ok_or("caller location requires the completed source Location nominal")?;
        let length = filename.len();
        // Prepay actual retained value slots, filename backing and transient copy.
        facts.charge_caller_inputs(length.saturating_add(32), length.saturating_add(1).saturating_mul(2)
            .saturating_add(16 * size_of::<Constant>()))?;
        let arguments = facts.types.intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?;
        let ty = facts.types.intern(Kind::Adt(ItemId(location.definition.0), arguments))?;
        let before = facts.aggregates.instantiation_steps();
        let ready = facts.aggregates.prepare_storage_fields(&mut facts.types, ty);
        let work = facts.aggregates.instantiation_steps().saturating_sub(before);
        facts.charge_caller_inputs(work, 0)?;
        ready?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length.saturating_add(1)).map_err(|_| "caller filename allocation failed")?;
        bytes.extend_from_slice(filename.as_bytes());
        bytes.push(0);
        let text = facts.texts.intern(&bytes)?;
        let filename_pointer = self.literal_address(Constant::bytes(text))?
            .with_metadata(Metadata::Length(length as u64))?;
        // These are the real four Location fields, already checked by site_records;
        // NonNull's actual Pattern field owns validity, its tag supplies no grant.
        let filename_value = self.fields(&[filename_pointer], None)?;
        let marker = self.fields(&[], None)?;
        let value = self.fields(&[filename_value,
            Constant::scalar(Scalar::Integer(u128::from(position.line))),
            Constant::scalar(Scalar::Integer(u128::from(position.column))), marker], None)?;
        self.borrow(value, ty)?.location().ok_or_else(|| "caller location allocation lost its reference".into())
    }
}

impl super::Reader<'_> {
    pub(crate) fn source_caller_location(self, instance: crate::types::instance::Instance,
        origin: crate::resolve::body_origin::BodyOrigin)
        -> Result<Option<crate::sema::compiler_items::CallerLocationSelection>, &'static str>
    {
        self.facts.source_bindings.as_deref().map(|catalog|
            catalog.caller_location_selection(self.facts, instance, origin)).transpose().map(Option::flatten)
    }
    pub(crate) fn caller_tracking(self, definition: crate::resolve::model::DefId) -> Result<bool, &'static str> {
        self.facts.recorded_caller_tracking(definition)
    }
    /// Typed immutable data validation, never call/instance permission.
    pub(crate) fn caller_location_value(self, location: LocationId) -> Result<Constant, &'static str> {
        let catalog = self.facts.source_bindings.as_deref().ok_or("caller site lacks its source catalog")?;
        let declaration = catalog.validated_language(LangItem::PanicLocation)?
            .ok_or("caller site lacks its completed actual Location declaration")?;
        let stored = self.location(location)?;
        let Some(Kind::Adt(item, arguments)) = self.facts.types.kind(stored.ty) else {
            return Err("caller site is not the actual Location type");
        };
        if item.0 != declaration.definition.0 || stored.origin != super::Origin::Allocation
            || self.facts.types.arguments(arguments).is_none_or(|mut arguments|
                arguments.next() != Some(Argument::Lifetime(RegionId::STATIC)) || arguments.next().is_some())
        { return Err("caller site changed its static nominal or allocation identity"); }
        let value = stored.value.ok_or("caller site has no immutable initialized value")?;
        if !matches!(self.view(value)?, super::View::Product { length: 4, variant: None }) {
            return Err("caller site changed its actual Location field data");
        }
        for index in [1, 2] {
            if !matches!(self.view(self.field(value, index)?)?, super::View::Scalar(Scalar::Integer(bits)) if bits > 0 && bits <= u128::from(u32::MAX)) {
                return Err("caller site has invalid line or column data");
            }
        }
        Ok(Constant::pointer(location))
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_source_caller_location(&self, instance: crate::types::instance::Instance,
        origin: crate::resolve::body_origin::BodyOrigin)
        -> Result<Option<crate::sema::compiler_items::CallerLocationSelection>, Failure>
    {
        let Some(catalog) = self.facts().source_bindings.as_deref() else { return Ok(None); };
        let Some(selected) = catalog.caller_location_selection(self.facts(), instance, origin)? else { return Ok(None); };
        catalog.require_caller_location_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
    pub(crate) fn checked_caller_tracking(&self, definition: crate::resolve::model::DefId) -> Result<bool, Failure> {
        self.facts().checked_caller_tracking(self.queries.report, definition)
            .map_err(|_| "caller tracking lost its current header".into())
    }
}
