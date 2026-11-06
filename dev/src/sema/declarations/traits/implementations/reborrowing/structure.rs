//! Ephemeral full-application fields are data, never impl or operation permission.
use crate::{
    sema::{members::Selection, model::Declarations},
    support::retained::Growth,
    types::{
        GenericArgs, ItemId, Kind, ListId, Store, TypeId, arguments::Argument, substitute::Bound,
    },
};

#[derive(Clone, Copy)]
pub(in crate::sema) struct Field {
    pub selection: Selection,
    pub ty: TypeId,
    pub declared: TypeId,
}

pub(in crate::sema) struct StructView {
    pub ty: TypeId,
    pub fields: Vec<Field>,
}
const _: [(); 16] = [(); size_of::<Field>()];
const _: [(); 32] = [(); size_of::<StructView>()];

struct Declared {
    owner: ItemId,
    arguments: GenericArgs,
    normal: ListId,
    raw: ListId,
    count: usize,
}

fn declaration(facts: &Declarations, ty: TypeId) -> Result<Declared, &'static str> {
    let Some(Kind::Adt(owner, arguments)) = facts.types.kind(ty) else {
        return Err("reborrowing implementation requires an actual struct Self");
    };
    if facts
        .types
        .arguments(arguments)
        .ok_or("reborrowing struct has no complete actual arguments")?
        .filter(|argument| matches!(argument, Argument::Lifetime(_)))
        .count()
        != 1
    {
        return Err("reborrowing struct requires exactly one actual lifetime argument");
    }
    let definition = facts
        .aggregates
        .get(owner)
        .ok_or("reborrowing struct declaration is pending")?;
    if definition.item != owner || definition.kind != crate::types::aggregate::Kind::Struct {
        return Err("reborrowing implementation requires a genuine struct owner");
    }
    let [variant] = facts.aggregates.variants(definition) else {
        return Err("reborrowing struct does not have its actual single field group");
    };
    let normal = variant.fields;
    let raw = facts.unreduced.fields(owner, 0).unwrap_or(normal);
    let count = facts
        .types
        .list(normal)
        .ok_or("reborrowing struct field declarations are missing")?
        .len();
    if facts
        .types
        .list(raw)
        .is_none_or(|fields| fields.len() != count)
    {
        return Err("reborrowing raw fields differ from the actual declaration coverage");
    }
    Ok(Declared {
        owner,
        arguments,
        normal,
        raw,
        count,
    })
}

fn member_start(facts: &Declarations, owner: ItemId, count: usize) -> Result<u32, &'static str> {
    let [variant] = facts.members.variants(owner) else {
        return Err("reborrowing struct has no actual member identities");
    };
    let group = facts
        .members
        .variants
        .get(variant.0 as usize)
        .ok_or("reborrowing struct member group is missing")?;
    if group.owner != owner
        || group.ordinal != 0
        || group
            .end
            .checked_sub(group.start)
            .and_then(|n| usize::try_from(n).ok())
            != Some(count)
    {
        return Err("reborrowing struct field identities differ from its declaration");
    }
    Ok(group.start)
}

pub(in crate::sema) fn view(
    facts: &mut Declarations,
    ty: TypeId,
) -> Result<StructView, &'static str> {
    let Declared {
        owner,
        arguments,
        normal,
        raw,
        count,
    } = declaration(facts, ty)?;
    let start = member_start(facts, owner, count)?;
    let pay = crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "reborrowing fields lack their completed source payer")?;
    let growth = Growth::vector::<Field>(0, 0, count);
    pay.charge(
        growth.work.saturating_add(1),
        growth
            .bytes
            .saturating_add(size_of::<StructView>() + size_of::<Bound>()),
    )?;
    let mut fields = Vec::new();
    fields
        .try_reserve(count)
        .map_err(|_| "reborrowing field scratch allocation failed")?;
    let charge = |w, b| pay.charge(w, b);
    let mut fold = if count == 0 {
        None
    } else {
        Some(Bound::joint_charged(
            &facts.types,
            owner,
            arguments,
            &charge,
        )?)
    };
    for ordinal in 0..count {
        charge(1, 0)?;
        let ty = facts
            .types
            .list(normal)
            .and_then(|fields| fields.get(ordinal))
            .copied()
            .ok_or("reborrowing actual field type is missing")?;
        let declared = facts
            .types
            .list(raw)
            .and_then(|fields| fields.get(ordinal))
            .copied()
            .ok_or("reborrowing declared field type is missing")?;
        let fold = fold
            .as_mut()
            .ok_or("reborrowing fields lost their actual argument fold")?;
        let ty = field_type(fold, &mut facts.types, ty, &charge)?;
        let declared = field_type(fold, &mut facts.types, declared, &charge)?;
        // Only the actual PhantomData nominal is excluded. Other zero-sized
        // fields remain real data fields and must satisfy ordinary obligations.
        if facts
            .aggregates
            .phantom_argument(&facts.types, declared)?
            .is_some()
        {
            continue;
        }
        let ordinal = u32::try_from(ordinal).map_err(|_| "reborrowing field ordinal overflow")?;
        let field = start
            .checked_add(ordinal)
            .ok_or("reborrowing field identity overflow")?;
        let field = crate::sema::members::FieldId(field);
        if facts.members.ordinal(field) != Some(ordinal) {
            return Err("reborrowing field changed its actual member ordinal");
        }
        fields.push(Field {
            selection: Selection::Nominal(field),
            ty,
            declared,
        });
    }
    Ok(StructView { ty, fields })
}

fn field_type(
    fold: &mut Bound,
    types: &mut Store,
    ty: TypeId,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<TypeId, &'static str> {
    match fold.argument_inside_charged(types, Argument::Type(ty), 0, charge)? {
        Argument::Type(ty) => Ok(ty),
        _ => Err("reborrowing field substitution returned a different argument kind"),
    }
}
