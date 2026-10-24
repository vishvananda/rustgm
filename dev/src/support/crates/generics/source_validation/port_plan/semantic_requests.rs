//! One complete request inventory is retained only after all real original data closes.
use super::{
    Context, DomainIndex, InputWork, OperationPort, OriginalInputPlan, RequiredInput, RequiredKind,
    Work,
    coverage::Coverage,
    g,
    nodes::{Arena, NodeId, Rewrite},
    s, uses,
};
use crate::support::{
    crates::{self, owner_catalog::BodyRef},
    retained,
};

pub(super) struct Builder<'a, 'index, 'rows> {
    pub domains: &'a DomainIndex<'index, 'rows>,
    pub arena: Arena,
    pub coverage: Coverage,
    pub requests: Vec<RequiredInput>,
    pub ranges: Vec<g::Range>,
    first: usize,
}
impl<'a, 'index, 'rows> Builder<'a, 'index, 'rows> {
    pub(super) fn new(
        domains: &'a DomainIndex<'index, 'rows>,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let first = work.steps;
        let extra = size_of::<Self>().saturating_sub(size_of::<Arena>());
        let mut arena = Arena::new(domains, extra, work)?;
        arena.signatures(domains, work)?;
        let coverage = Coverage::new(domains, &mut arena, work)?;
        Ok(Self {
            domains,
            arena,
            coverage,
            requests: Vec::new(),
            ranges: Vec::new(),
            first,
        })
    }
    pub(super) fn finish(
        mut self,
        work: &mut Work,
    ) -> Result<OriginalInputPlan<'rows>, &'static str> {
        let rows = self.domains.headers.rows();
        for original in &rows.originals {
            let start = u32::try_from(self.requests.len())
                .map_err(|_| "source required-input index overflow")?;
            self.declaration_inputs(original, work)?;
            self.operations(original, work)?;
            self.arena.reserve(4 * size_of::<g::Range>(), 0, work)?;
            self.ranges.push(g::Range {
                start,
                end: u32::try_from(self.requests.len())
                    .map_err(|_| "source required-input index overflow")?,
            });
        }
        work.charge(1)?;
        let extra = size_of::<Self>().saturating_sub(size_of::<Arena>());
        let live_bytes = self
            .arena
            .retained_bytes(self.domains, extra)
            .saturating_add(self.coverage.retained_bytes())
            .saturating_add(retained::vector(&self.requests))
            .saturating_add(retained::vector(&self.ranges));
        if live_bytes > self.arena.reserved || self.arena.peak > crates::MAX_BYTES {
            return Err("source original inputs exceeded their combined allocation preflight");
        }
        // Arena, coverage and work stacks are construction-only. The returned
        // plan retains only borrowed rows and its complete request/range arrays.
        // The caller's already retained DomainIndex stays in the combined base.
        let bytes = self
            .domains
            .retained_bytes()
            .saturating_add(size_of::<OriginalInputPlan<'_>>())
            .saturating_add(retained::vector(&self.requests))
            .saturating_add(retained::vector(&self.ranges));
        let observed = InputWork {
            steps: work.steps.saturating_sub(self.first),
            expression_nodes: self.arena.nodes.len(),
            rewritten_uses: self.arena.completed.len(),
            application_templates: self.arena.templates.len(),
            comparison_pairs: self.arena.matched.len(),
            cache_hits: self.arena.hits,
            required_inputs: self.requests.len(),
        };
        Ok(OriginalInputPlan {
            rows,
            ranges: self.ranges,
            requests: self.requests,
            observed,
            bytes,
            peak: self.arena.peak,
        })
    }
    pub(super) fn emit(
        &mut self,
        port: OperationPort,
        site: s::InputSite,
        kind: RequiredKind,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        self.arena
            .reserve(4 * size_of::<RequiredInput>(), 0, work)?;
        self.requests.push(RequiredInput { port, site, kind });
        Ok(())
    }
    pub(super) fn node(
        &mut self,
        original: &s::Original,
        root: g::TypeRef,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let context = Context {
            owner: original.declaration,
            frame: Some(self.domains.headers.rows().body_inputs[original.body.0 as usize].frame),
            body: Some(original.body),
        };
        let key = uses::key(self.domains, root, Rewrite::Original, Some(context), work)?;
        self.arena.root(self.domains, key, work)
    }
    fn declaration_inputs(
        &mut self,
        original: &s::Original,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.domains.headers.rows();
        let input = &rows.body_inputs[original.body.0 as usize];
        let header = self.domains.function(original.declaration, work)?;
        for (ordinal, &slot) in input.parameters.iter().enumerate() {
            work.charge(1)?;
            let s::ParameterKind::Type { sized } = rows.slots[slot.0 as usize].kind else {
                return Err("source original type-formal request changed kind");
            };
            self.emit(
                OperationPort::Formal {
                    body: original.body,
                    ordinal: u32::try_from(ordinal)
                        .map_err(|_| "source type-formal ordinal overflow")?,
                },
                s::InputSite::Declaration {
                    owner: original.declaration,
                    site: 0,
                },
                RequiredKind::Formal {
                    slot,
                    sizing: sized,
                },
                work,
            )?;
        }
        self.clauses(original.body, header.inputs, work)?;
        if let Some(parent) = header.parent {
            // The admitted header relation is a function's real trait/impl owner,
            // never lexical function capture. Thus no empty ancestry walk or
            // flattened inherited environment is needed for this source family.
            let range = match self.domains.headers.header(parent, work)? {
                Some(super::super::HeaderRef::Trait(index)) => rows.traits[index as usize].inputs,
                Some(super::super::HeaderRef::Implementation(index)) => {
                    rows.implementations[index as usize].inputs
                }
                _ => return Err("source original has an unsupported actual declaration parent"),
            };
            self.clauses(original.body, range, work)?;
        }
        if let s::EnvironmentOrigin::ConstTraitDefault { member, trait_ } = input.environment {
            self.emit(
                OperationPort::Declaration {
                    body: original.body,
                    owner: member,
                },
                s::InputSite::Declaration {
                    owner: member,
                    site: 0,
                },
                RequiredKind::DefiningCleanup { member, trait_ },
                work,
            )?;
        }
        Ok(())
    }
    fn clauses(
        &mut self,
        body: BodyRef,
        range: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.domains.headers.rows();
        for (offset, clause) in range.slice(&rows.clauses)?.iter().enumerate() {
            self.emit(
                OperationPort::Declaration {
                    body,
                    owner: clause.owner,
                },
                s::InputSite::Declaration {
                    owner: clause.owner,
                    site: clause.site,
                },
                RequiredKind::Declaration {
                    clause: range
                        .start
                        .checked_add(
                            u32::try_from(offset)
                                .map_err(|_| "source declaration clause index overflow")?,
                        )
                        .ok_or("source declaration clause index overflow")?,
                },
                work,
            )?;
        }
        Ok(())
    }
    pub(super) fn formation(
        &mut self,
        original: &s::Original,
        port: OperationPort,
        site: u32,
        ty: g::TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let node = self.node(original, ty, work)?;
        if !self.arena.intrinsically_formed(node, work)?
            && !self.coverage.formed(original.body, node, work)?
        {
            return Err("source operation omitted a required nontrivial formation input");
        }
        self.emit(
            port,
            s::InputSite::Body(site),
            RequiredKind::Formation(ty),
            work,
        )
    }
}
