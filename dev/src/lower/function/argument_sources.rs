//! Runtime tuple spreading reuses the ordinary checked call argument writer.
use super::{Lower, Representation, Selected, Type, ir};
use crate::lowir::program::Operand;
use crate::types::{Kind, TypeId, instance::Instance};

#[derive(Clone, Copy)]
pub(super) enum Arguments {
    Values(ir::ValueList),
    /// An already evaluated, owned ARG snapshot. Its fields move to G; this
    /// adapter grants neither Copy nor permission to read a different place.
    Tuple {
        owner: TypeId,
        value: Selected,
    },
}
impl Arguments {
    pub(super) fn len(self, lower: &Lower<'_, '_, '_>) -> Result<usize, &'static str> {
        match self {
            Self::Values(list) => lower
                .body
                .arguments
                .get(list.start as usize..list.end as usize)
                .map(<[_]>::len)
                .ok_or("invalid ordinary argument range"),
            Self::Tuple { owner, .. } => {
                let Some(Kind::Tuple(fields)) = lower.context.types.kind(owner) else {
                    return Err("runtime selector lost its actual tuple owner");
                };
                lower
                    .context
                    .types
                    .list(fields)
                    .map(<[_]>::len)
                    .ok_or("runtime selector tuple fields disappeared")
            }
        }
    }
    pub(super) fn get(
        self,
        lower: &mut Lower<'_, '_, '_>,
        index: usize,
    ) -> Result<(TypeId, Selected), &'static str> {
        match self {
            Self::Values(list) => {
                let id = *lower
                    .body
                    .arguments
                    .get(list.start as usize..list.end as usize)
                    .and_then(|ids| ids.get(index))
                    .ok_or("missing ordinary call argument")?;
                let ty = lower
                    .body
                    .values
                    .get(id.0 as usize)
                    .ok_or("ordinary argument value disappeared")?
                    .ty;
                Ok((ty, lower.value(id)?))
            }
            Self::Tuple { owner, value } => tuple_field(lower, owner, value, index),
        }
    }
}

fn tuple_field(
    lower: &mut Lower<'_, '_, '_>,
    owner: TypeId,
    value: Selected,
    index: usize,
) -> Result<(TypeId, Selected), &'static str> {
    let Some(Kind::Tuple(fields)) = lower.context.types.kind(owner) else {
        return Err("runtime selector arguments require their exact tuple identity");
    };
    let expected = *lower
        .context
        .types
        .list(fields)
        .and_then(|f| f.get(index))
        .ok_or("runtime selector tuple index exceeds its actual extent")?;
    let layout = lower.context.layout(owner)?;
    let (ty, offset) = lower.context.selected_field(
        &layout,
        ir::Selection::Tuple(
            u32::try_from(index).map_err(|_| "too many runtime selector tuple fields")?,
        ),
    )?;
    if ty != expected {
        return Err("runtime selector tuple layout differs from its actual field identity");
    }
    let representation = lower.context.representation(ty)?;
    if matches!(representation, Representation::Scalar(Type::Void)) {
        let selected = if matches!(
            lower.context.types.kind(ty),
            Some(Kind::FunctionItem { .. })
        ) {
            Selected::Item(
                lower
                    .body
                    .callable_target(Instance::new(lower.context.types, ty)?),
            )
        } else {
            Selected::Unit
        };
        return Ok((ty, selected));
    }
    let Selected::Object(base) = value else {
        return Err("runtime selector tuple lacks its evaluated owned snapshot");
    };
    let address = lower.offset(base, offset)?;
    let selected = match representation {
        Representation::Scalar(ty) => Selected::Runtime(lower.builder.load(ty, address.operand)?),
        Representation::Object(_) | Representation::ScalarObject(_) => Selected::Object(address),
    };
    Ok((ty, selected))
}

impl Lower<'_, '_, '_> {
    pub(super) fn call_arguments(
        &mut self,
        signature: TypeId,
        callee: Operand,
        arguments: Arguments,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        self.call_arguments_with_caller(signature, callee, arguments, result, None)
    }
    pub(super) fn call_arguments_with_caller(
        &mut self,
        signature: TypeId,
        callee: Operand,
        arguments: Arguments,
        result: TypeId,
        location: Option<crate::lowir::program::Value>,
    ) -> Result<Selected, &'static str> {
        let signature = self
            .context
            .caller_signature(signature, location.is_some())?;
        if matches!(
            self.context.output.signatures.get(signature.0 as usize),
            Some(crate::lowir::program::Signature::Object(_))
        ) || self.context.output.variadic(signature)?
        {
            let Arguments::Values(values) = arguments else {
                return Err("runtime selector callback requires the ordinary Rust ABI");
            };
            return self.boundary_call(signature, callee, values, result);
        }
        let count = arguments.len(self)?;
        if matches!(arguments, Arguments::Tuple { .. }) && count > 1_000_000 {
            return Err("call argument work budget exceeded");
        }
        let representation = self.context.representation(result)?;
        self.arguments.clear();
        let destination = if matches!(representation, Representation::Object(_)) {
            let address = self.allocate(result)?.ok_or("missing result allocation")?;
            let pointer = self.pointer(address)?;
            self.arguments.push(pointer);
            Some(address)
        } else {
            None
        };
        for index in 0..count {
            let (ty, selected) = arguments.get(self, index)?;
            self.push_argument(ty, selected)?;
        }
        if let Some(location) = location {
            self.arguments.push(location);
        }
        let value =
            self.builder
                .call(signature, representation.result(), callee, &self.arguments)?;
        if matches!(representation, Representation::ScalarObject(_)) {
            let destination = self
                .allocate(result)?
                .ok_or("missing nullable result storage")?;
            self.builder.store(
                value.ok_or("missing nullable result word")?,
                destination.operand,
            )?;
            return Ok(Selected::Object(destination));
        }
        Ok(destination.map_or_else(
            || value.map_or(Selected::Unit, Selected::Runtime),
            Selected::Object,
        ))
    }
    fn push_argument(&mut self, ty: TypeId, selected: Selected) -> Result<(), &'static str> {
        let value = match selected {
            Selected::Runtime(value) => value,
            Selected::Object(address) => {
                if let Representation::ScalarObject(storage) = self.context.representation(ty)? {
                    self.builder.load(storage.ty, address.operand)?
                } else {
                    self.pointer(address)?
                }
            }
            Selected::Unit | Selected::Item(_) => return Ok(()),
        };
        self.arguments.push(value);
        Ok(())
    }
}
