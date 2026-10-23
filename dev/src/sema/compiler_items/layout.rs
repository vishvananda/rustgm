//! Layout selections retain their existing usize contract and carry no permission.
use super::Catalog;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    support::bindings::{LangItem, LayoutIntrinsic, SourceIntrinsic},
    types::{Primitive, TypeId, instance::Instance},
};
/// Data describing a validated target; execution still requires its own checked call.
#[derive(Clone, Copy)]
pub(crate) struct Selection {
    pub definition: DefId,
    pub element: TypeId,
    pub kind: LayoutIntrinsic,
    pub fallback: bool,
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
    pub(crate) fn layout_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<Option<Selection>, &'static str> {
        let Some(selected) = self.selected_intrinsic(facts, instance)? else {
            return Ok(None);
        };
        let SourceIntrinsic::Layout(kind) = selected.kind else {
            return Ok(None);
        };
        Ok(Some(Selection {
            definition: selected.definition,
            element: selected.element,
            kind,
            fallback: selected.fallback,
        }))
    }
}
