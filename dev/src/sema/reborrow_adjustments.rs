//! Completed source value reborrows retain their real body and type-store owner.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::model::Declarations,
    support::{bindings::LangItem, hash::HashMap, retained::Growth},
    types::{TypeId, arena::Identity},
};

#[derive(Clone, Copy)]
pub(crate) struct Field {
    pub source: crate::sema::members::Selection,
    pub target: crate::sema::members::Selection,
    pub source_ty: TypeId,
    pub target_ty: TypeId,
}
#[derive(Clone, Copy)]
pub(crate) struct Adjustment {
    pub body: NodeKey,
    pub source: TypeId,
    pub target: TypeId,
    pub borrow: TypeId,
    pub mutable: bool,
    pub declaration: DefId,
    pub field: Option<Field>,
}
pub(crate) struct Adjustments {
    arena: Identity,
    entries: HashMap<NodeKey, Adjustment>,
}
impl Adjustments {
    pub(crate) fn get(
        &self,
        facts: &Declarations,
        node: NodeKey,
    ) -> Result<Option<Adjustment>, &'static str> {
        if !facts.types.has_identity(&self.arena) {
            return Err("value reborrow belongs to another type store");
        }
        let Some(row) = self.entries.get(&node).copied() else {
            return Ok(None);
        };
        let role = if row.mutable {
            LangItem::Reborrow
        } else {
            LangItem::CoerceShared
        };
        if facts
            .source_bindings
            .as_deref()
            .ok_or("value reborrow lost its source catalog")?
            .validated_language(role)?
            .is_none_or(|d| d.definition != row.declaration)
        {
            return Err("value reborrow changed its actual completed source role");
        }
        Ok(Some(row))
    }
}

pub(in crate::sema) fn retain(
    facts: &mut Declarations,
    node: NodeKey,
    adjustment: Adjustment,
) -> Result<(), &'static str> {
    if facts.types.existing_arena_identity().is_none() {
        facts.charge_caller_inputs(1, Identity::allocation_bytes())?;
        facts.types.arena_identity();
    }
    let arena = facts
        .types
        .existing_arena_identity()
        .ok_or("value reborrow lost its prepaid type-store identity")?;
    let previous = facts
        .source_bindings
        .as_deref()
        .ok_or("value reborrow requires its actual source catalog")?
        .value_reborrows
        .as_deref();
    if previous.is_some_and(|p| !facts.types.has_identity(&p.arena)) {
        return Err("value reborrow inventory belongs to another type store");
    }
    let (length, capacity) = previous.map_or((0, 0), |p| (p.entries.len(), p.entries.capacity()));
    let growth = Growth::map::<NodeKey, Adjustment>(length, capacity, 1);
    facts.charge_caller_inputs(
        growth.work.saturating_add(1),
        growth.bytes.saturating_add(if previous.is_none() {
            size_of::<Adjustments>()
        } else {
            0
        }),
    )?;
    let entries = &mut facts
        .source_bindings
        .as_deref_mut()
        .ok_or("value reborrow lost its actual source catalog")?
        .value_reborrows
        .get_or_insert_with(|| {
            Box::new(Adjustments {
                arena: arena.clone(),
                entries: HashMap::default(),
            })
        })
        .entries;
    entries
        .try_reserve(1)
        .map_err(|_| "value reborrow inventory allocation failed")?;
    entries.insert(node, adjustment);
    Ok(())
}
