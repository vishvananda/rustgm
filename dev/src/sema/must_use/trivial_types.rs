//! Unit and visible uninhabited domains suppress discard diagnostics, never proofs.
use super::{Contracts, Declarations, Report};
use crate::{
    resolve::package::ModuleId,
    sema::patterns::coverage::{self, Cache},
    types::{Kind, TypeId},
};
impl Declarations {
    pub(in crate::sema) fn must_use_trivial(&mut self, report: &Report, module: ModuleId,
        root: TypeId, cache: &mut Cache) -> Result<bool, &'static str> {
        if let Some(result) = immediate(&self.types, root)? { return Ok(result) }
        let table = self.must_use_contracts.get_or_insert_default();
        charge(table, 1, 0)?;
        if let Some(&result) = table.trivial.get(&(module, root)) { return result }
        let before = self.aggregates.instantiation_steps();
        let prepared = self.aggregates.prepare(&mut self.types, root);
        let work = self.aggregates.instantiation_steps().saturating_sub(before);
        let charged = charge(self.must_use_contracts.get_or_insert_default(), work, 4 * size_of::<(TypeId, bool)>());
        let result = prepared.and(charged).and_then(|()| self.trivial_graph(report, module, root, cache));
        if let Err(error) = result {
            // A failed budget cannot publish more storage; keep the actual first failure.
            let _ = self.remember_trivial(module, root, Err(error));
            return Err(error);
        }
        result
    }
    fn trivial_graph(&mut self, report: &Report, module: ModuleId, root: TypeId,
        cache: &mut Cache) -> Result<bool, &'static str> {
        let mut pending = vec![(root, false)];
        while let Some((ty, complete)) = pending.pop() {
            let table = self.must_use_contracts.get_or_insert_default();
            charge(table, 1, 0)?;
            if let Some(&result) = table.trivial.get(&(module, ty)) { result?; continue }
            if let Some(result) = immediate(&self.types, ty)? {
                self.remember_trivial(module, ty, Ok(result))?; continue;
            }
            if complete {
                let result = self.completed_trivial(module, ty)?;
                self.remember_trivial(module, ty, Ok(result))?; continue;
            }
            let query = coverage::is_inhabited(self, report, module, ty, cache);
            let table = self.must_use_contracts.get_or_insert_default();
            table.work.inhabitation_queries = table.work.inhabitation_queries.saturating_add(1);
            let work = usize::try_from(query.work).map_err(|_| "inhabitation work does not fit the host counter")?;
            let charged = charge(table, work, 0);
            let inhabited = query.result.and_then(|inhabited| charged.map(|()| inhabited))?;
            if !inhabited { self.remember_trivial(module, ty, Ok(true))?; continue }
            match self.types.kind(ty).ok_or("missing discarded domain type")? {
                Kind::Tuple(list) => {
                    let children = self.types.list(list).ok_or("missing discarded tuple domain")?;
                    charge(self.must_use_contracts.get_or_insert_default(), children.len(), 4 * (children.len() + 1) * size_of::<(TypeId, bool)>())?;
                    pending.push((ty, true));
                    pending.extend(children.iter().rev().map(|&child| (child, false)));
                }
                Kind::Array { element, length } if length != 0 => {
                    charge(self.must_use_contracts.get_or_insert_default(), 1, 8 * size_of::<(TypeId, bool)>())?;
                    pending.push((ty, true)); pending.push((element, false));
                }
                _ => self.remember_trivial(module, ty, Ok(false))?,
            }
        }
        self.must_use_contracts.as_deref().and_then(|table| table.trivial.get(&(module, root))).copied()
            .ok_or("discarded domain lost its root summary")?
    }
    fn completed_trivial(&self, module: ModuleId, ty: TypeId) -> Result<bool, &'static str> {
        let table = self.must_use_contracts.as_deref().ok_or("missing discarded domain owner")?;
        let ready = |ty| table.trivial.get(&(module, ty)).copied().ok_or("missing discarded child domain")?;
        match self.types.kind(ty).ok_or("missing completed discarded domain")? {
            Kind::Tuple(list) => {
                let mut trivial = true;
                for &child in self.types.list(list).ok_or("missing completed tuple domain")? { trivial &= ready(child)?; }
                Ok(trivial)
            }
            Kind::Array { element, length } if length != 0 => ready(element),
            _ => Err("invalid completed discarded domain"),
        }
    }
    fn remember_trivial(&mut self, module: ModuleId, ty: TypeId, value: Result<bool, &'static str>) -> Result<(), &'static str> {
        let table = self.must_use_contracts.get_or_insert_default();
        charge(table, 1, 4 * size_of::<((ModuleId, TypeId), Result<bool, &'static str>)>())?;
        if table.trivial.len() >= 1_000_000 { return Err("discarded domain storage budget exceeded") }
        table.trivial.insert((module, ty), value);
        Ok(())
    }
}
fn immediate(types: &crate::types::Store, ty: TypeId) -> Result<Option<bool>, &'static str> {
    Ok(match types.kind(ty).ok_or("discarded domain type is unknown")? {
        Kind::Never => Some(true),
        Kind::Tuple(list) if types.list(list).is_some_and(<[_]>::is_empty) => Some(true),
        Kind::Tuple(_) | Kind::Adt(..) | Kind::Array { length: 1.., .. } => None,
        _ => Some(false),
    })
}
fn charge(table: &mut Contracts, work: usize, bytes: usize) -> Result<(), &'static str> {
    table.work.inhabitation_steps = table.work.inhabitation_steps.saturating_add(work);
    table.trivial_bytes = table.trivial_bytes.saturating_add(bytes);
    if table.work.inhabitation_steps > 10_000_000 || table.trivial_bytes > 64 * 1024 * 1024 {
        return Err("discarded domain work or storage budget exceeded");
    }
    Ok(())
}
