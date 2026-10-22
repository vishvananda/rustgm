//! Storage heads share complete capture substitution without rewriting callable identity.
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::model::Declarations,
    types::{Kind, TypeId},
};

impl Declarations {
    /// Prepare appended canonical applications once, independently of caller proofs.
    /// Bodies still pass ordinary region and ownership verification before execution.
    pub(crate) fn prepare_opaque_storage(&mut self) -> Result<(), &'static str> {
        if self.opaques.declarations.is_empty() {
            return Ok(());
        }
        let mut pending = HashMap::default();
        let mut roots = Vec::new();
        while self.opaques.storage_cursor < self.types.len() {
            let index = self.opaques.storage_cursor;
            self.opaques.storage_cursor += 1;
            let ty = TypeId(u32::try_from(index).map_err(|_| "too many storage types")?);
            let Some(Kind::Opaque(id, arguments)) = self.types.kind(ty) else {
                continue;
            };
            let declaration = self
                .opaques
                .declarations
                .get(&id)
                .ok_or("missing storage opaque declaration")?;
            let Some(hidden) = declaration.hidden else {
                continue;
            };
            let mut substitution = declaration.substitution(&self.types, arguments)?;
            let hidden = substitution.apply(&mut self.types, hidden)?;
            self.aggregates.prepare(&mut self.types, hidden)?;
            pending.insert(ty, hidden);
            roots.push(ty);
            if pending.len() > 100_000 {
                return Err("opaque storage application budget exceeded");
            }
        }
        let mut path = Vec::new();
        let mut active = HashSet::default();
        for &root in &roots {
            if !pending.contains_key(&root) {
                continue;
            }
            let mut current = root;
            while let Some(&hidden) = pending.get(&current) {
                if !active.insert(current) {
                    return Err("recursive opaque hidden type");
                }
                path.push(current);
                current = hidden;
            }
            let head = self.types.storage_head(current)?;
            for ty in path.drain(..) {
                self.types.publish_storage_head(ty, head)?;
                pending.remove(&ty);
            }
            active.clear();
        }
        if !roots.is_empty() {
            let templates = self
                .opaques
                .storage_layouts
                .get_or_insert_with(Default::default);
            for root in roots {
                templates
                    .demand(
                        &mut self.types,
                        &mut self.aggregates,
                        &mut self.layouts,
                        root,
                    )
                    .map_err(|e| e.message)?;
            }
        }
        Ok(())
    }
}
