//! Publish portable hidden types without mixing different precise capture lists.
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{GenericArgs, Kind, opaque::RegionChoice, substitute::Bound},
};

impl Declarations {
    pub(crate) fn close_opaque_results(
        &mut self,
        owner: DefId,
        equalities: &[RegionChoice],
    ) -> Result<(), &'static str> {
        if let Some(error) = self
            .opaques
            .storage_preparation
            .as_ref()
            .and_then(|preparation| preparation.failed)
        {
            return Err(error);
        }
        let Some(ids) = self.opaques.owners.get(&owner) else {
            return Ok(());
        };
        if ids.iter().all(|id| {
            self.opaques
                .declarations
                .get(id)
                .is_some_and(|declaration| declaration.hidden.is_some())
        }) {
            return Ok(());
        }
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
        for id in ids {
            let declaration = self
                .opaques
                .declarations
                .get_mut(id)
                .ok_or("missing opaque declaration")?;
            if declaration.hidden.is_some() {
                continue;
            }
            if let Some(inferred) = declaration.inferred {
                let Some(Kind::Opaque(_, captures)) = self.types.kind(declaration.application)
                else {
                    return Err("invalid opaque capture context");
                };
                let folder = folders.get_mut(&captures).unwrap_or(&mut closed);
                let hidden = folder.apply(&mut self.types, inferred)?;
                declaration.check_hidden_captures(&self.types, hidden)?;
                declaration.hidden = Some(hidden);
                if let Some(preparation) = self.opaques.storage_preparation.as_mut() {
                    preparation.wake(*id)?;
                }
            }
        }
        Ok(())
    }
}
