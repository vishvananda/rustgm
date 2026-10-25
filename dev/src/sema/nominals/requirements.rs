//! Instantiated nominal requirements retain the scope of surrounding callable binders.
use super::Declarations;
use crate::support::{
    hash::{HashMap, HashSet},
    retained::Growth,
};
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

/// One immutable caller environment; immediate contracts remain shared in the store.
#[derive(Default)]
pub(in crate::sema) struct Requirements {
    types: HashSet<(TypeId, u16)>,
    scopes: HashMap<u16, Scope>,
    fold_work: usize,
    /// Reusable traversal storage; empty between queries.
    work: Vec<(TypeId, u16)>,
    unique: HashSet<Clause>,
}
struct Scope {
    captures: Captures,
    lower: substitute::Bound,
    predicates: HashMap<Clause, Option<Clause>>,
}
impl Requirements {
    pub(in crate::sema) fn steps(&self) -> usize {
        self.types.len().saturating_add(self.fold_work)
    }

    /// Lift one retained type-outlives debt through its actual callable depth.
    /// Locally captured whole clauses defer under the same RFC 1214 policy.
    pub(in crate::sema) fn outlives(
        &mut self,
        store: &mut Store,
        projection: crate::types::outlives::Projection,
        shorter: crate::types::RegionId,
    ) -> Result<Option<Predicate>, &'static str> {
        let clause = Predicate {
            ty: projection.ty,
            bound: Bound::Outlives(shorter),
        }
        .into();
        match self.lift(store, clause, projection.depth)? {
            Some(Clause::Predicate(predicate)) => Ok(Some(predicate)),
            None => Ok(None),
            Some(Clause::ForAll(_)) => Err("outlives lifting changed the retained clause kind"),
        }
    }

    fn lift(
        &mut self,
        store: &mut Store,
        predicate: Clause,
        depth: u16,
    ) -> Result<Option<Clause>, &'static str> {
        self.lift_observed::<false>(store, predicate, depth, &|_, _| Ok(()))
    }
    fn lift_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        predicate: Clause,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Option<Clause>, &'static str> {
        if CHARGED {
            charge(1, 0)?;
        }
        if depth == 0 || predicate.escaping_binders(store)? == 0 {
            return Ok(Some(predicate));
        }
        if CHARGED {
            let growth = Growth::map::<u16, Scope>(
                self.scopes.len(),
                self.scopes.capacity(),
                usize::from(!self.scopes.contains_key(&depth)),
            );
            charge(growth.work, growth.bytes)?;
        }
        let scope = self.scopes.entry(depth).or_insert_with(|| Scope {
            captures: Captures::new(depth),
            lower: substitute::Bound::lower(depth),
            predicates: HashMap::default(),
        });
        if let Some(&result) = scope.predicates.get(&predicate) {
            return Ok(result);
        }
        let before = scope.captures.steps.saturating_add(scope.lower.steps);
        // RFC 1214 defers only predicates mentioning a surrounding callable local.
        // Independent predicates are lifted back into the caller's proof scope.
        let result: Result<Option<Clause>, &'static str> = (|| {
            let captured = if CHARGED {
                scope.captures.clause_charged(store, predicate, charge)?
            } else {
                scope.captures.clause(store, predicate)?
            };
            if captured {
                Ok(None)
            } else {
                Ok(Some(if CHARGED {
                    scope.lower.clause_charged(store, predicate, charge)?
                } else {
                    scope.lower.clause(store, predicate)?
                }))
            }
        })();
        let after = scope.captures.steps.saturating_add(scope.lower.steps);
        if CHARGED {
            charge(after.saturating_sub(before), 0)?;
        }
        let result = result?;
        self.fold_work = self.fold_work.saturating_add(after - before);
        if self.fold_work.saturating_add(self.types.len()) > 1_000_000 {
            return Err("nominal requirement fold work budget exceeded");
        }
        if CHARGED {
            let growth = Growth::map::<Clause, Option<Clause>>(
                scope.predicates.len(),
                scope.predicates.capacity(),
                1,
            );
            charge(growth.work, growth.bytes)?;
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
    requirements_inside(facts, root, 0, checked)
}

/// Same immutable formation cache; actual growth is paid before mutation.
pub(in crate::sema) fn requirements_counted(
    facts: &mut Declarations,
    root: TypeId,
    checked: &mut Requirements,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Vec<Clause>, &'static str> {
    requirements_inside_observed::<true>(facts, root, 0, checked, charge)
}

/// Stop at nominal argument edges. The caller retains each edge as a separate
/// formation proof in the same immutable query, including its callable depth.
pub(in crate::sema) fn immediate(
    facts: &mut Declarations,
    root: TypeId,
    depth: u16,
    checked: &mut Requirements,
    children: &mut Vec<(TypeId, u16)>,
) -> Result<Vec<Clause>, &'static str> {
    let mut work = vec![(root, depth)];
    let mut unique = HashSet::default();
    collect::<false>(
        facts,
        &mut work,
        &mut unique,
        checked,
        Some(children),
        &|_, _| Ok(()),
    )
}

/// Equality results and outlives subjects recurse beneath their whole-clause binder.
pub(super) fn requirements_inside(
    facts: &mut Declarations,
    root: TypeId,
    depth: u16,
    checked: &mut Requirements,
) -> Result<Vec<Clause>, &'static str> {
    requirements_inside_observed::<false>(facts, root, depth, checked, &|_, _| Ok(()))
}
fn requirements_inside_observed<const CHARGED: bool>(
    facts: &mut Declarations,
    root: TypeId,
    depth: u16,
    checked: &mut Requirements,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Vec<Clause>, &'static str> {
    if CHARGED {
        charge(1, 0)?;
    }
    // Only nominal instances contribute requirements, and the structural
    // property covers every edge followed below.
    if facts.types.contains_nominals(root) == Some(false) {
        return Ok(Vec::new());
    }
    let mut work = std::mem::take(&mut checked.work);
    let mut unique = std::mem::take(&mut checked.unique);
    work.clear();
    unique.clear();
    reserve::<_, CHARGED>(&work, 1, charge)?;
    work.push((root, depth));
    let result = collect::<CHARGED>(facts, &mut work, &mut unique, checked, None, charge);
    work.clear();
    unique.clear();
    checked.work = work;
    checked.unique = unique;
    result
}
fn collect<const CHARGED: bool>(
    facts: &mut Declarations,
    work: &mut Vec<(TypeId, u16)>,
    unique: &mut HashSet<Clause>,
    checked: &mut Requirements,
    mut immediate: Option<&mut Vec<(TypeId, u16)>>,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Vec<Clause>, &'static str> {
    let mut predicates = Vec::new();
    while let Some((ty, depth)) = work.pop() {
        if CHARGED {
            charge(1, 0)?;
            let growth = Growth::map::<(TypeId, u16), ()>(
                checked.types.len(),
                checked.types.capacity(),
                usize::from(!checked.types.contains(&(ty, depth))),
            );
            charge(growth.work, growth.bytes)?;
        }
        if immediate.is_some() {
            checked.fold_work = checked.fold_work.saturating_add(1);
            if checked.fold_work > 1_000_000 {
                return Err("immediate nominal formation work budget exceeded");
            }
        }
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
                    instance::<CHARGED>(facts, item, arguments, &mut immediate, charge)?;
                    if CHARGED {
                        let growth = Growth::map::<TypeId, std::sync::Arc<[Clause]>>(
                            facts.generics.nominal_requirements.len(),
                            facts.generics.nominal_requirements.capacity(),
                            1,
                        );
                        charge(
                            growth.work + immediate.len(),
                            growth
                                .bytes
                                .saturating_add(immediate.len().saturating_mul(size_of::<Clause>()))
                                .saturating_add(2 * size_of::<usize>()),
                        )?;
                    }
                    facts
                        .generics
                        .nominal_requirements
                        .insert(ty, immediate.into());
                }
                let cached = std::sync::Arc::clone(&facts.generics.nominal_requirements[&ty]);
                for &predicate in cached.iter() {
                    if let Some(predicate) = checked.lift_observed::<CHARGED>(
                        &mut facts.types,
                        predicate,
                        depth,
                        charge,
                    )? {
                        if CHARGED {
                            let growth = Growth::map::<Clause, ()>(
                                unique.len(),
                                unique.capacity(),
                                usize::from(!unique.contains(&predicate)),
                            );
                            charge(1 + growth.work, growth.bytes)?;
                        }
                        if unique.insert(predicate) {
                            reserve::<_, CHARGED>(&predicates, 1, charge)?;
                            predicates.push(predicate);
                        }
                    }
                }
                nominal_arguments::<CHARGED>(
                    facts,
                    arguments,
                    depth,
                    checked,
                    work,
                    immediate.as_deref_mut(),
                    charge,
                )?;
            }
            Kind::Tuple(list) => {
                reserve::<_, CHARGED>(
                    work,
                    facts
                        .types
                        .list(list)
                        .ok_or("invalid nominal requirement tuple")?
                        .len(),
                    charge,
                )?;
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
            | Kind::Slice(element) => {
                reserve::<_, CHARGED>(work, 1, charge)?;
                work.push((element, depth));
            }
            Kind::Reference { referent, .. } => {
                reserve::<_, CHARGED>(work, 1, charge)?;
                work.push((referent, depth));
            }
            Kind::Pointer { pointee, .. } => {
                reserve::<_, CHARGED>(work, 1, charge)?;
                work.push((pointee, depth));
            }
            Kind::Function {
                parameters, result, ..
            } => {
                let depth = depth
                    .checked_add(1)
                    .ok_or("nominal requirement binder depth overflow")?;
                reserve::<_, CHARGED>(
                    work,
                    facts
                        .types
                        .list(parameters)
                        .ok_or("invalid nominal requirement signature")?
                        .len()
                        .saturating_add(1),
                    charge,
                )?;
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
            Kind::Closure {
                environment,
                arguments,
                ..
            } => {
                reserve::<_, CHARGED>(work, 1, charge)?;
                work.push((environment, depth));
                arguments_work::<CHARGED>(facts, arguments, depth, work, charge)?;
            }
            Kind::FunctionItem { arguments, .. } => {
                // The declaration already proved its signature, including local
                // late binders. Item-use obligations validate its substitutions;
                // replaying the template signature would capture caller lifetimes.
                arguments_work::<CHARGED>(facts, arguments, depth, work, charge)?;
            }
            _ => {}
        }
    }
    Ok(predicates)
}
fn nominal_arguments<const CHARGED: bool>(
    facts: &Declarations,
    arguments: GenericArgs,
    depth: u16,
    checked: &mut Requirements,
    work: &mut Vec<(TypeId, u16)>,
    immediate: Option<&mut Vec<(TypeId, u16)>>,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let Some(children) = immediate else {
        return arguments_work::<CHARGED>(facts, arguments, depth, work, charge);
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("missing immediate nominal arguments")?;
    reserve::<_, CHARGED>(children, arguments.len(), charge)?;
    if CHARGED {
        charge(arguments.len(), 0)?;
    }
    checked.fold_work = checked.fold_work.saturating_add(arguments.len());
    for argument in arguments {
        if let Argument::Type(ty) = argument
            && facts.types.contains_nominals(ty) != Some(false)
        {
            children.push((ty, depth));
        }
    }
    Ok(())
}

fn arguments_work<const CHARGED: bool>(
    facts: &Declarations,
    arguments: GenericArgs,
    depth: u16,
    work: &mut Vec<(TypeId, u16)>,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("missing nominal requirement arguments")?;
    if work.len().saturating_add(arguments.len()) > 1_000_000 {
        return Err("nominal requirement pending work budget exceeded");
    }
    reserve::<_, CHARGED>(work, arguments.len(), charge)?;
    if CHARGED {
        charge(arguments.len(), 0)?;
    }
    for argument in arguments {
        if let Argument::Type(ty) = argument {
            work.push((ty, depth));
        }
    }
    Ok(())
}
fn instance<const CHARGED: bool>(
    facts: &mut Declarations,
    item: ItemId,
    arguments: GenericArgs,
    output: &mut Vec<Clause>,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    if CHARGED {
        charge(1, 0)?;
    }
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
                    reserve::<_, CHARGED>(output, 1, charge)?;
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
        let predicate = if CHARGED {
            facts.generics.substitute_clause_charged(
                &mut facts.types,
                owner,
                arguments,
                predicate,
                charge,
            )?
        } else {
            facts
                .generics
                .substitute_clause(&mut facts.types, owner, arguments, predicate)?
        };
        reserve::<_, CHARGED>(output, 1, charge)?;
        output.push(predicate);
    }
    Ok(())
}

fn reserve<T, const CHARGED: bool>(
    values: &Vec<T>,
    additional: usize,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    if CHARGED {
        let growth = Growth::vector::<T>(values.len(), values.capacity(), additional);
        charge(growth.work, growth.bytes)?;
    }
    Ok(())
}
