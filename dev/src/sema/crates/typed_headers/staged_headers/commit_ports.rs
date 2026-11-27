//! Publish one completely prepared canonical inventory; executable proof stays separate.
use super::{Declarations, Error, Headers, Stage, error, mapped_source};
use crate::{
    resolve::model::DefId,
    sema::{
        alias_requirements::Requirements, associated::ImportedOwner, crates::ImportedOwners,
        traits::Contracts,
    },
    support::retained,
    types::{ItemId, callable::Templates, outlives::Pair},
};
use std::rc::Rc;

/// One compilation installs the canonical tables for its complete import inventory.
pub(in crate::sema::crates::typed_headers) struct Installation {
    aliases: Requirements,
    callables: Templates,
    contracts: Contracts,
    graphs: Vec<Rc<mapped_source::Graph>>,
    owners: Vec<ImportedOwner>,
    before: retained::Allocation,
}

impl Installation {
    pub(in crate::sema::crates::typed_headers) fn new(
        facts: &Declarations,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
        if facts.generics.traits.is_some()
            || !facts.lifetimes.callables.is_empty()
            || !facts.alias_requirements.is_empty()
        {
            return Err(error(
                "source canonical installation cannot replace an existing header inventory",
            ));
        }
        let before = table_allocation(facts, budget);
        budget.charge(before.visits.saturating_mul(2).saturating_add(1))?;
        budget.reserve(size_of::<Contracts>())?;
        Ok(Self {
            aliases: Requirements::default(),
            callables: Templates::default(),
            contracts: Contracts::default(),
            graphs: Vec::new(),
            owners: Vec::new(),
            before,
        })
    }

    pub(in crate::sema::crates::typed_headers) fn add(
        &mut self,
        context: &mapped_source::Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        Stage::prepare(context, facts, &mut self.aliases, budget)?
            .install(context, facts, self, budget)
    }

    fn reserve(&self, additional_owners: usize, budget: &mut Headers) -> Result<(), Error> {
        let owner_growth = retained::Growth::vector::<ImportedOwner>(
            self.owners.len(),
            self.owners.capacity(),
            additional_owners,
        );
        let graph_growth = retained::Growth::vector::<Rc<mapped_source::Graph>>(
            self.graphs.len(),
            self.graphs.capacity(),
            1,
        );
        budget.reserve(owner_growth.bytes.saturating_add(graph_growth.bytes))?;
        budget.charge(owner_growth.work.saturating_add(graph_growth.work))
    }

    pub(in crate::sema::crates::typed_headers) fn finish(
        self,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        self.contracts
            .check_cycles(&facts.types)
            .map_err(|(_, message)| error(message))?;
        budget.reserve(facts.associated.imported_allocation_bound(
            self.owners.len(),
            self.owners.iter().map(|row| row.members.len()).sum(),
            self.owners.iter().map(|row| row.results.len()).sum(),
        ))?;
        let graphs = Rc::new(mapped_source::Graphs::new(
            self.graphs,
            &facts.types,
            budget,
        )?);
        let mut allocation = self.callables.allocation();
        allocation.include(self.contracts.header_table_allocation());
        budget.charge(allocation.visits)?;
        budget.observed.include(allocation);
        budget.observed.add(size_of::<Contracts>());
        let source = ImportedOwners::Ordinary(Rc::clone(&graphs));
        let mut work = budget.work.get();
        let installed =
            facts
                .associated
                .install_imported(&facts.types, source, self.owners, &mut work);
        budget.work.set(work);
        installed.map_err(error)?;
        facts.lifetimes.callables = self.callables;
        facts.generics.traits = Some(Box::new(self.contracts));
        facts.alias_requirements = self.aliases;
        budget.source = Some(ImportedOwners::Ordinary(graphs));
        let after = table_allocation(facts, budget);
        budget.observed.include(retained::Allocation {
            bytes: after.bytes.saturating_sub(self.before.bytes),
            visits: self.before.visits.saturating_add(after.visits),
        });
        Ok(())
    }
}

impl Stage {
    fn install(
        mut self,
        context: &mapped_source::Context<'_>,
        facts: &mut Declarations,
        installation: &mut Installation,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        let constants = self.preflight(context, facts, budget)?;
        let effects = if context.descriptor.profile().bootstrap_bindings() {
            None
        } else {
            prepare_effects(&self.scopes, facts, constants, budget)?
        };
        let owners = self.associated_rows(context, &facts.types, budget)?;
        installation.reserve(owners.len(), budget)?;
        let functions = super::template_ports::prepare(
            context,
            &self.graph,
            facts,
            &self.scopes,
            &mut installation.callables,
            budget,
        )?;
        let exact = super::verbatim_exports::prepare(context, facts, &functions, budget)?;
        let cells = super::definition_cells::prepare(
            context,
            &self.graph,
            facts,
            &self.scopes,
            &functions,
            budget,
        )?;
        let self_bounds = super::contract_ports::prepare(
            context,
            &self.graph,
            &facts.types,
            &mut self.scopes,
            &mut installation.contracts,
            budget,
        )?;
        let rows = context.descriptor.rows();
        prepay_cycles(rows, budget)?;
        let nominal_bounds = prepare_nominal_bounds(context, &self.graph, facts, budget)?;
        let nominals = super::nominal_ports::prepare(context, &self.graph, facts, budget)?;
        let nominal_count = nominals
            .as_ref()
            .map_or(0, |nominals| nominals.owners.len());
        super::nominal_ports::reserve_owners(facts, nominal_count, budget)?;
        let generic = super::definition_cells::generic_nominals(rows, budget)?;
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
        let (member_rows, shapes, nominal_owners, cast_enums) = if let Some(nominals) = nominals {
            (
                Some(nominals.members.ready(&mut facts.members).map_err(error)?),
                Some(
                    nominals
                        .shapes
                        .ready(&mut facts.aggregates)
                        .map_err(error)?,
                ),
                nominals.owners,
                nominals.cast_enums,
            )
        } else {
            (None, None, Vec::new(), Vec::new())
        };
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
        facts.enum_casts.extend(cast_enums);
        installation.owners.extend(owners);
        installation.graphs.push(self.graph);
        publish_scopes(self.scopes, facts, budget);
        let linkage = facts.crates.get_or_insert_default();
        for function in functions {
            if let super::template_ports::Symbol::Canonical(symbol) = function.symbol {
                linkage.symbols.insert(function.definition, symbol);
            }
        }
        facts.generics.trait_self_bounds.extend(self_bounds);
        Ok(())
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
        if self.scopes.len() != rows.scopes.len() {
            return Err(error(
                "source canonical installation changed the scope inventory",
            ));
        }
        prepay_tables(facts, rows.scopes.len(), rows.functions.len(), budget)?;
        let associated = map_growth(&facts.associated.types, rows.associated_types.len());
        let bounds = self
            .scopes
            .iter()
            .filter_map(|scope| match &scope.inputs {
                super::Inputs::Associated { bounds, .. } => Some(bounds.len()),
                _ => None,
            })
            .sum::<usize>();
        budget.reserve(associated.bytes.saturating_add(
            bounds.saturating_mul(size_of::<crate::types::parameters::clauses::Clause>()),
        ))?;
        budget.charge(associated.work)?;
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
    let mut growth = retained::Growth::default();
    for table in [
        map_growth(&facts.generics.parameters, scopes),
        map_growth(&facts.generics.environments, scopes),
        map_growth(&facts.generics.parents, scopes),
        map_growth(&budget.entries, functions),
        map_growth(&facts.lifetimes.functions, functions),
        map_growth(&facts.methods.receivers, functions),
    ] {
        growth.include(table);
    }
    if let Some(linkage) = &facts.crates {
        growth.include(map_growth(&linkage.symbols, functions));
    }
    budget.charge(growth.work)?;
    budget.reserve(growth.bytes)
}

fn map_growth<K, V, S>(
    map: &std::collections::HashMap<K, V, S>,
    additional: usize,
) -> retained::Growth {
    retained::Growth::map::<K, V>(map.len(), map.capacity(), additional)
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
    result.map(&headers.written_signatures);
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
