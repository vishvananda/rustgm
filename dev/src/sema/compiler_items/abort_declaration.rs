//! A source abort declaration is a safe non-const claim, never call permission.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::bindings::SourceIntrinsic,
    syntax::{
        ast::Kind as Syntax,
        item::{Abi, FunctionFlags, ItemKind},
    },
    types::{Kind, TypeId, callable::Convention, instance::Instance},
};

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source abort requires a safe Rust signature without late regions");
    };
    if facts.types.list(parameters).is_none_or(|p| !p.is_empty())
        || facts.types.kind(result) != Some(Kind::Never)
        || facts
            .generics
            .parameters
            .get(&function.definition)
            .is_some_and(|p| !p.is_empty())
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|r| r.total != 0)
        || facts
            .generics
            .declared_predicates_in_scope(function.definition)
            .next()
            .is_some()
    {
        return Err("source abort requires no parameters or predicates and never result");
    }
    if facts.is_const_function(function.definition) || function.body.is_some() {
        return Err("source abort requires a non-const bodyless declaration");
    }
    Ok(())
}

/// Completed declaration identity, without reached-call or body permission.
pub(crate) struct AbortSelection {
    pub definition: DefId,
    pub signature: TypeId,
}
impl Catalog {
    pub(crate) fn abort_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<AbortSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, expected)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        if kind != SourceIntrinsic::Abort {
            return Ok(None);
        }
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("imported source abort requires transported operation provenance");
        }
        if declaration.parent.is_some()
            || signature != expected
            || facts
                .types
                .arguments(arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err("selected source abort changed its complete signature or arguments");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source abort has no completed declaration")?;
        if function.definition != definition
            || function.node != declaration.origin
            || function.signature != expected
        {
            return Err("source abort completed declaration changed");
        }
        validate(facts, *function)?;
        Ok(Some(AbortSelection {
            definition,
            signature,
        }))
    }

    /// The dependency consumer authenticates the live source header before
    /// omitting this exact direct Call's body demand. Lowering proves its body.
    pub(crate) fn require_abort_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selection: &AbortSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, SourceIntrinsic::Abort, signature)) =
            self.intrinsic_record(selection.definition)?
        else {
            return Err("source abort lost its actual attributed declaration");
        };
        let current = facts
            .header_attribute_contracts(report, selection.definition)
            .map_err(|_| "source abort lost current source-header authentication")?;
        if signature != selection.signature
            || current
                .origin
                .source()
                .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
        {
            return Err("source abort requires its positive current source-header site");
        }
        let unit = report.package.unit(declaration.origin.file);
        let Syntax::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
            return Err("source abort lost its actual function item");
        };
        let item = unit
            .configured
            .ast
            .items
            .get(index as usize)
            .ok_or("source abort has a stale function item")?;
        let ItemKind::Function {
            flags,
            abi: Abi::Rust,
            parameters,
            body: None,
            ..
        } = item.kind
        else {
            return Err("source abort requires its current bodyless Rust declaration");
        };
        if flags != FunctionFlags::default()
            || !unit.configured.ast.list(parameters).is_empty()
            || !unit.configured.ast.list(item.generics).is_empty()
            || !unit.configured.ast.list(item.where_clause).is_empty()
        {
            return Err("source abort changed its current nonconst safe header");
        }
        Ok(())
    }
}
