//! Scoped evidence is published immediately so later queries cannot reuse its region interval.
use super::{Ambient, Checker, Error, NodeKey, Plan, Request, ScopeId, Sink, map_binders, regions};
use crate::{
    sema::model::{BodyRegion, BodyRegionOrigin, QuantifiedProof},
    sema::traits::obligations::Reservation,
    types::lifetime_proof::Obligations,
};

impl Checker<'_> {
    pub(in super::super) fn retain_reserved_projection_proof(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
        proof: Obligations,
        mut reservation: Reservation,
    ) -> Result<(), Error> {
        let (regions::Position::Local(body) | regions::Position::Constant(body)) = context.position
        else {
            return Err(self.error(key, "body reservation requires a body projection"));
        };
        if self.projection_alias_owner(context).is_some()
            || context
                .binder
                .is_some_and(|binder| self.regions.parent(binder).is_some())
        {
            return Err(self.error(key, "body projection reservation has a declaration binder"));
        }
        let body = NodeKey { node: body, ..key };
        reservation
            .reserve(&mut self.facts, body, key, proof.regions.clone())
            .map_err(|e| self.error(key, e))?;
        if matches!(context.position, regions::Position::Constant(_)) {
            self.facts.constant_annotation_bodies.insert(body);
        }
        let query = self
            .projections
            .roots
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "missing reserved projection query"))?;
        let index = if let Some(index) = query.reserved_sink {
            index
        } else {
            let index = self.facts.quantified_proofs.len();
            self.facts.quantified_proofs.push(QuantifiedProof {
                body,
                evidence: crate::sema::model::LifetimeProof::new(
                    key,
                    Obligations::new(proof.regions.start),
                ),
            });
            query.reserved_sink = Some(index);
            index
        };
        self.append_scoped_projection(key, request, Sink::Body(index), proof)
            .map_err(|e| self.error(key, e))
    }

    pub(in super::super) fn retain_scoped_projection_proof(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
        proof: Obligations,
    ) -> Result<(), Error> {
        if proof.is_empty() {
            return Ok(());
        }
        if let Some(template) = self.projection_alias_owner(context) {
            let scope = self.projection_alias_request_scope(key, request, context, template)?;
            return self
                .facts
                .alias_requirements
                .direct(template.owner, template.parameters, proof, scope)
                .map_err(|e| self.error(key, e));
        }
        let query = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing scoped projection proof query"))?;
        let reserved = query.reserved_sink;
        if reserved.is_none() && request.depth == 0 && self.ambient_depth(key, query.ambient)? == 0
        {
            return self.retain_projection_proof(key, context, proof);
        }
        let allocated = proof.regions.clone();
        let sink = if let Some(index) = reserved {
            if usize::try_from(allocated.start).ok() != self.facts.body_regions.len().checked_add(1)
            {
                return Err(self.error(key, "unreserved projection proof has foreign identities"));
            }
            Sink::Body(index)
        } else {
            self.projection_sink(key, context, allocated.start)?
        };
        self.append_scoped_projection(key, request, sink, proof)
            .map_err(|e| self.error(key, e))?;
        if let Sink::Body(index) = sink {
            let body = self.facts.quantified_proofs[index].body;
            self.facts
                .body_regions
                .extend(allocated.map(|_| BodyRegion {
                    body,
                    source: key,
                    origin: BodyRegionOrigin::QuantifiedProof,
                }));
        }
        Ok(())
    }

    fn projection_sink(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        first: u32,
    ) -> Result<Sink, Error> {
        if let regions::Position::Local(body) | regions::Position::Constant(body) = context.position
        {
            if usize::try_from(first).ok() != self.facts.body_regions.len().checked_add(1) {
                return Err(self.error(key, "scoped proof has foreign body region identities"));
            }
            if context
                .binder
                .is_some_and(|binder| self.regions.parent(binder).is_some())
            {
                return Err(
                    self.error(key, "local projection proof requires its callable universe")
                );
            }
            let body = NodeKey { node: body, ..key };
            if matches!(context.position, regions::Position::Constant(_)) {
                self.facts.constant_annotation_bodies.insert(body);
            }
            if let Some(&index) = self.projections.body_sinks.get(&body) {
                return Ok(Sink::Body(index));
            }
            let index = self.facts.quantified_proofs.len();
            self.facts.quantified_proofs.push(QuantifiedProof {
                body,
                evidence: crate::sema::model::LifetimeProof::new(key, Obligations::new(first)),
            });
            self.projections.body_sinks.insert(body, index);
            return Ok(Sink::Body(index));
        }
        let index = if let Some(&index) = self.projections.scopes.get(&context.binder) {
            index
        } else {
            let index = self.projections.proofs.len();
            self.projections
                .proofs
                .push((key, context, Obligations::new(first)));
            self.projections.scopes.insert(context.binder, index);
            index
        };
        Ok(Sink::Declaration(index))
    }

    fn append_scoped_projection(
        &mut self,
        key: NodeKey,
        request: Request,
        sink: Sink,
        proof: Obligations,
    ) -> Result<(), &'static str> {
        let query = self
            .projections
            .roots
            .get_mut(&key)
            .ok_or("missing scoped projection query")?;
        let destination = match sink {
            Sink::Declaration(index) => &mut self.projections.proofs[index].2,
            Sink::Body(index) => &mut self.facts.quantified_proofs[index].evidence.proof,
        };
        if let Sink::Body(_) = sink {
            destination.resume_at(proof.regions.start)?;
        }
        let root = match query.ambient {
            Ambient::Root => ScopeId::ROOT,
            Ambient::Alias { plan, scope } => self
                .projections
                .plans
                .get_mut(plan)
                .ok_or("missing alias normalization scope plan")?
                .instantiate(sink, destination, scope)?,
        };
        if query.scopes.sink != Some(sink) {
            query.scopes.sink = Some(sink);
            query.scopes.bindings.clear();
        }
        let scope = map_binders(
            &mut query.scopes.bindings,
            &query.fold,
            root,
            request.scope,
            |parent, count| destination.scope(parent, count),
        )?;
        destination.append_streamed_at(proof, scope)
    }
}

impl Plan {
    fn instantiate(
        &mut self,
        sink: Sink,
        destination: &mut Obligations,
        needed: ScopeId,
    ) -> Result<ScopeId, &'static str> {
        if needed.0 as usize > self.source.len() {
            return Err("invalid retained projection scope");
        }
        if self.sink != Some(sink) {
            self.sink = Some(sink);
            self.bindings.clear();
            self.bindings.push(ScopeId::ROOT);
        }
        crate::types::lifetime_proof::map_scopes(
            &mut self.bindings,
            ScopeId::ROOT,
            needed,
            |id| self.source.binder(id),
            |parent, count| destination.scope(parent, count),
        )
    }
}
