//! Actual intrinsic declarations select data; normal calls retain all permissions.
use super::super::{Catalog, intrinsic_contracts, prototype_validation};
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId, unit::NodeKey},
    sema::model::{Declarations, Function},
    support::bindings::{SourceIntrinsic, prototype_specs::Declaration},
    types::{
        Kind, RegionId, TypeId, arguments::Argument, callable::Convention, instance::Instance,
    },
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Read {
    pub definition: DefId,
    pub callable: TypeId,
    pub output: TypeId,
}
fn record<'a>(
    catalog: &Catalog,
    facts: &'a Declarations,
    callable: TypeId,
) -> Result<Option<(Declaration, &'a Function, TypeId, crate::types::GenericArgs)>, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(callable)
    else {
        return Ok(None);
    };
    let Some((declaration, SourceIntrinsic::Declaration(identity), original)) =
        catalog.intrinsic_record(DefId(item.0))?
    else {
        return Ok(None);
    };
    if !matches!(identity.name(), "va_arg" | "va_copy" | "va_end") {
        return Ok(None);
    }
    super::charge(facts, 16)?;
    let function = facts
        .function_indices
        .get(&DefId(item.0))
        .and_then(|index| facts.functions.get(*index))
        .ok_or("source vararg intrinsic lost its completed declaration")?;
    if function.signature != original
        || function.node != declaration.origin
        || declaration.parent.is_some()
    {
        return Err("source vararg intrinsic changed its actual completed owner or signature");
    }
    prototype_validation::validate(facts, *function, identity)?;
    Ok(Some((identity, function, signature, arguments)))
}
impl Catalog {
    pub(crate) fn vararg_fallback(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<bool, &'static str> {
        let Some((identity, function, signature, arguments)) = record(self, facts, instance.ty())?
        else {
            return Ok(false);
        };
        if !matches!(identity.name(), "va_copy" | "va_end") {
            return Ok(false);
        }
        if matches!(
            self.bindings,
            super::super::binding_views::Bindings::Imported(_)
        ) {
            return Err("imported vararg fallback body provenance is not transported");
        }
        if function.body.is_none()
            || facts
                .types
                .arguments(arguments)
                .is_none_or(|args| args.len() != 0)
        {
            return Err(
                "source vararg fallback requires its actual declared body and lifetime-only instance",
            );
        }
        prototype_validation::validate(
            facts,
            Function {
                signature,
                ..*function
            },
            identity,
        )?;
        Ok(true)
    }
    pub(crate) fn vararg_read(
        &self,
        facts: &Declarations,
        callable: TypeId,
        origin: BodyOrigin,
    ) -> Result<Option<Read>, &'static str> {
        let Some((identity, function, signature, arguments)) = record(self, facts, callable)?
        else {
            return Ok(None);
        };
        if identity.name() != "va_arg" {
            return Ok(None);
        }
        if origin.source().is_none()
            || matches!(
                self.bindings,
                super::super::binding_views::Bindings::Imported(_)
            )
        {
            return Err("imported vararg read provenance is not transported");
        }
        if function.body.is_some() {
            return Err("source va_arg must retain its actual bodyless intrinsic declaration");
        }
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source va_arg arguments are missing")?;
        let (Some(Argument::Type(output)), None) = (args.next(), args.next()) else {
            return Err("source va_arg requires its complete actual type instance");
        };
        let Some(Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_: true,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("source va_arg changed its actual unsafe Rust signature");
        };
        let Some(&[input]) = facts.types.list(parameters) else {
            return Err("source va_arg requires its actual single mutable reference");
        };
        let Some(Kind::Reference {
            mutable: true,
            region,
            referent,
        }) = facts.types.kind(input)
        else {
            return Err("source va_arg lost its mutable source list input");
        };
        let (_, inner) = super::application(facts, referent)?;
        if !matches!((region.bound_parts(),inner.bound_parts()),(Some((0,a)),Some((0,b))) if a!=b && u32::from(a)<bound_regions && u32::from(b)<bound_regions)
            || intrinsic_contracts::item_roots(&facts.types, [result], |steps| {
                super::charge(facts, steps)
            })? != Some([output])
        {
            return Err(
                "source va_arg changed its independent public regions or actual instance result",
            );
        }
        Ok(Some(Read {
            definition: function.definition,
            callable,
            output,
        }))
    }
    pub(crate) fn require_vararg_report(
        &self,
        report: &Report,
        facts: &Declarations,
        read: &Read,
        origin: BodyOrigin,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        if self.vararg_read(facts, read.callable, origin)? != Some(*read) {
            return Err("source va_arg descriptor belongs to another current source declaration");
        }
        Ok(())
    }
    pub(crate) fn validate_vararg_opening(
        &self,
        report: &Report,
        facts: &Declarations,
        owner: NodeKey,
        call: NodeKey,
        checked: (TypeId, TypeId),
    ) -> Result<(), &'static str> {
        let Some(read) = self.vararg_read(facts, checked.0, owner.into())? else {
            return Ok(());
        };
        self.require_vararg_report(report, facts, &read, owner.into())?;
        let (list, regions) = read.opened(facts, checked.1)?;
        intrinsic_contracts::checked_opening(report,facts,owner,call,checked,&regions,
            intrinsic_contracts::OpeningReference{mutable:true,referent:list})
            .map_err(|error|match error {
                intrinsic_contracts::OpeningError::Effect(message)=>message,
                _=>"source va_arg opening changed its actual call, source use, adjusted argument or call-owned regions",
            })
    }
}
impl Read {
    pub(crate) fn opened(
        &self,
        facts: &Declarations,
        signature: TypeId,
    ) -> Result<(TypeId, [RegionId; 2]), &'static str> {
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: true,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("source va_arg requires its actual opened unsafe Rust Call");
        };
        let Some(&[parameter]) = facts.types.list(parameters) else {
            return Err("source va_arg Call requires one actual argument");
        };
        let Some(Kind::Reference {
            mutable: true,
            region,
            referent,
        }) = facts.types.kind(parameter)
        else {
            return Err("source va_arg Call lost its actual mutable list reference");
        };
        let (_, inner) = super::application(facts, referent)?;
        if result != self.output || region.bound_parts().is_some() || inner.bound_parts().is_some()
        {
            return Err("source va_arg opened Call changed its actual result or list lifetimes");
        }
        Ok((referent, [region, inner]))
    }
}
