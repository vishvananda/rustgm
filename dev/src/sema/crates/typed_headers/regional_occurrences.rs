//! Prepay the existing pointer occurrence kernel from selected real dependencies.
use super::{Cursor, Error, Kind, ReservationWork, Store, error, retained, s};
use crate::{support::crates::generics as g, types::TypeId};

pub(super) fn prepare(
    rows: s::ValidatedView<'_>,
    cursor: &Cursor,
    store: &Store,
    signature: g::Signature,
    stamp: u32,
    selected: &mut [Option<u32>],
    work: &mut ReservationWork,
) -> Result<usize, Error> {
    let roots = signature.parameters.slice(&rows.roots).map_err(error)?;
    work.charge(roots.len().saturating_add(1))?;
    let mut nodes = 0_usize;
    for root in roots
        .iter()
        .copied()
        .chain(std::iter::once(signature.result))
    {
        let mut current = root;
        loop {
            work.charge(1)?;
            let ty = cursor.child(current)?;
            if store.escaping_binders(ty) == Some(0) {
                break;
            }
            let cell = selected
                .get_mut(current.0 as usize)
                .ok_or_else(|| error("private callable occurrence lost its actual recipe row"))?;
            if *cell == Some(stamp) {
                break;
            }
            let row = rows.types.get(current.0 as usize).ok_or_else(|| {
                error("private callable occurrence is outside its actual recipe graph")
            })?;
            let s::Type::Reference { referent, .. } = *row else {
                return Err(error(
                    "private callable contextual occurrence requires its later profile",
                ));
            };
            let Some(Kind::Reference {
                referent: actual, ..
            }) = store.kind(ty)
            else {
                return Err(error(
                    "private callable occurrence changed its canonical reference kind",
                ));
            };
            if cursor.child(referent)? != actual {
                return Err(error(
                    "private callable occurrence changed its actual canonical child",
                ));
            }
            *cell = Some(stamp);
            nodes = nodes.saturating_add(1);
            current = referent;
        }
    }
    // The existing Uses collector pushes each selected reference and its child;
    // shared nodes are visited once per input/output collector. Both collectors
    // and the pointer's immediate fields are charged before the kernel runs.
    work.charge(
        nodes
            .saturating_mul(4)
            .saturating_add(roots.len())
            .saturating_add(1),
    )?;
    if nodes == 0 {
        return Ok(0);
    }
    let visits = nodes
        .saturating_mul(2)
        .saturating_add(roots.len())
        .saturating_add(1);
    Ok(retained::new_map::<(TypeId, u16), ()>(visits)
        .saturating_add(retained::new_map::<u16, ()>(nodes))
        .saturating_add(visits.saturating_mul(4 * size_of::<(TypeId, u16)>()))
        .saturating_mul(2))
}
