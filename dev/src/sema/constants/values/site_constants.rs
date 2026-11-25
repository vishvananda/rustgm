//! Compiler-created caller locations use the actual source nominal and immutable storage.
use super::{Constant, DefId, Evaluator, Failure, LocationId, Metadata, Reader};
use crate::{
    resolve::unit::NodeKey,
    sema::model::Scalar,
    support::bindings::LangItem,
    types::{ItemId, Kind, RegionId, arguments::Argument},
};

impl Evaluator<'_> {
    /// Demand the definition site only for the address of a tracked function.
    pub(crate) fn prepare_reified_location(&mut self, target: DefId) -> Result<(), Failure> {
        if Reader::new(self.facts())
            .declaration_location(target)
            .is_some()
            || !self.facts().recorded_caller_tracking(target)?
        {
            return Ok(());
        }
        if !self.checked_caller_tracking(target)? {
            return Err("reified location does not belong to a tracked function".into());
        }
        self.prepare_declaration_location(target)
    }

    pub(crate) fn prepare_invocation_location(
        &mut self,
        kind: crate::support::call_traits::CallTrait,
    ) -> Result<(), Failure> {
        let method = crate::sema::compiler_items::callable_method_definition(self.facts(), kind)?;
        self.prepare_declaration_location(method)
    }

    fn prepare_declaration_location(&mut self, target: DefId) -> Result<(), Failure> {
        if Reader::new(self.facts())
            .declaration_location(target)
            .is_some()
        {
            return Ok(());
        }
        let source = self
            .queries
            .report
            .resolution
            .definitions
            .get(target.0 as usize)
            .and_then(|definition| definition.node)
            .ok_or("caller location requires its original source declaration")?;
        let current = self
            .facts()
            .constant_references
            .caller_declarations
            .as_deref();
        let (len, capacity) =
            current.map_or((0, 0), |locations| (locations.len(), locations.capacity()));
        let growth =
            crate::support::retained::Growth::map::<DefId, super::references::Id>(len, capacity, 1);
        let owner = usize::from(current.is_none())
            * size_of::<crate::support::hash::HashMap<DefId, super::references::Id>>();
        self.facts().charge_caller_inputs(
            growth.work.saturating_add(1),
            growth.bytes.saturating_add(owner),
        )?;
        let location = self.prepare_source_location(source)?;
        let sites = self
            .queries
            .typing
            .facts
            .constant_references
            .caller_declarations
            .get_or_insert_with(Default::default);
        sites
            .try_reserve(1)
            .map_err(|_| "caller declaration location inventory allocation failed")?;
        sites.insert(target, location.0);
        Ok(())
    }

    pub(crate) fn prepare_source_location(
        &mut self,
        source: NodeKey,
    ) -> Result<LocationId, Failure> {
        let report = self.queries.report;
        let (filename, position) = coordinates(report, self.facts(), source)?;
        self.prepare_caller_location(filename, position.line, position.column)
    }

    pub(crate) fn prepare_caller_location(
        &mut self,
        filename: &str,
        line: u32,
        column: u32,
    ) -> Result<LocationId, Failure> {
        if filename.is_empty()
            || filename.len() > 4096
            || filename.as_bytes().contains(&0)
            || line == 0
            || column == 0
        {
            return Err("caller location has invalid filename, line or column".into());
        }
        let report = self.queries.report;
        let facts = &mut *self.queries.typing.facts;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("caller location lacks its actual source catalog")?;
        catalog.require_provider_report(report)?;
        let location = catalog
            .validated_role(LangItem::PanicLocation)?
            .ok_or("caller location requires the completed source Location nominal")?;
        let length = filename.len();
        // Prepay actual retained value slots, filename backing and transient copy.
        facts.charge_caller_inputs(
            length.saturating_add(32),
            length
                .saturating_add(1)
                .saturating_mul(2)
                .saturating_add(16 * size_of::<Constant>()),
        )?;
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?;
        let ty = facts
            .types
            .intern(Kind::Adt(ItemId(location.definition.0), arguments))?;
        let before = facts.aggregates.instantiation_steps();
        let ready = facts
            .aggregates
            .prepare_storage_fields(&mut facts.types, ty);
        let work = facts
            .aggregates
            .instantiation_steps()
            .saturating_sub(before);
        facts.charge_caller_inputs(work, 0)?;
        ready?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length.saturating_add(1))
            .map_err(|_| "caller filename allocation failed")?;
        bytes.extend_from_slice(filename.as_bytes());
        bytes.push(0);
        let text = facts.texts.intern(&bytes)?;
        let filename_pointer = self
            .literal_address(Constant::bytes(text))?
            .with_metadata(Metadata::Length(length as u64))?;
        // These are the real four Location fields, already checked by site_records;
        // NonNull's actual Pattern field owns validity, its tag supplies no grant.
        let filename_value = self.fields(&[filename_pointer], None)?;
        let marker = self.fields(&[], None)?;
        let value = self.fields(
            &[
                filename_value,
                Constant::scalar(Scalar::Integer(u128::from(line))),
                Constant::scalar(Scalar::Integer(u128::from(column))),
                marker,
            ],
            None,
        )?;
        self.borrow(value, ty)?
            .location()
            .ok_or_else(|| "caller location allocation lost its reference".into())
    }
}

impl<'a> Reader<'a> {
    pub(crate) fn declaration_location(self, target: DefId) -> Option<LocationId> {
        self.facts
            .constant_references
            .caller_declarations
            .as_deref()?
            .get(&target)
            .copied()
            .map(LocationId)
    }

    pub(crate) fn invocation_location(
        self,
        kind: crate::support::call_traits::CallTrait,
    ) -> Result<LocationId, &'static str> {
        let method = crate::sema::compiler_items::callable_method_definition(self.facts, kind)?;
        self.declaration_location(method)
            .ok_or("tracked invocation lacks its prepared callable member location")
    }

    pub(crate) fn source_caller_location(
        self,
        instance: crate::types::instance::Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::CallerLocationSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.caller_location_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
    pub(crate) fn caller_tracking(
        self,
        definition: crate::resolve::model::DefId,
    ) -> Result<bool, &'static str> {
        self.facts.recorded_caller_tracking(definition)
    }
    /// Typed immutable data validation, never call/instance permission.
    pub(crate) fn caller_location_value(
        self,
        location: LocationId,
    ) -> Result<Constant, &'static str> {
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or("caller site lacks its source catalog")?;
        let declaration = catalog
            .validated_role(LangItem::PanicLocation)?
            .ok_or("caller site lacks its completed actual Location declaration")?;
        let stored = self.location(location)?;
        let Some(Kind::Adt(item, arguments)) = self.facts.types.kind(stored.ty) else {
            return Err("caller site is not the actual Location type");
        };
        if item.0 != declaration.definition.0
            || stored.origin != super::Origin::Allocation
            || self
                .facts
                .types
                .arguments(arguments)
                .is_none_or(|mut arguments| {
                    arguments.next() != Some(Argument::Lifetime(RegionId::STATIC))
                        || arguments.next().is_some()
                })
        {
            return Err("caller site changed its static nominal or allocation identity");
        }
        let value = stored
            .value
            .ok_or("caller site has no immutable initialized value")?;
        if !matches!(
            self.view(value)?,
            super::View::Product {
                length: 4,
                variant: None
            }
        ) {
            return Err("caller site changed its actual Location field data");
        }
        for index in [1, 2] {
            if !matches!(self.view(self.field(value, index)?)?, super::View::Scalar(Scalar::Integer(bits)) if bits > 0 && bits <= u128::from(u32::MAX))
            {
                return Err("caller site has invalid line or column data");
            }
        }
        Ok(Constant::pointer(location))
    }
    pub(crate) fn caller_location_parts(
        self,
        location: LocationId,
    ) -> Result<(&'a str, u32, u32), &'static str> {
        self.caller_location_value(location)?;
        let value = self
            .location(location)?
            .value
            .ok_or("caller site lost its initialized value")?;
        let super::View::Scalar(Scalar::Integer(line)) = self.view(self.field(value, 1)?)? else {
            return Err("caller site line is not an integer");
        };
        let super::View::Scalar(Scalar::Integer(column)) = self.view(self.field(value, 2)?)? else {
            return Err("caller site column is not an integer");
        };
        let line = u32::try_from(line).map_err(|_| "caller site line exceeds u32")?;
        let column = u32::try_from(column).map_err(|_| "caller site column exceeds u32")?;
        let pointer = self.field(self.field(value, 0)?, 0)?;
        let Some(Metadata::Length(length)) = pointer.metadata() else {
            return Err("caller filename pointer lacks length metadata");
        };
        let length = usize::try_from(length).map_err(|_| "caller filename length overflow")?;
        let backing = pointer
            .location()
            .ok_or("caller filename lost its backing allocation")?;
        let stored = self
            .location(backing)?
            .value
            .ok_or("caller filename backing has no value")?;
        let super::View::Bytes(text) = self.view(stored)? else {
            return Err("caller filename backing changed its byte representation");
        };
        let bytes = self.bytes(text)?;
        if bytes.len() != length.saturating_add(1) || bytes.last() != Some(&0) {
            return Err("caller site filename lacks its NUL terminator");
        }
        let filename = std::str::from_utf8(
            bytes
                .get(..length)
                .ok_or("caller site filename slice out of range")?,
        )
        .map_err(|_| "caller site filename is not UTF-8")?;
        if filename.is_empty() || filename.len() > 4096 || filename.as_bytes().contains(&0) {
            return Err("caller site filename is empty, oversized or contains NUL");
        }
        Ok((filename, line, column))
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_source_caller_location(
        &self,
        instance: crate::types::instance::Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::CallerLocationSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.caller_location_selection(self.facts(), instance, origin)?
        else {
            return Ok(None);
        };
        catalog.require_caller_location_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
    pub(crate) fn checked_caller_tracking(
        &self,
        definition: crate::resolve::model::DefId,
    ) -> Result<bool, Failure> {
        self.facts()
            .checked_caller_tracking(self.queries.report, definition)
            .map_err(|_| "caller tracking lost its current header".into())
    }
}

fn coordinates<'report>(
    report: &'report crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
    source: NodeKey,
) -> Result<(&'report str, crate::support::source::Position), Failure> {
    facts.charge_caller_inputs(1, 0)?;
    let mut origin_steps = 0;
    let physical = report.physical_call_location(source, &mut origin_steps);
    facts.charge_caller_inputs(origin_steps, 0)?;
    let physical = physical.map_err(|_| "caller location has invalid expansion provenance")?;
    let source = &report.package.unit(physical.file).source;
    let growth = source.position_preparation();
    facts.charge_caller_inputs(growth.work.saturating_add(1), growth.bytes)?;
    let position = source.position(physical.span.start)?;
    let filename = report
        .package
        .file_paths
        .get(physical.file.0 as usize)
        .and_then(|path| path.to_str())
        .ok_or("caller location filename is not representable as UTF-8")?;
    Ok((filename, position))
}
impl Evaluator<'_> {
    pub(crate) fn validate_source_location(
        &self,
        source: NodeKey,
        location: LocationId,
    ) -> Result<(), Failure> {
        let (filename, position) = coordinates(self.queries.report, self.facts(), source)?;
        self.facts()
            .charge_caller_inputs(filename.len().saturating_add(16), 0)?;
        let reader = super::Reader::new(self.facts());
        reader.caller_location_value(location)?;
        let value = reader
            .location(location)?
            .value
            .ok_or("caller site lost its initialized value")?;
        if reader.field(value, 1)?.as_scalar()? != Scalar::Integer(u128::from(position.line))
            || reader.field(value, 2)?.as_scalar()? != Scalar::Integer(u128::from(position.column))
        {
            return Err("caller site no longer matches its actual physical source position".into());
        }
        let pointer = reader.field(reader.field(value, 0)?, 0)?;
        let backing = pointer
            .location()
            .ok_or("caller filename lost its backing allocation")?;
        let stored = reader
            .location(backing)?
            .value
            .ok_or("caller filename backing has no value")?;
        let super::View::Bytes(text) = reader.view(stored)? else {
            return Err("caller filename backing changed its byte representation".into());
        };
        let bytes = reader.bytes(text)?;
        if pointer.metadata() != Some(Metadata::Length(filename.len() as u64))
            || bytes.len() != filename.len().saturating_add(1)
            || bytes.last() != Some(&0)
            || bytes.get(..filename.len()) != Some(filename.as_bytes())
        {
            return Err("caller site filename or NUL backing changed".into());
        }
        Ok(())
    }
}
