//! Validated source intrinsic contracts retain actual declarations and substitutions.
use super::Catalog;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::{Declarations, Function},
    support::bindings::{LangItem, LayoutIntrinsic},
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};

pub(super) struct Intrinsic {
    declaration: crate::resolve::bindings::Declaration,
    kind: LayoutIntrinsic,
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
pub(crate) struct Selection {
    pub definition: DefId,
    pub element: TypeId,
    pub kind: LayoutIntrinsic,
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
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(error(
            "source layout intrinsic requires a safe Rust signature without late lifetimes",
        ));
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("missing source intrinsic inputs"))?;
    let generic = facts
        .generics
        .parameters
        .get(&function.definition)
        .ok_or_else(|| error("source layout intrinsic requires one type parameter"))?;
    if !parameters.is_empty()
        || facts.types.kind(result) != Some(Kind::Primitive(Primitive::Usize))
        || generic.len() != 1
    {
        return Err(error(
            "source layout intrinsic requires one type parameter, no values and usize result",
        ));
    }
    let Argument::Type(ty) = generic[0].argument else {
        return Err(error("source layout intrinsic requires a type parameter"));
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err(error("source layout intrinsic lost its type parameter"));
    };
    if facts.types.parameter(id).is_none_or(|p| {
        p.owner.0 != function.definition.0 || !matches!(p.kind, ParameterKind::Type { sized: true })
    }) {
        return Err(error(
            "source layout intrinsic requires its own Sized type parameter",
        ));
    }
    facts
        .source_bindings
        .as_deref_mut()
        .and_then(|c| c.intrinsics.get_mut(&function.definition))
        .ok_or_else(|| error("source intrinsic reservation disappeared"))?
        .signature = Some(function.signature);
    Ok(())
}

/// The actual source member has completed type elaboration, not evaluation.
pub(in crate::sema) fn validate_member(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(declaration) = [LangItem::MemSizeConst, LangItem::MemAlignConst]
        .into_iter()
        .filter_map(|kind| catalog.language(kind))
        .find(|entry| entry.definition == definition)
    else {
        return Ok(());
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    let parent = declaration
        .parent
        .ok_or_else(|| error("source layout member lost its parent"))?;
    let owner = facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
        .ok_or_else(|| error("source layout member has no actual associated owner"))?;
    let actual_parent = match owner.kind {
        crate::sema::associated::OwnerKind::Trait { definition, .. } => definition,
        crate::sema::associated::OwnerKind::Impl { .. } => owner.self_definition,
    };
    if parent != actual_parent
        || report
            .resolution
            .definitions
            .get(parent.0 as usize)
            .and_then(|d| d.node)
            != Some(owner.source)
    {
        return Err(error("source layout member associated owner changed"));
    }
    let ty = facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten()
        .ok_or_else(|| error("source layout member type is not complete"))?;
    if ty != TypeId::primitive(Primitive::Usize) {
        return Err(error(
            "source layout language member currently requires usize type",
        ));
    }
    Ok(())
}

impl Catalog {
    pub(crate) fn intrinsic(&self, definition: DefId) -> Result<bool, &'static str> {
        let Some(entry) = self.intrinsics.get(&definition) else {
            return Ok(false);
        };
        if entry.signature.is_none() {
            return Err("source intrinsic declaration is not validated");
        }
        Ok(true)
    }
    pub(crate) fn layout_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<Option<Selection>, &'static str> {
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
        // This profile's public signature has no parameter occurrences. Matching
        // the complete retained signature therefore validates its substitution.
        if signature != original {
            return Err("selected source intrinsic signature differs from its validated contract");
        }
        let mut arguments = facts
            .types
            .arguments(arguments)
            .ok_or("invalid source intrinsic arguments")?;
        let Some(Argument::Type(element)) = arguments.next() else {
            return Err("selected source layout intrinsic lacks its type argument");
        };
        if arguments.next().is_some()
            || facts.types.contains_parameters(element) != Some(false)
            || facts.types.contains_projections(element) != Some(false)
        {
            return Err("source layout intrinsic requires a complete closed type argument");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("source intrinsic has no completed declaration")?;
        if function.signature != original || function.node != entry.declaration.origin {
            return Err("source intrinsic completed declaration changed");
        }
        Ok(Some(Selection {
            definition,
            element,
            kind: entry.kind,
            fallback: function.body.is_some(),
        }))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/provider_layout.rs"]
mod tests;
