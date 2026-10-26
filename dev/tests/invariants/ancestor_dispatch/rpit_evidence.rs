//! The same final inherited result retains debts in both real normalization consumers.
use super::{TestResult, declaration, method, verified};
use crate::{
    resolve::model::DefKind,
    sema::{
        model::Declarations,
        nominals::traits::{Projected, normal_form},
        projections::Definition,
        traits::{assumptions::Assumptions, implementations::Availability, selection},
    },
    types::{
        ItemId, Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        infer::{Context, Instantiation},
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::predicates::{Bound, Predicate},
        relations::Constraints,
        substitute::{Normalizer, Step},
    },
};
fn valid(facts: &Declarations, proof: &Obligations, universals: u32) -> TestResult<bool> {
    let mut constraints = Constraints::new(
        &facts.types,
        &facts.lifetimes.variances,
        universals,
        100_000,
    )?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    Ok(constraints.solve().is_ok())
}
fn projected(
    facts: &mut Declarations,
    root: TypeId,
) -> TestResult<(crate::types::substitute::Request, Projected)> {
    let mut fold = Normalizer::default();
    let Step::Resolve(request) = fold.start(&mut facts.types, root)? else {
        return Err("actual trait RPIT did not request normalization".into());
    };
    let normal_form::Progress::Ready(projected) =
        normal_form::begin(facts, None, request, 1, false, Availability::Validated)
            .map_err(|error| format!("outer canonical projection: {error}"))?
    else {
        return Err("completed source headers unexpectedly pending".into());
    };
    Ok((request, projected))
}
fn search_result(facts: &mut Declarations, root: TypeId) -> TestResult<Obligations> {
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &facts.types, &[])
        .map_err(|error| format!("search inference scope: {error}"))?;
    let goal = context
        .instantiate_predicate(
            &mut facts.types,
            Predicate {
                ty: root,
                bound: Bound::Sized,
            },
            &mut substitution,
        )
        .map_err(|error| format!("search goal formation: {error}"))?;
    let empty = Assumptions::default();
    let caller = selection::Caller {
        assumptions: &empty,
        environment: None,
        first_region: 1,
        retain_sized: false,
    };
    let selection::Selection::Proven(evidence) =
        selection::select_available(facts, &mut context, &goal, caller, Availability::Validated)
            .map_err(|error| format!("search normalization: {error}"))?
    else {
        return Err("search did not retain an actual normalization proof".into());
    };
    assert!(!evidence.provisional_size && evidence.sized.is_empty());
    let mut proof = evidence.proof;
    proof.equalities.append(&mut context.region_equalities);
    Ok(proof)
}
#[test]
fn both_rpit_consumers_keep_final_inherited_identity_and_real_parent_lifetime_debt() -> TestResult {
    for (name, restricted) in [
        ("final-rpit-inherited-across-scalar-specialization", false),
        ("final-rpit-static-parent-nonstatic-child", true),
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/specialization-graph/execution/{name}.rs"
        ))?;
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = verified(&report)?;
        let owner = declaration(&report, DefKind::Trait, "V")?;
        let member = method(&facts, &report, owner, "f")?;
        let subject = if restricted {
            let nominal = declaration(&report, DefKind::Struct, "W")?;
            let reference = facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(0, 0)?,
                referent: TypeId::primitive(Primitive::U8),
            })?;
            let arguments = facts.types.intern_arguments(&[Argument::Type(reference)])?;
            facts
                .types
                .intern(Kind::Adt(ItemId(nominal.0), arguments))?
        } else {
            TypeId::primitive(Primitive::U16)
        };
        let opaque = *facts
            .opaques
            .owners
            .get(&member)
            .and_then(|ids| ids.first())
            .ok_or("actual trait RPIT owner")?;
        let arguments = facts.types.intern_arguments(&[Argument::Type(subject)])?;
        let root = facts.types.intern(Kind::Opaque(opaque, arguments))?;
        let capture = selection::copy_trace::Capture::new()?;
        let (request, projected) = projected(&mut facts, root)
            .map_err(|error| format!("{name}: {error}; trace={:?}", capture.rows()))?;
        assert_eq!(
            valid(&facts, &projected.proof, u32::from(restricted))?,
            !restricted,
            "outer result phase: {name}"
        );
        let definition = Definition::alias(
            &mut facts,
            request.alias,
            projected.family,
            projected.source.ok_or("actual selected source")?,
        )
        .map_err(|error| format!("{name}: defining target: {error}"))?
        .ok_or("actual inherited implementation result")?;
        let result = definition
            .result(&mut facts)?
            .ok_or("retained actual result template")?;
        let result = projected
            .proof
            .restoration()?
            .apply(&mut facts.types, result)?;
        let Some(Kind::Opaque(id, _)) = facts.types.kind(result) else {
            return Err("selected defining result lost its actual opaque declaration".into());
        };
        assert_eq!(
            facts
                .opaques
                .declarations
                .get(&id)
                .ok_or("actual defining opaque owner")?
                .owner,
            definition.id
        );
        // Sized is the genuine implicit formation obligation of these RPITs.
        // A bare ProjectionEqual with an Opaque subject is not a valid query.
        let proof = search_result(&mut facts, root)
            .map_err(|error| format!("{name}: {error}; trace={:?}", capture.rows()))?;
        assert_eq!(
            valid(&facts, &proof, u32::from(restricted))?,
            !restricted,
            "search result phase: {name}"
        );
        let rows = capture.rows()?;
        assert!(
            rows.iter()
                .any(|row| row.starts_with("specialization-rpit-outer"))
        );
        assert!(
            rows.iter()
                .any(|row| row.starts_with("specialization-rpit-search"))
        );
        let (frame, node, normalization) = selection::specialization_layouts();
        for row in rows
            .iter()
            .filter(|row| row.starts_with("specialization-rpit-"))
        {
            eprintln!("{name}: {row}");
        }
        eprintln!(
            "specialization result layouts Query={} Normalization={} Frame={frame} Node={node} SelectedMethod={} ProjectionRequirements={}",
            size_of::<normal_form::Query>(),
            normalization,
            size_of::<crate::sema::specialization::SelectedMethod>(),
            size_of::<crate::sema::specialization::ProjectionRequirements>()
        );
    }
    Ok(())
}
