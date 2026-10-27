//! Query-added clauses retain their own namespace above actual opened/quantified prefixes.
use super::*;
use crate::types::{
    Kind, RegionId,
    arguments::Argument,
    parameters::{Parameter, ParameterKind, clauses::Quantified, environment::Abilities},
    substitute,
    traits::Application,
};
use std::cell::Cell;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn predicate(
    types: &mut Store,
    subject: crate::types::TypeId,
    region: RegionId,
) -> Result<Predicate> {
    let arguments = types.intern_arguments(&[Argument::Lifetime(region)])?;
    let application = types.intern_trait_application(Application {
        item: ItemId(5),
        arguments,
    })?;
    Ok(Predicate {
        ty: subject,
        bound: Bound::Trait(application),
    })
}

pub(in crate::sema) fn controls() -> Result {
    let mut types = Store::default();
    let id = types.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let other = types.intern_parameter(Parameter {
        owner: ItemId(2),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let subject = types.intern(Kind::Parameter(id))?;
    let target = types.intern(Kind::Parameter(other))?;
    let original = predicate(&mut types, subject, RegionId::bound(0, 0)?)?;
    let environment = Arc::new(
        Environment::new(&types, &[(id, Abilities::default())])?
            .with_predicates(&types, &[original])?,
    );
    let steps = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let pay = |work, retained| {
        steps.set(steps.get().saturating_add(work));
        bytes.set(bytes.get().saturating_add(retained));
        Ok(())
    };
    let mut opening = substitute::Bound::open_regions_charged(100, &pay)?;
    let opened = opening.predicate_charged(&mut types, original, &pay)?;
    let mapped = environment.opened_regions_charged(&mut types, &mut opening, &pay)?;
    let base = Assumptions::from_environment_charged(&types, Some(&mapped), &pay)?;
    let given = predicate(&mut types, target, RegionId(200))?;
    let extension = base.extended(&types, &[given], &pay)?;
    assert!(
        base.prove_canonical(&mut types, &mut Context::default(), given)?
            .is_none()
    );
    for goal in [given, opened] {
        assert!(
            extension
                .prove_canonical(&mut types, &mut Context::default(), goal)?
                .is_some()
        );
    }
    let base_index = base.index.as_ref().ok_or("real base index")?;
    let extended_index = extension.index.as_ref().ok_or("query index")?;
    assert_eq!(extended_index.seen.len(), 1);
    let shared = base_index.layers().next().ok_or("actual mapped prefix")?.0;
    assert!(
        extended_index
            .layers()
            .any(|(layer, _)| std::ptr::eq(layer, shared))
    );
    let before = (steps.get(), bytes.get());
    let failed = base.extended(&types, &[given], &|work, retained| {
        pay(work, retained)?;
        Err("paid failure")
    });
    assert!(failed.is_err());
    assert!(steps.get() > before.0 && bytes.get() > before.1);
    assert!(
        base.extended(&types, &[given], &pay)?
            .prove_canonical(&mut types, &mut Context::default(), given)?
            .is_some()
    );
    quantified(&mut types, subject, target, id, &pay)?;
    println!(
        "receiver-prefix steps={} bytes={} Assumptions={} ClauseIndex={}",
        steps.get(),
        bytes.get(),
        size_of::<Assumptions>(),
        size_of::<ClauseIndex>()
    );
    Ok(())
}

fn quantified(
    types: &mut Store,
    subject: crate::types::TypeId,
    target: crate::types::TypeId,
    id: ParameterId,
    pay: &impl Fn(usize, usize) -> std::result::Result<(), &'static str>,
) -> Result {
    let body = predicate(types, subject, RegionId::bound(0, 0)?)?;
    let clause = Clause::ForAll(types.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: body,
    })?);
    let environment =
        Environment::new(types, &[(id, Abilities::default())])?.with_clauses(types, &[clause])?;
    let (base, _) = Assumptions::from_environment(types, Some(&environment))?;
    let given = predicate(types, target, RegionId(200))?;
    let extension = base.extended(types, &[given], pay)?;
    let wanted = predicate(types, subject, RegionId(200))?;
    let mut context = Context::default();
    let mut substitution = infer::Instantiation::new(&mut context, types, &[])?;
    let goal = context.instantiate_predicate(types, wanted, &mut substitution)?;
    assert!(matches!(
        extension.select_scoped(types, &mut context, &goal, 300)?,
        Selection::Proven(_)
    ));
    assert!(extension.has_quantified());
    Ok(())
}
