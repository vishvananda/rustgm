//! Instantiated nominal requirements retain the scope of surrounding callable binders.
use super::Declarations;
use crate::{
    resolve::model::DefId,
    types::{
        GenericArgs, ItemId, Kind, Store, TypeId,
        arguments::Argument,
        bound_uses::captures::Captures,
        parameters::{
            ParameterKind,
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        substitute,
    },
};
use std::collections::{HashMap, HashSet};

/// One immutable caller environment; immediate contracts remain shared in the store.
#[derive(Default)]
pub(in crate::sema) struct Requirements {
    types: HashSet<(TypeId, u16)>,
    scopes: HashMap<u16, Scope>,
    fold_work: usize,
}
struct Scope {
    captures: Captures,
    lower: substitute::Bound,
    predicates: HashMap<Clause, Option<Clause>>,
}
impl Requirements {
    #[cfg(test)]
    pub(in crate::sema) fn steps(&self) -> usize {
        self.types.len().saturating_add(self.fold_work)
    }

    fn lift(
        &mut self,
        store: &mut Store,
        predicate: Clause,
        depth: u16,
    ) -> Result<Option<Clause>, &'static str> {
        if depth == 0 || predicate.escaping_binders(store)? == 0 {
            return Ok(Some(predicate));
        }
        let scope = self.scopes.entry(depth).or_insert_with(|| Scope {
            captures: Captures::new(depth),
            lower: substitute::Bound::lower(depth),
            predicates: HashMap::new(),
        });
        if let Some(&result) = scope.predicates.get(&predicate) {
            return Ok(result);
        }
        let before = scope.captures.steps.saturating_add(scope.lower.steps);
        // RFC 1214 defers only predicates mentioning a surrounding callable local.
        // Independent predicates are lifted back into the caller's proof scope.
        let result = if scope.captures.clause(store, predicate)? {
            None
        } else {
            Some(scope.lower.clause(store, predicate)?)
        };
        let after = scope.captures.steps.saturating_add(scope.lower.steps);
        self.fold_work = self.fold_work.saturating_add(after - before);
        if self.fold_work.saturating_add(self.types.len()) > 1_000_000 {
            return Err("nominal requirement fold work budget exceeded");
        }
        scope.predicates.insert(predicate, result);
        Ok(result)
    }
}

/// `checked` belongs to one immutable caller environment. Cache only immediate
/// instance predicates; flattening every structural root would retain quadratic prefixes.
pub(in crate::sema) fn requirements(
    facts: &mut Declarations,
    root: TypeId,
    checked: &mut Requirements,
) -> Result<Vec<Clause>, &'static str> {
    let mut work = vec![(root, 0_u16)];
    let mut predicates = Vec::new();
    let mut unique = HashSet::new();
    while let Some((ty, depth)) = work.pop() {
        if !checked.types.insert((ty, depth)) {
            continue;
        }
        if checked
            .types
            .len()
            .saturating_add(checked.fold_work)
            .saturating_add(work.len())
            .saturating_add(predicates.len())
            > 1_000_000
        {
            return Err("nominal requirement work budget exceeded");
        }
        match facts
            .types
            .kind(ty)
            .ok_or("invalid nominal requirement type")?
        {
            Kind::Adt(item, arguments) => {
                if !facts.generics.nominal_requirements.contains_key(&ty) {
                    let mut immediate = Vec::new();
                    instance(facts, item, arguments, &mut immediate)?;
                    facts
                        .generics
                        .nominal_requirements
                        .insert(ty, immediate.into());
                }
                let immediate = std::sync::Arc::clone(&facts.generics.nominal_requirements[&ty]);
                for &predicate in immediate.iter() {
                    if let Some(predicate) = checked.lift(&mut facts.types, predicate, depth)?
                        && unique.insert(predicate)
                    {
                        predicates.push(predicate);
                    }
                }
                arguments_work(facts, arguments, depth, &mut work)?;
            }
            Kind::Tuple(list) => {
                work.extend(
                    facts
                        .types
                        .list(list)
                        .ok_or("invalid nominal requirement tuple")?
                        .iter()
                        .map(|&ty| (ty, depth)),
                );
            }
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => work.push((element, depth)),
            Kind::Reference { referent, .. } => work.push((referent, depth)),
            Kind::Pointer { pointee, .. } => work.push((pointee, depth)),
            Kind::Function {
                parameters, result, ..
            } => {
                let depth = depth
                    .checked_add(1)
                    .ok_or("nominal requirement binder depth overflow")?;
                work.push((result, depth));
                work.extend(
                    facts
                        .types
                        .list(parameters)
                        .ok_or("invalid nominal requirement signature")?
                        .iter()
                        .map(|&ty| (ty, depth)),
                );
            }
            Kind::FunctionItem { arguments, .. } => {
                // The declaration already proved its signature, including local
                // late binders. Item-use obligations validate its substitutions;
                // replaying the template signature would capture caller lifetimes.
                arguments_work(facts, arguments, depth, &mut work)?;
            }
            _ => {}
        }
    }
    Ok(predicates)
}
fn arguments_work(
    facts: &Declarations,
    arguments: GenericArgs,
    depth: u16,
    work: &mut Vec<(TypeId, u16)>,
) -> Result<(), &'static str> {
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("missing nominal requirement arguments")?;
    if work.len().saturating_add(arguments.len()) > 1_000_000 {
        return Err("nominal requirement pending work budget exceeded");
    }
    for argument in arguments {
        if let Argument::Type(ty) = argument {
            work.push((ty, depth));
        }
    }
    Ok(())
}
fn instance(
    facts: &mut Declarations,
    item: ItemId,
    arguments: GenericArgs,
    output: &mut Vec<Clause>,
) -> Result<(), &'static str> {
    let owner = DefId(item.0);
    if !facts.generics.nominal_contracts.contains(&owner) {
        return Err("nominal declaration requirements are not complete");
    }
    if let Some(parameters) = facts.generics.parameters.get(&owner) {
        let actual = facts
            .types
            .arguments(arguments)
            .ok_or("invalid nominal requirement arguments")?;
        if actual.len() != parameters.len() {
            return Err("nominal requirement argument arity mismatch");
        }
        for (parameter, argument) in parameters.iter().zip(actual) {
            if let Argument::Type(template) = parameter.argument {
                let Some(Kind::Parameter(id)) = facts.types.kind(template) else {
                    return Err("invalid nominal type parameter");
                };
                let Argument::Type(ty) = argument else {
                    return Err("nominal requirement argument kind mismatch");
                };
                if matches!(
                    facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                ) {
                    output.push(
                        Predicate {
                            ty,
                            bound: Bound::Sized,
                        }
                        .into(),
                    );
                }
            }
        }
    }
    let predicates = facts
        .generics
        .predicates
        .get(&owner)
        .map_or(&[][..], Vec::as_slice);
    let count = predicates.len();
    for index in 0..count {
        let predicate = facts.generics.predicates[&owner][index].value;
        if matches!(
            predicate,
            Clause::Predicate(Predicate {
                bound: Bound::Outlives(_),
                ..
            })
        ) {
            continue;
        }
        let predicate =
            facts
                .generics
                .substitute_clause(&mut facts.types, owner, arguments, predicate)?;
        output.push(predicate);
    }
    Ok(())
}
