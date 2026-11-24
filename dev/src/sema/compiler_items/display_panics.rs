//! The actual const panic hook describes a declaration; it grants no callable body.
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{
        Kind, Primitive, Store, TypeId, arguments::Argument, callable::Convention,
        instance::Instance,
    },
};
pub(crate) struct Selection {
    pub definition: DefId,
    pub signature: TypeId,
    pub input: TypeId,
    by_value: bool,
}
impl Selection {
    pub(crate) fn matches_call(&self, store: &Store, signature: TypeId) -> bool {
        if self.by_value {
            signature == self.signature
        } else {
            reference_input(store, signature, self.input, false)
        }
    }
    pub(crate) fn matches_input(&self, store: &Store, ty: TypeId) -> bool {
        if self.by_value {
            ty == self.input
        } else {
            matches!(store.kind(ty), Some(Kind::Reference { mutable: false, referent, .. })
                if referent == self.input)
        }
    }

    pub(crate) fn fault(&self, store: &Store) -> &'static str {
        if !self.by_value {
            "constant execution reached source panic_display"
        } else if matches!(
            store.kind(self.input),
            Some(Kind::Reference { referent, .. })
                if referent == TypeId::primitive(Primitive::Str)
        ) {
            "constant execution reached source begin_panic"
        } else {
            "source begin_panic requires a string slice payload during constant execution"
        }
    }
}
fn reference_input(store: &Store, signature: TypeId, input: TypeId, public: bool) -> bool {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result: TypeId::NEVER,
        unsafe_: false,
        convention: Convention::RUST,
    }) = store.kind(signature)
    else {
        return false;
    };
    let Some([parameter]) = store.list(parameters) else {
        return false;
    };
    let Some(Kind::Reference {
        mutable: false,
        referent,
        region,
    }) = store.kind(*parameter)
    else {
        return false;
    };
    referent == input
        && if public {
            bound_regions == 1 && region.bound_parts() == Some((0, 0))
        } else {
            bound_regions == 0 && region.bound_parts().is_none()
        }
}
/// This finite local hook still requires the current direct checked const Call.
pub(crate) fn select(
    report: &Report,
    facts: &Declarations,
    instance: Instance,
    origin: BodyOrigin,
) -> Result<Option<Selection>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(instance.ty())
    else {
        return Ok(None);
    };
    if catalog.uses_imported_roles() {
        for role in [
            LangItem::PanicFmt,
            LangItem::PanicNounwind,
            LangItem::PanicDisplay,
        ] {
            if catalog
                .role(role)?
                .is_some_and(|declaration| declaration.definition.0 == item.0)
            {
                catalog.validated_role(role)?;
                return Err("imported formatting panic requires its checked const operation");
            }
        }
    }
    let Some((role, declaration)) = [LangItem::PanicDisplay, LangItem::BeginPanic]
        .into_iter()
        .find_map(|role| {
            catalog
                .language(role)
                .filter(|declaration| item.0 == declaration.definition.0)
                .map(|declaration| (role, declaration))
        })
    else {
        return Ok(None);
    };
    crate::sema::header_attributes::charge_source_contract_query(facts, 16)
        .map_err(|_| "source const panic declaration work budget exceeded")?;
    if origin.source().is_none() {
        return Err("source const panic requires its actual source caller provenance");
    }
    catalog.require_provider_report(report)?;
    let function = facts
        .function_indices
        .get(&declaration.definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("source const panic requires its completed declaration")?;
    let body = function
        .body
        .ok_or("source const panic lost its actual defining body")?;
    // This authenticates the declaration/header only. Its runtime defining-body
    // context is never used as permission to execute that body during CTFE.
    let by_value = role == LangItem::BeginPanic;
    if by_value {
        if declaration.parent.is_some()
            || function.node != declaration.origin
            || !facts.unchecked_const_body(report, declaration.definition, body)?
        {
            return Err("source const panic lost its current genuine begin_panic role");
        }
    } else if !super::runtime_panics::runtime_body(report, facts, declaration.definition, body)? {
        return Err("source const panic lost its current genuine panic_display role");
    }
    let generics = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .ok_or("source const panic lost its type parameter")?;
    let [parameter] = generics.as_slice() else {
        return Err("source const panic requires its actual single type parameter");
    };
    let Argument::Type(generic) = parameter.argument else {
        return Err("source const panic requires a type parameter");
    };
    let mut args = facts
        .types
        .arguments(arguments)
        .ok_or("invalid source const panic arguments")?;
    let (Some(Argument::Type(input)), None) = (args.next(), args.next()) else {
        return Err("source const panic requires its complete single-type instance");
    };
    let public_matches = if by_value {
        value_input(&facts.types, function.signature, generic)
            && value_input(&facts.types, signature, input)
    } else {
        reference_input(&facts.types, function.signature, generic, true)
            && reference_input(&facts.types, signature, input, true)
    };
    if !public_matches {
        return Err("source const panic instance differs from its selected type/signature");
    }
    Ok(Some(Selection {
        definition: declaration.definition,
        signature,
        input,
        by_value,
    }))
}

fn value_input(store: &Store, signature: TypeId, input: TypeId) -> bool {
    matches!(
        store.kind(signature),
        Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result: TypeId::NEVER,
            unsafe_: false,
            convention: Convention::RUST,
        }) if store.list(parameters) == Some(&[input])
    )
}

/// Neither artifact writer has an executable const interception contract.
pub(in crate::sema) fn require_begin_panic_transport(
    facts: &Declarations,
) -> Result<(), &'static str> {
    if let Some(catalog) = facts.source_bindings.as_deref()
        && catalog.role(LangItem::BeginPanic)?.is_some()
    {
        return Err("selected Source begin_panic requires its checked artifact transport");
    }
    Ok(())
}
