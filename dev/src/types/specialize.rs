//! Specialize callable binders for type-hole shape inference, preserving item identity.
use super::{Kind, RegionId, Store, TypeId, substitute::Bound};
const LIMIT: usize = 1_000_000;
enum Work {
    Enter(TypeId),
    Finish(Kind, usize),
}
pub struct Specialized {
    pub ty: TypeId,
    pub regions: u32,
    pub steps: usize,
}
/// Replace late binders with distinct inference arguments per structural position.
/// Captured body universals must already use unbound aliases. The original source
/// type remains the subject of the separate, directed subsumption proof.
///
/// # Errors
/// Rejects escaping binders, invalid shapes, exhausted identities and traversal budgets.
pub fn callables(store: &mut Store, root: TypeId, first: u32) -> Result<Specialized, &'static str> {
    if store.escaping_binders(root) != Some(0) {
        return Err("callable specialization has escaping body lifetimes");
    }
    let mut work = vec![Work::Enter(root)];
    let mut values = Vec::new();
    let mut steps = 0_usize;
    let mut next = first;
    while let Some(task) = work.pop() {
        steps += 1;
        if steps.saturating_add(work.len()) > LIMIT {
            return Err("callable specialization work budget exceeded");
        }
        match task {
            Work::Enter(mut ty) => {
                if !store
                    .contains_late_callables(ty)
                    .ok_or("invalid callable specialization type")?
                {
                    values.push(ty);
                    continue;
                }
                if let Some(Kind::Function { bound_regions, .. }) = store.kind(ty)
                    && bound_regions != 0
                {
                    ty = instantiate(store, ty, bound_regions, &mut next, &mut steps)?;
                }
                let kind = store.kind(ty).ok_or("missing specialized type")?;
                let count = arity(store, kind)?;
                if steps.saturating_add(work.len()).saturating_add(count + 1) > LIMIT {
                    return Err("callable specialization work budget exceeded");
                }
                work.push(Work::Finish(kind, count));
                match kind {
                    Kind::Function {
                        parameters, result, ..
                    } => {
                        work.push(Work::Enter(result));
                        work.extend(
                            store
                                .list(parameters)
                                .ok_or("missing callable parameters")?
                                .iter()
                                .rev()
                                .copied()
                                .map(Work::Enter),
                        );
                    }
                    Kind::Tuple(list) => work.extend(
                        store
                            .list(list)
                            .ok_or("missing specialized tuple")?
                            .iter()
                            .rev()
                            .copied()
                            .map(Work::Enter),
                    ),
                    Kind::Reference { referent: ty, .. }
                    | Kind::Pointer { pointee: ty, .. }
                    | Kind::Array { element: ty, .. }
                    | Kind::Slice(ty) => work.push(Work::Enter(ty)),
                    _ => return Err("unexpected specialized callable shape"),
                }
            }
            Work::Finish(kind, count) => {
                let base = values
                    .len()
                    .checked_sub(count)
                    .ok_or("specialization stack underflow")?;
                let kind = rebuild(store, kind, &values[base..])?;
                let ty = store.intern(kind)?;
                values.truncate(base);
                values.push(ty);
            }
        }
    }
    let ty = values
        .pop()
        .ok_or("missing callable specialization result")?;
    if !values.is_empty() {
        return Err("unbalanced callable specialization stack");
    }
    Ok(Specialized {
        ty,
        regions: next - first,
        steps,
    })
}
fn instantiate(
    store: &mut Store,
    ty: TypeId,
    count: u32,
    next: &mut u32,
    steps: &mut usize,
) -> Result<TypeId, &'static str> {
    let end = next
        .checked_add(count)
        .filter(|end| *end < 0x8000_0000)
        .ok_or("callable specialization region overflow")?;
    if *next == 0 || count as usize > LIMIT {
        return Err("invalid callable specialization region range");
    }
    let arguments = (*next..end).map(RegionId).collect();
    *next = end;
    let mut substitution = Bound::new(arguments);
    let ty = substitution.signature(store, ty, 0)?;
    *steps = steps
        .saturating_add(substitution.steps)
        .saturating_add(count as usize);
    if *steps > LIMIT {
        return Err("callable specialization work budget exceeded");
    }
    Ok(ty)
}
fn arity(store: &Store, kind: Kind) -> Result<usize, &'static str> {
    match kind {
        Kind::Function { parameters, .. } => Ok(store
            .list(parameters)
            .ok_or("missing specialization parameters")?
            .len()
            + 1),
        Kind::Tuple(list) => Ok(store
            .list(list)
            .ok_or("missing specialization tuple")?
            .len()),
        Kind::Reference { .. } | Kind::Pointer { .. } | Kind::Array { .. } | Kind::Slice(_) => {
            Ok(1)
        }
        _ => Err("invalid specialization constructor"),
    }
}
fn rebuild(store: &mut Store, kind: Kind, children: &[TypeId]) -> Result<Kind, &'static str> {
    let first = || {
        children
            .first()
            .copied()
            .ok_or("missing specialization child")
    };
    Ok(match kind {
        Kind::Tuple(_) => Kind::Tuple(store.intern_list(children)?),
        Kind::Array { length, .. } => Kind::Array {
            length,
            element: first()?,
        },
        Kind::Slice(_) => Kind::Slice(first()?),
        Kind::Reference {
            region, mutable, ..
        } => Kind::Reference {
            region,
            mutable,
            referent: first()?,
        },
        Kind::Pointer { mutable, .. } => Kind::Pointer {
            mutable,
            pointee: first()?,
        },
        Kind::Function {
            bound_regions,
            unsafe_,
            ..
        } => {
            let (result, parameters) = children
                .split_last()
                .ok_or("missing specialization result")?;
            Kind::Function {
                bound_regions,
                unsafe_,
                result: *result,
                parameters: store.intern_list(parameters)?,
            }
        }
        _ => return Err("invalid specialization rebuild"),
    })
}
