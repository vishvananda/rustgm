//! Validate unused declarations too; closed shapes avoid quadratic concrete-impl scans.
use super::{Checker, DefId, Error, NodeKey, TypeId};
use crate::{sema::methods::overlap, types::substitute::Bound};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Header {
    owner: DefId,
    source: NodeKey,
    closed: Option<TypeId>,
}

#[derive(Default)]
struct Bucket {
    all: Vec<Header>,
    open: Vec<Header>,
    closed: HashMap<TypeId, Vec<Header>>,
}
impl Bucket {
    fn candidates(&self, header: Header) -> Vec<Header> {
        let Some(key) = header.closed else {
            return self.all.clone();
        };
        let mut candidates = self.open.clone();
        candidates.extend(self.closed.get(&key).into_iter().flatten().copied());
        candidates.sort_unstable_by_key(|header| header.owner.0);
        candidates
    }
    fn insert(&mut self, header: Header) {
        self.all.push(header);
        if let Some(key) = header.closed {
            self.closed.entry(key).or_default().push(header);
        } else {
            self.open.push(header);
        }
    }
}

impl Checker<'_> {
    pub(super) fn validate_inherent_overlaps(&mut self) -> Result<(), Error> {
        let mut buckets: Vec<_> = self
            .facts
            .methods
            .inherent
            .values()
            .filter(|members| members.len() > 1)
            .cloned()
            .collect();
        buckets.sort_unstable_by_key(|members| members[0].0);
        let mut headers = HashMap::new();
        let mut pairs = HashMap::new();
        let mut shapes = Bound::shape();
        for members in buckets {
            let mut bucket = Bucket::default();
            for method in members {
                let owner = self
                    .facts
                    .associated
                    .owner(method)
                    .and_then(|id| self.facts.associated.get(id))
                    .ok_or_else(|| {
                        Error::at(crate::resolve::unit::FileId(0), 0, "missing inherent owner")
                    })?;
                let (owner, source) = (owner.self_definition, owner.source);
                let header = if let Some(&header) = headers.get(&owner) {
                    header
                } else {
                    let header = self.overlap_header(owner, source, &mut shapes)?;
                    headers.insert(owner, header);
                    header
                };
                for previous in bucket.candidates(header) {
                    let key = if previous.owner.0 <= owner.0 {
                        (previous.owner, owner)
                    } else {
                        (owner, previous.owner)
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
                            header.source,
                            "duplicate inherent method in overlapping implementations",
                        ));
                    }
                }
                bucket.insert(header);
            }
        }
        Ok(())
    }

    fn overlap_header(
        &mut self,
        owner: DefId,
        source: NodeKey,
        shapes: &mut Bound,
    ) -> Result<Header, Error> {
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
        Ok(Header {
            owner,
            source,
            closed,
        })
    }
}
