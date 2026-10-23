//! Storage heads share complete capture substitution without rewriting callable identity.
use super::{StorageFailure, applications::Preparation};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::model::Declarations,
    types::{Kind, TypeId},
};

type Applications = (HashMap<TypeId, TypeId>, Vec<TypeId>);

impl Declarations {
    /// Prepare appended canonical applications once, independently of caller proofs.
    /// Bodies still pass ordinary region and ownership verification before execution.
    pub(crate) fn prepare_opaque_storage(&mut self) -> Result<(), &'static str> {
        self.prepare_opaque_storage_located()
            .map_err(|failure| failure.message)
    }
    /// Retain only the exact operation's diagnostic owner; never infer proof authority.
    pub(crate) fn prepare_opaque_storage_located(&mut self) -> Result<(), StorageFailure> {
        if self.opaques.declarations.is_empty() {
            return Ok(());
        }
        let mut preparation = self.opaques.storage_preparation.take().unwrap_or_default();
        let result = match preparation.failed {
            Some(error) => Err(error),
            None => self.prepare_opaque_heads(&mut preparation),
        };
        preparation.failed = result.as_ref().err().copied();
        self.opaques.storage_preparation = Some(preparation);
        result
    }
    fn prepare_opaque_heads(
        &mut self,
        preparation: &mut Preparation,
    ) -> Result<(), StorageFailure> {
        let (mut pending, roots) = self.collect_opaque_applications(preparation)?;
        let mut path = Vec::new();
        let mut active = HashSet::default();
        for &root in &roots {
            if !pending.contains_key(&root) {
                continue;
            }
            let result = (|| {
                let mut current = root;
                while let Some(&hidden) = pending.get(&current) {
                    preparation.charge(1)?;
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
                Ok(())
            })();
            result.map_err(|message| self.opaque_storage_failure(root, message))?;
        }
        for root in roots {
            let result = (|| {
                let steps = preparation.layouts.steps;
                let result = preparation
                    .layouts
                    .demand(
                        &mut self.types,
                        &mut self.aggregates,
                        &mut self.layouts,
                        root,
                    )
                    .map_err(|e| e.message);
                preparation.charge(preparation.layouts.steps.saturating_sub(steps))?;
                result?;
                Ok(())
            })();
            result.map_err(|message| self.opaque_storage_failure(root, message))?;
        }
        Ok(())
    }
    fn collect_opaque_applications(
        &mut self,
        preparation: &mut Preparation,
    ) -> Result<Applications, StorageFailure> {
        let mut pending = HashMap::default();
        let mut roots = Vec::new();
        let replay = std::mem::take(&mut preparation.ready);
        let replay_capacity = replay.capacity();
        let mut replay = replay.into_iter();
        loop {
            let ty = if let Some(ty) = replay.next() {
                ty
            } else if self.opaques.storage_cursor < self.types.len() {
                let index = self.opaques.storage_cursor;
                self.opaques.storage_cursor += 1;
                TypeId(u32::try_from(index).map_err(|_| "too many storage types")?)
            } else {
                break;
            };
            preparation.charge(1)?;
            let Some(Kind::Opaque(id, arguments)) = self.types.kind(ty) else {
                continue;
            };
            let result = (|| {
                let declaration = self
                    .opaques
                    .declarations
                    .get(&id)
                    .ok_or("missing storage opaque declaration")?;
                let Some(hidden) = declaration.hidden else {
                    preparation.wait(id, ty)?;
                    return Ok(None);
                };
                preparation.scratch(roots.len().saturating_add(1), replay_capacity)?;
                let mut substitution = declaration.substitution(&self.types, arguments)?;
                let hidden = substitution.apply(&mut self.types, hidden)?;
                preparation.charge(substitution.steps)?;
                self.aggregates.prepare(&mut self.types, hidden)?;
                Ok(Some(hidden))
            })();
            let Some(hidden) =
                result.map_err(|message| self.opaque_storage_failure(ty, message))?
            else {
                continue;
            };
            pending.insert(ty, hidden);
            roots.push(ty);
        }
        Ok((pending, roots))
    }
    fn opaque_storage_failure(&self, ty: TypeId, message: &'static str) -> StorageFailure {
        let owner = match self.types.kind(ty) {
            Some(Kind::Opaque(id, _)) => self.opaques.declarations.get(&id).map(|d| d.owner),
            _ => None,
        };
        StorageFailure { owner, message }
    }
}
