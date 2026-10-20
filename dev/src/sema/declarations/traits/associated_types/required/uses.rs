//! Canonical graph traversal preserves argument positions across nested binders.
use super::{DefId, Family, Use, charge};
use crate::types::{
    GenericArgs, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    outlives::{Projection, TypeComponents},
    parameters::{clauses::Clause, predicates::Bound},
    projection::Application,
    substitute,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Collector {
    seen: HashSet<(TypeId, u16)>,
    work: Vec<(TypeId, u16)>,
    applications: Vec<(Application, u16)>,
    pub steps: usize,
}
impl Collector {
    pub fn clause(&mut self, types: &Store, clause: Clause) -> Result<(), &'static str> {
        let (predicate, depth) = match clause {
            Clause::Predicate(p) => (p, 0),
            Clause::ForAll(id) => (
                types
                    .quantified(id)
                    .ok_or("invalid required-bound clause")?
                    .predicate,
                1,
            ),
        };
        if let Bound::Trait(id) = predicate.bound {
            self.arguments(
                types,
                types
                    .trait_application(id)
                    .ok_or("invalid required-bound trait")?
                    .arguments,
                depth,
            )?;
        } else if let Bound::ProjectionEqual(result) = predicate.bound {
            self.work.push((result, depth));
        }
        self.collect(types, predicate.ty, depth)
    }
    pub fn collect(&mut self, types: &Store, root: TypeId, depth: u16) -> Result<(), &'static str> {
        self.work.push((root, depth));
        while let Some((ty, depth)) = self.work.pop() {
            charge(&mut self.steps, 1)?;
            if types.contains_projections(ty) == Some(false) || !self.seen.insert((ty, depth)) {
                continue;
            }
            let list = match types.kind(ty).ok_or("invalid required-bound type")? {
                Kind::Projection(application) => {
                    self.applications.push((application, depth));
                    self.work.push((application.subject, depth));
                    let parent = types
                        .trait_application(application.trait_application)
                        .ok_or("invalid required-bound parent")?;
                    self.arguments(types, parent.arguments, depth)?;
                    self.arguments(types, application.arguments, depth)?;
                    None
                }
                Kind::Reference {
                    referent: child, ..
                }
                | Kind::Pointer { pointee: child, .. }
                | Kind::Slice(child)
                | Kind::Array { element: child, .. }
                | Kind::ArrayTerm { element: child, .. } => {
                    self.work.push((child, depth));
                    None
                }
                Kind::Closure {
                    signature,
                    environment,
                    arguments,
                    ..
                } => {
                    self.work.extend([(signature, depth), (environment, depth)]);
                    self.arguments(types, arguments, depth)?;
                    None
                }
                Kind::Tuple(list) => Some((list, depth)),
                Kind::Function {
                    parameters, result, ..
                } => {
                    let depth = depth
                        .checked_add(1)
                        .ok_or("required-bound binder depth exceeded")?;
                    self.work.push((result, depth));
                    Some((parameters, depth))
                }
                Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) => {
                    self.arguments(types, arguments, depth)?;
                    None
                }
                Kind::Dynamic { arguments, .. } => {
                    let nested = depth
                        .checked_add(1)
                        .ok_or("required-bound binder depth exceeded")?;
                    self.arguments(types, arguments, nested)?;
                    None
                }
                Kind::FunctionItem {
                    signature,
                    arguments,
                    ..
                } => {
                    self.work.push((signature, depth));
                    self.arguments(types, arguments, depth)?;
                    None
                }
                Kind::Parameter(_) | Kind::Primitive(_) | Kind::Never => None,
            };
            if let Some((list, depth)) = list {
                let children = types.list(list).ok_or("invalid required-bound type list")?;
                charge(&mut self.steps, children.len())?;
                self.work
                    .extend(children.iter().rev().map(|&ty| (ty, depth)));
            }
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        types: &Store,
        arguments: GenericArgs,
        depth: u16,
    ) -> Result<(), &'static str> {
        let arguments = types
            .arguments(arguments)
            .ok_or("invalid required-bound arguments")?;
        charge(&mut self.steps, arguments.len())?;
        self.work.extend(arguments.filter_map(|a| match a {
            Argument::Type(ty) => Some((ty, depth)),
            _ => None,
        }));
        Ok(())
    }
    pub fn finish(
        &mut self,
        types: &mut Store,
        families: &HashMap<DefId, Family>,
        owner: DefId,
        item: DefId,
    ) -> Result<Vec<Use>, &'static str> {
        let mut found =
            HashMap::<DefId, (HashSet<(TypeId, TypeId)>, HashSet<(RegionId, RegionId)>)>::new();
        let mut captures = TypeComponents::default();
        let mut lowering = HashMap::<u16, substitute::Bound>::new();
        for &(application, depth) in &self.applications {
            let target = DefId(application.item.0);
            let Some(family) = families
                .get(&target)
                .filter(|f| f.owner == owner && target != item)
            else {
                continue;
            };
            let formals: Vec<_> = types.projection_inputs(family.formal)?.collect();
            let actuals: Vec<_> = types.projection_inputs(application)?.collect();
            if formals.len() != actuals.len() {
                return Err("required-bound application arity mismatch");
            }
            charge(&mut self.steps, actuals.len())?;
            let (ty_uses, region_uses) = found.entry(target).or_default();
            for (formal, actual) in formals.into_iter().zip(actuals) {
                match (formal, actual) {
                    (Argument::Type(formal), Argument::Type(mut ty)) => {
                        if depth != 0 {
                            if captures.captures(types, Projection { ty, depth })? {
                                continue;
                            }
                            let fold = lowering
                                .entry(depth)
                                .or_insert_with(|| substitute::Bound::lower(depth));
                            let before = fold.steps;
                            ty = fold.apply(types, ty)?;
                            charge(&mut self.steps, fold.steps.saturating_sub(before))?;
                        }
                        ty_uses.insert((ty, formal));
                    }
                    (Argument::Lifetime(formal), Argument::Lifetime(mut region)) => {
                        if let Some((binder, slot)) = region.bound_parts() {
                            if binder < depth {
                                continue;
                            }
                            region = RegionId::bound(binder - depth, slot)?;
                        }
                        region_uses.insert((region, formal));
                    }
                    (Argument::Const(_), Argument::Const(_)) => {}
                    _ => return Err("required-bound application kind mismatch"),
                }
            }
        }
        charge(&mut self.steps, captures.steps())?;
        let mut result: Vec<_> = found
            .into_iter()
            .map(|(target, (types, regions))| {
                let mut types: Vec<_> = types.into_iter().collect();
                let mut regions: Vec<_> = regions.into_iter().collect();
                types.sort_unstable_by_key(|&(a, b)| (a.0, b.0));
                regions.sort_unstable_by_key(|&(a, b)| (a.0, b.0));
                Use {
                    target,
                    types,
                    regions,
                    contributed: HashSet::new(),
                }
            })
            .collect();
        result.sort_unstable_by_key(|u| u.target.0);
        Ok(result)
    }
}
