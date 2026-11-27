//! The same proof storage lends scoped views between selected raw appends.
use super::{Checked, Error, State, Templates};
use std::rc::Rc;

pub(crate) struct Retained {
    templates: Rc<Templates>,
    state: Option<State>,
}
impl Checked<'_> {
    pub(crate) fn retain(self, actual: Rc<Templates>) -> Result<Retained, Error> {
        if !std::ptr::eq(self.templates, actual.as_ref()) {
            return Err(Error {
                node: None,
                message: "retained proofs require their actual checked template allocation".into(),
            });
        }
        #[cfg(test)]
        eprintln!(
            "original-proof storage checked={} state={} retained={} index={}/{} regions={}/{} loans={}/{} contracts={}/{}",
            size_of::<Checked<'_>>(),
            size_of::<State>(),
            size_of::<Retained>(),
            self.state.index.len(),
            self.state.index.capacity(),
            self.state.regions.bodies.len(),
            self.state.regions.bodies.capacity(),
            self.state.loans.len(),
            self.state.loans.capacity(),
            self.state.contracts.len(),
            self.state.contracts.capacity()
        );
        Ok(Retained {
            templates: actual,
            state: Some(self.state),
        })
    }
}
impl Retained {
    pub(crate) fn append_bound(
        &self,
        input: &super::Contract,
        custom: usize,
    ) -> Result<crate::support::retained::Growth, Error> {
        use crate::support::retained::Growth;
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| error("retained proof storage is already borrowed"))?;
        let mut growth = self.templates.append_original_bound();
        growth.include(Growth::map::<crate::types::ItemId, usize>(
            state.index.len(),
            state.index.capacity(),
            1,
        ));
        growth.include(Growth::vector::<crate::borrow::regions::Body>(
            state.regions.bodies.len(),
            state.regions.bodies.capacity(),
            1,
        ));
        growth.include(Growth::vector::<crate::borrow::loans::Analysis>(
            state.loans.len(),
            state.loans.capacity(),
            1,
        ));
        growth.include(Growth::vector::<super::Contract>(
            state.contracts.len(),
            state.contracts.capacity(),
            1,
        ));
        growth.bytes = growth
            .bytes
            .saturating_add(input.retained_bytes())
            .saturating_add(
                crate::support::retained::new_map::<crate::types::ItemId, ()>(custom)
                    .saturating_mul(3),
            );
        growth.work = growth
            .work
            .saturating_add(custom.saturating_mul(3))
            .saturating_add(1);
        Ok(growth)
    }
    /// Consume the owner on failure; earlier rows are never replayed or exposed partially.
    pub(crate) fn append(
        self,
        original: crate::body::transported_originals::Original,
        prepared: &super::Contract,
        types: &crate::types::Store,
        regions: crate::borrow::regions::Body,
        loans: crate::borrow::loans::Analysis,
    ) -> Result<Self, Error> {
        self.append_with(original, prepared, types, regions, loans, false)
    }
    pub(crate) fn append_synthesized(
        self,
        original: crate::body::transported_originals::Original,
        prepared: &super::Contract,
        types: &crate::types::Store,
        regions: crate::borrow::regions::Body,
        loans: crate::borrow::loans::Analysis,
    ) -> Result<Self, Error> {
        self.append_with(original, prepared, types, regions, loans, true)
    }
    fn append_with(
        mut self,
        original: crate::body::transported_originals::Original,
        prepared: &super::Contract,
        types: &crate::types::Store,
        regions: crate::borrow::regions::Body,
        loans: crate::borrow::loans::Analysis,
        allow_source: bool,
    ) -> Result<Self, Error> {
        let templates = Rc::get_mut(&mut self.templates).ok_or_else(|| {
            error("selected original append requires its uniquely owned checked templates")
        })?;
        templates.require_unique_originals().map_err(error)?;
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| error("retained proof storage is already borrowed"))?;
        state.require_current_store(types).map_err(error)?;
        let template = original.template();
        let raw = original.contract();
        if (!allow_source && template.body.source.source().is_some())
            || template.body.instance.is_some()
            || prepared.key != raw.key
            || prepared.source != raw.source
            || prepared.signature != raw.signature
            || prepared.universals != raw.universals
            || regions.key != template.body.key()
            || regions.source != template.body.source
            || regions.instance != template.body.instance
        {
            return Err(error(
                "selected original proof changed its actual immutable input identity",
            ));
        }
        super::owners::insert(
            template,
            templates.len(),
            &mut state.index,
            &mut state.closures,
            &mut state.anonymous,
        )?;
        crate::borrow::loans::templates::validate(template).map_err(error)?;
        let raw = raw.clone();
        templates.append_original(original).map_err(error)?;
        state.contracts.push(raw);
        state.regions.bodies.push(regions);
        state.loans.push(loans);
        Ok(self)
    }
    pub(crate) fn with_checked<R>(
        &mut self,
        run: impl for<'a> FnOnce(&Checked<'a>) -> R,
    ) -> Result<R, Error> {
        let state = self.state.take().ok_or_else(|| Error {
            node: None,
            message: "retained template proof storage is already borrowed".into(),
        })?;
        let checked = Checked {
            templates: &self.templates,
            state,
        };
        let result = run(&checked);
        self.state = Some(checked.state);
        Ok(result)
    }
}

fn error(message: &str) -> Error {
    Error {
        node: None,
        message: message.into(),
    }
}
