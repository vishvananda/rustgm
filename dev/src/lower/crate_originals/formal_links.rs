//! Named formals follow the actual initializing Read/Constrain/Store chain once.
use super::{Action, Body, DefId, DefKind, Export, LocalId, LocalKind, Place, Rvalue, ValueId};
pub(super) fn record(export: &mut Export<'_>, body: &Body) -> Result<(), &'static str> {
    export.reserve(
        body.values
            .len()
            .saturating_mul(size_of::<Option<LocalId>>())
            .saturating_add(
                body.locals
                    .len()
                    .saturating_mul(size_of::<Option<ValueId>>()),
            ),
    )?;
    let mut origins = Vec::with_capacity(body.values.len());
    for value in &body.values {
        export.charge(1)?;
        origins.push(match value.kind {
            Rvalue::Read(Place::Local(local)) => Some(local),
            Rvalue::Constrain(value) => *origins
                .get(value.0 as usize)
                .ok_or("formal value graph is cyclic")?,
            _ => None,
        });
    }
    let mut stores = vec![None; body.locals.len()];
    for statement in &body.statements {
        export.charge(1)?;
        if let Action::Store {
            place: Place::Local(local),
            value,
        } = statement.action
            && stores[local.0 as usize].is_none()
        {
            stores[local.0 as usize] = Some(value);
        }
    }
    for (index, local) in body.locals.iter().enumerate() {
        export.charge(1)?;
        let Some(definition) = local.definition else {
            continue;
        };
        if export.report.resolution.definitions[definition.0 as usize].kind != DefKind::Parameter {
            continue;
        }
        let binding = LocalId(Export::ordinal(index)?);
        let input = if matches!(local.kind, LocalKind::Parameter(_)) {
            binding
        } else {
            let value = stores[index].ok_or("named formal has no actual initializing store")?;
            origins[value.0 as usize]
                .ok_or("named formal initializer has no actual input origin")?
        };
        let LocalKind::Parameter(ordinal) = body.locals[input.0 as usize].kind else {
            return Err("named formal initializer belongs to another local");
        };
        export.reserve(4 * size_of::<(DefId, (u32, u32, u32))>())?;
        if export
            .formals
            .insert(definition, (input.0, binding.0, ordinal))
            .is_some()
        {
            return Err("generic formal declaration occurs in another original body");
        }
    }
    Ok(())
}
