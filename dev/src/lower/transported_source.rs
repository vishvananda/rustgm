//! Source transport borrows actual successful originals; emitted rows grant no proof.
mod cfg_rows;
mod locations;
mod obligation_rows;
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

pub(super) struct Source {
    stage: Stage,
    /// A single pass over actual semantic allocations, not a log scan per call.
    openings: HashMap<(NodeKey, NodeKey), Vec<RegionId>>,
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
        facts: &Declarations,
        checked: &Checked<'_>,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        let mut source = Self {
            stage: Stage::begin(report, facts, identity)?,
            openings: HashMap::default(),
        };
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
        for function in &facts.functions {
            source.stage.charge(1)?;
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
        facts: &Declarations,
        proved: &Proved,
        input: &Contract,
    ) -> Result<(), &'static str> {
        self.collect_body(facts, proved.body(), input)
    }

    fn collect_body(
        &mut self,
        facts: &Declarations,
        body: &Body,
        input: &Contract,
    ) -> Result<(), &'static str> {
        // Validate before any operation/input row or collected bit can change.
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
        let mut sites = locations::Sites::new(&mut self.stage, body)?;
        let variables = obligation_rows::allocations(&mut self.stage, input, context, &mut sites)?;
        let mut original =
            cfg_rows::emit(&mut self.stage, facts, body, context, variables, &mut sites)?;
        obligation_rows::calls(self, facts, body, context, &mut original, &mut sites)?;
        let inputs = obligation_rows::emit(
            &mut self.stage,
            facts,
            definition,
            input,
            context,
            &mut sites,
        )?;
        original.sites = sites.len()?;
        self.stage.retain_original(context, original, inputs)
    }

    pub(super) fn complete(self) -> Result<SourceInspection, &'static str> {
        let (rows, work) = self.stage.into_rows()?;
        Ok(SourceInspection { rows, work })
    }
}
