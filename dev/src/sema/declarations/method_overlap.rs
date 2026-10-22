//! Validate unused declarations too; closed shapes avoid quadratic concrete-impl scans.
use super::{Checker, DefId, Error, NodeKey, TypeId};
use crate::support::hash::HashMap;
use crate::{sema::methods::overlap, types::substitute::Bound};

#[derive(Default)]
struct Bucket {
    all: Vec<DefId>,
    methods: crate::sema::methods::Candidates,
}
impl Bucket {
    fn candidates(&self, closed: Option<TypeId>) -> Vec<DefId> {
        let Some(key) = closed else {
            return self.all.clone();
        };
        let mut candidates = self.methods.open.clone();
        candidates.extend(self.methods.closed.get(&key).into_iter().flatten().copied());
        candidates.sort_unstable_by_key(|method| method.0);
        candidates
    }
    fn insert(&mut self, closed: Option<TypeId>, method: DefId) {
        self.all.push(method);
        if let Some(key) = closed {
            self.methods.closed.entry(key).or_default().push(method);
        } else {
            self.methods.open.push(method);
        }
    }
}

impl Checker<'_> {
    pub(super) fn validate_inherent_overlaps(&mut self) -> Result<(), Error> {
        let mut buckets: Vec<_> = self
            .facts
            .methods
            .inherent
            .iter()
            .filter(|(_, members)| members.len() > 1)
            .map(|(&key, members)| (key, members.clone()))
            .collect();
        buckets.sort_unstable_by_key(|(_, members)| members[0].0);
        if buckets.is_empty() {
            return Ok(());
        }
        let mut headers = HashMap::default();
        let mut pairs = HashMap::default();
        let mut shapes = Bound::shape();
        for (key, members) in buckets {
            let mut bucket = Bucket::default();
            for method in members {
                let owner = self.inherent_owner(method)?;
                let (owner, source) = (owner.self_definition, owner.source);
                let header = if let Some(&header) = headers.get(&owner) {
                    header
                } else {
                    let header = self.overlap_header(owner, source, &mut shapes)?;
                    headers.insert(owner, header);
                    header
                };
                for previous in bucket.candidates(header) {
                    let previous = self.inherent_owner(previous)?.self_definition;
                    let key = if previous.0 <= owner.0 {
                        (previous, owner)
                    } else {
                        (owner, previous)
                    };
                    let possible = if let Some(&possible) = pairs.get(&key) {
                        possible
                    } else {
                        let (possible, steps) = overlap::possible(&mut self.facts, key.0, key.1)
                            .map_err(|message| self.error(source, message))?;
                        self.facts.methods.overlap_pairs += 1;
                        self.facts.methods.overlap_steps =
                            self.facts.methods.overlap_steps.saturating_add(steps);
                        if self.facts.methods.overlap_steps > 1_000_000 {
                            return Err(self.error(source, "inherent overlap work budget exceeded"));
                        }
                        pairs.insert(key, possible);
                        possible
                    };
                    if possible {
                        return Err(self.error(
                            source,
                            "duplicate inherent method in overlapping implementations",
                        ));
                    }
                }
                bucket.insert(header, method);
            }
            self.facts.methods.indices.insert(key, bucket.methods);
        }
        self.facts.methods.shapes = Some(Box::new(shapes));
        Ok(())
    }

    fn overlap_header(
        &mut self,
        owner: DefId,
        source: NodeKey,
        shapes: &mut Bound,
    ) -> Result<Option<TypeId>, Error> {
        let ty = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| self.error(source, "missing overlap target"))?
            .self_type;
        let closed = if self.facts.types.contains_parameters(ty) == Some(false) {
            Some(
                shapes
                    .apply(&mut self.facts.types, ty)
                    .map_err(|message| self.error(source, message))?,
            )
        } else {
            None
        };
        Ok(closed)
    }
    fn inherent_owner(&self, member: DefId) -> Result<crate::sema::associated::Owner, Error> {
        self.facts
            .associated
            .owner(member)
            .and_then(|id| self.facts.associated.get(id))
            .copied()
            .ok_or_else(|| Error::at(crate::resolve::unit::FileId(0), 0, "missing inherent owner"))
    }
}
