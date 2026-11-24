//! Re-prove the caller record and its transparent non-null filename storage.
use super::{Declarations, ItemId, Kind, LangItem, argument, fields, location_fields};
use crate::{
    resolve::model::DefKind,
    sema::compiler_items::{Catalog, Role},
    support::crates::generics::source::DiagnosticRole,
    syntax::item::Shape,
    types::{Constraint, aggregate, arguments::Argument},
};

pub(in crate::sema::compiler_items) fn location(
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(16);
    let item = ItemId(declaration.definition.0);
    record(facts, declaration, aggregate::Repr::default())?;
    let filename = catalog
        .imported_diagnostic(DiagnosticRole::NonNull, DefKind::Struct, steps)?
        .ok_or("imported Location requires the actual NonNull diagnostic identity")?;
    let phantom = catalog
        .role(LangItem::PhantomData)?
        .ok_or("imported Location requires the actual PhantomData role")?;
    location_fields(
        facts,
        fields(facts, item)?,
        ItemId(filename.definition.0),
        ItemId(phantom.definition.0),
    )?;
    nonnull(facts, filename)
}

fn nonnull(facts: &Declarations, role: Role) -> Result<(), &'static str> {
    let item = ItemId(role.definition.0);
    let shape = record(
        facts,
        role,
        aggregate::Repr {
            representation: aggregate::Representation::Transparent,
            ..aggregate::Repr::default()
        },
    )?;
    let ty = facts.definitions[item.0 as usize]
        .ok_or("imported NonNull is missing its canonical type")?;
    let Argument::Type(target) = argument(facts, ty, item)? else {
        return Err("imported NonNull requires its actual type formal");
    };
    let own_formal = matches!(facts.types.kind(target), Some(Kind::Parameter(id)) if facts.types.parameter(id).is_some_and(|p| p.owner == item && p.index == 0));
    let [variant] = facts.aggregates.variants(shape) else {
        return Err("imported NonNull requires exactly one field group");
    };
    let Some([pointer]) = facts.types.list(variant.fields) else {
        return Err("imported NonNull requires its one actual pointer field");
    };
    if !own_formal
        || !matches!(facts.types.kind(*pointer), Some(Kind::Pattern { base, constraint: Constraint::NonNull }) if facts.types.kind(base) == Some(Kind::Pointer { mutable: false, pointee: target }))
    {
        return Err("imported NonNull requires its actual !null const pointer field");
    }
    Ok(())
}

fn record(
    facts: &Declarations,
    declaration: Role,
    repr: aggregate::Repr,
) -> Result<&aggregate::Definition, &'static str> {
    let item = ItemId(declaration.definition.0);
    let [variant] = facts.members.variants(item) else {
        return Err("imported caller site type requires one actual record group");
    };
    let member = &facts.members.variants[variant.0 as usize];
    if declaration.parent.is_some()
        || member.owner != item
        || member.shape != Shape::Record
        || !facts
            .generics
            .nominal_contracts
            .contains(&declaration.definition)
    {
        return Err("imported caller site type requires its module-owned record declaration");
    }
    facts
        .aggregates
        .get(item)
        .filter(|shape| shape.kind == aggregate::Kind::Struct && shape.repr == repr)
        .ok_or("imported caller site type changed its actual struct representation")
}
