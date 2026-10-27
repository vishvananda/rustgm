//! Cold owner for actual removal receipts and source-owned lexical lint changes.
use super::{Engine, Error, NodeKey, Report, Sources, Visit, removed_functions::Removals};
use crate::resolve::lint_policy::{
    Policy,
    producer::{Producer, Site, Tail},
};
#[derive(Default)]
pub(crate) struct Effects {
    pub removals: Removals,
    pub policy: Option<Producer>,
    pub(super) derives: Option<Box<super::derive_lists::Transactions>>,
    pub(super) visibilities: Option<Box<super::unnamed_visibility::Causes>>,
}
impl Report {
    pub(crate) fn removed_functions(&self) -> Option<&Removals> {
        self.attribute_effects
            .as_deref()
            .map(|effects| &effects.removals)
    }
    pub(crate) fn lint_producer(&self) -> Option<&Producer> {
        self.attribute_effects
            .as_deref()
            .and_then(|effects| effects.policy.as_ref())
    }
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn lint_frame(&mut self, frame: u32, key: NodeKey) -> Result<Option<Tail>, Error> {
        self.attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .map_or(Ok(None), |producer| {
                producer.frame(frame, key).map_err(Error::from)
            })
    }
    pub(super) fn lint_owner(&mut self, visit: Visit, frame: u32) -> Result<Option<Tail>, Error> {
        let key = self.visit_key(visit);
        let parent = self.lint_frame(frame, key)?;
        let unit = self.loader.package.unit(key.file);
        let Some(index) = unit
            .owner_indices
            .get(key.node.get() as usize)
            .copied()
            .flatten()
        else {
            return Ok(parent);
        };
        if unit.configured.owners[index.get() as usize - 1]
            .attributes
            .is_empty()
        {
            return Ok(parent);
        }
        let frames = self.frame_mark()?;
        let calls = u32::try_from(self.calls.len())
            .map_err(|_| Error::at(key.file, 0, "lint call index overflow"))?;
        if self.attribute_effects.is_none() {
            let mut producer = Producer::new(frames, calls);
            producer.reserve_diagnostic(key, size_of::<Effects>())?;
            self.attribute_effects = Some(Box::new(Effects {
                removals: Removals::default(),
                policy: Some(producer),
                visibilities: None,
                derives: None,
            }));
        }
        let producer = self
            .attribute_effects
            .get_or_insert_default()
            .policy
            .get_or_insert_with(|| Producer::new(frames, calls));
        producer
            .owner(&self.loader.package, key, parent)
            .map_err(Error::from)
    }
    pub(super) fn visit_key(&self, visit: Visit) -> NodeKey {
        NodeKey {
            file: self.loader.package.modules[visit.module.0 as usize].file,
            node: visit.node,
        }
    }
    pub(super) fn lint_set_frame(
        &mut self,
        frame: u32,
        visit: Visit,
        tail: Option<Tail>,
    ) -> Result<(), Error> {
        let key = self.visit_key(visit);
        if let Some(producer) = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
        {
            producer.set_frame(frame, tail, key)?;
        }
        Ok(())
    }
    fn lint_site(&self, index: usize) -> Result<Site, Error> {
        let call = &self.calls[index];
        let scope = self
            .paths
            .state
            .facts
            .scopes
            .get(call.visit.path_scope.0 as usize)
            .filter(|scope| scope.module == call.visit.module)
            .ok_or_else(|| call.diagnostic("lint call has no actual caller scope"))?;
        Ok(Site {
            call: u32::try_from(index).map_err(|_| call.diagnostic("lint call index overflow"))?,
            owner: NodeKey {
                file: call.file,
                node: call.visit.node,
            },
            capture: call.capture,
            kind: call.grammar.kind(),
            module: call.visit.module,
            scope: call.visit.path_scope,
            scope_parent: scope.parent,
        })
    }
    pub(super) fn lint_save_call(&mut self, index: usize, frame: u32) -> Result<(), Error> {
        if self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.policy.as_ref())
            .is_none()
        {
            return Ok(());
        }
        let site = self.lint_site(index)?;
        let tail = self.lint_frame(frame, site.owner)?;
        if let Some(producer) = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
        {
            producer.call(site, tail)?;
        }
        Ok(())
    }
    pub(super) fn lint_derive_call(&mut self, index: usize, parent: usize) -> Result<(), Error> {
        let tail = self.lint_call_tail(parent)?;
        let site = self.lint_site(index)?;
        if let Some(producer) = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
        {
            producer.call(site, tail)?;
        }
        Ok(())
    }
    pub(super) fn lint_call_tail(&mut self, index: usize) -> Result<Option<Tail>, Error> {
        if self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.policy.as_ref())
            .is_none()
        {
            return Ok(None);
        }
        let site = self.lint_site(index)?;
        self.attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .map_or(Ok(None), |producer| {
                producer.call_tail(site).map_err(Error::from)
            })
    }
    pub(super) fn lint_call_policy(&mut self, index: usize) -> Result<Policy, Error> {
        let tail = self.lint_call_tail(index)?;
        let key = self.lint_site(index)?.owner;
        self.attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .map_or(Ok(Policy::default()), |producer| {
                producer
                    .policy(&self.loader.package, tail, key)
                    .map_err(Error::from)
            })
    }
    pub(super) fn lint_warning(
        &mut self,
        index: usize,
        offset: u32,
    ) -> Result<crate::resolve::package::Warning, Error> {
        let key = self.lint_site(index)?.owner;
        let bytes = 4 * size_of::<crate::resolve::package::Warning>()
            + size_of::<crate::resolve::package::ErrorData>()
            + "duplicated test attribute".len();
        if let Some(producer) = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
        {
            producer.reserve_diagnostic(key, bytes)?;
        }
        let mut diagnostic = Error::at(key.file, offset, "duplicated test attribute");
        diagnostic.introduction = self.calls[index].visit.introduction;
        Ok(crate::resolve::package::Warning {
            code: crate::support::diagnostics::WarningCode::DuplicateMacroAttributes,
            diagnostic: self.diagnostic(diagnostic),
        })
    }
}
