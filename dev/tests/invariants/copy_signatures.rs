//! Public callable identity and the opened body signature are separate guards.
use super::*;
use crate::body::ir::Owner;

#[test]
fn specialization_retains_exact_public_and_owned_signatures() -> Result {
    let (header, _) = cases::GENERIC_COPY.split_once("fn twice").ok_or("header")?;
    let source =
        format!("{header}fn inspect<T>(x:u8)->u8{{x}}fn answer()->u8{{inspect::<&u8>(7)}}");
    let (mut facts, templates, contracts) = fixture_source(&source)?;
    let selected = facts.nodes.iter().flatten().filter_map(|node| node.ty).find(|&ty| {
        matches!(facts.types.kind(ty), Some(Kind::FunctionItem { arguments, .. })
            if arguments != GenericArgs::EMPTY && facts.types.contains_parameters(ty) == Some(false))
    }).ok_or("selected reference instance")?;
    let instance = Instance::new(&facts.types, selected)?;
    let Some(Kind::FunctionItem {
        item,
        signature: public,
        ..
    }) = facts.types.kind(selected)
    else {
        return Err("selected callable".into());
    };
    let template = templates.bodies.iter().find(|t|
        matches!(t.body.owner, Owner::Function { definition, .. } if definition.0 == item.0)
    ).ok_or("matching source template")?;
    let original = contracts
        .iter()
        .find(|c| c.key == template.body.key())
        .ok_or("original contract")?;
    let mut materialized =
        body::instantiate::with_contract(&mut facts, template, instance, original)?;
    let owned = materialized.contract.signature.ok_or("owned signature")?;
    assert_ne!(owned, public);
    assert!(matches!(
        facts.types.kind(public),
        Some(Kind::Function {
            bound_regions: 0,
            ..
        })
    ));
    assert!(matches!(
        facts.types.kind(owned),
        Some(Kind::Function {
            bound_regions: 1,
            ..
        })
    ));
    assert!(matches!(materialized.body.owner,
        Owner::Function { signature, execution: Some(execution), .. }
            if signature == public && execution == owned));
    prepare(&mut facts, &materialized.body, &mut materialized.contract)?;
    check_pair(&materialized.body, &materialized.contract, public, owned)?;

    let mut foreign = original.clone();
    foreign.signature = Some(owned);
    assert!(body::instantiate::with_contract(&mut facts, template, instance, &foreign).is_err());
    let mut foreign = template.clone();
    if let Owner::Function { execution, .. } = &mut foreign.body.owner {
        *execution = Some(owned);
    }
    assert!(body::instantiate::with_contract(&mut facts, &foreign, instance, original).is_err());
    Ok(())
}

fn check_pair(body: &Body, contract: &Contract, public: TypeId, owned: TypeId) -> Result {
    let builtins = contract.builtins.as_ref().ok_or("builtins")?;
    let prepared = builtins.prepared_for_test()?;
    let requirements = builtins.source_copy.as_ref().ok_or("Copy requirements")?;
    let environment = builtins.environment.as_ref();
    let ty = prepared
        .decisions
        .keys()
        .next()
        .copied()
        .ok_or("read decision")?;
    prepared.decision(body, requirements, environment, ty)?;
    for change_public in [false, true] {
        let mut wrong = body.clone();
        if let Owner::Function {
            signature,
            execution,
            ..
        } = &mut wrong.owner
        {
            if change_public {
                *signature = owned;
            } else {
                *execution = Some(public);
            }
        }
        assert!(
            prepared
                .decision(&wrong, requirements, environment, ty)
                .is_err()
        );
    }
    let mut wrong = body.clone();
    wrong.instance = None;
    assert!(
        prepared
            .decision(&wrong, requirements, environment, ty)
            .is_err()
    );
    Ok(())
}

#[test]
fn signature_pair_preserves_hot_body_record_sizes() {
    assert_eq!(size_of::<Owner>(), 20);
    assert_eq!(size_of::<Body>(), 448);
    assert_eq!(size_of::<Contract>(), 184);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    println!("Prepared signature pair bytes={}", size_of::<Prepared>());
}
