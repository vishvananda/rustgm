//! Stage real source headers without installing roles, declarations or originals.
mod commit_ports;
pub(super) mod contract_ports;
mod definition_cells;
pub(super) mod environments;
mod formal_headers;
mod materialized_defaults;
pub(in crate::sema::crates::typed_headers) use formal_headers::Callable;
mod impl_headers;
#[path = "staged_headers/mapped_records.rs"]
mod nominal_ports;
mod owner_packets;
pub(super) mod rebuilt_environments;
pub(super) mod scheme_ports;
pub(super) mod template_ports;
mod verbatim_exports;
use super::{Declarations, Error, Header, Headers, error, mapped_source, predicate_remap};
use crate::{
    resolve::model::DefId,
    sema::{alias_requirements::Requirements, declaration_inputs::Input, generics::Parameter},
    support::crates::generics::source as s,
    types::{
        arguments::Argument,
        lifetime_proof::ScopeId,
        parameters::{
            ParameterId,
            clauses::Clause,
            environment::{Abilities, Environment},
        },
    },
};
use std::sync::Arc;
pub(super) use template_ports::register as register_callable;

/// This is a transaction-local canonical correspondence, never an installed
/// descriptor, completed role inventory or proof of an original/call instance.
pub(super) struct Stage {
    graph: std::rc::Rc<mapped_source::Graph>,
    scopes: Vec<Scope>,
    aliases: Requirements,
}
struct Scope {
    definition: DefId,
    parent: Option<DefId>,
    parameters: Vec<Parameter>,
    inputs: Inputs,
    environment: Arc<Environment>,
    function: Option<formal_headers::Callable>,
    implementation: Option<impl_headers::Implementation>,
}
enum Inputs {
    SelfType,
    Alias(Option<crate::types::TypeId>),
    Nominal(Arc<[Input]>),
    Associated {
        inputs: Arc<[Input]>,
        bounds: Arc<[Input]>,
        projection: crate::types::TypeId,
        family: Option<crate::sema::associated::Family>,
    },
    Trait(predicate_remap::Prepared),
    TraitAlias(Arc<[Input]>),
    Function(Arc<[Input]>),
    Implementation(Arc<[Input]>),
    /// An implementation's associated-result definition: the version-thirteen
    /// profile admits no own family inputs, so it inherits its impl's frame.
    Definition,
    /// The family conformance proof opens these premises with its own
    /// slots after reconstructing the enclosing implementation.
    FamilyDefinition(crate::sema::associated::Family),
}
impl Inputs {
    fn all(&self) -> &[Input] {
        match self {
            Self::SelfType | Self::Definition | Self::FamilyDefinition(_) | Self::Alias(_) => &[],
            Self::Trait(prepared) => prepared.all(),
            Self::Associated { inputs, .. }
            | Self::TraitAlias(inputs)
            | Self::Function(inputs)
            | Self::Implementation(inputs)
            | Self::Nominal(inputs) => inputs,
        }
    }
}
struct Active {
    owner: crate::support::crates::owner_catalog::DeclarationRef,
    parameters: usize,
    constants: usize,
    clauses: usize,
    environment: Arc<Environment>,
}
struct Walk<'a, 'b: 'a> {
    context: &'a mapped_source::Context<'b>,
    graph: &'a mapped_source::Graph,
    remap: predicate_remap::Context<'a>,
    active: Vec<Active>,
    abilities: Vec<(ParameterId, Abilities)>,
    constants: Vec<ParameterId>,
    clauses: Vec<Clause>,
}
impl Stage {
    pub(super) fn prepare(
        context: &mapped_source::Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
        let mut graph = mapped_source::Graph::prepare(context, facts, budget)?;
        graph.finish_opaques(context, facts, budget)?;
        let rows = context.descriptor.rows();
        budget.reserve(
            rows.scopes
                .len()
                .saturating_mul(size_of::<Scope>() + size_of::<Active>())
                .saturating_add(
                    rows.slots
                        .len()
                        .saturating_mul(size_of::<(ParameterId, Abilities)>()),
                )
                .saturating_add(rows.clauses.len().saturating_mul(size_of::<Clause>())),
        )?;
        let mut remap = predicate_remap::Context::new(
            context.imported,
            context.catalog,
            context.descriptor,
            &graph,
            budget,
        )?;
        graph.finish_opaque_results(context, &mut remap, facts, budget)?;
        let mut scopes = Vec::with_capacity(rows.scopes.len());
        let mut aliases = Requirements::default();
        let mut walk = Walk {
            context,
            graph: &graph,
            remap,
            active: Vec::with_capacity(rows.scopes.len()),
            abilities: Vec::with_capacity(rows.slots.len()),
            constants: Vec::new(),
            clauses: Vec::with_capacity(rows.clauses.len()),
        };
        for raw in &rows.scopes {
            let scope = walk.enter(facts, raw, budget)?;
            if let Inputs::Alias(default) = scope.inputs {
                install_alias_normalizations(
                    rows,
                    &graph,
                    (scope.definition, raw.owner),
                    default,
                    &mut aliases,
                    budget,
                )?;
            }
            scopes.push(scope);
        }
        budget.charge(1)?;
        let Walk { remap, .. } = walk;
        let applications = remap.finish(&mut facts.types, budget)?;
        graph.finish_applications(applications, &facts.types, budget)?;
        finish_inputs(context, &mut graph, &scopes, budget)?;
        rebuilt_environments::complete(context, &graph, facts, &mut scopes, budget)?;
        budget.reserve(2 * size_of::<usize>())?;
        budget
            .observed
            .add(size_of::<mapped_source::Graph>() + 2 * size_of::<usize>());
        Ok(Self {
            graph: std::rc::Rc::new(graph),
            scopes,
            aliases,
        })
    }
}

fn install_alias_normalizations(
    rows: s::ValidatedView<'_>,
    graph: &mapped_source::Graph,
    (definition, row): (DefId, crate::support::crates::owner_catalog::DeclarationRef),
    default: Option<crate::types::TypeId>,
    aliases: &mut Requirements,
    budget: &mut Headers,
) -> Result<(), Error> {
    budget.charge(rows.alias_normalization_lookup_work())?;
    let records = rows.alias_normalizations(row);
    let count = if records.is_empty() {
        usize::from(default.is_some())
    } else {
        records.len()
    };
    if count == 0 {
        return Ok(());
    }
    let allocation = aliases.source_projection_allocation(count);
    budget.charge(allocation.visits)?;
    budget.reserve(allocation.bytes)?;
    if records.is_empty() {
        let ty = default.ok_or_else(|| error("source alias normalization lost its target"))?;
        return aliases.source_projection(definition, ty).map_err(error);
    }
    for record in records {
        aliases
            .normalization(
                definition,
                definition,
                graph.ty(record.target)?,
                ScopeId::ROOT,
            )
            .map_err(error)?;
    }
    aliases.seal(definition).map_err(error)
}

fn finish_inputs(
    context: &mapped_source::Context<'_>,
    graph: &mut mapped_source::Graph,
    scopes: &[Scope],
    budget: &mut Headers,
) -> Result<(), Error> {
    let rows = context.descriptor.rows();
    budget.reserve(
        rows.traits
            .len()
            .saturating_add(rows.trait_aliases().len())
            .saturating_add(rows.implementations.len())
            .saturating_add(rows.nominals.len())
            .saturating_mul(size_of::<Option<Arc<[Input]>>>()),
    )?;
    let mut nominals = vec![None; rows.nominals.len()];
    let mut associated = vec![None; rows.associated_types.len()];
    let mut traits = vec![None; rows.traits.len()];
    let mut trait_aliases = vec![None; rows.trait_aliases().len()];
    let mut implementations = vec![None; rows.implementations.len()];
    for (raw, scope) in rows.scopes.iter().zip(scopes) {
        budget.charge(1)?;
        let (slot, inputs) = match (rows.header(raw.owner).map_err(error)?, &scope.inputs) {
            (Some(s::HeaderRef::AssociatedType(index)), Inputs::Associated { inputs, .. }) => {
                (&mut associated[index as usize], inputs)
            }
            (Some(s::HeaderRef::Nominal(index)), Inputs::Nominal(inputs)) => {
                (&mut nominals[index as usize], inputs)
            }
            (Some(s::HeaderRef::Trait(index)), Inputs::Trait(prepared)) => {
                (&mut traits[index as usize], prepared.all())
            }
            (Some(s::HeaderRef::TraitAlias(index)), Inputs::TraitAlias(inputs)) => {
                (&mut trait_aliases[index as usize], inputs)
            }
            (Some(s::HeaderRef::Implementation(index)), Inputs::Implementation(inputs)) => {
                (&mut implementations[index as usize], inputs)
            }
            (None, Inputs::SelfType)
            | (Some(s::HeaderRef::TypeAlias(_)), Inputs::Alias(_))
            | (Some(s::HeaderRef::Function(_)), Inputs::Function(_))
            | (
                Some(s::HeaderRef::AssociatedDefinition(_) | s::HeaderRef::Constant(_)),
                Inputs::Definition,
            )
            | (Some(s::HeaderRef::AssociatedDefinition(_)), Inputs::FamilyDefinition(_)) => {
                continue;
            }
            _ => {
                return Err(error(
                    "source full inputs changed their exact scope/header kind",
                ));
            }
        };
        if slot.replace(Arc::clone(inputs)).is_some() {
            return Err(error(
                "source full input header has duplicate actual scope coverage",
            ));
        }
    }
    budget.charge(traits.len().saturating_add(implementations.len()))?;
    budget.observed.vector(&nominals);
    budget.observed.vector(&traits);
    budget.observed.vector(&trait_aliases);
    budget.observed.vector(&implementations);
    graph
        .finish_inputs(nominals, traits, implementations, associated, trait_aliases)
        .map_err(error)
}
impl Walk<'_, '_> {
    fn inputs(
        &mut self,
        facts: &mut Declarations,
        raw: &s::ParameterScope,
        definition: DefId,
        budget: &mut Headers,
    ) -> Result<Inputs, Error> {
        let rows = self.context.descriptor.rows();
        match rows.header(raw.owner).map_err(error)? {
            Some(s::HeaderRef::Trait(index))
                if raw.parent == Some(rows.traits[index as usize].self_definition) =>
            {
                Ok(Inputs::Trait(self.remap.trait_inputs(
                    definition,
                    &mut facts.types,
                    budget,
                )?))
            }
            Some(s::HeaderRef::TraitAlias(index))
                if raw.parent == Some(rows.trait_aliases()[index as usize].self_definition) =>
            {
                Ok(Inputs::TraitAlias(self.remap.declaration_inputs(
                    definition,
                    &mut facts.types,
                    budget,
                )?))
            }
            Some(s::HeaderRef::AssociatedType(index)) => self.associated_type_inputs(
                facts,
                raw,
                definition,
                &rows.associated_types[index as usize],
                budget,
            ),
            Some(s::HeaderRef::Nominal(_)) if raw.parent.is_none() => Ok(Inputs::Nominal(
                self.remap
                    .declaration_inputs(definition, &mut facts.types, budget)?,
            )),
            Some(s::HeaderRef::Function(index)) => {
                let header = &rows.functions[index as usize];
                if header.parent != raw.parent || self.active.len() > 2 {
                    return Err(error(
                        "source function has an unsupported parameter-scope owner chain",
                    ));
                }
                Ok(Inputs::Function(self.remap.declaration_inputs(
                    definition,
                    &mut facts.types,
                    budget,
                )?))
            }
            Some(s::HeaderRef::Implementation(index)) if raw.parent.is_none() => {
                if rows.implementations[index as usize].self_definition != raw.owner {
                    return Err(error(
                        "source impl input scope changed its actual Self owner",
                    ));
                }
                Ok(Inputs::Implementation(self.remap.declaration_inputs(
                    definition,
                    &mut facts.types,
                    budget,
                )?))
            }
            Some(s::HeaderRef::AssociatedDefinition(index)) => {
                self.associated_definition_inputs(facts, raw, definition, index, budget)
            }
            Some(s::HeaderRef::Constant(index))
                if !rows.constants()[index as usize].body.evaluated()
                    && (raw.parent
                        == rows.declarations[raw.owner.0 as usize].definition.parent
                        || raw.parent.is_none()
                            && matches!(
                                rows.constants()[index as usize].body,
                                s::ConstantBody::Original(_)
                            )
                            && rows.declarations[raw.owner.0 as usize]
                                .definition
                                .parent
                                .and_then(|parent| rows.declarations.get(parent.0 as usize))
                                .is_some_and(|parent| {
                                    matches!(
                                        parent.definition.kind,
                                        s::DefinitionKind::Module | s::DefinitionKind::Function
                                    )
                                })) =>
            {
                Ok(Inputs::Definition)
            }
            Some(s::HeaderRef::TypeAlias(index)) if raw.parent.is_none() => {
                let target = self.graph.ty(rows.type_aliases()[index as usize].target)?;
                let projected = facts
                    .types
                    .contains_projections(target)
                    .ok_or_else(|| error("source alias target has no actual projection summary"))?;
                Ok(Inputs::Alias(projected.then_some(target)))
            }
            None if raw.parent.is_none() => Self::self_type_inputs(rows, raw.owner),
            _ => Err(error(
                "source scope requires its wider actual header consumer",
            )),
        }
    }
    fn associated_definition_inputs(
        &mut self,
        facts: &mut Declarations,
        raw: &s::ParameterScope,
        definition: DefId,
        index: u32,
        budget: &mut Headers,
    ) -> Result<Inputs, Error> {
        let rows = self.context.descriptor.rows();
        let parent = raw
            .parent
            .ok_or_else(|| error("source associated definition has no impl scope"))?;
        let Some(s::HeaderRef::Implementation(parent_index)) =
            rows.header(parent).map_err(error)?
        else {
            return Err(error(
                "source associated definition scope lost its actual impl owner",
            ));
        };
        if self.active.len() != 1 {
            return Err(error(
                "source associated definition scope lost its actual impl owner",
            ));
        }
        let own = raw.declared.end - raw.declared.start;
        if own == 0 {
            return Ok(Inputs::Definition);
        }
        if !self.context.descriptor.profile().declaration_policies() {
            return Err(error(
                "source associated definition families require their wider profile",
            ));
        }
        let captures = rows.implementations[parent_index as usize].region_captures;
        let total = captures.checked_add(own).ok_or_else(|| {
            error("associated definition lifetime count exceeds its checked scope")
        })?;
        let count = rows.cold().associated_definition_inputs.len();
        budget.charge(count.checked_ilog2().unwrap_or(0) as usize + 1)?;
        let inputs = if rows.cold().definition_inputs(index).is_some() {
            self.remap
                .declaration_inputs(definition, &mut facts.types, budget)?
        } else {
            Arc::default()
        };
        budget.reserve(
            inputs
                .len()
                .saturating_mul(size_of::<crate::types::parameters::clauses::Clause>()),
        )?;
        let requirements = inputs.iter().map(|input| input.value).collect::<Vec<_>>();
        budget.observed.vector(&requirements);
        Ok(Inputs::FamilyDefinition(crate::sema::associated::Family {
            captures,
            total,
            regions: Vec::new(),
            requirements: Some(requirements),
        }))
    }
    fn associated_type_inputs(
        &mut self,
        facts: &mut Declarations,
        scope: &s::ParameterScope,
        definition: DefId,
        header: &s::AssociatedTypeHeader,
        budget: &mut Headers,
    ) -> Result<Inputs, Error> {
        if scope.parent != Some(header.parent) {
            return Err(error(
                "associated type scope changed its actual trait parent",
            ));
        }
        let inputs = self
            .remap
            .declaration_inputs(definition, &mut facts.types, budget)?;
        let family = contract_ports::associated_family(
            self.context.descriptor.rows(),
            header,
            scope,
            &inputs,
            budget,
        )?;
        Ok(Inputs::Associated {
            family,
            inputs,
            bounds: self
                .remap
                .associated_bounds(definition, &mut facts.types, budget)?,
            projection: self.graph.ty(header.projection)?,
        })
    }

    fn self_type_inputs(
        rows: s::ValidatedView<'_>,
        owner: crate::support::crates::owner_catalog::DeclarationRef,
    ) -> Result<Inputs, Error> {
        let declaration = rows
            .declarations
            .get(owner.0 as usize)
            .ok_or_else(|| error("source Self scope is outside its declaration table"))?;
        let parent = declaration
            .definition
            .parent
            .ok_or_else(|| error("source Self scope has no actual lexical owner"))?;
        let self_definition = match rows.header(parent).map_err(error)? {
            Some(s::HeaderRef::Trait(index)) => rows.traits[index as usize].self_definition,
            Some(s::HeaderRef::TraitAlias(index)) => {
                rows.trait_aliases()[index as usize].self_definition
            }
            _ => return Err(error("source Self scope lacks its actual lexical owner")),
        };
        if declaration.definition.kind != s::DefinitionKind::SelfType || self_definition != owner {
            return Err(error("source scope is not the actual retained trait Self"));
        }
        Ok(Inputs::SelfType)
    }
    fn append_formals(
        &mut self,
        store: &crate::types::Store,
        parameters: &[Parameter],
        budget: &mut Headers,
    ) -> Result<(), Error> {
        for parameter in parameters {
            budget.charge(1)?;
            let argument = parameter.argument;
            if matches!(argument, Argument::Lifetime(_)) {
                continue;
            }
            let parameter = crate::sema::generics::parameter_id(store, argument).map_err(error)?;
            if matches!(argument, Argument::Const(_)) {
                let growth = crate::support::retained::Growth::vector::<ParameterId>(
                    self.constants.len(),
                    self.constants.capacity(),
                    1,
                );
                budget.pay(growth.work, growth.bytes).map_err(error)?;
                self.constants
                    .try_reserve(1)
                    .map_err(|_| error("source const formal scope allocation failed"))?;
                self.constants.push(parameter);
            } else {
                self.abilities.push((parameter, Abilities::default()));
            }
        }
        Ok(())
    }

    fn enter(
        &mut self,
        facts: &mut Declarations,
        raw: &s::ParameterScope,
        budget: &mut Headers,
    ) -> Result<Scope, Error> {
        let context = self.context;
        budget.charge(1)?;
        while self.active.last().map(|a| a.owner) != raw.parent {
            budget.charge(1)?;
            let closed = self
                .active
                .pop()
                .ok_or_else(|| error("source header parent is missing or closed"))?;
            self.abilities.truncate(closed.parameters);
            self.constants.truncate(closed.constants);
            self.clauses.truncate(closed.clauses);
        }
        let definition = context.definition(raw.owner, budget)?;
        let inputs = self.inputs(facts, raw, definition, budget)?;
        let previous_parameters = self.abilities.len();
        let previous_constants = self.constants.len();
        let previous_clauses = self.clauses.len();
        let parameters =
            formal_headers::parameters(context, self.graph, raw, &mut facts.types, budget)?;
        self.append_formals(&facts.types, &parameters, budget)?;
        for input in inputs.all() {
            budget.charge(1)?;
            self.clauses.push(input.value);
        }
        let environment = environments::prepare(
            facts,
            self.active.last().map(|a| &a.environment),
            (&self.abilities, &self.constants),
            (&self.clauses, Some(context.descriptor.profile())),
            (previous_parameters, previous_constants),
            previous_clauses,
            budget,
        )?;
        let function = if let Inputs::Function(inputs) = &inputs {
            formal_headers::header(
                context,
                self.graph,
                facts,
                raw,
                inputs,
                &environment,
                budget,
            )?
        } else {
            None
        };
        let parent = raw
            .parent
            .map(|owner| context.definition(owner, budget))
            .transpose()?;
        let implementation = if matches!(inputs, Inputs::Implementation(_)) {
            Some(impl_headers::prepare(
                context,
                self.graph,
                &mut self.remap,
                facts,
                raw.owner,
                budget,
            )?)
        } else {
            None
        };
        self.active.push(Active {
            owner: raw.owner,
            parameters: previous_parameters,
            constants: previous_constants,
            clauses: previous_clauses,
            environment: Arc::clone(&environment),
        });
        Ok(Scope {
            definition,
            parent,
            parameters,
            inputs,
            environment,
            function,
            implementation,
        })
    }
}
