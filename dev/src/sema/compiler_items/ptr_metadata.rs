//! A source pointer call keeps its actual P/M contract separate from representation data.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::{
        bindings::{LangItem, SourceIntrinsic},
        constness::Constness,
    },
    types::{
        Kind, TypeId,
        arguments::Argument,
        callable::Convention,
        instance::Instance,
        parameters::{ParameterKind, clauses::Clause, predicates::Bound},
        projection,
    },
};

/// Ephemeral data; the actual caller/body/const witness is required separately.
pub(crate) struct PtrMetadataSelection {
    definition: DefId,
    signature: TypeId,
    pointee: TypeId,
    output: TypeId,
    instance: Instance,
}
impl PtrMetadataSelection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) const fn output(&self) -> TypeId {
        self.output
    }
    pub(crate) fn input(&self, facts: &Declarations) -> Result<TypeId, &'static str> {
        let Some(Kind::Function { parameters, .. }) = facts.types.kind(self.signature) else {
            return Err("source ptr_metadata lost its callable signature");
        };
        let inputs = facts
            .types
            .list(parameters)
            .ok_or("source ptr_metadata input disappeared")?;
        match inputs {
            [input] => Ok(*input),
            _ => Err("source ptr_metadata input arity changed"),
        }
    }
    pub(crate) fn current(&self, facts: &Declarations) -> Result<(), &'static str> {
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("source ptr_metadata lost its actual catalog")?;
        for role in [LangItem::PointeeTrait, LangItem::MetadataType] {
            catalog
                .validated_language(role)?
                .ok_or("source ptr_metadata role is not complete")?;
        }
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("source ptr_metadata lost its actual function item");
        };
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source ptr_metadata lost its arguments")?;
        if item.0 != self.definition.0
            || signature != self.signature
            || args.next() != Some(Argument::Type(self.pointee))
            || args.next() != Some(Argument::Type(self.output))
            || args.next().is_some()
        {
            return Err("source ptr_metadata changed its full instance or signature");
        }
        Ok(())
    }
}

pub(super) fn validate(
    facts: &Declarations,
    function: Function,
) -> Result<projection::Application, &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source ptr_metadata requires a safe Rust signature without late regions");
    };
    let generic = facts
        .generics
        .parameters
        .get(&function.definition)
        .ok_or("source ptr_metadata requires its two own type parameters")?;
    if generic.len() != 2
        || !facts.is_const_function(function.definition)
        || function.body.is_some()
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|r| r.total != 0)
    {
        return Err(
            "source ptr_metadata requires a const bodyless declaration with two owned types",
        );
    }
    let mut types = [TypeId::UNIT; 2];
    for (index, formal) in generic.iter().enumerate() {
        let Argument::Type(ty) = formal.argument else {
            return Err("source ptr_metadata requires only type formals");
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("source ptr_metadata lost its formal identity");
        };
        if facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != function.definition.0
                || p.index as usize != index
                || !matches!(p.kind, ParameterKind::Type { sized } if sized == (index == 1))
        }) {
            return Err(
                "source ptr_metadata requires its ordered PointeeSized P and Sized M formals",
            );
        }
        types[index] = ty;
    }
    let input = facts
        .types
        .list(parameters)
        .ok_or("source ptr_metadata inputs disappeared")?;
    if input.len() != 1
        || result != types[1]
        || facts.types.kind(input[0])
            != Some(Kind::Pointer {
                mutable: false,
                pointee: types[0],
            })
    {
        return Err("source ptr_metadata requires exactly *const P to its own M");
    }
    protocol(facts, function.definition, types)
}

fn protocol(
    facts: &Declarations,
    definition: DefId,
    types: [TypeId; 2],
) -> Result<projection::Application, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source ptr_metadata requires its actual catalog")?;
    let parent = catalog
        .language(LangItem::PointeeTrait)
        .ok_or("source ptr_metadata requires actual Pointee")?;
    let member = catalog
        .language(LangItem::MetadataType)
        .ok_or("source ptr_metadata requires actual Metadata")?;
    let sizing = catalog
        .sizing
        .ok_or("source ptr_metadata requires completed sizing declarations")?;
    if member.parent != Some(parent.definition) {
        return Err("source ptr_metadata Metadata belongs to another trait");
    }
    let mut found = 0;
    let mut projection = None;
    for clause in facts.generics.declared_predicates(definition) {
        let predicate = match clause.value {
            Clause::UnstableFeature(_) => {
                return Err("source ptr_metadata feature conditions require an explicit contract");
            }
            Clause::Predicate(p) => p,
            Clause::ForAll(id) => {
                let clause = facts
                    .types
                    .quantified(id)
                    .ok_or("source ptr_metadata clause disappeared")?;
                if clause.bound_regions != 0 {
                    return Err("source ptr_metadata has an unsupported quantified constraint");
                }
                clause.predicate
            }
        };
        if let Some((id, effect)) = predicate.bound.trait_parts() {
            let app = facts
                .types
                .trait_application(id)
                .ok_or("invalid source ptr_metadata trait bound")?;
            if effect != Constness::Ordinary
                || facts
                    .types
                    .arguments(app.arguments)
                    .is_none_or(|a| a.len() != 0)
            {
                return Err("source ptr_metadata requires ordinary complete trait bounds");
            }
            if predicate.ty == types[0] && app.item.0 == parent.definition.0 {
                found |= 1;
            } else if predicate.ty == types[0] && app.item.0 == sizing[2].0 {
                found |= 2;
            } else if predicate.ty != types[1] || !sizing.contains(&DefId(app.item.0)) {
                return Err("source ptr_metadata has an unsupported defining trait constraint");
            }
        } else if let Bound::ProjectionEqual(result) = predicate.bound {
            let Some(Kind::Projection(app)) = facts.types.kind(predicate.ty) else {
                return Err("source ptr_metadata equality lost its actual projection");
            };
            if result != types[1]
                || app.subject != types[0]
                || app.item.0 != member.definition.0
                || facts
                    .types
                    .arguments(app.arguments)
                    .is_none_or(|a| a.len() != 0)
                || facts
                    .types
                    .trait_application(app.trait_application)
                    .is_none_or(|a| {
                        a.item.0 != parent.definition.0
                            || facts
                                .types
                                .arguments(a.arguments)
                                .is_none_or(|a| a.len() != 0)
                    })
            {
                return Err("source ptr_metadata requires its exact Metadata=M equality");
            }
            if projection.is_some_and(|previous| previous != app) {
                return Err("source ptr_metadata has conflicting Metadata constraints");
            }
            projection = Some(app);
        } else if predicate.ty != types[1] || predicate.bound != Bound::Sized {
            return Err("source ptr_metadata has an unsupported defining constraint");
        }
    }
    if found != 3 {
        return Err("source ptr_metadata requires actual Pointee and PointeeSized bounds");
    }
    projection.ok_or("source ptr_metadata requires the actual Metadata=M equality")
}

impl Catalog {
    pub(crate) fn ptr_metadata_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<PtrMetadataSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, original)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        if kind != SourceIntrinsic::PtrMetadata {
            return Ok(None);
        }
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("imported ptr_metadata requires transported source operation provenance");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("source ptr_metadata has no completed declaration")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
        {
            return Err("source ptr_metadata completed declaration changed");
        }
        let clauses = facts.generics.declared_predicates(definition).len();
        crate::sema::header_attributes::charge_source_contract_query(facts, 16 + clauses)
            .map_err(|_| "source ptr_metadata source query budget exhausted")?;
        validate(facts, *function)?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("invalid source ptr_metadata arguments")?;
        let (Some(Argument::Type(pointee)), Some(Argument::Type(output)), None) =
            (args.next(), args.next(), args.next())
        else {
            return Err("source ptr_metadata requires its complete two-type instance");
        };
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: false,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("selected source ptr_metadata changed its safe Rust signature");
        };
        let inputs = facts
            .types
            .list(parameters)
            .ok_or("selected source ptr_metadata input disappeared")?;
        if result != output
            || inputs.len() != 1
            || facts.types.kind(inputs[0])
                != Some(Kind::Pointer {
                    mutable: false,
                    pointee,
                })
        {
            return Err(
                "selected source ptr_metadata signature differs from its complete instance",
            );
        }
        for role in [LangItem::PointeeTrait, LangItem::MetadataType] {
            self.validated_language(role)?
                .ok_or("source ptr_metadata role is not complete")?;
        }
        Ok(Some(PtrMetadataSelection {
            definition,
            signature,
            pointee,
            output,
            instance,
        }))
    }

    pub(crate) fn require_ptr_metadata_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &PtrMetadataSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        selected.current(facts)?;
        super::pointee::authenticate(report, facts)
            .map_err(|_| "source ptr_metadata lost its current source role image")?;
        let Some((declaration, SourceIntrinsic::PtrMetadata, _)) =
            self.intrinsic_record(selected.definition)?
        else {
            return Err("source ptr_metadata lost its actual intrinsic declaration");
        };
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source ptr_metadata lost its current source header")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
        {
            return Err("source ptr_metadata requires its positive source header contract");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/ptr_intrinsic.rs"]
mod tests;
