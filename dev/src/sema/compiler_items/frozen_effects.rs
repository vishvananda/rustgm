//! Frozen imported declaration inputs retain call debt, never execution permission.
use super::{Catalog, calls::Effect};
use crate::{
    resolve::model::DefId,
    sema::declaration_inputs::Input,
    support::{hash::HashMap, retained},
    types::Store,
};

pub(in crate::sema) struct Prepared {
    entries: Box<HashMap<DefId, Effect>>,
}
impl Catalog {
    pub(in crate::sema) fn call_effect_bytes(&self) -> usize {
        self.call_effects.as_deref().map_or(0, |entries| {
            size_of::<HashMap<DefId, Effect>>().saturating_add(retained::map(entries))
        })
    }
}
impl Prepared {
    pub(in crate::sema) fn new<'a>(
        catalog: &Catalog,
        types: &Store,
        count: usize,
        declarations: impl Iterator<Item = (DefId, &'a [Input])>,
        pay: retained::Payment<'_>,
    ) -> Result<Self, &'static str> {
        pay(1, 0)?;
        if count == 0 {
            return Err("imported constant effect plan requires actual declarations");
        }
        if catalog.call_effects.is_some() {
            return Err("imported const effects cannot replace an existing declaration inventory");
        }
        let growth = retained::Growth::map::<DefId, Effect>(0, 0, count);
        pay(
            growth.work,
            growth
                .bytes
                .saturating_add(size_of::<HashMap<DefId, Effect>>()),
        )?;
        let mut entries = HashMap::with_capacity_and_hasher(count, Default::default());
        for (definition, inputs) in declarations {
            pay(1, 0)?;
            let mut effect = Effect::Ordinary;
            for input in inputs {
                pay(2, 0)?;
                if super::calls::conditional_clause(types, input.value)? {
                    effect = Effect::Conditional;
                }
            }
            if entries.len() == count || entries.insert(definition, effect).is_some() {
                return Err("imported const effects changed their unique declaration count");
            }
        }
        if entries.len() != count {
            return Err("imported const effects omitted an actual constant declaration");
        }
        Ok(Self {
            entries: Box::new(entries),
        })
    }

    pub(in crate::sema) fn install(self, catalog: &mut Catalog) {
        catalog.call_effects = Some(self.entries);
    }
}
