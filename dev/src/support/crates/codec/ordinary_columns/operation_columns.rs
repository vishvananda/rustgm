//! The ordinary move profile retains complete CFG, cleanup and owned-value rows.
use super::{
    BodyRef, DeclarationRef, Input, Writer, allocation_columns as allocations,
    cfg_columns as storage, crates, g, id, index_write, invoked_rows, s, write, write_copy,
};

pub(super) fn original_write(out: &mut Writer, row: &s::Original) -> Result<(), &'static str> {
    for value in [row.body.0, row.declaration.0, row.result.0, row.sites] {
        id(out, value)?;
    }
    out.u8(match row.context {
        s::CallContext::Runtime => 0,
        s::CallContext::ConstDefinition => 1,
        s::CallContext::ClosedConst => 2,
    })?;
    write(out, &row.variables, allocations::owned_write)?;
    write(out, &row.locals, storage::local_write)?;
    write(out, &row.parameter_bindings, storage::parameter_write)?;
    write(out, &row.projections, allocations::projection_write)?;
    write(out, &row.values, value_write)?;
    write(out, &row.statements, statement_write)?;
    write(out, &row.blocks, storage::block_write)?;
    write(out, &row.scopes, storage::scope_write)?;
    write(out, &row.exits, storage::exit_write)?;
    write(out, &row.temporaries, storage::temporary_write)?;
    out.u8(u8::from(row.owned.is_some()))?;
    if let Some(owned) = &row.owned {
        write(out, &owned.created, storage::temporary_write)?;
        write(out, &owned.returned, release_write)?;
    }
    write_copy(out, &row.arguments, index_write)?;
    write_copy(out, &row.openings, super::recipe_columns::root_region_write)?;
    write(out, &row.calls, invoked_rows::write)
}
pub(super) fn original_read(read: &mut Input<'_, '_>) -> Result<s::Original, &'static str> {
    let body = BodyRef(read.id()?);
    let declaration = DeclarationRef(read.id()?);
    let result = g::TypeRef(read.id()?);
    let sites = read.id()?;
    let context = match read.reader.u8()? {
        0 => s::CallContext::Runtime,
        tag => {
            read.require_provided("source defining context requires a wider source profile")?;
            match tag {
                1 => s::CallContext::ConstDefinition,
                2 => s::CallContext::ClosedConst,
                _ => return Err("unknown source defining call context"),
            }
        }
    };
    let variables = read.provided_list(crates::MAX_COMPONENTS, 6, allocations::owned_read)?;
    let locals = read.list(crates::MAX_COMPONENTS, 16, storage::local_read)?;
    let parameter_bindings = read.list(crates::MAX_COMPONENTS, 12, storage::parameter_read)?;
    let projections =
        read.provided_list(crates::MAX_COMPONENTS, 14, allocations::projection_read)?;
    let values = read.list(crates::MAX_COMPONENTS, 5, value_read)?;
    let statements = read.list(crates::MAX_COMPONENTS, 9, statement_read)?;
    let blocks = read.list(crates::MAX_COMPONENTS, 13, storage::block_read)?;
    let scopes = read.list(crates::MAX_COMPONENTS, 6, storage::scope_read)?;
    let exits = read.list(crates::MAX_COMPONENTS, 15, storage::exit_read)?;
    let temporaries = read.list(crates::MAX_COMPONENTS, 21, storage::temporary_read)?;
    let owned = if read.boolean()? {
        Some(s::OwnedValues {
            created: read.list(crates::MAX_COMPONENTS, 21, storage::temporary_read)?,
            returned: read.list(crates::MAX_COMPONENTS, 17, release_read)?,
        })
    } else {
        None
    };
    let arguments = read.call_list(crates::MAX_COMPONENTS, 4, Input::id)?;
    let openings = read.provided_list(
        crates::MAX_COMPONENTS,
        4,
        super::recipe_columns::root_region_read,
    )?;
    let calls = read.call_list(crates::MAX_COMPONENTS, 19, invoked_rows::read)?;
    Ok(s::Original {
        body,
        declaration,
        result,
        sites,
        context,
        variables,
        locals,
        parameter_bindings,
        projections,
        values,
        statements,
        blocks,
        scopes,
        exits,
        temporaries,
        owned,
        arguments,
        openings,
        calls,
    })
}
pub(super) fn place_write(out: &mut Writer, place: s::Place) -> Result<(), &'static str> {
    let (tag, index) = match place {
        s::Place::Local(index) => (0, index),
        s::Place::Projection(index) => (1, index),
    };
    out.u8(tag)?;
    id(out, index)
}
pub(super) fn place_read(read: &mut Input<'_, '_>) -> Result<s::Place, &'static str> {
    match read.reader.u8()? {
        0 => Ok(s::Place::Local(read.id()?)),
        1 => {
            read.require_provided("source projection requires a wider source profile")?;
            Ok(s::Place::Projection(read.id()?))
        }
        _ => Err("source projection requires a wider source profile"),
    }
}
fn value_write(out: &mut Writer, row: &s::Value) -> Result<(), &'static str> {
    id(out, row.ty.0)?;
    match row.kind {
        s::ValueKind::Unit => out.u8(0),
        s::ValueKind::Read(place) => {
            out.u8(1)?;
            place_write(out, place)
        }
        s::ValueKind::Constrain(value) => {
            out.u8(2)?;
            id(out, value)
        }
        s::ValueKind::Borrow(place) => {
            out.u8(3)?;
            place_write(out, place)
        }
        s::ValueKind::Function(application) => {
            out.u8(4)?;
            id(out, application.0)
        }
        s::ValueKind::Call {
            signature,
            callee,
            arguments,
        } => {
            out.u8(5)?;
            id(out, signature.0)?;
            id(out, callee)?;
            super::range(out, arguments)
        }
    }
}
fn value_read(read: &mut Input<'_, '_>) -> Result<s::Value, &'static str> {
    let ty = g::TypeRef(read.id()?);
    let kind = match read.reader.u8()? {
        0 => s::ValueKind::Unit,
        1 => s::ValueKind::Read(place_read(read)?),
        2 => s::ValueKind::Constrain(read.id()?),
        3 => {
            read.require_provided("unknown source ordinary value operation")?;
            s::ValueKind::Borrow(place_read(read)?)
        }
        tag @ (4 | 5) => {
            read.require_calls("unknown source ordinary value operation")?;
            match tag {
                4 => s::ValueKind::Function(s::FunctionApplicationRef(read.id()?)),
                _ => s::ValueKind::Call {
                    signature: g::TypeRef(read.id()?),
                    callee: read.id()?,
                    arguments: read.range()?,
                },
            }
        }
        _ => return Err("unknown source ordinary value operation"),
    };
    Ok(s::Value { ty, kind })
}
fn statement_write(out: &mut Writer, row: &s::Statement) -> Result<(), &'static str> {
    id(out, row.site)?;
    match row.action {
        s::Action::StorageLive(local) => {
            out.u8(0)?;
            id(out, local)
        }
        s::Action::StorageDead(local) => {
            out.u8(1)?;
            id(out, local)
        }
        s::Action::StorageEnd { first, last } => {
            out.u8(2)?;
            id(out, first)?;
            id(out, last)
        }
        s::Action::Evaluate(value) => {
            out.u8(3)?;
            id(out, value)
        }
        s::Action::Store { place, value } => {
            out.u8(4)?;
            place_write(out, place)?;
            id(out, value)
        }
        s::Action::CheckInitialized(place) => {
            out.u8(5)?;
            place_write(out, place)
        }
        s::Action::Mention(place) => {
            out.u8(6)?;
            place_write(out, place)
        }
        s::Action::Project(projection) => {
            out.u8(7)?;
            id(out, projection)
        }
    }
}
fn statement_read(read: &mut Input<'_, '_>) -> Result<s::Statement, &'static str> {
    let site = read.id()?;
    let action = match read.reader.u8()? {
        0 => s::Action::StorageLive(read.id()?),
        1 => s::Action::StorageDead(read.id()?),
        2 => s::Action::StorageEnd {
            first: read.id()?,
            last: read.id()?,
        },
        3 => s::Action::Evaluate(read.id()?),
        4 => s::Action::Store {
            place: place_read(read)?,
            value: read.id()?,
        },
        5 => s::Action::CheckInitialized(place_read(read)?),
        6 => s::Action::Mention(place_read(read)?),
        7 => {
            read.require_provided("unknown source ordinary statement operation")?;
            s::Action::Project(read.id()?)
        }
        _ => return Err("unknown source ordinary statement operation"),
    };
    Ok(s::Statement { site, action })
}
fn release_write(out: &mut Writer, row: &s::Release) -> Result<(), &'static str> {
    id(out, row.value)?;
    storage::location_write(out, row.location)?;
    id(out, row.exits_before)
}
fn release_read(read: &mut Input<'_, '_>) -> Result<s::Release, &'static str> {
    Ok(s::Release {
        value: read.id()?,
        location: storage::location_read(read)?,
        exits_before: read.id()?,
    })
}
