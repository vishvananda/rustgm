//! Stable type-debt positions survive append-only evidence from normalization.
use super::{Obligations, ScopeId};
use crate::types::TypeId;

#[derive(Clone, Copy)]
enum Position {
    Root(usize),
    Scoped { block: usize, index: usize },
}

/// A position within the proof that produced it, not cross-proof authority.
#[derive(Clone, Copy)]
pub(crate) struct TypeRoot {
    pub scope: ScopeId,
    pub ty: TypeId,
    position: Position,
}

impl Obligations {
    /// Snapshot positions before an operation appends further debts. Iteration
    /// ordinals would be unstable when new root debts precede old scoped blocks.
    pub(crate) fn type_root_positions(&self) -> impl Iterator<Item = TypeRoot> + '_ {
        self.types
            .iter()
            .enumerate()
            .map(|(index, &(ty, _))| TypeRoot {
                scope: ScopeId::ROOT,
                ty,
                position: Position::Root(index),
            })
            .chain(self.quantified.iter().flat_map(|q| {
                q.blocks.iter().enumerate().flat_map(|(block, value)| {
                    value
                        .proof
                        .types
                        .iter()
                        .enumerate()
                        .map(move |(index, &(ty, _))| TypeRoot {
                            scope: value.scope,
                            ty,
                            position: Position::Scoped { block, index },
                        })
                })
            }))
    }

    /// Update exactly one saved debt; appended debts, targets and scopes stay
    /// intact. The caller retains all evidence used to normalize its old type.
    pub(crate) fn replace_type_root(
        &mut self,
        root: TypeRoot,
        replacement: TypeId,
    ) -> Result<(), &'static str> {
        let (scope, entry) = match root.position {
            Position::Root(index) => (ScopeId::ROOT, self.types.get_mut(index)),
            Position::Scoped { block, index } => {
                let block = self
                    .quantified
                    .as_mut()
                    .and_then(|q| q.blocks.get_mut(block))
                    .ok_or("missing retained type-debt block")?;
                (block.scope, block.proof.types.get_mut(index))
            }
        };
        let entry = entry.ok_or("missing retained type-debt position")?;
        if scope != root.scope || entry.0 != root.ty {
            return Err("retained type-debt position changed before replacement");
        }
        entry.0 = replacement;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RegionId;

    #[test]
    fn appended_debts_do_not_redirect_saved_positions() -> Result<(), &'static str> {
        let boolean = TypeId::primitive(crate::types::Primitive::Bool);
        let mut proof = Obligations::new(1);
        let scope = proof.scope(ScopeId::ROOT, 1)?;
        proof.types.push((TypeId::UNIT, RegionId::STATIC));
        let mut scoped = Obligations::new(1);
        scoped.types.push((boolean, RegionId::bound(0, 0)?));
        proof.append_at(scoped, scope)?;
        let roots: Vec<_> = proof.type_root_positions().collect();
        let mut appended = Obligations::new(1);
        appended.types.push((boolean, RegionId::STATIC));
        proof.extend(appended.clone())?;
        proof.append_at(appended, scope)?;
        proof.replace_type_root(roots[0], boolean)?;
        proof.replace_type_root(roots[1], TypeId::UNIT)?;
        assert!(proof.replace_type_root(roots[1], boolean).is_err());
        assert_eq!(
            proof.type_roots().collect::<Vec<_>>(),
            vec![
                (ScopeId::ROOT, boolean),
                (ScopeId::ROOT, boolean),
                (scope, TypeId::UNIT),
                (scope, boolean),
            ]
        );
        assert_eq!(
            proof.quantified.as_ref().ok_or("missing scopes")?.blocks[0]
                .proof
                .types[0]
                .1,
            RegionId::bound(0, 0)?
        );
        Ok(())
    }
}
