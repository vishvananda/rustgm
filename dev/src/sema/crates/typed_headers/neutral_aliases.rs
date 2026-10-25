//! A recorded body's neutral finish binds its immutable result Arc, never proof.
use super::{DomainStage, Owner, Ready, RecipeView, ReservationWork, Store, error};
use crate::{
    resolve::{
        body_origin::{ImportedBodyId, ImportedDeclarationId},
        package::Error,
    },
    sema::function_context::assumption_environment as defining,
    support::crates::owner_catalog::BodyRef,
    types::{arena::Identity, parameters::environment::Environment},
};
use std::sync::{Arc, Weak};

pub(in super::super) struct Key {
    root: ImportedDeclarationId,
    body: ImportedBodyId,
    raw: BodyRef,
    arena: Identity,
    environment: Weak<Environment>,
    recording: Weak<super::RecordedDomain>,
}
pub(in super::super) struct Request<'a, 'rows> {
    pub view: &'a RecipeView<'rows>,
    pub store: &'a Store,
    pub expected: &'a Ready,
    pub declaration: &'a Arc<Environment>,
}
pub(in super::super) struct Completed {
    pub key: Key,
    pub environment: Arc<Environment>,
    pub bytes: usize,
}
impl Owner {
    fn selected<'a>(
        &'a self,
        request: &Request<'_, '_>,
        work: &mut ReservationWork,
    ) -> Result<&'a Arc<super::RecordedDomain>, Error> {
        let stage = DomainStage::new(request.view, request.store, request.expected, work)?;
        let record = self
            .entries
            .get(request.expected.pair.function_row as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| error("neutral body lost its actual immutable recording"))?;
        record.require(&stage, request.store, work)?;
        Ok(record)
    }
    pub(in super::super) fn finish_neutral(
        &self,
        request: &Request<'_, '_>,
        prepared: defining::Prepared,
        work: &mut ReservationWork,
    ) -> Result<Completed, Error> {
        let record = self.selected(request, work)?;
        let count = record.aliases.as_ref().map_or(0, Vec::len);
        work.charge(3)?;
        let growth = request
            .declaration
            .neutral_alias_finish_bound(count)
            .map_err(error)?;
        work.charge(growth.work.saturating_add(count).saturating_add(16))?;
        let before = work.retained;
        work.reserve(growth.bytes)?;
        if prepared.needs_aliases() {
            return Err(error(
                "neutral body unexpectedly needs regional equality aliases",
            ));
        }
        // Clone only the authenticated recorded ledger, after its entire bound.
        let aliases = record.aliases.clone().unwrap_or_default();
        let completed = prepared
            .finish(request.expected.header.regions.total, aliases)
            .map_err(Error::unlocated)?;
        let environment = completed
            .environment
            .ok_or_else(|| error("neutral numbered body lost its actual environment"))?;
        let bytes = if count == 0 {
            if completed.changed || !Arc::ptr_eq(&environment, request.declaration) {
                return Err(error(
                    "neutral ledger-free body changed its declaration Arc",
                ));
            }
            0
        } else {
            if !completed.changed
                || Arc::ptr_eq(&environment, request.declaration)
                || environment
                    .body_aliases()
                    .is_none_or(|actual| Some(actual.ordered()) != record.aliases.as_deref())
            {
                return Err(error(
                    "neutral body changed its exact actual recorded ledger",
                ));
            }
            let actual = environment
                .neutral_alias_owned_from(request.declaration)
                .map_err(error)?;
            if actual.visits > 16 {
                return Err(error(
                    "neutral alias finish exceeded its selected observer work",
                ));
            }
            actual.bytes
        };
        if bytes > growth.bytes {
            return Err(error(
                "neutral alias finish exceeded its selected allocation",
            ));
        }
        work.retained = before.saturating_add(bytes);
        Ok(Completed {
            key: Key {
                root: record.seal.root,
                body: record.seal.body,
                raw: record.seal.raw,
                arena: record.seal.arena.clone(),
                environment: Arc::downgrade(&environment),
                recording: Arc::downgrade(record),
            },
            environment,
            bytes,
        })
    }
    pub(in super::super) fn require_neutral(
        &self,
        request: &Request<'_, '_>,
        key: &Key,
        environment: &Arc<Environment>,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        let record = self.selected(request, work)?;
        work.charge(9)?;
        request
            .declaration
            .neutral_alias_finish_bound(0)
            .map_err(error)?;
        if key.root != record.seal.root
            || key.body != record.seal.body
            || key.raw != record.seal.raw
            || !request.store.has_identity(&key.arena)
            || !key.environment.ptr_eq(&Arc::downgrade(environment))
            || !key.recording.ptr_eq(&Arc::downgrade(record))
        {
            return Err(error(
                "neutral numbered body changed its recorded owner or result Arc",
            ));
        }
        match record.aliases.as_deref() {
            None | Some([])
                if Arc::ptr_eq(environment, request.declaration)
                    && environment.body_aliases().is_none() =>
            {
                Ok(())
            }
            Some(aliases)
                if !aliases.is_empty()
                    && !Arc::ptr_eq(environment, request.declaration)
                    && environment
                        .body_aliases()
                        .is_some_and(|actual| actual.len() == aliases.len()) =>
            {
                Ok(())
            }
            _ => Err(error(
                "neutral numbered body changed its immutable ledger relationship",
            )),
        }
    }
}
