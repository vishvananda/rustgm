//! One structural target identifies the actual defining impl without granting its body proof.
use super::{
    graph_pairs,
    header_conditions::{self, Conditions},
    registry_graph::{Relation, State},
    retention_work::Budget,
};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{GenericArgs, arguments::Argument},
};
pub(in crate::sema) struct SelectedMethod {
    definition: DefId,
    declaration: DefId,
    implementation: Option<(DefId, GenericArgs)>,
    inherited: Option<Budget>,
}
impl SelectedMethod {
    pub(in crate::sema) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(in crate::sema) const fn implementation(&self) -> Option<(DefId, GenericArgs)> {
        self.implementation
    }
    pub(in crate::sema) fn projection_conditions(
        &self,
        facts: &mut Declarations,
    ) -> Result<Option<Conditions>, &'static str> {
        let Some(budget) = &self.inherited else {
            return Ok(None);
        };
        self.current(facts)?;
        let (owner, arguments) = self
            .implementation
            .ok_or("inherited member has no actual defining implementation")?;
        header_conditions::defining(facts, owner, arguments, budget).map(Some)
    }
    pub(in crate::sema) fn current(&self, facts: &Declarations) -> Result<(), &'static str> {
        let Some(budget) = &self.inherited else {
            return Ok(());
        };
        let registry = graph_pairs::registry(facts)?;
        let graph = registry
            .specialization_graph()
            .ok_or("inherited member lost its graph owner")?;
        graph.current(facts)?;
        if !matches!(graph.state, State::Sealed(_)) || !budget.same_owner(&graph.budget) {
            return Err("inherited member proof belongs to another or unfinished graph");
        }
        let (owner, _) = self
            .implementation
            .ok_or("inherited member has no actual defining implementation")?;
        let State::Sealed(forest) = &graph.state else {
            return Err("inherited member lost its graph seal");
        };
        super::member_paths::checked(forest, registry, registry.entry_key(owner)?)?
            .ok_or("inherited member lost its defining source witness")?;
        if registry.direct_method(registry.entry_key(owner)?, self.declaration)?
            != Some(self.definition)
        {
            return Err("inherited member defining declaration changed");
        }
        Ok(())
    }
    pub(in crate::sema) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.inherited
            .as_ref()
            .ok_or("ordinary member has no inherited proof budget")?
            .charge(steps, bytes)
    }
}
pub(in crate::sema) fn resolve(
    facts: &mut Declarations,
    owner: DefId,
    arguments: GenericArgs,
    declaration: DefId,
) -> Result<SelectedMethod, &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return ordinary(registry, owner, arguments, declaration);
    };
    graph.current(facts)?;
    let key = registry.entry_key(owner)?;
    let forest = match &graph.state {
        State::Sealed(forest) => forest,
        State::Building(builder) if !builder.sources.contains_key(&key) => {
            return ordinary(registry, owner, arguments, declaration);
        }
        State::Conforming(forest) if !forest.by_entry.contains_key(&key) => {
            return ordinary(registry, owner, arguments, declaration);
        }
        _ => return Err("inherited member selection requires sealed specialization conformance"),
    };
    if !forest.by_entry.contains_key(&key) {
        return ordinary(registry, owner, arguments, declaration);
    }
    let budget = graph.budget.alias()?;
    let header = registry.checked_entry(key)?;
    let (application, _) = header
        .predicate
        .bound
        .trait_parts()
        .ok_or("selected method owner has no actual trait application")?;
    let trait_owner = facts
        .types
        .trait_application(application)
        .ok_or("selected method trait application disappeared")?
        .item;
    if facts.associated.trait_owner(declaration) != Some(DefId(trait_owner.0)) {
        return Err("requested method is not a declaration of the selected trait");
    }
    // Validate the complete source schema even when the child supplies its own
    // body. This does not collect or prove any unused ancestor conditions.
    header_conditions::mapping(facts, owner, arguments, &budget)?
        .fold
        .pay()?;
    let mut current = key;
    let mut arguments = arguments;
    loop {
        budget.charge(1, 0)?;
        let registry = graph_pairs::registry(facts)?;
        let graph = registry
            .specialization_graph()
            .ok_or("specialization graph disappeared during member selection")?;
        graph.current(facts)?;
        let State::Sealed(forest) = &graph.state else {
            return Err("specialization member selection lost its seal");
        };
        let vertex = super::member_paths::checked(forest, registry, current)?
            .ok_or("inherited member lost its actual vertex")?;
        if let Some(definition) = registry.direct_method(current, declaration)? {
            return Ok(SelectedMethod {
                definition,
                declaration,
                implementation: Some((current.owner(), arguments)),
                inherited: if current == key { None } else { Some(budget) },
            });
        }
        let Some(edge) = vertex.parent else {
            // This is the trait's own provided body, not a defining impl body.
            return Ok(SelectedMethod {
                definition: declaration,
                declaration,
                implementation: None,
                inherited: None,
            });
        };
        let relation = *forest
            .parents
            .get(edge.index())
            .ok_or("inherited member parent edge disappeared")?;
        let parent = super::member_paths::checked(forest, registry, relation.parent)?
            .ok_or("inherited member parent vertex disappeared")?;
        if relation.child != current || parent.depth.checked_add(1) != Some(vertex.depth) {
            return Err("inherited member parent rank or identity changed");
        }
        arguments = translate(facts, relation, arguments, &budget)?;
        current = relation.parent;
    }
}
fn ordinary(
    registry: &crate::sema::traits::implementations::Implementations,
    owner: DefId,
    arguments: GenericArgs,
    declaration: DefId,
) -> Result<SelectedMethod, &'static str> {
    let definition = registry
        .method(owner, declaration)
        .ok_or("selected implementation lacks the actual trait member")?;
    Ok(SelectedMethod {
        definition,
        declaration,
        implementation: (definition != declaration).then_some((owner, arguments)),
        inherited: None,
    })
}
fn translate(
    facts: &mut Declarations,
    relation: Relation,
    arguments: GenericArgs,
    budget: &Budget,
) -> Result<GenericArgs, &'static str> {
    let mut mapping = header_conditions::mapping(facts, relation.child.owner(), arguments, budget)?;
    let source = facts
        .types
        .arguments(relation.parent_arguments)
        .ok_or("inherited member translation inputs disappeared")?;
    budget.charge(
        source.len(),
        source.len().saturating_mul(size_of::<Argument>()),
    )?;
    let mut values: Vec<_> = source.collect();
    for argument in &mut values {
        *argument =
            mapping
                .fold
                .apply_argument_charged(&mut facts.types, *argument, &|steps, bytes| {
                    budget.charge(steps, bytes)
                })?;
    }
    mapping.fold.pay()?;
    facts.types.intern_arguments(&values)
}
