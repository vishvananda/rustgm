//! A wrapper interface admits only catalogued external Self lifetime premises.
use super::*;
use crate::{
    support::call_traits::CallTrait,
    types::{
        GenericArgs, Kind, TypeId, callable::Convention, closure::CallKind, instance::Instance,
    },
};
type Result<T = ()> = std::result::Result<T, &'static str>;
fn reference(types: &mut Store, region: RegionId, referent: TypeId) -> Result<TypeId> {
    types.intern(Kind::Reference {
        region,
        referent,
        mutable: false,
    })
}
fn function(types: &mut Store, inputs: &[TypeId], count: u32) -> Result<TypeId> {
    let parameters = types.intern_list(inputs)?;
    types.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: count,
        unsafe_: false,
        convention: Convention::RUST,
    })
}
fn fixture() -> Result<(Store, Execution, Adapter)> {
    let mut types = Store::default();
    let identity = types.declare_closure(CallKind::Shared)?;
    let input = reference(&mut types, RegionId::bound(0, 0)?, TypeId::UNIT)?;
    let public = function(&mut types, &[input], 1)?;
    let external = reference(&mut types, RegionId::bound(0, 1)?, TypeId::UNIT)?;
    let fields = types.intern_list(&[external])?;
    let environment = types.intern(Kind::Tuple(fields))?;
    let self_ty = types.intern(Kind::Closure {
        identity,
        signature: public,
        environment,
        arguments: GenericArgs::EMPTY,
    })?;
    let receiver = reference(&mut types, RegionId::bound(0, 2)?, self_ty)?;
    let signature = function(&mut types, &[receiver, input], 3)?;
    let execution = Execution {
        self_ty,
        signature,
        universals: 3,
        receiver: CallKind::Shared,
    };
    let adapter = Adapter::new(
        &types,
        Instance::closure(&types, self_ty)?,
        CallTrait::FnOnce,
    )?;
    Ok((types, execution, adapter))
}
#[test]
fn wrapper_premises_exclude_public_receiver_private_and_unused_slots() -> Result {
    let (types, execution, adapter) = fixture()?;
    let external = RegionId::bound(0, 1)?;
    let declared = [(external, RegionId::STATIC), (RegionId::STATIC, external)];
    validate(&types, execution, 2, adapter, &declared)?;
    for invalid in [
        RegionId::bound(0, 0)?,
        RegionId::bound(0, 2)?,
        RegionId::bound(0, 3)?,
        RegionId::bound(1, 1)?,
        RegionId(1),
    ] {
        assert!(validate(&types, execution, 2, adapter, &[(invalid, external)]).is_err());
        assert!(validate(&types, execution, 2, adapter, &[(external, invalid)]).is_err());
    }
    Ok(())
}
#[test]
fn wrapper_contract_cannot_inject_or_replace_declared_premises() -> Result {
    let (types, execution, adapter) = fixture()?;
    let source = crate::resolve::unit::NodeKey {
        file: crate::support::source::FileId(0),
        node: crate::syntax::ast::NodeId(0),
    };
    let pair = (RegionId::bound(0, 1)?, RegionId::STATIC);
    let required = Requirements(vec![pair]);
    let mut contract = super::super::contract(source, adapter, execution);
    assert!(required.validate(&contract).is_err());
    contract.assumptions.push(pair);
    required.validate(&contract)?;
    assert!(Requirements(Vec::new()).validate(&contract).is_err());
    contract.assumptions.push((RegionId(1), RegionId::STATIC));
    assert!(required.validate(&contract).is_err());
    contract.assumptions = vec![(pair.1, pair.0)];
    assert!(required.validate(&contract).is_err());
    contract.assumptions = vec![pair];
    let input = Requirements(vec![pair]);
    input.validate(&contract)?;
    assert!(validate(&types, execution, 2, adapter, &[(RegionId(1), pair.1)]).is_err());
    Ok(())
}
