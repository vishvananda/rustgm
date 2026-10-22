//! Original declaration contexts retain exact source binders, never call authority.
use super::*;
use crate::{
    body::{ir::BodyKey, regions},
    sema::inline_constants::{Context, InlineInstance},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn lifetime_only_original_context_rejects_forged_headers_and_preserves_call_admission() -> Result {
    let report = crate::test_source::resolve(
        "fn f<'a:'b,'b>(_:&'a u8,_:&'b u8)->u8{const{7}}fn main(){}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let inline = facts
        .inline_constants
        .as_deref()
        .ok_or("registry")?
        .bodies()
        .next()
        .ok_or("inline")?;
    let source = inline.source;
    let owner = inline.declaration.ok_or("owner")?;
    let index = *facts.function_indices.get(&owner).ok_or("function")?;
    let ty = facts.definitions[owner.0 as usize].ok_or("definition")?;
    let callable = Instance::new(&facts.types, ty)?;
    let selected = InlineInstance::select(&facts, source, Context::Callable(callable))?;
    let mut contract = regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|input| input.key == BodyKey::Anonymous(source))
        .ok_or("contract")?;
    let original =
        original_function_context(&mut facts, selected, &contract)?.ok_or("original context")?;
    assert!(
        matches!(original.caller, crate::sema::values::execution::CallerInstance::Inline(actual) if actual==selected)
    );
    assert!(
        original.instance.is_none() && original.captures.is_empty() && original.bindings.is_empty()
    );
    assert!(
        select(&mut facts, callable).is_err(),
        "declaration evidence must not grant invocation admission"
    );
    let universals = contract.universals;
    for forged in [universals - 1, universals + 1] {
        contract.universals = forged;
        assert!(original_function_context(&mut facts, selected, &contract).is_err());
    }
    contract.universals = universals;
    contract.key = BodyKey::Declaration(owner);
    assert!(original_function_context(&mut facts, selected, &contract).is_err());
    contract.key = BodyKey::Anonymous(source);
    let original_source = contract.source;
    contract.source = facts.functions[index].node;
    assert!(original_function_context(&mut facts, selected, &contract).is_err());
    contract.source = original_source;
    let function = facts.functions[index];
    facts.functions[index].body = Some(source);
    assert!(original_function_context(&mut facts, selected, &contract).is_err());
    facts.functions[index] = function;
    facts.functions[index].signature = TypeId::UNIT;
    assert!(original_function_context(&mut facts, selected, &contract).is_err());
    facts.functions[index] = function;
    facts.functions[index].definition = DefId(owner.0 + 1);
    assert!(original_function_context(&mut facts, selected, &contract).is_err());
    facts.functions[index] = function;
    assert!(original_function_context(&mut facts, selected, &contract)?.is_some());
    forged_headers(
        &mut facts,
        selected,
        &contract,
        crate::types::ItemId(owner.0),
        function.signature,
    )?;
    Ok(())
}

fn forged_headers(
    facts: &mut Declarations,
    selected: InlineInstance,
    contract: &regions::Contract,
    item: crate::types::ItemId,
    signature: TypeId,
) -> Result {
    let header = facts.lifetimes.callables.get(item).ok_or("header")?;
    let Some(Kind::Function {
        bound_regions,
        unsafe_,
        convention,
        parameters,
        result: _,
    }) = facts.types.kind(header.signature)
    else {
        return Err("signature".into());
    };
    let forged = facts.types.intern(Kind::Function {
        bound_regions,
        unsafe_,
        convention,
        parameters,
        result: TypeId::UNIT,
    })?;
    let original_headers = std::mem::take(&mut facts.lifetimes.callables);
    facts
        .lifetimes
        .callables
        .insert(&facts.types, item, forged, header.parameters, &[])?;
    assert!(
        original_function_context(facts, selected, contract).is_err(),
        "a same-arity changed result cannot pass exact relocation"
    );
    facts.lifetimes.callables = original_headers;
    assert!(original_function_context(facts, selected, contract)?.is_some());
    let original_headers = std::mem::take(&mut facts.lifetimes.callables);
    assert!(
        original_function_context(facts, selected, contract).is_err(),
        "a missing original header must not select the invocation fallback"
    );
    facts
        .lifetimes
        .callables
        .insert_monomorphic(&facts.types, item, signature)?;
    assert!(
        original_function_context(facts, selected, contract).is_err(),
        "a zero-early forged header must not weaken original classification"
    );
    facts.lifetimes.callables = original_headers;
    Ok(())
}
