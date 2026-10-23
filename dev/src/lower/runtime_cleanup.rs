//! Actual helper bodies consume proved recipes, never their placeholder CFG.
mod empty;
#[cfg(test)]
#[path = "../../tests/invariants/glue_witnesses.rs"]
mod tests;
use super::{
    Error,
    context::{Context, Representation},
};
use crate::{
    body::ir::{Body, BodyKey, LocalKind, Owner},
    lowir::{
        program::{Function, Operand, Signature, SignatureId, Terminator, Value},
        vocabulary::Type,
    },
    resolve::model::DefId,
    sema::{compiler_items::DropGlueSelection, values::Reader},
    types::{
        Kind, Store, TypeId, aggregate::Definitions, callable::Convention, destruction::NeedsDrop,
    },
};
const LIMIT: usize = 64 * 1024 * 1024;

/// One immutable emission invocation. Ordinary paths allocate no query.
pub(super) struct State<'a> {
    types: &'a Store,
    definitions: &'a Definitions,
    definition: Option<DefId>,
    empty: Option<Box<NeedsDrop>>,
}
impl<'a> State<'a> {
    pub(super) fn new(
        types: &'a Store,
        definitions: &'a Definitions,
        reader: Reader<'_>,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            types,
            definitions,
            definition: reader.source_drop_glue_definition()?,
            empty: None,
        })
    }
    pub(super) fn lower(
        &mut self,
        context: &mut Context<'_>,
        index: usize,
        body: &Body,
        native: SignatureId,
    ) -> Result<Option<Function>, Error> {
        if self.definition.is_none() {
            return Ok(None);
        }
        self.emit(context, index, body, native)
            .map_err(|message| Error {
                node: context.source_body_site(index),
                message,
            })
    }
    fn emit(
        &mut self,
        context: &mut Context<'_>,
        index: usize,
        body: &Body,
        native: SignatureId,
    ) -> Result<Option<Function>, &'static str> {
        let Owner::Function { definition, .. } = body.owner else {
            return Ok(None);
        };
        if self.definition != Some(definition) {
            return Ok(None);
        }
        if !std::ptr::eq(self.types, context.types)
            || !std::ptr::eq(self.definitions, context.aggregates)
        {
            return Err("source drop glue query belongs to another structural inventory");
        }
        context.require_source_body(index, body)?;
        let instance = body
            .instance
            .ok_or("source drop glue requires its proved concrete instance")?;
        let selection = context
            .constants
            .source_drop_glue(instance)?
            .ok_or("source drop glue lost its exact actual declaration")?;
        checked(context.types, body, selection)?;
        if context.types.is_sized(selection.element) != Some(true) {
            return Err("unsized source drop glue requires metadata-aware cleanup");
        }
        context.layout(selection.element)?;
        if !matches!(
            context.representation(parameter(context.types, selection.signature)?)?,
            Representation::Scalar(Type::Ptr)
        ) {
            return Err("source drop glue requires its selected thin-reference ABI");
        }
        native_signature(context, native)?;
        let recipe = if let Some(proof) = context.destruction {
            proof.recipes().exposed(selection.element)?
        } else {
            self.dropless(
                context.types,
                context.aggregates,
                selection.element,
                selection.empty_registry,
                LIMIT,
            )?;
            None
        };
        let mut builder = context.output.body_builder();
        let entry = builder.block()?;
        builder.enter(entry)?;
        if let Some(recipe) = recipe {
            let symbol = context.demand_drop(recipe)?;
            let signature = context.output.symbols[symbol.0 as usize].signature;
            builder.call(
                signature,
                Type::Void,
                Operand::Symbol(symbol),
                &[Value {
                    ty: Type::Ptr,
                    operand: Operand::Parameter(0),
                }],
            )?;
        }
        builder.terminate(Terminator::Return(None))?;
        builder.finish().map(Some)
    }
}

/// The consumed body must preserve its immutable proved header, including both
/// signature namespaces. A same-arity public signature is not an owned proof.
pub(super) fn retained_header(retained: &Body, body: &Body) -> Result<(), &'static str> {
    let identity = |owner| match owner {
        Owner::Function {
            definition,
            signature,
            execution,
        } => Some((definition, signature, execution)),
        _ => None,
    };
    if retained.instance != body.instance
        || retained.source != body.source
        || retained.result != body.result
        || identity(retained.owner).is_none()
        || identity(retained.owner) != identity(body.owner)
    {
        return Err("source cleanup differs from its retained proved body");
    }
    Ok(())
}

/// Called only for the actual immutable proved Body entering mechanical lowering.
fn checked(types: &Store, body: &Body, selection: DropGlueSelection) -> Result<(), &'static str> {
    selected_element(types, selection)?;
    let Owner::Function {
        definition,
        signature,
        execution: Some(execution),
    } = body.owner
    else {
        return Err("source drop glue has no actual materialized signature pair");
    };
    if definition != selection.definition
        || body.key() != BodyKey::Declaration(definition)
        || body.instance != Some(selection.instance)
        || signature != selection.signature
        || body.source.source() != Some(selection.source)
        || body.result != TypeId::UNIT
    {
        return Err("source drop glue body belongs to another selected contract");
    }
    let public = parameter(types, signature)?;
    let owned = parameter(types, execution)?;
    if !matches!(
        types.kind(signature),
        Some(Kind::Function {
            bound_regions: 1,
            ..
        })
    ) || !matches!(types.kind(public), Some(Kind::Reference { mutable: true, region, .. })
            if region == crate::types::RegionId::bound(0,0)?)
        || !matches!(
            types.kind(owned),
            Some(Kind::Reference { mutable: true, .. })
        )
    {
        return Err("source drop glue lost its actual mutable-reference input");
    }
    let mut parameters = body.locals.iter().filter_map(|local| {
        if let LocalKind::Parameter(index) = local.kind {
            Some((index, local.ty))
        } else {
            None
        }
    });
    if parameters.next() != Some((0, owned)) || parameters.next().is_some() {
        return Err("source drop glue input differs from its owned signature");
    }
    Ok(())
}
fn selected_element(types: &Store, selection: DropGlueSelection) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = types.kind(selection.instance.ty())
    else {
        return Err("source drop glue selection lost its full function instance");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source drop glue arguments are missing")?;
    if item.0 != selection.definition.0
        || signature != selection.signature
        || arguments.next() != Some(crate::types::arguments::Argument::Type(selection.element))
        || arguments.next().is_some()
    {
        return Err("source drop glue selection belongs to another type or signature");
    }
    Ok(())
}
fn parameter(types: &Store, signature: TypeId) -> Result<TypeId, &'static str> {
    let Some(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        unsafe_: true,
        convention: Convention::RUST,
        ..
    }) = types.kind(signature)
    else {
        return Err("source drop glue lost its unsafe Rust unit contract");
    };
    match types.list(parameters) {
        Some([parameter]) => Ok(*parameter),
        _ => Err("source drop glue requires its one retained input"),
    }
}
fn native_signature(context: &Context<'_>, native: SignatureId) -> Result<(), &'static str> {
    let Some(Signature::Scalar(signature)) = context.output.signatures.get(native.0 as usize)
    else {
        return Err("source drop glue has an unsupported selected call ABI");
    };
    if signature.result != Type::Void
        || signature.noreturn
        || signature.variadic
        || context
            .output
            .parameters
            .get(signature.parameters.start as usize..signature.parameters.end as usize)
            != Some(&[Type::Ptr])
    {
        return Err("source drop glue ABI differs from its checked reference call");
    }
    Ok(())
}
