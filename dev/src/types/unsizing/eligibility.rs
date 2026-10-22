//! Projected tails retain every eligible argument; actual field conversion needs proof.
use super::{Definitions, ItemId, Kind, Rules, Store, Uses};
use crate::support::hash::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Candidates {
    tails: HashMap<ItemId, bool>,
    slots: HashMap<ItemId, Vec<u32>>,
}
impl Rules {
    /// These candidates are not unsizing permissions. Only parameters confined
    /// to the last field may change; that field's normalized types must unsize.
    /// # Errors
    /// Rejects incomplete declarations, malformed parameters and exhausted work.
    pub fn projected_parameters(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        item: ItemId,
    ) -> Result<&[u32], &'static str> {
        if !self.projected.slots.contains_key(&item) {
            let mut slots = Vec::new();
            if self.projected_tail(types, definitions, item)? {
                let definition = definitions
                    .get(item)
                    .ok_or("missing projected unsizing owner")?;
                let fields = definitions
                    .variants(definition)
                    .first()
                    .and_then(|variant| types.list(variant.fields))
                    .ok_or("missing projected unsizing fields")?;
                if let Some((&tail, prefix)) = fields.split_last() {
                    for &field in prefix {
                        self.exclude(types, field)?;
                    }
                    let mut uses = Uses::default();
                    uses.collect(types, tail)?;
                    self.charge(uses.steps)?;
                    for id in uses.parameters.into_iter().chain(uses.constants) {
                        let parameter = types
                            .parameter(id)
                            .ok_or("invalid projected unsizing parameter")?;
                        if parameter.owner == item
                            && !self.excluded.parameters.contains(&id)
                            && !self.excluded.constants.contains(&id)
                        {
                            slots.push(parameter.index);
                        }
                    }
                    slots.sort_unstable();
                }
            }
            self.projected.slots.insert(item, slots);
        }
        self.projected
            .slots
            .get(&item)
            .map(Vec::as_slice)
            .ok_or("missing projected unsizing candidates")
    }
    fn projected_tail(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        mut item: ItemId,
    ) -> Result<bool, &'static str> {
        let mut seen = HashSet::default();
        let result = loop {
            self.charge(1)?;
            if let Some(&result) = self.projected.tails.get(&item) {
                break result;
            }
            if !seen.insert(item) {
                return Err("recursive projected unsizing tail");
            }
            let definition = definitions
                .get(item)
                .ok_or("missing projected unsizing declaration")?;
            if definition.kind != super::aggregate::Kind::Struct {
                break false;
            }
            let tail = definitions
                .variants(definition)
                .first()
                .and_then(|variant| types.list(variant.fields))
                .and_then(|fields| fields.last())
                .copied();
            let Some(tail) = tail else {
                break false;
            };
            if types.contains_projections(tail) == Some(true) {
                break true;
            }
            let Some(Kind::Adt(child, _)) = types.kind(tail) else {
                break false;
            };
            item = child;
        };
        self.projected
            .tails
            .extend(seen.into_iter().map(|item| (item, result)));
        Ok(result)
    }
}
