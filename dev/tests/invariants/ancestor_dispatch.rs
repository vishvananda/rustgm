//! Actual sealed facts exercise member consumers while whole-program execution stays fenced.
#[path = "ancestor_dispatch/rpit_evidence.rs"]
mod rpit_evidence;
use super::{Checker, Declarations};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
    sema::{
        dispatch::{self, Dispatcher},
        specialization,
    },
    types::{Kind, Primitive, TypeId, arguments::Argument},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn verified(report: &Report) -> TestResult<Declarations> {
    let mut checker = Checker::new(report)?;
    let error = checker.run().err().ok_or("execution fence disappeared")?;
    assert_eq!(
        error.message,
        "specialization graph execution is not implemented"
    );
    Ok(checker.facts)
}
fn declaration(report: &Report, kind: DefKind, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|row| row.kind == kind && report.package.names.get(row.name) == name)
        .ok_or("actual named declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
fn method(facts: &Declarations, report: &Report, owner: DefId, name: &str) -> TestResult<DefId> {
    let id = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|&(index, row)| {
            row.kind == DefKind::Function
                && report.package.names.get(row.name) == name
                && u32::try_from(index)
                    .ok()
                    .is_some_and(|index| facts.associated.trait_owner(DefId(index)) == Some(owner))
        })
        .map(|(index, _)| u32::try_from(index))
        .transpose()?
        .ok_or("actual trait member")?;
    Ok(DefId(id))
}
fn dispatch(
    facts: &mut Declarations,
    method: DefId,
    subject: TypeId,
) -> TestResult<(DefId, Vec<Argument>)> {
    let mut work = 0;
    let callable =
        dispatch::instantiate_callable(facts, method, &[Argument::Type(subject)], &mut work)?;
    let mut query = Dispatcher::new(0, 1)?;
    let selected = query.select(facts, callable)?;
    let Some(Kind::FunctionItem {
        item, arguments, ..
    }) = facts.types.kind(selected)
    else {
        return Err("selected method is not a real function item".into());
    };
    let values = facts
        .types
        .arguments(arguments)
        .ok_or("complete selected arguments")?
        .collect();
    assert!(
        query.finish().is_empty(),
        "scalar selection left unpaid lifetime evidence"
    );
    Ok((DefId(item.0), values))
}
fn generic_array_member(facts: &Declarations, member: DefId) -> TestResult<DefId> {
    let header = facts
        .generics
        .implementations
        .as_deref()
        .ok_or("actual registry")?
        .headers()
        .find(|header| {
            matches!(facts.types.kind(header.predicate.ty), Some(Kind::Array { element, length: 1 })
                if matches!(facts.types.kind(element), Some(Kind::Parameter(_))))
        })
        .ok_or("actual intermediate generic-array implementation")?;
    header
        .methods
        .iter()
        .find_map(|&(declaration, actual)| (declaration == member).then_some(actual).flatten())
        .ok_or_else(|| "actual directly declared array member".into())
}

#[test]
fn sealed_method_dispatch_selects_nearest_defining_owner_and_complete_arguments() -> TestResult {
    for name in [
        "chain-and-inherited-body-explicit-middle",
        "reverse-chain-order-explicit-middle",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/specialization-graph/corrected-chains/{name}.rs"
        ))?;
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = verified(&report)?;
        let before = facts
            .generics
            .implementations
            .as_deref()
            .and_then(|r| r.specialization_graph())
            .ok_or("actual graph")?
            .observation();
        let owner = declaration(&report, DefKind::Trait, "V")?;
        let inherited = method(&facts, &report, owner, "g")?;
        let target = generic_array_member(&facts, inherited)?;
        for primitive in [Primitive::U8, Primitive::U16] {
            let element = TypeId::primitive(primitive);
            let subject = facts.types.intern(Kind::Array { element, length: 1 })?;
            let (actual, arguments) = dispatch(&mut facts, inherited, subject)?;
            assert_eq!(
                actual, target,
                "{name}: requested inherited g selected another body"
            );
            assert_eq!(
                arguments,
                [Argument::Type(element)],
                "{name}: defining T must differ from selected array Self"
            );
        }
        let own = method(&facts, &report, owner, "f")?;
        let subject = facts.types.intern(Kind::Array {
            element: TypeId::primitive(Primitive::U8),
            length: 1,
        })?;
        let (actual, arguments) = dispatch(&mut facts, own, subject)?;
        assert_ne!(actual, own);
        assert!(
            arguments.is_empty(),
            "the actual concrete child has no generic inputs"
        );
        assert_ne!(actual, target);
        assert!(specialization::reject_execution(&facts, &report).is_err());
        let after = facts
            .generics
            .implementations
            .as_deref()
            .and_then(|r| r.specialization_graph())
            .ok_or("same graph")?
            .observation();
        assert_eq!(before.capacity, after.capacity);
        eprintln!(
            "specialization member dispatch {name} work_delta={} logical_bytes_delta={} graph_capacity={}",
            after.work - before.work,
            after.bytes - before.bytes,
            after.capacity
        );
    }
    let report = crate::test_source::resolve(
        "trait V{fn f()->u8;}impl V for u8{fn f()->u8{17}}fn main(){}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let owner = declaration(&report, DefKind::Trait, "V")?;
    let member = method(&facts, &report, owner, "f")?;
    let (selected, arguments) = dispatch(&mut facts, member, TypeId::primitive(Primitive::U8))?;
    assert_ne!(selected, member);
    assert!(arguments.is_empty());
    assert!(
        facts
            .generics
            .implementations
            .as_deref()
            .ok_or("ordinary registry")?
            .specialization_graph()
            .is_none()
    );
    Ok(())
}

#[test]
fn inherited_scalar_dispatch_preserves_nonstatic_caller_and_actual_definition_context() -> TestResult
{
    use crate::types::{ItemId, RegionId, outlives::Queries, relations::Constraints};
    let source = crate::test_source::fixture(
        "pa31/tests/spec/specialization-graph/execution/inherited-static-parent-nonstatic-child.rs",
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = verified(&report)?;
    let trait_owner = declaration(&report, DefKind::Trait, "V")?;
    let callable = method(&facts, &report, trait_owner, "f")?;
    let nominal = declaration(&report, DefKind::Struct, "W")?;
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(reference)])?;
    let subject = facts
        .types
        .intern(Kind::Adt(ItemId(nominal.0), arguments))?;
    let mut work = 0;
    let ty = dispatch::instantiate_callable(
        &mut facts,
        callable,
        &[Argument::Type(subject)],
        &mut work,
    )?;
    let mut query = Dispatcher::new(1, 1)?;
    let selected = query.select(&mut facts, ty)?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(selected) else {
        return Err("inherited callable lost its actual function identity".into());
    };
    assert_ne!(DefId(item.0), callable);
    assert!(facts.function_indices.contains_key(&DefId(item.0)));
    let proof = query.finish();
    let mut constraints = Constraints::new(&facts.types, &facts.lifetimes.variances, 1, 100_000)?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    constraints.solve()?;
    assert!(specialization::reject_execution(&facts, &report).is_err());
    Ok(())
}

#[test]
fn default_rpit_rejection_binds_trait_declaration_and_keeps_final_inheritance() -> TestResult {
    for name in [
        "inherited-default-rpit",
        "own-child-rpit-override",
        "default-rpit-with-concrete-implementation-output",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/specialization-graph/execution/{name}.rs"
        ))?;
        let report = crate::test_source::resolve(&source, true)?;
        let error = Checker::new(&report)?
            .run()
            .err()
            .ok_or("default RPIT accepted")?;
        assert!(
            error
                .message
                .contains("return-position impl Trait in trait cannot be specialized"),
            "{name}: {error}"
        );
    }
    let source = crate::test_source::fixture(
        "pa31/tests/spec/specialization-graph/execution/final-rpit-inherited-across-scalar-specialization.rs",
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    verified(&report)?;
    Ok(())
}

#[test]
fn checked_defining_bodies_and_instances_keep_original_contracts_while_public_execution_stays_fenced()
-> TestResult {
    for path in [
        "corrected-chains/chain-and-inherited-body-explicit-middle",
        "corrected-chains/reverse-chain-order-explicit-middle",
        "execution/inherited-static-parent-nonstatic-child",
        "execution/final-rpit-inherited-across-scalar-specialization",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/specialization-graph/{path}.rs"
        ))?;
        let mut report = crate::test_source::resolve(&source, true)?;
        let facts = verified(&report)?;
        let mut analysis = crate::pipeline::analyze(&report, facts)?;
        let entry = crate::sema::entry::select(&mut report, &analysis.facts)?;
        let _program = crate::lower::instances::build_prepared(
            &report,
            &mut analysis.facts,
            entry,
            analysis.prepared,
        )?;
        assert!(specialization::reject_execution(&analysis.facts, &report).is_err());
    }
    Ok(())
}

#[test]
fn selected_member_rejects_wrong_trait_arguments_arena_and_graph_owner_and_keeps_failed_work()
-> TestResult {
    use crate::types::GenericArgs;
    let source = crate::test_source::fixture(
        "pa31/tests/spec/specialization-graph/corrected-chains/chain-and-inherited-body-explicit-middle.rs",
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = verified(&report)?;
    let owner = declaration(&report, DefKind::Trait, "V")?;
    let requested = method(&facts, &report, owner, "g")?;
    let child = facts.generics.implementations.as_deref().ok_or("actual registry")?.headers()
        .find(|header| matches!(facts.types.kind(header.predicate.ty), Some(Kind::Array { element, length: 1 }) if element == TypeId::primitive(Primitive::U8)))
        .ok_or("actual concrete child")?.owner;
    assert!(specialization::selected_method(&mut facts, child, GenericArgs::EMPTY, owner).is_err());
    let wrong = facts
        .types
        .intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U8))])?;
    assert!(specialization::selected_method(&mut facts, child, wrong, requested).is_err());
    let selected =
        specialization::selected_method(&mut facts, child, GenericArgs::EMPTY, requested)?;
    let other_report = crate::test_source::resolve(&source, true)?;
    let mut other = verified(&other_report)?;
    std::mem::swap(
        &mut facts.generics.implementations,
        &mut other.generics.implementations,
    );
    assert!(selected.current(&facts).is_err(), "foreign arena accepted");
    std::mem::swap(&mut facts.types, &mut other.types);
    assert!(
        selected.current(&facts).is_err(),
        "same numeric rows in another graph owner accepted"
    );
    std::mem::swap(&mut facts.types, &mut other.types);
    std::mem::swap(
        &mut facts.generics.implementations,
        &mut other.generics.implementations,
    );
    selected.current(&facts)?;
    assert!(selected.charge(10_000_001, 0).is_err());
    let before = facts
        .generics
        .implementations
        .as_deref()
        .and_then(|r| r.specialization_graph())
        .ok_or("same graph")?
        .observation();
    assert!(selected.current(&facts).is_err());
    let after = facts
        .generics
        .implementations
        .as_deref()
        .and_then(|r| r.specialization_graph())
        .ok_or("same graph")?
        .observation();
    assert!(after.work > before.work && before.work > 10_000_000);
    eprintln!(
        "specialization member failed work before={} after={} logical_bytes={}",
        before.work, after.work, after.bytes
    );
    Ok(())
}
