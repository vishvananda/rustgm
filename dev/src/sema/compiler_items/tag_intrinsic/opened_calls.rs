//! Public late binders open once in the existing checked source call context.
use super::{Catalog, Declarations, DiscriminantSelection, Kind, TypeId};
use crate::{
    resolve::{expansion::Report, model::DefId, unit::NodeKey},
    sema::model::{BodyRegionOrigin, Operation},
    support::bindings::SourceIntrinsic,
    syntax::ast::Kind as Syntax,
    types::{Store, arguments::Argument, callable::Convention},
};

fn opened_input(
    store: &Store,
    signature: TypeId,
    input: TypeId,
    output: TypeId,
) -> Result<crate::types::RegionId, &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = store.kind(signature)
    else {
        return Err("source discriminant requires its actual opened safe Rust Call signature");
    };
    let Some([parameter]) = store.list(parameters) else {
        return Err("source discriminant Call requires one shared reference");
    };
    let Some(Kind::Reference {
        mutable: false,
        referent,
        region,
    }) = store.kind(*parameter)
    else {
        return Err("source discriminant Call requires its actual shared input");
    };
    if referent != input || result != output || region.bound_parts().is_some() {
        return Err("source discriminant opened input/result differs from its complete instance");
    }
    Ok(region)
}

fn public_signature(store: &Store, signature: TypeId) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = store.kind(signature)
    else {
        return Err("source discriminant opening lost its safe Rust public binder");
    };
    let Some([parameter]) = store.list(parameters) else {
        return Err("source discriminant public signature requires one reference");
    };
    if !matches!(store.kind(*parameter),Some(Kind::Reference{mutable:false,region,..}) if region.bound_parts()==Some((0,0)))
    {
        return Err("source discriminant public signature changed its actual shared late input");
    }
    Ok(())
}

impl DiscriminantSelection {
    /// This reads data only after the ordinary original/instance call proof.
    pub(crate) fn opened_signature_matches(&self, store: &Store, signature: TypeId) -> bool {
        opened_input(store, signature, self.input, self.output).is_ok()
    }
}

impl Catalog {
    /// Called by the existing CFG Call producer while its actual typed node is available.
    /// No new call/region proof or retained token is created here.
    pub(crate) fn validate_discriminant_opening(
        &self,
        report: &Report,
        facts: &Declarations,
        owner: NodeKey,
        call: NodeKey,
        checked: (TypeId, TypeId),
    ) -> Result<(), &'static str> {
        if facts
            .source_bindings
            .as_deref()
            .is_none_or(|actual| !std::ptr::eq(actual, self))
            || owner.file != call.file
        {
            return Err("source discriminant opening belongs to another caller catalog");
        }
        self.require_provider_report(report)?;
        super::super::discriminant::authenticate(report, facts)
            .map_err(|_| "source discriminant opening lost its current source protocol seal")?;
        crate::sema::header_attributes::charge_source_contract_query(facts, 8)
            .map_err(|_| "source discriminant opening exceeds its existing source work budget")?;
        let (callable, signature) = checked;
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature: public,
        }) = facts.types.kind(callable)
        else {
            return Err("source discriminant opening lost its full FunctionItem");
        };
        let definition = DefId(item.0);
        let Some((declaration, SourceIntrinsic::DiscriminantValue, original)) =
            self.intrinsic_record(definition)?
        else {
            return Err("source discriminant opening has no validated intrinsic declaration");
        };
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|i| facts.functions.get(*i))
            .ok_or("source discriminant opening lost its actual function")?;
        if function.signature != original || function.node != declaration.origin {
            return Err("source discriminant opening changed its source declaration");
        }
        super::validate(facts, *function)?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source discriminant opening lost its full arguments")?;
        let (Some(Argument::Type(input)), None) = (args.next(), args.next()) else {
            return Err("source discriminant opening requires exactly one own type argument");
        };
        public_signature(&facts.types, public)?;
        let output = facts
            .node(call)
            .and_then(|node| node.ty)
            .ok_or("source discriminant opening lost its current checked result")?;
        // The ordinary checked FnItem/Call relation owns substitution under
        // the public binder, including aliases and outer captured lifetimes.
        let region = opened_input(&facts.types, signature, input, output)?;
        checked_node(
            report,
            facts,
            owner,
            call,
            (callable, signature, input, region),
        )
    }
}

fn checked_node(
    report: &Report,
    facts: &Declarations,
    owner: NodeKey,
    call: NodeKey,
    expected: (TypeId, TypeId, TypeId, crate::types::RegionId),
) -> Result<(), &'static str> {
    let (callable, signature, input, region) = expected;
    let Some(Operation::Call {
        signature: actual,
        source_use: Some(source_use),
    }) = facts.node(call).and_then(|fact| fact.operation)
    else {
        return Err("source discriminant opening requires its current checked Operation::Call");
    };
    if actual != signature {
        return Err("source discriminant opening changed its current checked signature");
    }
    source_use.effect_for(facts, owner)?;
    let allocated = facts
        .body_regions
        .get(owner, region)
        .ok_or("source discriminant opening has no owned call-argument region")?;
    if allocated.body != owner
        || allocated.source != call
        || !matches!(allocated.origin, BodyRegionOrigin::CallArgument)
    {
        return Err("source discriminant opening borrowed another call's region");
    }
    let ast = &report.package.unit(call.file).configured.ast;
    let Syntax::Call { callee, arguments } = ast.node(call.node).kind else {
        return Err("source discriminant opening requires its actual call syntax");
    };
    if facts
        .node(NodeKey {
            node: callee,
            ..call
        })
        .and_then(|fact| fact.ty)
        != Some(callable)
    {
        return Err("source discriminant opening changed its actual complete callee");
    }
    let [argument] = ast.list(arguments) else {
        return Err("source discriminant opening requires one actual argument");
    };
    let actual = facts
        .node(NodeKey {
            node: *argument,
            ..call
        })
        .ok_or("source discriminant argument was not checked")?;
    let argument = actual
        .adjustment
        .map(|a| a.target)
        .or(actual.ty)
        .ok_or("source discriminant argument has no checked type")?;
    if !matches!(facts.types.kind(argument),Some(Kind::Reference{mutable:false,referent,..}) if referent==input)
    {
        return Err("source discriminant argument differs from its typed shared input");
    }
    Ok(())
}
