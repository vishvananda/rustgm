//! Reached discriminant calls retain actual source identity and ordinary call proof.
mod opened_calls;
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::bindings::{LangItem, SourceIntrinsic},
    types::{
        Kind, RegionId, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};

/// Data authority only; const/body/call/region permission is checked separately.
pub(crate) struct DiscriminantSelection {
    definition: DefId,
    signature: TypeId,
    input: TypeId,
    output: TypeId,
    instance: Instance,
}
impl DiscriminantSelection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) const fn referent(&self) -> TypeId {
        self.input
    }
    pub(crate) const fn output(&self) -> TypeId {
        self.output
    }
    pub(crate) fn input_matches(&self, store: &crate::types::Store, ty: TypeId) -> bool {
        matches!(store.kind(ty), Some(Kind::Reference { mutable: false, referent, .. }) if referent == self.input)
    }
    fn current(&self, facts: &Declarations) -> Result<(), &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("source discriminant lost its actual function item");
        };
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source discriminant lost actual arguments")?;
        if item.0 != self.definition.0
            || signature != self.signature
            || args.next() != Some(Argument::Type(self.input))
            || args.next().is_some()
        {
            return Err("source discriminant changed its complete instance");
        }
        selected_signature(facts, self.signature, self.input, self.output)
    }
}

/// Prepare the selected input's relocation before read-only execution checks.
pub(crate) fn prepare_discriminant(
    facts: &mut Declarations,
    instance: Instance,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem { arguments, .. }) = facts.types.kind(instance.ty()) else {
        return Err("source discriminant lacks its actual function item");
    };
    let input = instance_input(&facts.types, arguments)?;
    let before = facts.types.capture_shift_steps();
    let result = facts.types.prepare_shift(input, 1);
    let work = facts.types.capture_shift_steps().saturating_sub(before);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result
}

fn instance_input(
    types: &crate::types::Store,
    arguments: crate::types::GenericArgs,
) -> Result<TypeId, &'static str> {
    let mut args = types
        .arguments(arguments)
        .ok_or("invalid discriminant instance arguments")?;
    let (Some(Argument::Type(input)), None) = (args.next(), args.next()) else {
        return Err("source discriminant requires its complete single-type instance");
    };
    Ok(input)
}

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let parameters = facts
        .generics
        .parameters
        .get(&function.definition)
        .ok_or("source discriminant requires one own Sized type parameter")?;
    let [parameter] = parameters.as_slice() else {
        return Err("source discriminant requires exactly one own type parameter");
    };
    let Argument::Type(input) = parameter.argument else {
        return Err("source discriminant requires a type parameter");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(input) else {
        return Err("source discriminant lost its type parameter");
    };
    if facts.types.parameter(id).is_none_or(|p| {
        p.owner.0 != function.definition.0
            || p.index != 0
            || !matches!(p.kind, ParameterKind::Type { sized: true })
    }) || !facts.is_const_function(function.definition)
        || function.body.is_some()
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_none_or(|r| {
                r.captures != 0 || !r.early.is_empty() || r.late != [0] || !r.bounds.is_empty()
            })
    {
        return Err(
            "source discriminant requires its actual bodyless const declaration and one input lifetime",
        );
    }
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("source discriminant lacks its actual callable signature");
    };
    let Some(Kind::Projection(application)) = facts.types.kind(result) else {
        return Err("source discriminant requires its actual associated result");
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source discriminant has no actual catalog")?;
    let trait_ = catalog
        .language(LangItem::DiscriminantKind)
        .ok_or("source discriminant requires actual DiscriminantKind")?;
    let member = catalog
        .language(LangItem::DiscriminantType)
        .ok_or("source discriminant requires actual discriminant_type")?;
    if member.parent != Some(trait_.definition)
        || application.item.0 != member.definition.0
        || application.subject != input
        || facts
            .types
            .trait_application(application.trait_application)
            .is_none_or(|a| {
                a.item.0 != trait_.definition.0
                    || facts
                        .types
                        .arguments(a.arguments)
                        .is_none_or(|a| a.len() != 0)
            })
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err("source discriminant requires its complete genuine trait/member application");
    }
    selected_signature(facts, function.signature, input, result)
}

fn selected_signature(
    facts: &Declarations,
    signature: TypeId,
    input: TypeId,
    output: TypeId,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err(
            "source discriminant requires a safe Rust signature with one late input lifetime",
        );
    };
    let Some([parameter]) = facts.types.list(parameters) else {
        return Err("source discriminant requires exactly one shared reference");
    };
    // A lifetime of the caller's own scopes sits one binder deeper under the
    // late input lifetime; only its already prepared relocation is accepted.
    let referent = if facts.types.escaping_binders(input) == Some(0) {
        Some(input)
    } else {
        facts.types.prepared_shift(input, 1)?
    };
    let region = RegionId::bound(0, 0)?;
    if result != output
        || referent.is_none_or(|referent| {
            facts.types.kind(*parameter)
                != Some(Kind::Reference {
                    mutable: false,
                    region,
                    referent,
                })
        })
    {
        return Err("source discriminant signature differs from its actual shared input/result");
    }
    Ok(())
}

impl Catalog {
    pub(super) fn discriminant_header(
        &self,
        facts: &Declarations,
        definition: DefId,
        origin: Option<BodyOrigin>,
    ) -> Result<bool, &'static str> {
        if let Some((declaration, kind, original)) = self.intrinsic_record(definition)? {
            if kind != SourceIntrinsic::DiscriminantValue {
                return Ok(false);
            }
            if origin.is_some_and(|origin| origin.source().is_none())
                || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
            {
                return Err(
                    "imported discriminant intrinsic operation provenance is not transported",
                );
            }
            if origin.is_some() {
                crate::sema::header_attributes::charge_source_contract_query(facts, 16)
                    .map_err(|_| "source discriminant call exceeds its existing work budget")?;
            }
            let function = facts
                .function_indices
                .get(&definition)
                .and_then(|i| facts.functions.get(*i))
                .ok_or("source discriminant lacks its completed declaration")?;
            if function.signature != original
                || function.node != declaration.origin
                || declaration.parent.is_some()
            {
                return Err("source discriminant completed header changed");
            }
            validate(facts, *function)?;
            return Ok(true);
        }
        let Some(imported) = self.imported_declared_intrinsic(definition) else {
            return Ok(false);
        };
        if imported.kind != SourceIntrinsic::DiscriminantValue {
            return Ok(false);
        }
        let actual = facts
            .declaration_header(definition)?
            .ok_or("imported discriminant lost its canonical declaration")?;
        if actual.declaration != imported.origin
            || actual.signature != imported.signature
            || actual.body.is_some()
            || !facts.is_const_function(definition)
        {
            return Err("imported discriminant changed its canonical declaration");
        }
        Ok(true)
    }

    pub(crate) fn discriminant_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<DiscriminantSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        if !self.discriminant_header(facts, definition, Some(origin))? {
            return Ok(None);
        }
        let input = instance_input(&facts.types, arguments)?;
        let owner = self
            .traits
            .as_deref()
            .and_then(|r| r.discriminant.as_ref())
            .ok_or("source discriminant protocol is not complete")?;
        owner
            .current_facts(facts)
            .map_err(|_| "source discriminant protocol is stale")?;
        let output = match super::discriminant::classify(facts, input)
            .map_err(|_| "invalid discriminant type query")?
        {
            super::discriminant::Output::Ready(output) => output,
            super::discriminant::Output::Fields(_) => {
                return Err("source discriminant requires completed nominal representation");
            }
            super::discriminant::Output::Unresolved => {
                return Err("source discriminant output remains an unresolved projection");
            }
        };
        selected_signature(facts, signature, input, output)?;
        Ok(Some(DiscriminantSelection {
            definition,
            signature,
            input,
            output,
            instance,
        }))
    }

    pub(crate) fn require_discriminant_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &DiscriminantSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        selected.current(facts)?;
        super::discriminant::authenticate(report, facts)
            .map_err(|_| "source discriminant role image changed")?;
        let Some((declaration, SourceIntrinsic::DiscriminantValue, _)) =
            self.intrinsic_record(selected.definition)?
        else {
            let Some(imported) = self.imported_declared_intrinsic(selected.definition) else {
                return Err("source discriminant intrinsic declaration changed");
            };
            if imported.kind != SourceIntrinsic::DiscriminantValue
                || !facts.is_const_function(selected.definition)
                || facts
                    .declaration_header(selected.definition)?
                    .is_none_or(|actual| {
                        actual.declaration != imported.origin || actual.body.is_some()
                    })
            {
                return Err("imported discriminant changed its canonical declaration");
            }
            return Ok(());
        };
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source discriminant source header changed")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
        {
            return Err("source discriminant requires positive actual source-header completion");
        }
        Ok(())
    }
}
