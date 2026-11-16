//! rustc's `identify_constrained_generic_params` for a nominal declaration:
//! a parameter that variance leaves bivariant is still used when a
//! projection-equality clause of the declaration constrains it. alloc's
//! `struct DedupSortedIter<K, V, I> where I: Iterator<Item = (K, V)>` stores
//! only `I`, and the clause makes `K` and `V` follow from it.
use super::{Argument, ConstId, ConstKind, GenericArgs, ItemId, Kind, RegionId, Store, TypeId};
use crate::support::hash::HashSet;

/// Work bound shared with the variance solver.
const LIMIT: usize = 1_000_000;

/// Extend `constrained`, one flag per declaration slot of `owner`, to a
/// fixpoint. A clause `<P as Trait<..>>::Item == T` whose projection mentions
/// only constrained slots, anywhere, constrains every slot that `T` mentions
/// outside a projection, opaque type or future.
/// # Errors
/// Rejects invalid type identities and exhausted work.
pub(crate) fn constrain(
    types: &Store,
    owner: ItemId,
    constrained: &mut [bool],
    clauses: &[(TypeId, TypeId)],
) -> Result<(), &'static str> {
    let mut walk = Walk {
        types,
        owner,
        steps: 0,
    };
    let mut pending: Vec<_> = clauses.to_vec();
    let mut changed = true;
    while changed {
        changed = false;
        let mut index = 0;
        while index < pending.len() {
            let (projection, result) = pending[index];
            let inputs = walk.slots(projection, true)?;
            if inputs
                .iter()
                .all(|&slot| constrained.get(slot).copied().unwrap_or(false))
            {
                for slot in walk.slots(result, false)? {
                    *constrained
                        .get_mut(slot)
                        .ok_or("constrained parameter outside its declaration")? = true;
                }
                pending.swap_remove(index);
                changed = true;
            } else {
                index += 1;
            }
        }
    }
    Ok(())
}

struct Walk<'a> {
    types: &'a Store,
    owner: ItemId,
    steps: usize,
}
impl Walk<'_> {
    fn charge(&mut self) -> Result<(), &'static str> {
        self.steps += 1;
        if self.steps > LIMIT {
            return Err("constrained parameter work budget exceeded");
        }
        Ok(())
    }
    /// The declaration slots `ty` mentions, all of them when `everywhere`,
    /// otherwise only those outside projections, opaque types and futures.
    fn slots(&mut self, ty: TypeId, everywhere: bool) -> Result<Vec<usize>, &'static str> {
        let mut found = Vec::new();
        let mut seen = HashSet::default();
        let mut work = vec![(ty, 0_u16)];
        while let Some((ty, depth)) = work.pop() {
            self.charge()?;
            if !seen.insert((ty, depth)) {
                continue;
            }
            let nested = || {
                depth
                    .checked_add(1)
                    .ok_or("parameter binder depth overflow")
            };
            match self
                .types
                .kind(ty)
                .ok_or("invalid constrained parameter type")?
            {
                Kind::Parameter(id) => {
                    let parameter = self
                        .types
                        .parameter(id)
                        .ok_or("missing constrained parameter")?;
                    if parameter.owner == self.owner {
                        found.push(parameter.index as usize);
                    }
                }
                Kind::Reference {
                    region, referent, ..
                } => {
                    Self::region(region, depth, &mut found);
                    work.push((referent, depth));
                }
                Kind::Pattern { base, .. } => work.push((base, depth)),
                Kind::Pointer { pointee, .. } => work.push((pointee, depth)),
                Kind::ArrayTerm { element, length } => {
                    self.constant(length, &mut found)?;
                    work.push((element, depth));
                }
                Kind::Array { element, .. } | Kind::Slice(element) => work.push((element, depth)),
                Kind::Tuple(fields) => {
                    for field in self.types.list(fields).ok_or("invalid constrained tuple")? {
                        work.push((*field, depth));
                    }
                }
                Kind::Function {
                    parameters, result, ..
                } => {
                    let depth = nested()?;
                    for parameter in self
                        .types
                        .list(parameters)
                        .ok_or("invalid constrained signature")?
                    {
                        work.push((*parameter, depth));
                    }
                    work.push((result, depth));
                }
                Kind::FunctionItem { signature, .. } => work.push((signature, depth)),
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => {
                    self.arguments(arguments, depth, &mut work, &mut found)?;
                    work.push((signature, depth));
                    work.push((environment, depth));
                }
                Kind::Adt(_, arguments) => {
                    self.arguments(arguments, depth, &mut work, &mut found)?;
                }
                Kind::Dynamic {
                    arguments, region, ..
                } => {
                    Self::region(region, depth, &mut found);
                    self.arguments(arguments, nested()?, &mut work, &mut found)?;
                }
                Kind::Opaque(_, arguments) | Kind::Future { arguments, .. } => {
                    if everywhere {
                        self.arguments(arguments, depth, &mut work, &mut found)?;
                    }
                }
                Kind::Projection(application) => {
                    if everywhere {
                        work.push((application.subject, depth));
                        let trait_ = self
                            .types
                            .trait_application(application.trait_application)
                            .ok_or("invalid constrained projection trait")?;
                        for arguments in [trait_.arguments, application.arguments] {
                            self.arguments(arguments, depth, &mut work, &mut found)?;
                        }
                    }
                }
                Kind::Never | Kind::Foreign(_) | Kind::Primitive(_) => {}
            }
        }
        Ok(found)
    }
    /// A lifetime of the declaration itself; lifetimes bound inside the type
    /// and `'static` are no declaration slot.
    fn region(region: RegionId, depth: u16, found: &mut Vec<usize>) {
        if let Some((binder, index)) = region.bound_parts()
            && binder == depth
        {
            found.push(usize::from(index));
        }
    }
    fn constant(&mut self, id: ConstId, found: &mut Vec<usize>) -> Result<(), &'static str> {
        self.charge()?;
        if let ConstKind::Parameter(id) = self
            .types
            .const_kind(id)
            .ok_or("invalid constrained constant")?
        {
            let parameter = self
                .types
                .parameter(id)
                .ok_or("missing constrained const parameter")?;
            if parameter.owner == self.owner {
                found.push(parameter.index as usize);
            }
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        arguments: GenericArgs,
        depth: u16,
        work: &mut Vec<(TypeId, u16)>,
        found: &mut Vec<usize>,
    ) -> Result<(), &'static str> {
        for argument in self
            .types
            .arguments(arguments)
            .ok_or("invalid constrained arguments")?
        {
            self.charge()?;
            match argument {
                Argument::Type(ty) => work.push((ty, depth)),
                Argument::Lifetime(region) => Self::region(region, depth, found),
                Argument::Const(id) => self.constant(id, found)?,
            }
        }
        Ok(())
    }
}
