//! Validated source intrinsic contracts retain actual declarations and substitutions.
use super::Catalog;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::{Declarations, Function},
    support::bindings::SourceIntrinsic,
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};

pub(super) struct Intrinsic {
    declaration: crate::resolve::bindings::Declaration,
    kind: SourceIntrinsic,
    signature: Option<TypeId>,
}
impl Intrinsic {
    pub(super) fn reserved(entry: crate::resolve::bindings::IntrinsicDeclaration) -> Self {
        Self {
            declaration: entry.declaration,
            kind: entry.kind,
            signature: None,
        }
    }
}
/// Data describing a validated target; execution still requires its own checked call.
#[derive(Clone, Copy)]
pub(crate) struct Selected {
    pub definition: DefId,
    pub element: TypeId,
    pub kind: SourceIntrinsic,
    pub fallback: bool,
}

pub(in crate::sema) fn validate(
    report: &Report,
    facts: &mut Declarations,
    function: Function,
) -> Result<(), Error> {
    let Some(entry) = facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.intrinsics.get(&function.definition))
    else {
        return Ok(());
    };
    let error = |message| Declarations::error(report, function.node, message);
    if entry.signature.is_some()
        || entry.declaration.origin != function.node
        || entry.declaration.parent.is_some()
    {
        return Err(error("source intrinsic declaration identity changed"));
    }
    signature(facts, function, entry.kind).map_err(error)?;
    facts
        .source_bindings
        .as_deref_mut()
        .and_then(|c| c.intrinsics.get_mut(&function.definition))
        .ok_or_else(|| error("source intrinsic reservation disappeared"))?
        .signature = Some(function.signature);
    Ok(())
}

fn signature(
    facts: &Declarations,
    function: Function,
    kind: SourceIntrinsic,
) -> Result<(), &'static str> {
    if kind == SourceIntrinsic::OffsetOf {
        return super::displacement::validate(facts, function);
    }
    if kind == SourceIntrinsic::DiscriminantValue {
        return super::tag_intrinsic::validate(facts, function);
    }
    if kind == SourceIntrinsic::AggregateRawPtr {
        return super::compose_pointer::validate(facts, function);
    }
    if kind == SourceIntrinsic::PtrMetadata {
        return super::ptr_metadata::validate(facts, function).map(|_| ());
    }
    if kind == SourceIntrinsic::ConstEvalSelect {
        return super::select_contract::validate(facts, function);
    }
    if kind == SourceIntrinsic::UbChecks {
        return super::runtime_checks::validate(facts, function);
    }
    if kind == SourceIntrinsic::Transmute {
        return super::transmute_contract::validate(facts, function);
    }
    if kind == SourceIntrinsic::CallerLocation {
        return super::caller_location::validate(facts, function);
    }
    if kind == SourceIntrinsic::Abort {
        return super::abort_declaration::validate(facts, function);
    }
    let needs_drop = kind == SourceIntrinsic::NeedsDrop;
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(if needs_drop {
            "source needs_drop requires a safe Rust signature without late lifetimes"
        } else {
            "source layout intrinsic requires a safe Rust signature without late lifetimes"
        });
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("missing source intrinsic inputs")?;
    let generic = facts
        .generics
        .parameters
        .get(&function.definition)
        .ok_or(if needs_drop {
            "source needs_drop requires one type parameter"
        } else {
            "source layout intrinsic requires one type parameter"
        })?;
    let expected = if needs_drop {
        Primitive::Bool
    } else {
        Primitive::Usize
    };
    if !parameters.is_empty()
        || facts.types.kind(result) != Some(Kind::Primitive(expected))
        || generic.len() != 1
    {
        return Err(if needs_drop {
            "source needs_drop requires one type parameter, no values and bool result"
        } else {
            "source layout intrinsic requires one type parameter, no values and usize result"
        });
    }
    let Argument::Type(ty) = generic[0].argument else {
        return Err(if needs_drop {
            "source needs_drop requires a type parameter"
        } else {
            "source layout intrinsic requires a type parameter"
        });
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err(if needs_drop {
            "source needs_drop lost its type parameter"
        } else {
            "source layout intrinsic lost its type parameter"
        });
    };
    if facts.types.parameter(id).is_none_or(|p| {
        p.owner.0 != function.definition.0
            || !matches!(p.kind, ParameterKind::Type { sized } if sized != needs_drop)
    }) {
        return Err(if needs_drop {
            "source needs_drop requires its own ?Sized type parameter"
        } else {
            "source layout intrinsic requires its own Sized type parameter"
        });
    }
    if needs_drop && !facts.is_const_function(function.definition) {
        return Err("source needs_drop declaration must be const");
    }
    Ok(())
}

impl Catalog {
    pub(super) fn intrinsic_record(
        &self,
        definition: DefId,
    ) -> Result<
        Option<(
            crate::resolve::bindings::Declaration,
            SourceIntrinsic,
            TypeId,
        )>,
        &'static str,
    > {
        self.intrinsics
            .get(&definition)
            .map(|entry| {
                Ok((
                    entry.declaration,
                    entry.kind,
                    entry
                        .signature
                        .ok_or("source intrinsic declaration is not validated")?,
                ))
            })
            .transpose()
    }
    pub(crate) fn intrinsic(&self, definition: DefId) -> Result<bool, &'static str> {
        Ok(self.intrinsic_kind(definition)?.is_some())
    }
    pub(crate) fn intrinsic_kind(
        &self,
        definition: DefId,
    ) -> Result<Option<SourceIntrinsic>, &'static str> {
        let Some(entry) = self.intrinsics.get(&definition) else {
            return Ok(None);
        };
        if entry.signature.is_none() {
            return Err("source intrinsic declaration is not validated");
        }
        Ok(Some(entry.kind))
    }
    pub(crate) fn selected_intrinsic(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<Option<Selected>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some(entry) = self.intrinsics.get(&definition) else {
            return Ok(None);
        };
        let original = entry
            .signature
            .ok_or("source intrinsic declaration is not validated")?;
        if matches!(
            entry.kind,
            SourceIntrinsic::ConstEvalSelect
                | SourceIntrinsic::UbChecks
                | SourceIntrinsic::Abort
                | SourceIntrinsic::Transmute
                | SourceIntrinsic::CallerLocation
                | SourceIntrinsic::PtrMetadata
                | SourceIntrinsic::AggregateRawPtr
                | SourceIntrinsic::DiscriminantValue
        ) {
            // These contracts have dedicated descriptors, without a layout element.
            return Ok(None);
        }
        // This profile's public signature has no parameter occurrences. Matching
        // the complete retained signature therefore validates its substitution.
        if signature != original {
            return Err("selected source intrinsic signature differs from its validated contract");
        }
        let element = layout_element(&facts.types, arguments, entry.kind)?;
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("source intrinsic has no completed declaration")?;
        if function.signature != original || function.node != entry.declaration.origin {
            return Err("source intrinsic completed declaration changed");
        }
        Ok(Some(Selected {
            definition,
            element,
            kind: entry.kind,
            fallback: function.body.is_some(),
        }))
    }
}

fn layout_element(
    types: &crate::types::Store,
    arguments: crate::types::GenericArgs,
    kind: SourceIntrinsic,
) -> Result<TypeId, &'static str> {
    let mut arguments = types
        .arguments(arguments)
        .ok_or("invalid source intrinsic arguments")?;
    let Some(Argument::Type(element)) = arguments.next() else {
        return Err(match kind {
            SourceIntrinsic::Layout(_) | SourceIntrinsic::OffsetOf => {
                "selected source layout intrinsic lacks its type argument"
            }
            SourceIntrinsic::NeedsDrop => "selected source needs_drop lacks its type argument",
            SourceIntrinsic::ConstEvalSelect => {
                "source callback selection requires its own descriptor"
            }
            SourceIntrinsic::DiscriminantValue => {
                "source discriminant requires its actual reached reference-call descriptor"
            }
            SourceIntrinsic::AggregateRawPtr => {
                "source aggregate_raw_ptr requires its actual composition-call descriptor"
            }
            SourceIntrinsic::PtrMetadata => {
                "source ptr_metadata requires its actual pointer-call descriptor"
            }
            SourceIntrinsic::CallerLocation => {
                "source caller location requires its exact caller descriptor"
            }
            SourceIntrinsic::UbChecks => "source UB check requires its own descriptor",
            SourceIntrinsic::Abort => "source abort requires its own reached-call descriptor",
            SourceIntrinsic::Transmute => {
                "source transmute requires its own checked call descriptor"
            }
        });
    };
    if arguments.next().is_some()
        || types.contains_parameters(element) != Some(false)
        || types.contains_projections(element) != Some(false)
    {
        return Err(match kind {
            SourceIntrinsic::Layout(_) | SourceIntrinsic::OffsetOf => {
                "source layout intrinsic requires a complete closed type argument"
            }
            SourceIntrinsic::NeedsDrop => {
                "source needs_drop requires a complete closed type argument"
            }
            SourceIntrinsic::ConstEvalSelect => {
                "source callback selection requires its own descriptor"
            }
            SourceIntrinsic::DiscriminantValue => {
                "source discriminant requires its actual reached reference-call descriptor"
            }
            SourceIntrinsic::AggregateRawPtr => {
                "source aggregate_raw_ptr requires its actual composition-call descriptor"
            }
            SourceIntrinsic::PtrMetadata => {
                "source ptr_metadata requires its actual pointer-call descriptor"
            }
            SourceIntrinsic::CallerLocation => {
                "source caller location requires its exact caller descriptor"
            }
            SourceIntrinsic::UbChecks => "source UB check requires its own descriptor",
            SourceIntrinsic::Abort => "source abort requires its own reached-call descriptor",
            SourceIntrinsic::Transmute => {
                "source transmute requires its own checked call descriptor"
            }
        });
    }
    Ok(element)
}

#[cfg(test)]
use super::layout::validate_member;
#[cfg(test)]
use crate::support::bindings::LangItem;
#[cfg(test)]
#[path = "../../../tests/invariants/provider_layout.rs"]
mod tests;
