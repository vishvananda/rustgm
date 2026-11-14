//! Closed source member requests retain exact typed facts; executable entry is separate.
use super::*;
#[test]
fn closed_const_member_requests_reject_forged_owner_site_signature_and_selection() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-add-sub/macro-const-with-location.rs")?;
    let report = crate::test_source::source_provider(&source, "const_member_initializers")?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let initializer = facts
        .initializers
        .iter()
        .find(|i| {
            report
                .package
                .names
                .get(report.resolution.definitions[i.definition.0 as usize].name)
                == "SHARED"
        })
        .ok_or("actual SHARED initializer")?;
    let body = prepared
        .templates
        .initializer(&report, &facts, initializer)?
        .body;
    let different = facts
        .initializers
        .iter()
        .find(|i| {
            report
                .package
                .names
                .get(report.resolution.definitions[i.definition.0 as usize].name)
                == "SUM"
        })
        .ok_or("different actual initializer")?;
    let index = statement(&body)?;
    let mut work = Work::new(1_000_000);
    assert!(source_request(
        &report,
        &facts,
        &body,
        &body.statements[index],
        &mut work
    )?);
    let Action::Evaluate(id) = body.statements[index].action else {
        return Err("statement".into());
    };
    let Rvalue::Call {
        callee,
        signature: _,
        arguments,
    } = body.values[id.0 as usize].kind
    else {
        return Err("Call".into());
    };
    for mutation in 0..5 {
        let mut changed = body.clone();
        match mutation {
            0 => changed.values[id.0 as usize].ty = crate::types::TypeId::UNIT,
            1 => {
                changed.values[id.0 as usize].kind = Rvalue::Call {
                    callee,
                    signature: crate::types::TypeId::UNIT,
                    arguments,
                }
            }
            2 => changed.values[callee.0 as usize].kind = Rvalue::Function(initializer.definition),
            3 => changed.statements[index].source = different.semantic_source().node.into(),
            _ => {
                changed.owner = Owner::Initializer {
                    definition: different.definition,
                    kind: different.kind,
                }
            }
        }
        assert!(
            source_request(
                &report,
                &facts,
                &changed,
                &changed.statements[index],
                &mut work
            )
            .is_err(),
            "changed actual owner/site/signature/value {mutation} was accepted"
        );
    }
    let mut tiny = Work::new(1);
    assert!(source_request(&report, &facts, &body, &body.statements[index], &mut tiny).is_err());
    assert!(tiny.steps > 1);
    Ok(())
}
#[test]
fn runtime_member_calls_cannot_acquire_const_permission_from_a_body_flag() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-add-sub/explicit-shared-bodies.rs")?;
    let report = crate::test_source::source_provider(&source, "runtime_member_flags")?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "explicit_add"
        })
        .ok_or("function")?;
    let template = prepared.templates.function(&report, &facts, function)?;
    let mut body = template.body;
    body.const_context = true;
    let index = statement(&body)?;
    assert!(!source_request(
        &report,
        &facts,
        &body,
        &body.statements[index],
        &mut Work::new(1_000_000)
    )?);
    Ok(())
}

#[test]
fn const_index_operand_and_expected_receiver_remain_distinct_authenticated_inputs() -> Result {
    let source = crate::test_source::fixture("pa31/tests/spec/source-index-places/const-valid.rs")?;
    let report = crate::test_source::source_provider(&source, "const_index_operands")?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let (site, original) = facts
        .methods
        .calls
        .iter()
        .find(|(site, _)| {
            matches!(
                report
                    .package
                    .unit(site.file)
                    .configured
                    .ast
                    .node(site.node)
                    .kind,
                crate::syntax::ast::Kind::Index { .. }
            )
        })
        .map(|(site, method)| (*site, *method))
        .ok_or("actual closed index")?;
    let Some(crate::sema::model::Operation::Call { signature, .. }) =
        facts.node(site).and_then(|node| node.operation)
    else {
        return Err("actual index Call".into());
    };
    let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
        return Err("actual index signature".into());
    };
    let call = Call {
        site,
        target: original.callable,
        signature,
        result,
    };
    let node_type = facts.node(site).and_then(|node| node.ty);
    let mut work = Work::new(1_000_000);
    place_result(&report, &facts, call, node_type, &mut work)?;
    assert_ne!(original.borrow, Some(original.receiver));
    let Some(Kind::Reference {
        mutable,
        region,
        referent,
    }) = original.borrow.and_then(|ty| facts.types.kind(ty))
    else {
        return Err("actual operand borrow".into());
    };
    let wrong_mutability = facts.types.intern(Kind::Reference {
        mutable: !mutable,
        region,
        referent,
    })?;
    let wrong_referent = facts.types.intern(Kind::Reference {
        mutable,
        region,
        referent: crate::types::TypeId::UNIT,
    })?;
    for mutation in 0..4 {
        let mut changed = original;
        match mutation {
            0 => changed.receiver = crate::types::TypeId::UNIT,
            1 => changed.borrow = Some(wrong_mutability),
            2 => changed.borrow = Some(wrong_referent),
            _ => changed.borrow = None,
        }
        facts.methods.calls.insert(site, changed);
        assert!(place_result(&report, &facts, call, node_type, &mut work).is_err());
    }
    facts.methods.calls.insert(site, original);
    assert!(place_operand(&facts, site, site.node, original.borrow, mutable, &mut work).is_err());
    place_result(&report, &facts, call, node_type, &mut work)?;
    Ok(())
}

#[test]
fn generic_const_field_requests_reject_changed_arguments_and_body_flags() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/conditional-member-calls/shared-field.rs")?;
    let report = crate::test_source::source_provider(&source, "generic_member_requests")?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "field"
        })
        .ok_or("actual generic field function")?;
    let body = prepared.templates.function(&report, &facts, function)?.body;
    let index = statement(&body)?;
    let mut work = Work::new(1_000_000);
    assert!(source_request(
        &report,
        &facts,
        &body,
        &body.statements[index],
        &mut work
    )?);
    let Action::Evaluate(value) = body.statements[index].action else {
        return Err("actual field statement".into());
    };
    let Rvalue::Call { callee, .. } = body.values[value.0 as usize].kind else {
        return Err("actual field call".into());
    };
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(body.values[callee.0 as usize].ty)
    else {
        return Err("actual field callee".into());
    };
    let wrong = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments: crate::types::GenericArgs::EMPTY,
    })?;
    let mut changed = body.clone();
    changed.values[callee.0 as usize].ty = wrong;
    assert!(
        source_request(
            &report,
            &facts,
            &changed,
            &changed.statements[index],
            &mut work
        )
        .is_err()
    );
    let mut changed = body.clone();
    changed.const_context = false;
    assert!(!source_request(
        &report,
        &facts,
        &changed,
        &changed.statements[index],
        &mut work
    )?);
    Ok(())
}

#[test]
fn explicit_deref_place_keeps_its_actual_receiver_signature_and_referent() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/explicit-const-dereference/shared.rs")?;
    let report = crate::test_source::source_provider(&source, "explicit_place_proof")?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "field"
        })
        .ok_or("actual explicit deref function")?;
    let body = prepared.templates.function(&report, &facts, function)?.body;
    let index = statement(&body)?;
    let mut work = Work::new(1_000_000);
    let call = source_call(&facts, &body, &body.statements[index], &mut work)?;
    let node_type = facts.node(call.site).and_then(|node| node.ty);
    let original = *facts
        .methods
        .calls
        .get(&call.site)
        .ok_or("actual selected deref")?;
    place_result(&report, &facts, call, node_type, &mut work)?;
    for mutation in 0..3 {
        let mut changed = original;
        match mutation {
            0 => changed.receiver = crate::types::TypeId::UNIT,
            1 => changed.borrow = None,
            _ => changed.callable = crate::types::TypeId::UNIT,
        }
        facts.methods.calls.insert(call.site, changed);
        assert!(place_result(&report, &facts, call, node_type, &mut work).is_err());
    }
    facts.methods.calls.insert(call.site, original);
    assert!(
        place_result(
            &report,
            &facts,
            call,
            Some(crate::types::TypeId::UNIT),
            &mut work,
        )
        .is_err()
    );
    let Some(Kind::Function {
        result,
        unsafe_,
        convention,
        ..
    }) = facts.types.kind(call.signature)
    else {
        return Err("actual opened deref signature".into());
    };
    let parameters = facts.types.intern_list(&[])?;
    let signature = facts.types.intern(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_,
        convention,
    })?;
    assert!(
        place_result(
            &report,
            &facts,
            Call { signature, ..call },
            node_type,
            &mut work
        )
        .is_err()
    );
    place_result(&report, &facts, call, node_type, &mut work)?;
    Ok(())
}

#[test]
fn normalized_default_requests_require_their_actual_defining_context() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/default-generic-subjects/reference.rs")?;
    let report = crate::test_source::source_provider(&source, "default_requests")?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "check"
        })
        .ok_or("actual default")?;
    let definition = function.definition;
    let body = prepared.templates.function(&report, &facts, function)?.body;
    let index = statement(&body)?;
    let mut work = Work::new(1_000_000);
    let context = prepare(&facts, &body, definition, &mut work)?;
    assert!(!source_request(
        &report,
        &facts,
        &body,
        &body.statements[index],
        &mut work
    )?);
    assert!(source_request_in_scope(
        &report,
        &facts,
        &body,
        &body.statements[index],
        Some(&context),
        &mut work
    )?);
    let call = source_call(&facts, &body, &body.statements[index], &mut work)?;
    let other = facts
        .functions
        .iter()
        .filter_map(|function| function.body)
        .find(|source| Some(*source) != body.source.source())
        .ok_or("another actual body")?;
    assert!(
        context
            .source_default_request(other, call.target, &mut |amount| work.charge(amount))
            .is_err()
    );
    Ok(())
}
