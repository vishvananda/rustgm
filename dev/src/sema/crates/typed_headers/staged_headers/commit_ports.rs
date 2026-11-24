//! Publish one completely prepared canonical inventory; executable proof stays separate.
use super::{Declarations, Error, Headers, Stage, error, mapped_source};
use crate::{
    resolve::model::DefId,
    sema::{generics::Parameter, lifetimes::FunctionRegions},
    support::retained,
    types::{ItemId, TypeId, outlives::Pair, parameters::environment::Environment},
};
use std::{rc::Rc, sync::Arc};

impl Stage {
    pub(in crate::sema::crates::typed_headers) fn install(
        mut self,
        context: &mapped_source::Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        let constants = self.preflight(context, facts, budget)?;
        let effects = prepare_effects(&self.scopes, facts, constants, budget)?;
        let owners = self.associated_rows(context, &facts.types, budget)?;
        budget.reserve(facts.associated.imported_allocation_bound(
            owners.len(),
            owners.iter().map(|row| row.members.len()).sum(),
            owners.iter().map(|row| row.results.len()).sum(),
        ))?;
        let (callables, functions) =
            super::template_ports::prepare(context, &self.graph, facts, &self.scopes, budget)?;
        let exact = super::verbatim_exports::prepare(context, facts, &functions, budget)?;
        let cells = super::definition_cells::prepare(
            context,
            &self.graph,
            facts,
            &self.scopes,
            &functions,
            budget,
        )?;
        let (contracts, self_bounds) = super::contract_ports::prepare(
            context,
            &self.graph,
            &facts.types,
            &mut self.scopes,
            budget,
        )?;
        let rows = context.descriptor.rows();
        prepay_cycles(rows, budget)?;
        contracts
            .check_cycles(&facts.types)
            .map_err(|(_, message)| error(message))?;
        let nominal_bounds = prepare_nominal_bounds(context, &self.graph, facts, budget)?;
        let nominals = super::nominal_ports::prepare(context, &self.graph, facts, budget)?;
        let nominal_count = nominals
            .as_ref()
            .map_or(0, |nominals| nominals.owners.len());
        super::nominal_ports::reserve_owners(facts, nominal_count, budget)?;
        let generic = super::definition_cells::generic_nominals(rows, budget)?;
        let before = table_allocation(facts, budget);
        budget.charge(before.visits.saturating_mul(2).saturating_add(1))?;
        // This shared work count is checked inside the owner installer BEFORE
        // its first write; failure cannot leave a partially installed owner.
        let mut work = budget.work.get();
        let exact = super::verbatim_exports::destination(exact, facts.foreign.as_deref_mut())?;
        let effects = effects
            .map(|plan| {
                facts
                    .source_bindings
                    .as_deref_mut()
                    .map(|catalog| (plan, catalog))
                    .ok_or_else(|| error("source constant effect destination disappeared"))
            })
            .transpose()?;
        let (member_rows, shapes, nominal_owners) = if let Some(nominals) = nominals {
            (
                Some(nominals.members.ready(&mut facts.members).map_err(error)?),
                Some(
                    nominals
                        .shapes
                        .ready(&mut facts.aggregates)
                        .map_err(error)?,
                ),
                nominals.owners,
            )
        } else {
            (None, None, Vec::new())
        };
        let installed = facts.associated.install_imported(
            &facts.types,
            crate::sema::crates::ImportedOwners::Ordinary(Rc::clone(&self.graph)),
            owners,
            &mut work,
        );
        budget.work.set(work);
        installed.map_err(error)?;
        if let Some(rows) = member_rows {
            rows.install();
        }
        if let Some(shapes) = shapes {
            shapes.install();
        }
        // Every remaining key, index and allocation bound was checked above.
        // No fallible remap, quota check or proof follows this first publication.
        for (definition, ty) in cells {
            facts.definitions[definition.0 as usize] = Some(ty);
        }
        if let Some((plan, registry)) = exact {
            plan.install(registry);
        }
        if let Some((plan, catalog)) = effects {
            plan.install(catalog);
        }
        install_nominal_facts(facts, nominal_owners, nominal_bounds, generic);
        facts.lifetimes.callables = callables;
        if !self.aliases.is_empty() {
            facts.alias_requirements = std::mem::take(&mut self.aliases);
        }
        self.publish(facts, budget, functions, contracts, self_bounds, before);
        Ok(())
    }
    fn publish(
        self,
        facts: &mut Declarations,
        budget: &mut Headers,
        functions: Vec<super::template_ports::Callable>,
        contracts: crate::sema::traits::Contracts,
        self_bounds: Vec<(DefId, crate::types::parameters::predicates::Predicate)>,
        before: retained::Allocation,
    ) {
        publish_scopes(self.scopes, facts, budget);
        let linkage = facts.crates.get_or_insert_default();
        for function in functions {
            if let Some(symbol) = function.symbol {
                linkage.symbols.insert(function.definition, symbol);
            }
        }
        facts.generics.traits = Some(Box::new(contracts));
        facts.generics.trait_self_bounds.extend(self_bounds);
        budget.source = Some(crate::sema::crates::ImportedOwners::Ordinary(self.graph));
        let after = table_allocation(facts, budget);
        budget.observed.include(retained::Allocation {
            bytes: after.bytes.saturating_sub(before.bytes),
            visits: before.visits.saturating_add(after.visits),
        });
    }

    fn preflight(
        &self,
        context: &mapped_source::Context<'_>,
        facts: &Declarations,
        budget: &mut Headers,
    ) -> Result<usize, Error> {
        self.graph.require(
            context.imported,
            context.catalog,
            context.descriptor,
            budget,
        )?;
        self.graph.require_store(&facts.types).map_err(error)?;
        if facts
            .crates
            .as_ref()
            .and_then(|linkage| linkage.origin_catalog())
            .is_none_or(|catalog| !std::ptr::eq(catalog, context.catalog))
        {
            return Err(error(
                "source installation changed its actual semantic ownership catalog",
            ));
        }
        let rows = context.descriptor.rows();
        if self.scopes.len() != rows.scopes.len()
            || budget.source.is_some()
            || !budget.entries.is_empty()
            || facts.generics.traits.is_some()
            || !facts.lifetimes.callables.is_empty()
            || !self.aliases.is_empty() && !facts.alias_requirements.is_empty()
        {
            return Err(error(
                "source canonical installation cannot replace a partial or existing header inventory",
            ));
        }
        prepay_tables(facts, rows.scopes.len(), rows.functions.len(), budget)?;
        budget.reserve(
            retained::new_map::<DefId, crate::sema::associated::TypeContract>(
                facts
                    .associated
                    .types
                    .len()
                    .saturating_add(rows.associated_types.len()),
            )
            .saturating_add(
                self.scopes
                    .iter()
                    .filter_map(|scope| match &scope.inputs {
                        super::Inputs::Associated { bounds, .. } => Some(bounds.len()),
                        _ => None,
                    })
                    .sum::<usize>()
                    .saturating_mul(size_of::<crate::types::parameters::clauses::Clause>()),
            ),
        )?;
        let mut constants = 0usize;
        let mut families = 0usize;
        for scope in &self.scopes {
            budget.charge(1)?;
            constants += usize::from(scope.function.as_ref().is_some_and(|c| c.header.constant));
            let definition = scope.definition;
            families += usize::from(matches!(
                scope.inputs,
                super::Inputs::Associated {
                    family: Some(_),
                    ..
                } | super::Inputs::FamilyDefinition(_)
            ));
            if facts.associated.families.contains_key(&definition)
                || facts.generics.parameters.contains_key(&definition)
                || facts.generics.environments.contains_key(&definition)
                || facts.generics.parents.contains_key(&definition)
                || facts.generics.predicates.contains_key(&definition)
                || facts.generics.trait_self_bounds.contains_key(&definition)
                || facts.lifetimes.functions.contains_key(&definition)
                || facts.methods.receivers.contains_key(&definition)
                || facts.function_indices.contains_key(&definition)
                || facts.constant_functions.contains(&definition)
                || facts
                    .crates
                    .as_ref()
                    .is_some_and(|linkage| linkage.symbols.contains_key(&definition))
            {
                return Err(error(
                    "source scope destination contains previously installed declaration facts",
                ));
            }
        }
        let family_growth = retained::Growth::map::<DefId, crate::sema::associated::Family>(
            facts.associated.families.len(),
            facts.associated.families.capacity(),
            families,
        );
        budget.charge(family_growth.work)?;
        budget.reserve(family_growth.bytes)?;
        let growth = retained::Growth::map::<DefId, ()>(
            facts.constant_functions.len(),
            facts.constant_functions.capacity(),
            constants,
        );
        budget.charge(growth.work)?;
        budget.reserve(growth.bytes)?;
        Ok(constants)
    }
}

fn prepare_effects(
    scopes: &[super::Scope],
    facts: &Declarations,
    constants: usize,
    budget: &mut Headers,
) -> Result<Option<crate::sema::compiler_items::ImportedCallEffects>, Error> {
    if constants == 0 {
        return Ok(None);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("source constant declarations lost their actual role catalog"))?;
    let declarations = scopes.iter().filter_map(|scope| {
        scope
            .function
            .as_ref()
            .filter(|c| c.header.constant)
            .map(|c| (scope.definition, &c.header.inputs[..c.header.declared]))
    });
    super::super::predicate_remap::translation::feature_names::paid(budget, |pay| {
        crate::sema::compiler_items::ImportedCallEffects::new(
            catalog,
            &facts.types,
            constants,
            declarations,
            pay,
        )
        .map(Some)
    })
}

fn prepay_tables(
    facts: &Declarations,
    scopes: usize,
    functions: usize,
    budget: &mut Headers,
) -> Result<(), Error> {
    budget.reserve(retained::sum([
        retained::new_map::<DefId, Vec<Parameter>>(
            facts.generics.parameters.len().saturating_add(scopes),
        ),
        retained::new_map::<DefId, Arc<Environment>>(
            facts.generics.environments.len().saturating_add(scopes),
        ),
        retained::new_map::<DefId, DefId>(facts.generics.parents.len().saturating_add(scopes)),
        retained::new_map::<DefId, super::Header>(functions),
        retained::new_map::<DefId, FunctionRegions>(
            facts.lifetimes.functions.len().saturating_add(functions),
        ),
        retained::new_map::<DefId, String>(
            facts
                .crates
                .as_ref()
                .map_or(0, |l| l.symbols.len())
                .saturating_add(functions),
        ),
        retained::new_map::<DefId, (TypeId, u32)>(
            facts.methods.receivers.len().saturating_add(functions),
        ),
    ]))
}

fn install_nominal_facts(
    facts: &mut Declarations,
    nominal_owners: Vec<DefId>,
    nominal_bounds: Vec<(ItemId, Pair)>,
    generic: bool,
) {
    facts
        .crates
        .get_or_insert_default()
        .imported_records
        .extend(nominal_owners.iter().copied());
    facts.generics.nominal_contracts.extend(nominal_owners);
    facts.lifetimes.nominal_bounds.extend(nominal_bounds);
    facts.generics.has_nominal_requirements |= generic;
}

fn prepare_nominal_bounds(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    budget: &mut Headers,
) -> Result<Vec<(ItemId, Pair)>, Error> {
    let rows = context.descriptor.rows();
    let mut prepared = Vec::new();
    for row in &rows.cold().nominal_region_bounds {
        let owner = ItemId(context.definition(row.declaration, budget)?.0);
        let bounds = row.bounds.slice(&rows.region_bounds).map_err(error)?;
        budget.charge(bounds.len().saturating_mul(3))?;
        budget.reserve(bounds.len().saturating_mul(4 * size_of::<(ItemId, Pair)>()))?;
        for bound in bounds {
            prepared.push((
                owner,
                (graph.region(bound.longer)?, graph.region(bound.shorter)?),
            ));
        }
    }
    let destination = &mut facts.lifetimes.nominal_bounds;
    let growth = retained::Growth::vector::<(ItemId, Pair)>(
        destination.len(),
        destination.capacity(),
        prepared.len(),
    );
    budget.pay(growth.work, growth.bytes).map_err(error)?;
    destination
        .try_reserve(prepared.len())
        .map_err(|_| error("imported nominal lifetime allocation failed"))?;
    Ok(prepared)
}

fn publish_scopes(scopes: Vec<super::Scope>, facts: &mut Declarations, budget: &mut Headers) {
    for scope in scopes {
        let family = match scope.inputs {
            super::Inputs::Associated {
                bounds,
                projection,
                family,
                ..
            } => {
                facts.associated.types.insert(
                    scope.definition,
                    crate::sema::associated::TypeContract {
                        projection,
                        bounds: bounds.iter().map(|input| input.value).collect(),
                    },
                );
                family
            }
            super::Inputs::FamilyDefinition(family) => Some(family),
            _ => None,
        };
        if let Some(family) = family {
            facts.associated.families.insert(scope.definition, family);
        }
        if let Some(parent) = scope.parent {
            facts.generics.parents.insert(scope.definition, parent);
        }
        facts
            .generics
            .parameters
            .insert(scope.definition, scope.parameters);
        facts
            .generics
            .environments
            .insert(scope.definition, scope.environment);
        if let Some(callable) = scope.function {
            facts
                .lifetimes
                .functions
                .insert(scope.definition, callable.regions);
            if let Some(receiver) = callable.receiver {
                facts.methods.receivers.insert(scope.definition, receiver);
            }
            if callable.header.constant {
                facts.constant_functions.insert(scope.definition);
            }
            budget.entries.insert(scope.definition, callable.header);
        }
    }
}

fn table_allocation(facts: &Declarations, headers: &Headers) -> retained::Allocation {
    let mut result = facts.associated.imported_table_allocation();
    result.map(&facts.associated.families);
    result.map(&facts.generics.parameters);
    result.map(&facts.generics.parents);
    result.map(&facts.generics.environments);
    result.map(&facts.generics.trait_self_bounds);
    result.map(&facts.lifetimes.functions);
    result.map(&facts.methods.receivers);
    result.set(&facts.constant_functions);
    if let Some(bytes) = facts
        .source_bindings
        .as_deref()
        .map(crate::sema::compiler_items::Catalog::call_effect_bytes)
        .filter(|bytes| *bytes != 0)
    {
        result.add(bytes);
    }
    result.map(&headers.entries);
    if let Some(linkage) = &facts.crates {
        result.map(&linkage.symbols);
    }
    result
}

fn prepay_cycles(
    rows: crate::support::crates::generics::source::ValidatedView<'_>,
    budget: &mut Headers,
) -> Result<(), Error> {
    budget.charge(
        rows.traits
            .len()
            .saturating_add(rows.clauses.len())
            .saturating_mul(2),
    )?;
    budget.reserve(
        retained::new_map::<ItemId, bool>(rows.traits.len()).saturating_add(
            rows.traits
                .len()
                .saturating_add(rows.clauses.len())
                .saturating_mul(size_of::<(ItemId, bool)>()),
        ),
    )?;
    Ok(())
}
