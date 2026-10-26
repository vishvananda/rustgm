//! Only private trait collection retains actual source incarnations and arena.
use super::{Declarations, Report, Stage};
use crate::{
    resolve::unit::NodeKey,
    support::{crates::CrateIdentity, runtime_checks::RuntimeCheckConfig, source::OriginalImage},
    types::arena::Identity,
};
use std::sync::Arc;

pub(super) struct CollectorOrigin {
    provider: CrateIdentity,
    root: NodeKey,
    units: Vec<Option<Arc<OriginalImage>>>,
    arena: Identity,
    runtime: RuntimeCheckConfig,
    preserves_ub: bool,
}
impl CollectorOrigin {
    pub(super) fn new(
        report: &Report,
        facts: &mut Declarations,
        stage: &mut Stage,
    ) -> Result<Box<Self>, &'static str> {
        stage.charge(report.resolution.definitions.len().saturating_add(1))?;
        stage.reserve(
            size_of::<Self>()
                .saturating_add(
                    report
                        .package
                        .units
                        .len()
                        .saturating_mul(size_of::<Option<Arc<OriginalImage>>>()),
                )
                .saturating_add(if facts.types.existing_arena_identity().is_none() {
                    Identity::allocation_bytes()
                } else {
                    0
                }),
        )?;
        let provider = report
            .package
            .source_core()
            .ok_or("collected source requires its actual provider")?;
        if provider.current != provider.provider {
            return Err("collected source identity is not its actual provider");
        }
        let root_unit = report
            .package
            .units
            .first()
            .ok_or("collected source root is absent")?;
        let mut units = vec![None; report.package.units.len()];
        units[0] = Some(Arc::clone(root_unit.source.original_image()));
        for index in 0..stage.definitions.len() {
            let Some(scope) = stage
                .collected_scope(report, facts, index)
                .map_err(|_| "collected source header has no actual declaration context")?
            else {
                continue;
            };
            let declaration = scope.source();
            stage.charge(2)?;
            let unit = report
                .package
                .units
                .get(declaration.file.0 as usize)
                .ok_or("collected source header unit is absent")?;
            let slot = units
                .get_mut(declaration.file.0 as usize)
                .ok_or("collected source header unit is foreign")?;
            if slot.is_none() {
                *slot = Some(Arc::clone(unit.source.original_image()));
            }
        }
        Ok(Box::new(Self {
            provider: provider.provider.clone(),
            root: NodeKey {
                file: crate::resolve::unit::FileId(0),
                node: root_unit.configured.ast.root,
            },
            units,
            arena: facts.types.arena_identity(),
            runtime: report.package.runtime_checks(),
            preserves_ub: report.package.preserves_ub_checks(),
        }))
    }
    pub(super) fn require_store(
        &self,
        facts: &Declarations,
        stage: &mut Stage,
    ) -> Result<(), &'static str> {
        stage.charge(1)?;
        if !facts.types.has_identity(&self.arena) {
            return Err("collected source belongs to another actual canonical arena");
        }
        Ok(())
    }
    pub(super) fn require_root(
        &self,
        report: &Report,
        facts: &Declarations,
        stage: &mut Stage,
    ) -> Result<(), &'static str> {
        self.require_store(facts, stage)?;
        stage.charge(4)?;
        let provider = report
            .package
            .source_core()
            .ok_or("collected source lost its provider policy")?;
        if provider.current != self.provider
            || provider.provider != self.provider
            || report.package.runtime_checks() != self.runtime
            || report.package.preserves_ub_checks() != self.preserves_ub
            || report.package.units.len() != self.units.len()
            || report
                .package
                .units
                .first()
                .map(|unit| unit.configured.ast.root)
                != Some(self.root.node)
        {
            return Err("collected source changed its actual provider/source/configuration");
        }
        self.require_header(report, self.root, stage)
    }
    pub(super) fn require_header(
        &self,
        report: &Report,
        declaration: NodeKey,
        stage: &mut Stage,
    ) -> Result<(), &'static str> {
        stage.charge(2)?;
        let image = self
            .units
            .get(declaration.file.0 as usize)
            .and_then(Option::as_ref)
            .ok_or("collected header lost its selected source incarnation")?;
        let unit = report
            .package
            .units
            .get(declaration.file.0 as usize)
            .ok_or("collected header lost its selected actual unit")?;
        if !Arc::ptr_eq(image, unit.source.original_image()) {
            return Err("collected header belongs to a replaced source incarnation");
        }
        Ok(())
    }
}
