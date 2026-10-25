//! Stage real source headers without installing roles, declarations or originals.
mod commit_ports;
mod contract_ports;
mod definition_cells;
mod environments;
mod formal_headers;
mod impl_headers;
mod owner_packets;
mod rebuilt_environments;
mod scheme_ports;
mod template_ports;
use super::{Declarations, Error, Header, Headers, error, mapped_source, predicate_remap};
use crate::{
    resolve::model::DefId,
    sema::{declaration_inputs::Input, generics::Parameter},
    support::crates::generics::source as s,
    types::parameters::{
        ParameterId,
        clauses::Clause,
        environment::{Abilities, Environment},
    },
};
use std::sync::Arc;
pub(super) use template_ports::register as register_callable;

/// This is a transaction-local canonical correspondence, never an installed
/// descriptor, completed role inventory or proof of an original/call instance.
pub(super) struct Stage {
    graph: std::rc::Rc<mapped_source::Graph>,
    scopes: Vec<Scope>,
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
    Trait(predicate_remap::Prepared),
    Function(Arc<[Input]>),
    Implementation(Arc<[Input]>),
}
impl Inputs {
    fn all(&self) -> &[Input] {
        match self {
            Self::SelfType => &[],
            Self::Trait(prepared) => prepared.all(),
            Self::Function(inputs) | Self::Implementation(inputs) => inputs,
        }
    }
}
struct Active {
    owner: crate::support::crates::owner_catalog::DeclarationRef,
    parameters: usize,
    clauses: usize,
    environment: Arc<Environment>,
}
struct Walk<'a, 'b: 'a> {
    context: &'a mapped_source::Context<'b>,
    graph: &'a mapped_source::Graph,
    remap: predicate_remap::Context<'a>,
    active: Vec<Active>,
    abilities: Vec<(ParameterId, Abilities)>,
    clauses: Vec<Clause>,
}
impl Stage {
    pub(super) fn prepare(
        context: &mapped_source::Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
        let mut graph = mapped_source::Graph::prepare(context, facts, budget)?;
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
        let remap = predicate_remap::Context::new(
            context.imported,
            context.catalog,
            context.descriptor,
            &graph,
            budget,
        )?;
        let mut scopes = Vec::with_capacity(rows.scopes.len());
        let mut walk = Walk {
            context,
            graph: &graph,
            remap,
            active: Vec::with_capacity(rows.scopes.len()),
            abilities: Vec::with_capacity(rows.slots.len()),
            clauses: Vec::with_capacity(rows.clauses.len()),
        };
        for raw in &rows.scopes {
            scopes.push(walk.enter(facts, raw, budget)?);
        }
        budget.charge(1)?;
        let Walk { remap, .. } = walk;
        let applications = remap.finish(&mut facts.types, budget)?;
        graph.finish_applications(applications);
        finish_inputs(context, &mut graph, &scopes, budget)?;
        rebuilt_environments::complete(context, &graph, facts, &mut scopes, budget)?;
        budget.reserve(2 * size_of::<usize>())?;
        budget
            .observed
            .add(size_of::<mapped_source::Graph>() + 2 * size_of::<usize>());
        Ok(Self {
            graph: std::rc::Rc::new(graph),
            scopes,
        })
    }
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
            .saturating_add(rows.implementations.len())
            .saturating_mul(size_of::<Option<Arc<[Input]>>>()),
    )?;
    let mut traits = vec![None; rows.traits.len()];
    let mut implementations = vec![None; rows.implementations.len()];
    for (raw, scope) in rows.scopes.iter().zip(scopes) {
        budget.charge(1)?;
        let (slot, inputs) = match (rows.header(raw.owner).map_err(error)?, &scope.inputs) {
            (Some(s::HeaderRef::Trait(index)), Inputs::Trait(prepared)) => {
                (&mut traits[index as usize], prepared.all())
            }
            (Some(s::HeaderRef::Implementation(index)), Inputs::Implementation(inputs)) => {
                (&mut implementations[index as usize], inputs)
            }
            (None, Inputs::SelfType) | (Some(s::HeaderRef::Function(_)), Inputs::Function(_)) => {
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
    budget.observed.vector(&traits);
    budget.observed.vector(&implementations);
    graph.finish_inputs(traits, implementations).map_err(error)
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
            None if raw.parent.is_none() => {
                let declaration = rows
                    .declarations
                    .get(raw.owner.0 as usize)
                    .ok_or_else(|| error("source Self scope is outside its declaration table"))?;
                let parent = declaration
                    .definition
                    .parent
                    .ok_or_else(|| error("source Self scope has no actual lexical owner"))?;
                let Some(s::HeaderRef::Trait(index)) = rows.header(parent).map_err(error)? else {
                    return Err(error(
                        "source Self scope lacks its actual owning trait header",
                    ));
                };
                if declaration.definition.kind != s::DefinitionKind::SelfType
                    || rows.traits[index as usize].self_definition != raw.owner
                {
                    return Err(error("source scope is not the actual retained trait Self"));
                }
                Ok(Inputs::SelfType)
            }
            _ => Err(error(
                "source scope requires its wider actual header consumer",
            )),
        }
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
            self.clauses.truncate(closed.clauses);
        }
        let definition = context.definition(raw.owner, budget)?;
        let inputs = self.inputs(facts, raw, definition, budget)?;
        let previous_parameters = self.abilities.len();
        let previous_clauses = self.clauses.len();
        let parameters = formal_headers::parameters(context, self.graph, raw, budget)?;
        for parameter in &parameters {
            budget.charge(1)?;
            if let crate::types::arguments::Argument::Type(ty) = parameter.argument {
                let Some(crate::types::Kind::Parameter(id)) = facts.types.kind(ty) else {
                    return Err(error(
                        "source declared formal lost its actual rigid identity",
                    ));
                };
                self.abilities.push((id, Abilities::default()));
            }
        }
        for input in inputs.all() {
            budget.charge(1)?;
            self.clauses.push(input.value);
        }
        let environment = environments::prepare(
            facts,
            self.active.last().map(|a| &a.environment),
            &self.abilities,
            &self.clauses,
            previous_parameters,
            previous_clauses,
            budget,
        )?;
        let function = if let Inputs::Function(inputs) = &inputs {
            formal_headers::header(
                context,
                self.graph,
                facts,
                raw.owner,
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
