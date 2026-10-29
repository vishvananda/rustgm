//! Source transport borrows actual successful originals; emitted rows grant no proof.
mod cfg_rows;
mod collector_origin;
mod locations;
mod numbered_inputs;
mod obligation_rows;
mod publication;
#[cfg(test)]
#[path = "../../tests/invariants/publication_attributes.rs"]
mod publication_tests;
mod published_annotations;
#[path = "transported_source/collected_headers.rs"]
mod trait_inputs;
use crate::{
    body::{
        ir::{self, Body, Owner},
        regions::Contract,
    },
    borrow::{incremental::Proved, templates::Checked},
    resolve::{expansion::Report, model::DefId, unit::NodeKey},
    sema::{
        crates::authored_export::{Context, Stage},
        model::{BodyRegionOrigin, Declarations},
    },
    support::{
        crates::{
            CrateIdentity,
            generics::{self as g, source as s},
        },
        hash::HashMap,
        retained,
    },
    types::{ItemId, RegionId},
};
pub(crate) use trait_inputs::TraitInspection;

pub(super) struct Source {
    stage: Stage,
    /// A single pass over actual semantic allocations, not a log scan per call.
    openings: HashMap<(NodeKey, NodeKey), Vec<RegionId>>,
    numbering: Option<Vec<Option<s::BodyUnit>>>,
    collector: Option<Box<collector_origin::CollectorOrigin>>,
}
/// Neutral staged observations. Rows still require complete validation,
/// serialization/authenticated import and fresh original/instance proof.
pub struct SourceInspection {
    pub rows: s::SourceRows,
    pub work: crate::sema::crates::authored_export::SourceWork,
}

impl Source {
    pub(super) fn begin(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_with_numbering(report, facts, checked, identity, false)
    }

    pub(super) fn begin_numbered(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_with_numbering(report, facts, checked, identity, true)
    }

    pub(super) fn begin_trait_numbered(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<Self, crate::resolve::package::Error> {
        let mut stage = Stage::begin_with_ordinary_traits(report, facts, identity)
            .map_err(crate::resolve::package::Error::unlocated)?;
        let collector = collector_origin::CollectorOrigin::new(report, facts, &mut stage)?;
        Self::begin_stage(facts, checked, stage, true, Some(collector))
            .map_err(crate::resolve::package::Error::unlocated)
    }

    fn begin_with_numbering(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
        numbered: bool,
    ) -> Result<Self, &'static str> {
        let stage = Stage::begin(report, facts, identity)?;
        Self::begin_stage(facts, checked, stage, numbered, None)
    }

    pub(super) fn begin_namespace(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<(Self, s::NamespaceColumns), &'static str> {
        let mut stage = Stage::begin_with_ordinary_traits(report, facts, identity)?;
        let names = stage.namespace_columns(report)?;
        Ok((
            Self::begin_stage(facts, checked, stage, false, None)?,
            names,
        ))
    }

    pub(super) fn begin_module_contracts(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<(Self, s::ModuleContractRows), &'static str> {
        let mut stage = Stage::begin_module_contracts(report, facts, identity)?;
        let modules = stage.module_contract_rows(report, facts)?;
        Ok((
            Self::begin_stage(facts, checked, stage, false, None)?,
            modules,
        ))
    }
    pub(super) fn begin_module_availability(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<(Self, s::ModuleContractRows), &'static str> {
        let mut stage = Stage::begin_module_availability(report, facts, identity)?;
        let modules = stage.module_contract_rows(report, facts)?;
        Ok((
            Self::begin_stage(facts, checked, stage, false, None)?,
            modules,
        ))
    }
    pub(super) fn begin_module_feature_predicates(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<(Self, s::ModuleContractRows), &'static str> {
        let mut stage = Stage::begin_module_feature_predicates(report, facts, identity)?;
        let modules = stage.module_contract_rows(report, facts)?;
        Ok((
            Self::begin_stage(facts, checked, stage, false, None)?,
            modules,
        ))
    }
    pub(super) fn begin_module_definitions(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<(Self, s::ModuleContractRows), &'static str> {
        let mut stage = Stage::begin_module_definitions(report, facts, identity)?;
        let modules = stage.module_contract_rows(report, facts)?;
        Ok((
            Self::begin_stage(facts, checked, stage, false, None)?,
            modules,
        ))
    }
    pub(super) fn begin_publication(
        report: &Report,
        facts: &mut Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
        profile: super::instances::SourcePublication,
    ) -> Result<(Self, Option<s::ModuleContractRows>), crate::resolve::package::Error> {
        let legacy_error = |message: &'static str| report.body_optional_error(None, message);
        match profile {
            super::instances::SourcePublication::ProvidedContracts => Ok((
                Self::begin_trait_numbered(report, facts, checked, identity)?,
                None,
            )),
            super::instances::SourcePublication::ModuleContracts => {
                let (source, modules) =
                    Self::begin_module_contracts(report, facts, checked, identity)
                        .map_err(legacy_error)?;
                Ok((source, Some(modules)))
            }
            super::instances::SourcePublication::ModuleAvailability => {
                let (source, modules) =
                    Self::begin_module_availability(report, facts, checked, identity)
                        .map_err(legacy_error)?;
                Ok((source, Some(modules)))
            }
            super::instances::SourcePublication::ModuleFeaturePredicates => {
                let (source, modules) =
                    Self::begin_module_feature_predicates(report, facts, checked, identity)
                        .map_err(legacy_error)?;
                Ok((source, Some(modules)))
            }
            super::instances::SourcePublication::ModuleDefinitions => {
                let (source, modules) =
                    Self::begin_module_definitions(report, facts, checked, identity)
                        .map_err(legacy_error)?;
                Ok((source, Some(modules)))
            }
            super::instances::SourcePublication::LegacyFunctions
            | super::instances::SourcePublication::FunctionContracts => Ok((
                Self::begin(report, facts, checked, identity).map_err(legacy_error)?,
                None,
            )),
        }
    }
    fn begin_stage(
        facts: &mut Declarations,
        checked: &Checked<'_>,
        stage: Stage,
        numbered: bool,
        collector: Option<Box<collector_origin::CollectorOrigin>>,
    ) -> Result<Self, &'static str> {
        let mut source = Self {
            stage,
            openings: HashMap::default(),
            numbering: None,
            collector,
        };
        if numbered {
            let count = source.stage.rows.body_owners.len();
            source.stage.charge(count)?;
            source
                .stage
                .reserve(count.saturating_mul(size_of::<Option<s::BodyUnit>>()))?;
            source.numbering = Some((0..count).map(|_| None).collect());
        }
        for row in facts.body_regions.log() {
            source.stage.charge(1)?;
            if !matches!(row.region.origin, BodyRegionOrigin::CallArgument) {
                continue;
            }
            source.stage.reserve(
                retained::new_map::<(NodeKey, NodeKey), Vec<RegionId>>(1)
                    + 4 * size_of::<RegionId>(),
            )?;
            source
                .openings
                .entry((row.region.body, row.region.source))
                .or_default()
                .push(row.id);
        }
        for index in 0..facts.functions.len() {
            source.stage.charge(1)?;
            let function = facts.functions[index];
            let Some((template, input, proof)) =
                checked.function_proof(ItemId(function.definition.0))
            else {
                continue;
            };
            if proof.key != input.key
                || proof.source != input.source
                || proof.instance.is_some()
                || facts
                    .generics
                    .body_environment(function.definition)
                    .is_none_or(|environment| {
                        !std::sync::Arc::ptr_eq(environment, &template.environment)
                    })
            {
                return Err(
                    "source template lost its actual checked owner or defining environment",
                );
            }
            source.collect_body(facts, &template.body, input)?;
        }
        Ok(source)
    }

    pub(super) fn collect(
        &mut self,
        facts: &mut Declarations,
        proved: &Proved,
        input: &Contract,
    ) -> Result<(), &'static str> {
        self.collect_body(facts, proved.body(), input)
    }

    fn collect_body(
        &mut self,
        facts: &mut Declarations,
        body: &Body,
        input: &Contract,
    ) -> Result<(), &'static str> {
        // Validate before any operation/input row or collected bit can change.
        if let Some(collector) = self.collector.as_deref() {
            collector.require_store(facts, &mut self.stage)?;
        }
        let Owner::Function {
            definition,
            signature,
            execution: None,
        } = body.owner
        else {
            return Err(
                "source transport currently requires an original function owner without a relocated execution view",
            );
        };
        let context = self.stage.original_context(definition)?;
        let header = facts
            .function_header(definition)?
            .ok_or("source original declaration header is absent")?;
        if body.instance.is_some()
            || body.key() != input.key
            || body.source != input.source
            || header.body != Some(body.source)
            || header.signature != signature
            || input.signature != Some(signature)
            || input.universals != self.stage.universals(context)?
            || input.output.is_some()
            || input.require_result_formation
        {
            return Err("source original changed its exact body, signature or paired input");
        }
        self.stage.prepare_type_arena(facts, body.result)?;
        let mut sites = locations::Sites::new(&mut self.stage, body)?;
        let variables = obligation_rows::allocations(&mut self.stage, input, context, &mut sites)?;
        let mut original =
            cfg_rows::emit(&mut self.stage, facts, body, context, variables, &mut sites)?;
        obligation_rows::calls(self, facts, body, context, &mut original, &mut sites)?;
        let inputs = obligation_rows::emit(
            &mut self.stage,
            facts,
            body,
            definition,
            input,
            context,
            &mut sites,
        )?;
        original.sites = sites.len()?;
        let unit = self
            .numbering
            .as_ref()
            .map(|_| {
                numbered_inputs::emit(
                    &mut self.stage,
                    facts,
                    body,
                    definition,
                    input,
                    context,
                    original.variables.len(),
                )
            })
            .transpose()?;
        self.stage.retain_original(context, original, inputs)?;
        if let (Some(numbering), Some(unit)) = (&mut self.numbering, unit) {
            let slot = numbering
                .get_mut(unit.body.0 as usize)
                .ok_or("source numbering lost its exact reserved body")?;
            if slot.replace(unit).is_some() {
                return Err("source numbering has duplicate original coverage");
            }
        }
        Ok(())
    }

    fn finish_rows(
        self,
    ) -> Result<
        (
            s::SourceRows,
            crate::sema::crates::authored_export::SourceWork,
        ),
        &'static str,
    > {
        // No original can still arrive after this ownership transfer. Release
        // collection-only opening scratch before allocating validation indexes.
        drop(self.openings);
        let (mut rows, work) = self.stage.into_rows()?;
        if let Some(numbering) = self.numbering {
            let mut units = Vec::with_capacity(numbering.len());
            for unit in numbering {
                units.push(unit.ok_or("source numbering omitted an actual original")?);
            }
            rows.body_units = units;
        }
        Ok((rows, work))
    }

    pub(super) fn complete_namespace(
        self,
        names: s::NamespaceColumns,
        identity: &CrateIdentity,
    ) -> Result<std::sync::Arc<s::UnpublishedNamespace>, &'static str> {
        let (rows, observed) = self.finish_rows()?;
        let mut work = g::Work {
            steps: observed.steps,
            ..g::Work::default()
        };
        s::UnpublishedNamespace::stage(identity, rows, names, &mut work)
    }

    pub(super) fn complete(self) -> Result<SourceInspection, &'static str> {
        self.complete_with_extra(0)
    }
    /// The publishing path consumes the same completed collector directly.
    /// Inspection's indexes are not constructed and discarded before validation.
    pub(super) fn complete_payload(
        mut self,
        report: &Report,
        facts: &Declarations,
        base: crate::support::crates::Interface,
    ) -> Result<crate::support::crates::Interface, crate::resolve::package::Error> {
        if report
            .package
            .source_core()
            .is_none_or(|provider| &provider.provider != base.identity())
        {
            return Err(crate::resolve::package::Error::unlocated(
                "source attribute publication changed its actual artifact identity",
            ));
        }
        self.require_plain_attributes(report, facts)?;
        self.complete_profile_payload(base, None)
            .map_err(|message| report.diagnostic(report.body_optional_error(None, message)))
    }
    pub(super) fn complete_contract_payload(
        mut self,
        report: &Report,
        facts: &Declarations,
        base: crate::support::crates::Interface,
    ) -> Result<crate::support::crates::Interface, crate::resolve::package::Error> {
        if report
            .package
            .source_core()
            .is_none_or(|provider| &provider.provider != base.identity())
        {
            return Err(crate::resolve::package::Error::unlocated(
                "source attribute publication changed its actual artifact identity",
            ));
        }
        let attributes = self.collect_header_columns(report, facts)?;
        self.complete_profile_payload(base, Some(attributes))
            .map_err(|message| report.diagnostic(report.body_optional_error(None, message)))
    }

    pub(super) fn complete_module_policy_payload(
        mut self,
        modules: s::ModuleContractRows,
        report: &Report,
        facts: &Declarations,
        base: crate::support::crates::Interface,
        profile: super::instances::SourcePublication,
    ) -> Result<crate::support::crates::Interface, crate::resolve::package::Error> {
        if report
            .package
            .source_core()
            .is_none_or(|provider| &provider.provider != base.identity())
        {
            return Err(crate::resolve::package::Error::unlocated(
                "module publication changed its actual provider",
            ));
        }
        // Reuse the complete selected-header producer, including its final-header
        // fence when defaultness syntax is integrated. No absence is invented.
        let attributes = self.collect_header_columns(report, facts)?;
        let availability = if matches!(
            profile,
            super::instances::SourcePublication::ModuleAvailability
                | super::instances::SourcePublication::ModuleFeaturePredicates
                | super::instances::SourcePublication::ModuleDefinitions
        ) {
            Some(
                self.stage
                    .availability_rows(report, facts, &modules)
                    .map_err(crate::resolve::package::Error::unlocated)?,
            )
        } else {
            None
        };
        if self.numbering.is_some() {
            return Err(crate::resolve::package::Error::unlocated(
                "Provided numbering has no module-contract publisher",
            ));
        }
        let (rows, observed) = self
            .finish_rows()
            .map_err(crate::resolve::package::Error::unlocated)?;
        let base_bytes = base.base_retained_bytes();
        if base_bytes
            .saturating_add(observed.reserved_bytes)
            .saturating_add(observed.scratch_bytes)
            > crate::support::crates::MAX_BYTES
        {
            return Err(crate::resolve::package::Error::unlocated(
                "module publication exceeds combined collection storage",
            ));
        }
        let mut work = g::Work {
            steps: observed.steps,
            ..g::Work::default()
        };
        let payload = if let Some(availability) = availability {
            let publish = if profile == super::instances::SourcePublication::ModuleDefinitions {
                g::Payload::from_source_module_definitions
            } else if profile == super::instances::SourcePublication::ModuleFeaturePredicates {
                g::Payload::from_source_module_feature_predicates
            } else {
                g::Payload::from_source_module_availability
            };
            publish(
                base.identity(),
                rows,
                attributes,
                modules,
                availability,
                base_bytes,
                &mut work,
            )
        } else {
            g::Payload::from_source_module_contracts(
                base.identity(),
                rows,
                attributes,
                modules,
                base_bytes,
                &mut work,
            )
        }
        .map_err(|message| report.diagnostic(report.body_optional_error(None, message)))?;
        base.with_generic_payload(payload)
            .map_err(crate::resolve::package::Error::unlocated)
    }

    fn complete_profile_payload(
        self,
        base: crate::support::crates::Interface,
        attributes: Option<s::HeaderAttributeColumns>,
    ) -> Result<crate::support::crates::Interface, &'static str> {
        if self.numbering.is_some() {
            return Err("numbered source inspection has no provided-profile publisher");
        }
        drop(self.openings);
        let (rows, work) = self.stage.into_rows()?;
        let base_bytes = base.base_retained_bytes();
        if work
            .reserved_bytes
            .saturating_add(work.scratch_bytes)
            .saturating_add(base_bytes)
            > crate::support::crates::MAX_BYTES
        {
            return Err("source export base exceeds its combined collection envelope");
        }
        let mut validation = g::Work {
            steps: work.steps,
            ..g::Work::default()
        };
        let payload = if let Some(attributes) = attributes {
            g::Payload::from_source_contracts(
                base.identity(),
                rows,
                attributes,
                base_bytes,
                &mut validation,
            )?
        } else {
            g::Payload::from_source(base.identity(), rows, base_bytes, &mut validation)?
        };
        base.with_generic_payload(payload)
    }
}
