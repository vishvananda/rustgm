//! Precise receiver captures consume only their retained synthetic dereference prefix.
use super::{Builder, Error, NodeId, NodeKey, Work, environments::Access};
use crate::sema::closures::{AccessKind, AdjustedAccess, captures::Step};

impl Builder<'_> {
    fn adjusted_access(&self, site: NodeId, kind: AccessKind) -> Option<AdjustedAccess> {
        self.capture_environment()?
            .adjusted_accesses
            .get(&(
                NodeKey {
                    node: site,
                    ..self.source
                },
                kind,
            ))
            .copied()
    }

    pub(super) fn receiver_place_work(
        &self,
        operand: NodeId,
        site: NodeId,
        kind: AccessKind,
    ) -> Work {
        if self.adjusted_access(site, kind).is_some() {
            Work::AdjustedPlace {
                operand,
                site,
                kind,
            }
        } else {
            Work::Place(operand)
        }
    }

    pub(super) fn enter_adjusted_place(
        &mut self,
        operand: NodeId,
        site: NodeId,
        kind: AccessKind,
    ) -> Result<(), Error> {
        if self.current.is_none() {
            self.scratch.places.push(None);
            return Ok(());
        }
        let Some(access) = self.adjusted_access(site, kind) else {
            self.scratch.work.push(Work::Place(operand));
            return Ok(());
        };
        let facts = self.facts;
        let super::Owner::Closure { identity, .. } = self.body.owner else {
            return Err(self.error(site, "adjusted capture has no closure owner"));
        };
        let environment = facts
            .closures
            .environment(identity)
            .ok_or_else(|| self.error(site, "missing adjusted capture environment"))?;
        let original = environment
            .accesses
            .get(&NodeKey {
                node: operand,
                ..self.source
            })
            .ok_or_else(|| self.error(site, "adjusted capture has no original source place"))?;
        if access.dereferences == 0 || access.dereferences as usize > environment.plan.path_count()
        {
            return Err(self.error(site, "invalid adjusted capture dereference count"));
        }
        let key = (*original, access.path, access.dereferences);
        if !self
            .capture_places
            .as_deref()
            .is_some_and(|places| places.validated.contains(&key))
        {
            let mut cursor = access.path;
            for _ in 0..access.dereferences {
                let (parent, Step::Dereference { mutable }) =
                    environment.plan.parent(cursor).ok_or_else(|| {
                        self.error(site, "adjusted capture lacks a dereference prefix")
                    })?
                else {
                    return Err(self.error(site, "adjusted capture prefix is not a dereference"));
                };
                let parent_ty = environment.path_types.get(parent.index()).copied();
                let child_ty = environment.path_types.get(cursor.index()).copied();
                if !matches!(parent_ty.and_then(|ty| facts.types.kind(ty)),
                    Some(super::Type::Reference {referent, mutable: actual, ..})
                        if Some(referent) == child_ty && actual == mutable)
                {
                    return Err(self.error(
                        site,
                        "adjusted dereference differs from retained path types",
                    ));
                }
                cursor = parent;
            }
            if cursor != *original {
                return Err(self.error(site, "adjusted capture differs from original source place"));
            }
            if let Some(places) = self.capture_places.as_deref_mut() {
                places.validated.insert(key);
            }
        }
        let Access::Place(place) = self.capture_path_place(site, environment, access.path)? else {
            return Err(self.error(site, "adjusted receiver lacks a physical capture place"));
        };
        self.scratch.places.push(Some(place));
        Ok(())
    }

    pub(super) fn remaining_receiver_dereferences(
        &self,
        site: NodeId,
        kind: AccessKind,
        selected: u32,
    ) -> Result<u32, Error> {
        let consumed = self
            .adjusted_access(site, kind)
            .map_or(0, |access| access.dereferences);
        selected.checked_sub(consumed).ok_or_else(|| {
            self.error(
                site,
                "capture prefix exceeds selected receiver dereferences",
            )
        })
    }
}
