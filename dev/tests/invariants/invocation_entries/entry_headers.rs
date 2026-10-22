//! Shared selectors require an opened physical signature owned by the actual caller.
use super::*;
#[test]
fn malformed_opened_headers_cannot_select_execution()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut input = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .next()
        .ok_or("contract")?;
    let caller = input.source;
    let mut proofs = crate::body::instantiate::ProofCursor::default();
    let selected = function(&mut facts.types, &[TypeId::UNIT], 0)?;
    assert!(
        select_execution(
            &mut facts,
            &mut input,
            caller,
            CallTrait::FnOnce,
            TypeId::UNIT,
            selected,
            &mut proofs,
        )?
        .is_none()
    );
    let wrong = crate::resolve::unit::NodeKey {
        node: crate::syntax::ast::NodeId::new(u32::MAX),
        ..caller
    };
    assert!(
        select_execution(
            &mut facts,
            &mut input,
            wrong,
            CallTrait::FnOnce,
            TypeId::UNIT,
            selected,
            &mut proofs,
        )
        .is_err()
    );
    let different = TypeId::primitive(crate::types::Primitive::U8);
    assert!(
        select_execution(
            &mut facts,
            &mut input,
            caller,
            CallTrait::FnOnce,
            different,
            selected,
            &mut proofs,
        )
        .is_err()
    );
    let bound = function(&mut facts.types, &[TypeId::UNIT], 1)?;
    assert!(
        select_execution(
            &mut facts,
            &mut input,
            caller,
            CallTrait::FnOnce,
            TypeId::UNIT,
            bound,
            &mut proofs,
        )
        .is_err()
    );
    let Type::Function {
        parameters, result, ..
    } = facts.types.kind(selected).ok_or("signature")?
    else {
        return Err("function".into());
    };
    for (unsafe_, convention) in [(true, Convention::RUST), (false, Convention::C)] {
        let invalid = facts.types.intern(Type::Function {
            parameters,
            result,
            bound_regions: 0,
            unsafe_,
            convention,
        })?;
        assert!(
            select_execution(
                &mut facts,
                &mut input,
                caller,
                CallTrait::FnOnce,
                TypeId::UNIT,
                invalid,
                &mut proofs,
            )
            .is_err()
        );
    }
    Ok(())
}
