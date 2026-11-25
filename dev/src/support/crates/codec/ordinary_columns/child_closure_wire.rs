//! Version-fourteen source closure headers, bridges and formation inputs.
use super::{BodyRef, DeclarationRef, Input, Writer, crates, g, id, option, range, s, write};
use crate::support::call_traits::CallTrait;

pub(super) const TAG: u8 = 22;

const CALL_TRAITS: [CallTrait; 3] = [CallTrait::FnOnce, CallTrait::FnMut, CallTrait::Fn];

pub(super) fn encode(out: &mut Writer, columns: &s::ClosureColumns) -> Result<(), &'static str> {
    if columns.headers.is_empty() {
        return Err("source closure trailer requires actual closure headers");
    }
    write(out, &columns.headers, header_write)?;
    write(out, &columns.bridges, |out, &row| bridge_write(out, row))?;
    write(out, &columns.formation_inputs, formation_input_write)?;
    out.u32(columns.callable_slots.len())?;
    for &slot in &columns.callable_slots {
        out.u16(slot)?;
    }
    Ok(())
}

pub(super) fn decode(
    read: &mut Input<'_, '_>,
    types: Vec<s::ClosureType>,
) -> Result<Box<s::ClosureColumns>, &'static str> {
    read.reserve(size_of::<s::ClosureColumns>())?;
    let headers = read.list(crates::MAX_FUNCTIONS, 30, header_read)?;
    if headers.is_empty() {
        return Err("source closure trailer requires actual closure headers");
    }
    let bridges = read.list(crates::MAX_COMPONENTS, 7, bridge_read)?;
    let formation_inputs = read.list(crates::MAX_COMPONENTS, 12, formation_input_read)?;
    let callable_slots = read.list(crates::MAX_COMPONENTS, 2, |read| read.reader.u16())?;
    Ok(Box::new(s::ClosureColumns {
        types,
        headers,
        bridges,
        formation_inputs,
        callable_slots,
        anonymous: Vec::new(),
        anonymous_bridges: Vec::new(),
    }))
}

pub(super) fn encode_anonymous(
    out: &mut Writer,
    columns: &s::ClosureColumns,
) -> Result<(), &'static str> {
    if columns.anonymous.is_empty() {
        return Err("source anonymous trailer requires actual anonymous headers");
    }
    write(out, &columns.anonymous, anonymous_header_write)?;
    write(out, &columns.anonymous_bridges, |out, &row| {
        bridge_write(out, row)
    })
}

pub(super) fn decode_anonymous(
    read: &mut Input<'_, '_>,
    columns: &mut s::ClosureColumns,
) -> Result<(), &'static str> {
    let anonymous = read.list(crates::MAX_FUNCTIONS, 17, anonymous_header_read)?;
    if anonymous.is_empty() {
        return Err("source anonymous trailer requires actual anonymous headers");
    }
    let bridges = read.list(crates::MAX_COMPONENTS, 7, bridge_read)?;
    columns.anonymous = anonymous;
    columns.anonymous_bridges = bridges;
    Ok(())
}

fn anonymous_header_write(out: &mut Writer, row: &s::AnonymousHeader) -> Result<(), &'static str> {
    id(out, row.body.0)?;
    id(out, row.owner.0)?;
    id(out, row.binder.0)?;
    let flags = u8::from(row.deferred) | (u8::from(row.output.is_some()) << 1);
    out.u8(flags)?;
    id(out, row.universals)?;
    if let Some(output) = row.output {
        if output.formation.is_some() || output.creator_only_from.is_some() {
            return Err("source anonymous output cannot carry closure formation data");
        }
        id(out, output.revision)?;
        id(out, output.universals)?;
        range(out, output.bridges)?;
    }
    Ok(())
}

fn anonymous_header_read(read: &mut Input<'_, '_>) -> Result<s::AnonymousHeader, &'static str> {
    let body = BodyRef(read.id()?);
    let owner = DeclarationRef(read.id()?);
    let binder = g::BinderRef(read.id()?);
    let flags = read.reader.u8()?;
    if flags & !3 != 0 {
        return Err("unknown source anonymous header flags");
    }
    let universals = read.id()?;
    let output = if flags & 2 != 0 {
        Some(s::ClosureOutput {
            revision: read.id()?,
            universals: read.id()?,
            bridges: read.range()?,
            formation: None,
            creator_only_from: None,
        })
    } else {
        None
    };
    Ok(s::AnonymousHeader {
        body,
        owner,
        binder,
        universals,
        deferred: flags & 1 != 0,
        output,
    })
}

fn call_trait_write(out: &mut Writer, kind: CallTrait) -> Result<(), &'static str> {
    out.u8(match kind {
        CallTrait::FnOnce => 0,
        CallTrait::FnMut => 1,
        CallTrait::Fn => 2,
    })
}

fn call_trait_read(read: &mut Input<'_, '_>) -> Result<CallTrait, &'static str> {
    CALL_TRAITS
        .get(usize::from(read.reader.u8()?))
        .copied()
        .ok_or("unknown source closure call kind")
}

fn header_write(out: &mut Writer, row: &s::ClosureHeader) -> Result<(), &'static str> {
    id(out, row.body.0)?;
    id(out, row.owner.0)?;
    id(out, row.binder.0)?;
    call_trait_write(out, row.call_kind)?;
    let flags = u8::from(row.constant)
        | (u8::from(row.require_result_formation) << 1)
        | (u8::from(row.execution.is_some()) << 2)
        | (u8::from(row.output.is_some()) << 3);
    out.u8(flags)?;
    id(out, row.signature.0)?;
    id(out, row.universals)?;
    id(out, row.inherited)?;
    id(out, row.public_binders)?;
    if let Some(execution) = row.execution {
        id(out, execution.self_ty.0)?;
        id(out, execution.signature.0)?;
        id(out, execution.universals)?;
        call_trait_write(out, execution.receiver)?;
    }
    if let Some(output) = row.output {
        id(out, output.revision)?;
        id(out, output.universals)?;
        range(out, output.bridges)?;
        option(out, output.creator_only_from)?;
        out.u8(u8::from(output.formation.is_some()))?;
        if let Some(formation) = output.formation {
            range(out, formation.inputs)?;
            id(out, formation.inherited)?;
            range(out, formation.callable_slots)?;
        }
    }
    Ok(())
}

fn header_read(read: &mut Input<'_, '_>) -> Result<s::ClosureHeader, &'static str> {
    let body = BodyRef(read.id()?);
    let owner = DeclarationRef(read.id()?);
    let binder = g::BinderRef(read.id()?);
    let call_kind = call_trait_read(read)?;
    let flags = read.reader.u8()?;
    if flags & !15 != 0 {
        return Err("unknown source closure header flags");
    }
    let signature = g::TypeRef(read.id()?);
    let universals = read.id()?;
    let inherited = read.id()?;
    let public_binders = read.id()?;
    let execution = if flags & 4 != 0 {
        Some(s::ClosureExecution {
            self_ty: g::TypeRef(read.id()?),
            signature: g::TypeRef(read.id()?),
            universals: read.id()?,
            receiver: call_trait_read(read)?,
        })
    } else {
        None
    };
    let output = if flags & 8 != 0 {
        let revision = read.id()?;
        let universals = read.id()?;
        let bridges = read.range()?;
        let creator_only_from = read.option()?;
        let formation = if read.boolean()? {
            Some(s::ClosureFormation {
                inputs: read.range()?,
                inherited: read.id()?,
                callable_slots: read.range()?,
            })
        } else {
            None
        };
        Some(s::ClosureOutput {
            revision,
            universals,
            bridges,
            formation,
            creator_only_from,
        })
    } else {
        None
    };
    Ok(s::ClosureHeader {
        body,
        owner,
        binder,
        call_kind,
        constant: flags & 1 != 0,
        signature,
        universals,
        inherited,
        public_binders,
        execution,
        output,
        require_result_formation: flags & 2 != 0,
    })
}

fn bridge_write(out: &mut Writer, row: s::ClosureBridge) -> Result<(), &'static str> {
    out.u16(row.slot)?;
    id(out, row.parent.0)?;
    out.u8(u8::from(row.output))
}

fn bridge_read(read: &mut Input<'_, '_>) -> Result<s::ClosureBridge, &'static str> {
    Ok(s::ClosureBridge {
        slot: read.reader.u16()?,
        parent: s::RegionRef(read.id()?),
        output: read.boolean()?,
    })
}

fn formation_input_write(
    out: &mut Writer,
    row: &s::ClosureFormationInput,
) -> Result<(), &'static str> {
    id(out, row.root)?;
    id(out, row.child.0)?;
    id(out, row.parent.0)
}

fn formation_input_read(
    read: &mut Input<'_, '_>,
) -> Result<s::ClosureFormationInput, &'static str> {
    Ok(s::ClosureFormationInput {
        root: read.id()?,
        child: g::TypeRef(read.id()?),
        parent: g::TypeRef(read.id()?),
    })
}
