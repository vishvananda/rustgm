//! Publish portable hidden types without mixing different precise capture lists.
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::model::Declarations,
    types::{
        GenericArgs, Kind,
        opaque::{OpaqueId, RegionChoice},
        substitute::Bound,
    },
};

impl Declarations {
    /// A body without free hidden lifetimes publishes its hidden type itself.
    /// Applications that an earlier storage preparation parked for this
    /// identity are replayed by the next one.
    pub(crate) fn wake_opaque_applications(&mut self, id: OpaqueId) -> Result<(), &'static str> {
        match self.opaques.storage_preparation.as_mut() {
            Some(preparation) => preparation.wake(id),
            None => Ok(()),
        }
    }
    pub(crate) fn close_opaque_results(
        &mut self,
        report: &Report,
        owner: DefId,
        equalities: &[RegionChoice],
    ) -> Result<(), &'static str> {
        if let Some(error) = self
            .opaques
            .storage_preparation
            .as_ref()
            .and_then(|p| p.failed)
        {
            return Err(error.message);
        }
        let Some(ids) = self.opaques.owners.get(&owner) else {
            return Ok(());
        };
        for &id in ids {
            if self.is_named_opaque(id) {
                report.package.pay_named_opaque(1, 0)?;
                self.require_named_application(owner, id)?;
            }
        }
        if ids.iter().all(|id| {
            self.opaques
                .declarations
                .get(id)
                .is_some_and(|d| d.hidden.is_some())
        }) {
            return Ok(());
        }
        let count = ids.len();
        let mut mappings: HashMap<_, HashMap<_, _>> = HashMap::default();
        for &RegionChoice {
            captures,
            region,
            choice,
        } in equalities
        {
            if let Some(previous) = mappings.entry(captures).or_default().insert(region, choice)
                && previous != choice
            {
                return Err("inconsistent opaque lifetime choices");
            }
        }
        let mut folders: HashMap<GenericArgs, Bound> = mappings
            .into_iter()
            .map(|(captures, mapping)| {
                Bound::close_projection_regions(1, 0, mapping).map(|fold| (captures, fold))
            })
            .collect::<Result<_, _>>()?;
        let mut closed = Bound::close_projection_regions(1, 0, HashMap::default())?;
        for index in 0..count {
            let id = self.opaques.owners[&owner][index];
            self.close_opaque_result(report, owner, id, &mut folders, &mut closed)?;
        }
        Ok(())
    }
    fn close_opaque_result(
        &mut self,
        report: &Report,
        owner: DefId,
        id: OpaqueId,
        folders: &mut HashMap<GenericArgs, Bound>,
        closed: &mut Bound,
    ) -> Result<(), &'static str> {
        let declaration = self
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing opaque declaration")?;
        if declaration.hidden.is_some() {
            return Ok(());
        }
        let Some(inferred) = declaration.inferred else {
            return Ok(());
        };
        let application = self
            .named_opaque_application(owner, id)
            .unwrap_or(declaration.application);
        let Some(Kind::Opaque(_, captures)) = self.types.kind(application) else {
            return Err("invalid opaque capture context");
        };
        let folder = folders.get_mut(&captures).unwrap_or(closed);
        let hidden = folder.apply(&mut self.types, inferred)?;
        let hidden = self.close_named_hidden(report, owner, id, hidden)?;
        self.compare_named_hidden(id, hidden)?;
        let declaration = self
            .opaques
            .declarations
            .get_mut(&id)
            .ok_or("missing opaque declaration")?;
        declaration.check_hidden_captures(&self.types, hidden)?;
        declaration.hidden = Some(hidden);
        if self.is_named_opaque(id) {
            let declaration = self
                .opaques
                .declarations
                .get_mut(&id)
                .ok_or("missing named declaration")?;
            declaration.inferred = Some(hidden);
            declaration.free_regions.clear();
        }
        self.wake_opaque_applications(id)
    }
}
