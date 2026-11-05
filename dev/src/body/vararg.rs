//! One actual ellipsis local joins source provenance, initialized storage and emission.
use super::ir::{Action, Body, BodySite, LocalId, LocalKind, Owner};
use crate::{
    sema::{compiler_items, values::Reader},
    types::{Kind, Store, arguments::Argument, callable::Abi},
};

pub(crate) struct Start<'body> {
    body: &'body Body,
    local: LocalId,
    statement: u32,
}
pub(crate) fn variadic(types: &Store, body: &Body) -> bool {
    matches!(body.owner,Owner::Function{signature,..} if matches!(types.kind(signature),Some(Kind::Function{unsafe_:true,convention,..}) if convention.abi==Abi::C && convention.variadic))
}
/// Structural checks are shared by semantic initialization and physical emission.
pub(crate) fn structural(
    types: &Store,
    body: &Body,
) -> Result<Option<(LocalId, u32)>, &'static str> {
    if !variadic(types, body) {
        return Ok(None);
    }
    let mut local = None;
    for (index, value) in body.locals.iter().enumerate() {
        if value.kind == LocalKind::Variadic {
            if local.is_some() {
                return Err("variadic function has more than one ellipsis local");
            }
            local = Some(LocalId(
                u32::try_from(index).map_err(|_| "variadic local identity overflow")?,
            ));
        }
    }
    let local = local.ok_or("variadic function has no actual ellipsis local")?;
    let mut live = None;
    let entry = body
        .blocks
        .first()
        .ok_or("variadic function has no entry block")?;
    for (index, statement) in body.statements.iter().enumerate() {
        if matches!(statement.action,Action::StorageLive(id) if id==local) {
            let index = u32::try_from(index).map_err(|_| "variadic statement identity overflow")?;
            if live.is_some() || index < entry.start || index >= entry.end {
                return Err(
                    "variadic initialization must occur exactly once in the actual entry block",
                );
            }
            live = Some(index);
        }
    }
    Ok(Some((
        local,
        live.ok_or("variadic local has no actual entry StorageLive")?,
    )))
}
impl<'body> Start<'body> {
    pub(crate) fn prepare(
        reader: Reader<'_>,
        types: &Store,
        body: &'body Body,
    ) -> Result<Option<Self>, &'static str> {
        if !variadic(types, body) {
            return Ok(None);
        }
        reader.vararg_work(16 + body.locals.len() + body.statements.len())?;
        let Some((local, statement)) = structural(types, body)? else {
            return Ok(None);
        };
        let Owner::Function {
            definition,
            signature,
            execution: None,
        } = body.owner
        else {
            return Err("variadic initialization requires its actual original function owner");
        };
        if body.instance.is_some() {
            return Err("generic variadic definition specialization is not implemented");
        }
        let actual = body
            .locals
            .get(local.0 as usize)
            .ok_or("variadic local disappeared")?;
        reader.require_vararg_binding(compiler_items::vararg::BindingRequest {
            definition,
            signature,
            body: body.source,
            parameter: body.syntax_site(actual.source)?,
            ty: actual.ty,
            binding: actual.definition,
        })?;
        Ok(Some(Self {
            body,
            local,
            statement,
        }))
    }
    pub(crate) const fn local(&self) -> LocalId {
        self.local
    }
    pub(crate) fn publication(
        &self,
        body: &Body,
        local: LocalId,
        site: BodySite,
        before: u32,
    ) -> Result<(), &'static str> {
        if !std::ptr::eq(body, self.body)
            || local != self.local
            || before != self.statement
            || body
                .statements
                .get(before as usize)
                .is_none_or(|statement| {
                    statement.source != site
                        || !matches!(statement.action,Action::StorageLive(id) if id==local)
                })
        {
            return Err("variadic initialization plan belongs to another body or storage event");
        }
        Ok(())
    }
}
pub(crate) fn local_region(
    types: &Store,
    body: &Body,
    id: LocalId,
) -> Result<crate::types::RegionId, &'static str> {
    let ty = body
        .locals
        .get(id.0 as usize)
        .ok_or("variadic local is missing")?
        .ty;
    let Some(Kind::Adt(_, arguments)) = types.kind(ty) else {
        return Err("variadic local has no actual nominal type");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("variadic local arguments are missing")?;
    let (Some(Argument::Lifetime(region)), None) = (arguments.next(), arguments.next()) else {
        return Err("variadic local requires its actual sole lifetime argument");
    };
    Ok(region)
}

pub(crate) struct ReadCall {
    pub selected: compiler_items::vararg::Read,
    pub input: super::ir::ValueId,
    pub list: crate::types::TypeId,
}
pub(crate) fn read_call(
    reader: Reader<'_>,
    types: &Store,
    body: &Body,
    value: &super::ir::Value,
    callable: crate::types::TypeId,
) -> Result<Option<ReadCall>, &'static str> {
    let Some(selected) = reader.source_vararg(callable, body.source)? else {
        return Ok(None);
    };
    let super::ir::Rvalue::Call {
        signature,
        callee,
        arguments,
    } = value.kind
    else {
        return Err("source va_arg requires its actual direct Call");
    };
    let [input] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source va_arg argument range is missing")?
    else {
        return Err("source va_arg requires its actual one argument");
    };
    let (list, _) = reader.vararg_opened(&selected, signature)?;
    let Some(Kind::Function { parameters, .. }) = types.kind(signature) else {
        return Err("source va_arg opened signature disappeared");
    };
    let Some(&[input_type]) = types.list(parameters) else {
        return Err("source va_arg opened input disappeared");
    };
    if value.ty != selected.output
        || body
            .values
            .get(callee.0 as usize)
            .is_none_or(|value| value.ty != callable)
        || body
            .values
            .get(input.0 as usize)
            .is_none_or(|value| value.ty != input_type)
    {
        return Err("source va_arg differs from its complete actual callee, argument or result");
    }
    Ok(Some(ReadCall {
        selected,
        input: *input,
        list,
    }))
}
