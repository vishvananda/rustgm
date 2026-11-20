//! Cold declaration snapshots share source handles and the existing header budget.
use super::{Declaration, Declarations, Error, LangItem, Report, schemas::Plan};
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::{
        compiler_items::Role,
        declaration_inputs::{self, Input, Inputs},
        generics::Parameter,
        header_attributes::{self, SourceImage},
    },
    syntax::{ast::Kind as Syntax, item::Item},
    types::{ItemId, arena, outlives::Pair, parameters::clauses::Clause, projection},
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Row {
    name: crate::support::names::NameId,
    namespace: crate::resolve::model::Namespace,
    kind: crate::resolve::model::DefKind,
    scope: crate::resolve::model::ScopeId,
    access: crate::resolve::model::Access,
    members: Option<crate::resolve::model::ScopeId>,
}
impl Row {
    fn read(report: &Report, declaration: Declaration) -> Result<Self, Error> {
        let row = report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .ok_or_else(|| Error::unlocated("source Pointee definition is outside its report"))?;
        if row.node != Some(declaration.origin) {
            return Err(Error::unlocated(
                "source Pointee definition changed its current source",
            ));
        }
        Ok(Self {
            name: row.name,
            namespace: row.namespace,
            kind: row.kind,
            scope: row.scope,
            access: row.access,
            members: row.members,
        })
    }
}
struct SourceState {
    declaration: Declaration,
    row: Row,
    image: SourceImage,
    item: Item,
}
struct State {
    role: Role,
    source: Option<SourceState>,
    parameters: Vec<Parameter>,
    clauses: Vec<Input>,
}
pub(in crate::sema::compiler_items) struct Contracts {
    pub(super) arena: arena::Identity,
    pub(super) plan: Plan,
    states: [State; 2],
    nominal: Option<State>,
    inputs: Vec<Input>,
    trait_regions: Vec<Pair>,
    canonical_results: Vec<(ItemId, crate::sema::declaration_inputs::Input)>,
    family_identity: Option<(u32, u32)>,
    requirements: Vec<Clause>,
    regions: Vec<Pair>,
    results: Vec<Clause>,
}
fn item(report: &Report, declaration: Declaration) -> Result<Item, Error> {
    let unit = report.package.unit(declaration.origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err(Error::unlocated("source Pointee declaration lost its item"));
    };
    unit.configured
        .ast
        .items
        .get(index as usize)
        .copied()
        .ok_or_else(|| Error::unlocated("source Pointee item is outside its source"))
}
fn parameters(facts: &Declarations, declaration: Role) -> &[Parameter] {
    facts
        .generics
        .parameters
        .get(&declaration.definition)
        .map_or(&[], Vec::as_slice)
}
fn clauses(facts: &Declarations, role: Role) -> Result<Inputs<'_>, Error> {
    declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), role.definition)
        .map_err(Error::unlocated)
}

impl SourceState {
    fn capture(
        report: &Report,
        facts: &Declarations,
        role: Role,
        trait_header: bool,
    ) -> Result<Option<Self>, Error> {
        let DeclarationOrigin::Source(origin) = role.origin else {
            return Ok(None);
        };
        let declaration = Declaration {
            definition: role.definition,
            parent: role.parent,
            origin,
        };
        let image = if trait_header {
            header_attributes::completed_source_image(facts, report, role.definition)?
        } else {
            header_attributes::role_source_image(facts, report, declaration)?
        };
        Ok(Some(Self {
            declaration,
            row: Row::read(report, declaration)?,
            image,
            item: item(report, declaration)?,
        }))
    }

    fn current(&self, report: &Report, facts: &Declarations) -> Result<(), Error> {
        let declaration = self.declaration;
        let unit = report.package.unit(declaration.origin.file);
        header_attributes::role_source_image(facts, report, declaration)?;
        if Row::read(report, declaration)? != self.row
            || !self.image.current(unit, declaration.origin.node)
            || item(report, declaration)? != self.item
        {
            return Err(Error::unlocated(
                "source Pointee retained declaration changed",
            ));
        }
        Ok(())
    }
}

impl State {
    fn capture(
        report: &Report,
        facts: &Declarations,
        role: Role,
        trait_header: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            role,
            source: SourceState::capture(report, facts, role, trait_header)?,
            parameters: parameters(facts, role).to_vec(),
            clauses: clauses(facts, role)?.iter().collect(),
        })
    }
    fn current(&self, report: &Report, facts: &Declarations) -> Result<(), Error> {
        if let Some(source) = &self.source {
            source.current(report, facts)?;
        }
        self.current_facts(facts)
    }
    fn current_facts(&self, facts: &Declarations) -> Result<(), Error> {
        let role = self.role;
        let steps = self
            .parameters
            .len()
            .saturating_add(self.clauses.len())
            .saturating_add(1);
        super::charge(facts, role, steps)?;
        if let Some(source) = &self.source {
            header_attributes::require_recorded_pointee_image(
                facts,
                source.declaration,
                &source.image,
            )?;
        }
        let params = parameters(facts, role);
        let conditions = clauses(facts, role)?;
        if params.len() != self.parameters.len()
            || conditions.len() != self.clauses.len()
            || !params.iter().zip(&self.parameters).all(|(a, b)| {
                a.binding == b.binding
                    && a.argument == b.argument
                    && a.anonymous == b.anonymous
                    && a.default == b.default
            })
            || !conditions
                .iter()
                .zip(&self.clauses)
                .all(|(a, b)| a.source == b.source && a.value == b.value)
        {
            return Err(Error::unlocated(
                "source Pointee retained declaration changed",
            ));
        }
        Ok(())
    }
}
fn trait_header(facts: &Declarations, plan: Plan) -> Result<&crate::sema::traits::Header, Error> {
    facts
        .generics
        .traits
        .as_deref()
        .and_then(|traits| traits.header(ItemId(plan.trait_.definition.0)))
        .ok_or_else(|| Error::unlocated("source Pointee lost its canonical trait header"))
}
struct FamilyInputs<'a> {
    identity: Option<(u32, u32)>,
    requirements: &'a [Clause],
    regions: &'a [Pair],
}
fn family_inputs(facts: &Declarations, plan: Plan) -> Result<FamilyInputs<'_>, Error> {
    match facts.associated.families.get(&plan.member.definition) {
        Some(family) => Ok(FamilyInputs {
            identity: Some((family.captures, family.total)),
            requirements: family.requirements.as_deref().ok_or_else(|| {
                Error::unlocated("source Metadata family inputs are not complete")
            })?,
            regions: &family.regions,
        }),
        // schemas::plan authenticated the actual required no-generics/no-where
        // item and completed TypeContract. Absence alone supplies no authority.
        None => Ok(FamilyInputs {
            identity: None,
            requirements: &[],
            regions: &[],
        }),
    }
}
impl Contracts {
    pub(super) fn payment(facts: &Declarations, plan: Plan) -> Result<(usize, usize), Error> {
        let header = trait_header(facts, plan)?;
        let FamilyInputs {
            requirements,
            regions,
            ..
        } = family_inputs(facts, plan)?;
        let results = &facts
            .associated
            .require_projection_header(plan.member.definition)
            .map_err(Error::unlocated)?
            .bounds;
        let mut bytes = size_of::<Self>().saturating_add(2 * size_of::<usize>());
        let mut steps = 1usize;
        for declaration in [Some(plan.trait_), Some(plan.member), plan.nominal]
            .into_iter()
            .flatten()
        {
            let params = parameters(facts, declaration);
            let inputs = clauses(facts, declaration)?.len();
            bytes = bytes
                .saturating_add(size_of_val(params))
                .saturating_add(inputs.saturating_mul(size_of::<Input>()));
            steps = steps.saturating_add(params.len()).saturating_add(inputs);
        }
        bytes = bytes
            .saturating_add(
                header
                    .inputs
                    .view()
                    .len()
                    .saturating_mul(size_of::<Input>()),
            )
            .saturating_add(size_of_val(header.regions.as_slice()))
            .saturating_add(size_of_val(header.results.as_slice()))
            .saturating_add(size_of_val(requirements))
            .saturating_add(size_of_val(regions))
            .saturating_add(size_of_val(results.as_slice()));
        steps = steps
            .saturating_add(header.inputs.view().len())
            .saturating_add(header.regions.len())
            .saturating_add(header.results.len())
            .saturating_add(requirements.len())
            .saturating_add(regions.len())
            .saturating_add(results.len());
        if facts.types.existing_arena_identity().is_none() {
            bytes = bytes.saturating_add(arena::Identity::allocation_bytes());
        }
        Ok((steps, bytes))
    }
    pub(super) fn capture(
        report: &Report,
        facts: &Declarations,
        plan: Plan,
        arena: arena::Identity,
    ) -> Result<Self, Error> {
        let header = trait_header(facts, plan)?;
        let FamilyInputs {
            identity: family_identity,
            requirements,
            regions,
        } = family_inputs(facts, plan)?;
        Ok(Self {
            arena,
            plan,
            states: [
                State::capture(report, facts, plan.trait_, true)?,
                State::capture(report, facts, plan.member, false)?,
            ],
            nominal: plan
                .nominal
                .map(|declaration| State::capture(report, facts, declaration, false))
                .transpose()?,
            inputs: header.inputs.view().iter().collect(),
            trait_regions: header.regions.clone(),
            canonical_results: header.results.clone(),
            family_identity,
            requirements: requirements.to_vec(),
            regions: regions.to_vec(),
            results: facts
                .associated
                .require_projection_header(plan.member.definition)
                .map_err(Error::unlocated)?
                .bounds
                .clone(),
        })
    }
    pub(super) fn current(&self, report: &Report, facts: &Declarations) -> Result<(), Error> {
        super::charge(facts, self.plan.trait_, 1)?;
        if !facts.types.has_identity(&self.arena) {
            return Err(Error::unlocated(
                "source Pointee contract belongs to another type arena",
            ));
        }
        let plan = match self.plan.trait_.origin {
            DeclarationOrigin::Source(_) => super::schemas::plan(report, facts)?,
            DeclarationOrigin::Imported(_) => {
                let mut steps = 0;
                let checked = super::published_family::plan(report, facts, &mut steps);
                super::charge(facts, self.plan.trait_, steps)?;
                checked.map_err(Error::unlocated)?
            }
        };
        if plan != Some(self.plan) {
            return Err(Error::unlocated(
                "source Pointee checked owner or schema changed",
            ));
        }
        for state in &self.states {
            state.current(report, facts)?;
        }
        if let Some(nominal) = &self.nominal {
            nominal.current(report, facts)?;
        }
        self.current_typed(facts)
    }
    /// This replay belongs to the immutable Report phase authenticated by
    /// capture/current; it does not claim access to a separately changed Report.
    pub(super) fn current_facts(&self, facts: &Declarations) -> Result<(), Error> {
        super::charge(facts, self.plan.trait_, 1)?;
        if !facts.types.has_identity(&self.arena) {
            return Err(Error::unlocated(
                "source Pointee contract belongs to another type arena",
            ));
        }
        self.completed_roles(facts)?;
        for state in &self.states {
            state.current_facts(facts)?;
        }
        if let Some(nominal) = &self.nominal {
            nominal.current_facts(facts)?;
        }
        self.current_typed(facts)
    }
    fn current_typed(&self, facts: &Declarations) -> Result<(), Error> {
        let plan = self.plan;
        let header = trait_header(facts, plan)?;
        let FamilyInputs {
            identity: family_identity,
            requirements,
            regions,
        } = family_inputs(facts, plan)?;
        let contract = facts
            .associated
            .require_projection_header(plan.member.definition)
            .map_err(Error::unlocated)?;
        let results = &contract.bounds;
        super::charge(
            facts,
            plan.trait_,
            self.inputs
                .len()
                .saturating_add(self.trait_regions.len())
                .saturating_add(self.canonical_results.len())
                .saturating_add(self.requirements.len())
                .saturating_add(self.regions.len())
                .saturating_add(self.results.len()),
        )?;
        if header.self_type != plan.self_type
            || facts.types.kind(contract.projection)
                != Some(crate::types::Kind::Projection(plan.projection))
            || facts
                .types
                .projection_schema(crate::types::ItemId(plan.member.definition.0))
                != Some(plan.schema)
            || header.inputs.view().len() != self.inputs.len()
            || !header
                .inputs
                .view()
                .iter()
                .zip(&self.inputs)
                .all(|(a, b)| a.source == b.source && a.value == b.value)
            || header.regions != self.trait_regions
            || header.results.len() != self.canonical_results.len()
            || !header
                .results
                .iter()
                .zip(&self.canonical_results)
                .all(|((ai, a), (bi, b))| ai == bi && a.source == b.source && a.value == b.value)
            || family_identity != self.family_identity
            || requirements != self.requirements
            || regions != self.regions
            || *results != self.results
        {
            return Err(Error::unlocated(
                "source Metadata retained typed obligations changed",
            ));
        }
        Ok(())
    }
    pub(super) fn completed_roles(&self, facts: &Declarations) -> Result<(), Error> {
        super::charge(facts, self.plan.trait_, 3)?;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| Error::unlocated("source Pointee role catalog disappeared"))?;
        if catalog
            .neutral_attributes(self.plan.trait_)
            .map_err(Error::unlocated)?
            != self.plan.attributes
        {
            return Err(Error::unlocated(
                "source Pointee role changed its retained header attributes",
            ));
        }
        for (role, expected) in [
            (LangItem::PointeeTrait, Some(self.plan.trait_)),
            (LangItem::MetadataType, Some(self.plan.member)),
            (LangItem::DynMetadata, self.plan.nominal),
        ] {
            if catalog.validated_role(role).map_err(Error::unlocated)? != expected {
                return Err(Error::unlocated(
                    "source Pointee completed role changed its actual declaration",
                ));
            }
        }
        Ok(())
    }
    pub(super) fn application(
        &self,
        facts: &Declarations,
        application: projection::Application,
    ) -> Result<(), Error> {
        if application.item != ItemId(self.plan.member.definition.0)
            || facts.types.kind(application.subject).is_none()
            || facts
                .types
                .trait_application(application.trait_application)
                .is_none_or(|a| {
                    a.item != self.plan.schema.trait_item
                        || facts
                            .types
                            .arguments(a.arguments)
                            .is_none_or(|a| a.len() != 0)
                })
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err(Error::unlocated(
                "source Metadata use changed its actual member or complete inputs",
            ));
        }
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn capacities(&self) -> [usize; 15] {
        let mut result = [0; 15];
        for (slot, state) in result[..9]
            .as_chunks_mut::<3>()
            .0
            .iter_mut()
            .zip(&self.states)
        {
            slot.copy_from_slice(&[state.parameters.capacity(), state.clauses.capacity(), 0]);
        }
        if let Some(nominal) = &self.nominal {
            result[6..9].copy_from_slice(&[
                nominal.parameters.capacity(),
                nominal.clauses.capacity(),
                0,
            ]);
        }
        result[9..].copy_from_slice(&[
            self.inputs.capacity(),
            self.trait_regions.capacity(),
            self.canonical_results.capacity(),
            self.requirements.capacity(),
            self.regions.capacity(),
            self.results.capacity(),
        ]);
        result
    }
    #[cfg(test)]
    pub(super) fn layouts() -> (usize, usize) {
        (size_of::<Self>(), size_of::<State>())
    }
}
