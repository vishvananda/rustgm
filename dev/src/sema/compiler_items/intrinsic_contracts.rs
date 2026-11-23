//! Validated source intrinsic contracts retain actual declarations and substitutions.
use super::Catalog;
mod open;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::{Declarations, Function},
    support::bindings::{LayoutIntrinsic, SourceIntrinsic},
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};
pub(super) use open::{
    Error as OpeningError, Reference as OpeningReference, checked as checked_opening,
};

/// Native lowering is available for these completed bodyless declarations.
pub(crate) fn strict_bodiless_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "catch_unwind"
            | "size_of_val"
            | "align_of_val"
            | "offset"
            | "ptr_offset_from"
            | "ptr_offset_from_unsigned"
            | "slice_get_unchecked"
            | "compare_bytes"
            | "atomic_load"
            | "atomic_store"
            | "black_box"
            | "read_via_copy"
            | "write_via_move"
            | "forget"
            | "unreachable"
    ) || super::bit_counts::named(name).is_some()
        || crate::support::word_operation::Update::intrinsic(name).is_some()
        || crate::support::intrinsics::Intrinsic::source_transfer(name).is_some()
        || super::checked_atomic_access::Fence::named(name).is_some()
}

/// A Body Call's roots are already opened into its caller, unlike an item signature.
pub(super) fn opened_signature_matches(
    store: &crate::types::Store,
    signature: TypeId,
    inputs: &[TypeId],
    output: TypeId,
    unsafe_: bool,
    convention: Convention,
) -> bool {
    matches!(store.kind(signature), Some(Kind::Function {
        bound_regions: 0, parameters, result, unsafe_: actual_unsafe, convention: actual_convention,
    }) if actual_unsafe == unsafe_ && actual_convention == convention
        && store.list(parameters) == Some(inputs) && result == output)
}

/// Remove only the item's surrounding function binder, retaining nested local binders.
pub(super) fn item_roots<const N: usize>(
    store: &crate::types::Store,
    roots: [TypeId; N],
    mut charge: impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<Option<[TypeId; N]>, &'static str> {
    charge(1)?;
    let mut lookup = crate::types::substitute::lookup::Lookup::new(1);
    let result = (|| {
        let mut opened = [TypeId::UNIT; N];
        for (slot, root) in opened.iter_mut().zip(roots) {
            let Some(actual) = lookup.ty(store, root)? else {
                return Ok(None);
            };
            *slot = actual;
        }
        Ok(Some(opened))
    })();
    // Lookup is bounded internally; failed work and failed payment publish no answer.
    charge(lookup.steps())?;
    result
}

/// A `black_box` instance retains one type and the checked opening of its identity signature.
pub(crate) fn black_box_type(
    facts: &Declarations,
    callee: TypeId,
    signature: TypeId,
) -> Result<TypeId, &'static str> {
    let types = &facts.types;
    let Some(Kind::FunctionItem {
        arguments,
        signature: public,
        ..
    }) = types.kind(callee)
    else {
        return Err("source black_box requires its complete function item");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source black_box lost its type argument")?;
    let (Some(Argument::Type(ty)), None) = (arguments.next(), arguments.next()) else {
        return Err("source black_box requires exactly one type argument");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = types.kind(public)
    else {
        return Err("source black_box requires its public safe Rust signature");
    };
    let Some(&[input]) = types.list(parameters) else {
        return Err("source black_box requires one public value input");
    };
    let roots = item_roots(types, [input, result], |work| {
        crate::sema::header_attributes::charge_source_contract_query(facts, work)
            .map_err(|_| "source black_box instance query work exhausted")
    })?;
    if roots != Some([ty; 2]) {
        return Err("source black_box public signature does not open to its type argument");
    }
    if !opened_signature_matches(types, signature, &[ty], ty, false, Convention::RUST) {
        return Err("source black_box opened signature differs from its type argument");
    }
    Ok(ty)
}

#[derive(Clone, Copy)]
enum HeaderState {
    Reserved,
    Validated(TypeId),
    SourceFallback(TypeId),
}

impl HeaderState {
    const fn signature(self) -> Option<TypeId> {
        match self {
            Self::Reserved => None,
            Self::Validated(signature) | Self::SourceFallback(signature) => Some(signature),
        }
    }
}

pub(super) struct Intrinsic {
    declaration: crate::resolve::bindings::Declaration,
    kind: SourceIntrinsic,
    header: HeaderState,
}
impl Intrinsic {
    pub(super) fn reserved(entry: crate::resolve::bindings::IntrinsicDeclaration) -> Self {
        Self {
            declaration: entry.declaration,
            kind: entry.kind,
            header: HeaderState::Reserved,
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
    if !matches!(entry.header, HeaderState::Reserved)
        || entry.declaration.origin != function.node
        || entry.declaration.parent.is_some()
    {
        return Err(error("source intrinsic declaration identity changed"));
    }
    if entry.kind == SourceIntrinsic::ColdPath {
        super::cold_fallback::validate(report, facts, function).map_err(error)?;
    } else {
        signature(facts, function, entry.kind).map_err(error)?;
    }
    let source_fallback = source_fallback(report, facts, function, entry.kind)?;
    let completed = facts
        .source_bindings
        .as_deref_mut()
        .and_then(|c| c.intrinsics.get_mut(&function.definition))
        .ok_or_else(|| error("source intrinsic reservation disappeared"))?;
    completed.header = if source_fallback {
        HeaderState::SourceFallback(function.signature)
    } else {
        HeaderState::Validated(function.signature)
    };
    Ok(())
}

fn source_fallback(
    report: &Report,
    facts: &Declarations,
    function: Function,
    kind: SourceIntrinsic,
) -> Result<bool, Error> {
    let SourceIntrinsic::Declaration(identity) = kind else {
        return Ok(false);
    };
    if function.body.is_none() || !facts.is_const_function(function.definition) {
        return Ok(false);
    }
    match identity.name() {
        "select_unpredictable" => crate::sema::header_attributes::intrinsic_fallback_is_spec(
            facts,
            report,
            function.definition,
        ),
        "type_id_eq" => facts
            .const_panic_markers(report, function.definition)
            .map(|(runtime_body, _)| runtime_body),
        _ => Ok(false),
    }
}

fn signature(
    facts: &Declarations,
    function: Function,
    kind: SourceIntrinsic,
) -> Result<(), &'static str> {
    match kind {
        SourceIntrinsic::Declaration(identity) => {
            super::prototype_validation::validate(facts, function, identity)
        }
        SourceIntrinsic::UncheckedSub | SourceIntrinsic::SubWithOverflow => {
            super::subtraction_contract::validate(facts, function, kind)
        }
        SourceIntrinsic::ArithOffset => super::wrapping_contract::validate(facts, function),
        SourceIntrinsic::ThreeWayCompare => super::compare_intrinsic::validate(facts, function),
        SourceIntrinsic::OffsetOf => super::displacement::validate(facts, function),
        SourceIntrinsic::DiscriminantValue => super::tag_intrinsic::validate(facts, function),
        SourceIntrinsic::AggregateRawPtr => super::compose_pointer::validate(facts, function),
        SourceIntrinsic::PtrMetadata => super::ptr_metadata::validate(facts, function).map(|_| ()),
        SourceIntrinsic::ConstEvalSelect => super::select_contract::validate(facts, function),
        SourceIntrinsic::UbChecks => super::runtime_checks::validate(facts, function),
        SourceIntrinsic::Transmute | SourceIntrinsic::TransmuteUnchecked => {
            super::transmute_contract::validate(facts, function)
        }
        SourceIntrinsic::CallerLocation => super::caller_location::validate(facts, function),
        SourceIntrinsic::Abort => super::abort_declaration::validate(facts, function),
        _ => scalar_signature(facts, function, kind),
    }
}
fn scalar_signature(
    facts: &Declarations,
    function: Function,
    kind: SourceIntrinsic,
) -> Result<(), &'static str> {
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
        TypeId::primitive(Primitive::Bool)
    } else if kind == SourceIntrinsic::Layout(LayoutIntrinsic::AssertInhabited) {
        TypeId::UNIT
    } else {
        TypeId::primitive(Primitive::Usize)
    };
    if !parameters.is_empty() || result != expected || generic.len() != 1 {
        return Err(if needs_drop {
            "source needs_drop requires one type parameter, no values and bool result"
        } else {
            "source layout intrinsic requires one type parameter, no values and its declared property result"
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
                        .header
                        .signature()
                        .ok_or("source intrinsic declaration is not validated")?,
                ))
            })
            .transpose()
    }
    /// A source fallback sealed during header validation may follow normal body proofs.
    pub(crate) fn uses_source_fallback(&self, definition: DefId) -> bool {
        self.intrinsics
            .get(&definition)
            .is_some_and(|entry| matches!(entry.header, HeaderState::SourceFallback(_)))
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
        if matches!(entry.header, HeaderState::Reserved) {
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
            .header
            .signature()
            .ok_or("source intrinsic declaration is not validated")?;
        if matches!(
            entry.kind,
            SourceIntrinsic::ArithOffset
                | SourceIntrinsic::Declaration(_)
                | SourceIntrinsic::UncheckedSub
                | SourceIntrinsic::SubWithOverflow
                | SourceIntrinsic::ColdPath
                | SourceIntrinsic::ConstEvalSelect
                | SourceIntrinsic::UbChecks
                | SourceIntrinsic::Abort
                | SourceIntrinsic::Transmute
                | SourceIntrinsic::TransmuteUnchecked
                | SourceIntrinsic::CallerLocation
                | SourceIntrinsic::PtrMetadata
                | SourceIntrinsic::AggregateRawPtr
                | SourceIntrinsic::DiscriminantValue
                | SourceIntrinsic::ThreeWayCompare
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
    let (missing, incomplete) = match kind {
        SourceIntrinsic::Layout(_) | SourceIntrinsic::OffsetOf => (
            "selected source layout intrinsic lacks its type argument",
            "source layout intrinsic requires a complete closed type argument",
        ),
        SourceIntrinsic::NeedsDrop => (
            "selected source needs_drop lacks its type argument",
            "source needs_drop requires a complete closed type argument",
        ),
        _ => return Err("source intrinsic requires its own checked execution descriptor"),
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("invalid source intrinsic arguments")?;
    let Some(Argument::Type(element)) = arguments.next() else {
        return Err(missing);
    };
    if arguments.next().is_some()
        || types.contains_parameters(element) != Some(false)
        || types.contains_projections(element) != Some(false)
    {
        return Err(incomplete);
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
