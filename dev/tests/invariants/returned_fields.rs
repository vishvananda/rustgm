//! Lifetime-only returned receivers preserve exact owner and binder arity.
use super::{Checker, ItemId, NodeId};
use crate::{
    sema,
    types::{Kind, Primitive, RegionId, TypeId, infer::arguments::Argument},
};
#[test]
fn lifetime_only_receiver_checks_owner_arity_and_substitution()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve(
        "struct D<'a>(&'a u8);struct E<'a>(&'a u8);fn main(){}",
        true,
    )?;
    let mut facts = sema::declarations::analyze(&report)?;
    let owner = |name| {
        report
            .resolution
            .definitions
            .iter()
            .position(|definition| report.package.names.get(definition.name) == name)
            .and_then(|index| u32::try_from(index).ok())
            .map(ItemId)
            .ok_or("missing owner")
    };
    let d = owner("D")?;
    let e = owner("E")?;
    let mut checker = Checker::new(&report, &mut facts);
    let receiver = checker
        .state
        .infer
        .nominal(d, &[Argument::Lifetime(RegionId::STATIC)])?;
    let mut fields = checker.receiver_fields(NodeId::new(0), receiver, d)?;
    let template = checker.facts.types.intern(Kind::Reference {
        region: RegionId::bound(0, 0)?,
        mutable: false,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let actual = fields.regions.apply(&mut checker.facts.types, template)?;
    assert!(matches!(
        checker.facts.types.kind(actual),
        Some(Kind::Reference {
            region: RegionId::STATIC,
            ..
        })
    ));
    assert!(
        checker
            .receiver_fields(NodeId::new(0), receiver, e)
            .is_err()
    );
    for arguments in [
        Vec::new(),
        vec![Argument::Lifetime(RegionId::STATIC); 2],
        vec![Argument::Type(receiver)],
    ] {
        let malformed = checker.state.infer.nominal(d, &arguments)?;
        assert!(
            checker
                .receiver_fields(NodeId::new(0), malformed, d)
                .is_err()
        );
    }
    Ok(())
}
