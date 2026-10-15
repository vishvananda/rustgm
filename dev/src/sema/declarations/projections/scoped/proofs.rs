//! Scoped evidence is published immediately so later queries cannot reuse its region interval.
use super::{Ambient, Checker, Error, NodeKey, Plan, Request, ScopeId, Sink, map_binders, regions};
use crate::{
    sema::model::{BodyRegion, BodyRegionOrigin, QuantifiedProof},
    types::lifetime_proof::Obligations,
};

impl Checker<'_> {
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
        let ambient = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing scoped projection proof query"))?
            .ambient;
        if request.depth == 0 && self.ambient_depth(key, ambient)? == 0 {
            return self.retain_projection_proof(key, context, proof);
        }
        let allocated = proof.regions.clone();
        let sink = self.projection_sink(key, context, allocated.start)?;
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
            if let Some(&index) = self.projections.body_sinks.get(&body)
                && self.facts.quantified_proofs[index].proof.regions.end == first
            {
                return Ok(Sink::Body(index));
            }
            let index = self.facts.quantified_proofs.len();
            self.facts.quantified_proofs.push(QuantifiedProof {
                body,
                source: key,
                proof: Obligations::new(first),
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
            Sink::Body(index) => &mut self.facts.quantified_proofs[index].proof,
        };
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
        destination.append_at(proof, scope)
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
        while self.bindings.len() <= needed.0 as usize {
            let id = ScopeId(
                u32::try_from(self.bindings.len())
                    .map_err(|_| "too many retained projection scopes")?,
            );
            let (parent, count) = self.source.binder(id)?;
            let parent = *self
                .bindings
                .get(parent.0 as usize)
                .ok_or("missing retained parent scope")?;
            self.bindings.push(destination.scope(parent, count)?);
        }
        self.bindings
            .get(needed.0 as usize)
            .copied()
            .ok_or("missing retained projection scope")
    }
}
