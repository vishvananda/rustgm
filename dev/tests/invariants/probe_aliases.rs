//! Owned result aliases preserve source inputs and the ordinary export fence.
use super::*;
use crate::types::{ItemId, Primitive, RegionId, TypeId, copy, projection::Schema};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn store() -> Result<Store, &'static str> {
    let mut store = Store::default();
    store.declare_projection(
        ItemId(10),
        Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: copy::Properties::default(),
        },
    )?;
    Ok(store)
}

fn projection(context: &mut Context, store: &Store, input: TermId) -> Result<TermId, Error> {
    context.projected(store, ItemId(10), input, &[], &[])
}

#[test]
fn fresh_result_alias_exports_expression_without_inverting_its_input() -> TestResult {
    let mut store = store()?;
    let mut source = Context::default();
    let input = source.variable(Domain::Integer)?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[input])?;
    let expression = projection(probe.context(), &store, copied[0])?;
    let result = probe.context().variable(Domain::Any)?;
    probe.alias_fresh_results(&store, [(expression, result)])?;
    assert_eq!(probe.context().root(result)?, expression);
    assert!(probe.preserves_inputs(&mut source)?);
    let output = probe
        .export(&store, &mut source, result)?
        .ok_or("alias export")?;
    assert_eq!(source.projection_item(&store, output)?, Some(ItemId(10)));
    let known = source.known(&store, TypeId::primitive(Primitive::U32))?;
    assert_eq!(
        source.equate_scoped_terms(&store, output, known),
        Err(Error::Normalization)
    );
    assert_eq!(source.unresolved_domain(input)?, Some(Domain::Integer));
    assert_eq!(source.resolved(&mut store, input), Err(Error::Unresolved));
    Ok(())
}

#[test]
fn mapped_alias_representatives_and_nonbare_results_are_untouched() -> TestResult {
    let store = store()?;
    for reverse in [false, true] {
        let mut source = Context::default();
        let input = source.variable(Domain::Any)?;
        let (mut probe, copied) = Probe::new(&store, &mut source, &[input])?;
        let joined = probe.context().variable(Domain::Any)?;
        let (left, right) = if reverse {
            (joined, copied[0])
        } else {
            (copied[0], joined)
        };
        probe.context().equate(&store, left, right)?;
        let expression = projection(probe.context(), &store, copied[0])?;
        let numeric = probe.context().variable(Domain::Integer)?;
        let float = probe.context().variable(Domain::Float)?;
        let fresh = probe.context().variable(Domain::Any)?;
        let compound = probe.context().construct(Constructor::Tuple, &[fresh])?;
        let known = probe
            .context()
            .known(&store, TypeId::primitive(Primitive::U8))?;
        probe.alias_fresh_results(
            &store,
            [
                (expression, joined),
                (expression, numeric),
                (expression, float),
                (expression, compound),
                (expression, known),
                (compound, fresh),
            ],
        )?;
        assert!(probe.preserves_inputs(&mut source)?);
        assert_eq!(
            probe.context().unresolved_domain(joined)?,
            Some(Domain::Any)
        );
        assert_eq!(
            probe.context().unresolved_domain(numeric)?,
            Some(Domain::Integer)
        );
        assert_eq!(
            probe.context().unresolved_domain(float)?,
            Some(Domain::Float)
        );
        assert_eq!(probe.context().unresolved_domain(fresh)?, Some(Domain::Any));
        assert_eq!(probe.export(&store, &mut source, compound)?, None);
    }
    Ok(())
}

#[test]
fn cyclic_aliases_and_other_unowned_leaves_cannot_escape() -> TestResult {
    let store = store()?;
    for cycle in [false, true] {
        let mut source = Context::default();
        let (mut probe, _) = Probe::new(&store, &mut source, &[])?;
        let result = probe.context().variable(Domain::Any)?;
        let other = probe.context().variable(Domain::Any)?;
        let expression = projection(probe.context(), &store, if cycle { result } else { other })?;
        probe.alias_fresh_results(&store, [(expression, result)])?;
        if cycle {
            assert_eq!(
                probe.context().unresolved_domain(result)?,
                Some(Domain::Any)
            );
        }
        let sentinel = source.variable(Domain::Any)?;
        assert_eq!(probe.export(&store, &mut source, result)?, None);
        assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    }
    Ok(())
}

#[test]
fn aliases_preserve_inherited_regions_and_do_not_export_lifetime_proof() -> TestResult {
    let mut store = store()?;
    let mut source = Context::default();
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    let region = RegionId::bound(0, 0)?;
    let input = source.construct(
        Constructor::Reference {
            region,
            mutable: false,
        },
        &[byte],
    )?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[input])?;
    let expression = projection(probe.context(), &store, copied[0])?;
    let result = probe.context().variable(Domain::Any)?;
    probe
        .context()
        .region_equalities
        .push((RegionId(1), RegionId(2)));
    probe.alias_fresh_results(&store, [(expression, result)])?;
    let output = probe
        .export(&store, &mut source, result)?
        .ok_or("region alias export")?;
    let output = source.resolved(&mut store, output)?;
    let Some(crate::types::Kind::Projection(application)) = store.kind(output) else {
        return Err("associated alias identity".into());
    };
    assert!(
        matches!(store.kind(application.subject), Some(crate::types::Kind::Reference { region: actual, .. }) if actual == region)
    );
    assert!(source.region_equalities.is_empty());
    assert_eq!(
        probe.context().region_equalities,
        [(RegionId(1), RegionId(2))]
    );
    Ok(())
}

#[test]
fn mapped_root_protection_work_is_linear_in_one_alias_batch() -> TestResult {
    let store = store()?;
    for width in [8, 32, 128] {
        let mut source = Context::default();
        let inputs = (0..width)
            .map(|_| source.variable(Domain::Any))
            .collect::<Result<Vec<_>, _>>()?;
        let (mut probe, copied) = Probe::new(&store, &mut source, &inputs)?;
        let mut aliases = Vec::new();
        for input in copied {
            let expression = projection(probe.context(), &store, input)?;
            let result = probe.context().variable(Domain::Any)?;
            aliases.push((expression, result));
        }
        let before = probe.context().steps;
        probe.alias_fresh_results(&store, aliases)?;
        let work = probe.context().steps - before;
        assert!(work <= 32 * width + 64, "width={width} work={work}");
        assert!(probe.preserves_inputs(&mut source)?);
    }
    Ok(())
}

#[test]
fn marked_normal_forms_keep_their_existing_projection_guard() -> TestResult {
    let mut store = store()?;
    let mut source = Context::default();
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[byte])?;
    let expression = projection(probe.context(), &store, copied[0])?;
    let normalized = probe.context().resolved(&mut store, expression)?;
    probe
        .context()
        .apply_normalization(&mut store, expression, normalized)?;
    let result = probe.context().variable(Domain::Any)?;
    probe.alias_fresh_results(&store, [(expression, result)])?;
    assert_eq!(
        probe.context().unresolved_domain(result)?,
        Some(Domain::Any)
    );
    assert_eq!(probe.export(&store, &mut source, result)?, None);
    Ok(())
}

#[test]
fn invalid_alias_identities_and_work_exhaustion_are_errors() -> TestResult {
    let store = store()?;
    let mut source = Context::default();
    let (mut probe, _) = Probe::new(&store, &mut source, &[])?;
    let fresh = probe.context().variable(Domain::Any)?;
    assert_eq!(
        probe.alias_fresh_results(&store, [(TermId(u32::MAX), fresh)]),
        Err(Error::InvalidIdentity)
    );
    probe.context().steps = 1_000_000;
    assert_eq!(
        probe.alias_fresh_results(&store, [(fresh, fresh)]),
        Err(Error::Budget)
    );
    Ok(())
}

#[test]
fn repeated_aliases_do_not_rescan_shared_source_inputs() -> TestResult {
    let store = store()?;
    for width in [8, 32, 128] {
        let mut source = Context::default();
        let inputs = (0..width)
            .map(|_| source.variable(Domain::Any))
            .collect::<Result<Vec<_>, _>>()?;
        let tuple = source.construct(Constructor::Tuple, &inputs)?;
        let (mut probe, copied) = Probe::new(&store, &mut source, &[tuple])?;
        let expression = projection(probe.context(), &store, copied[0])?;
        let mut aliases = Vec::new();
        for _ in 0..width {
            let result = probe.context().variable(Domain::Any)?;
            aliases.push((expression, result));
        }
        let before = probe.context().steps;
        probe.alias_fresh_results(&store, aliases)?;
        let work = probe.context().steps - before;
        assert!(work <= 40 * width + 64, "width={width} work={work}");
        assert!(probe.preserves_inputs(&mut source)?);
    }
    Ok(())
}

#[test]
fn refined_input_graph_cannot_be_an_occurrence_check_boundary() -> TestResult {
    let store = store()?;
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[input])?;
    let result = probe.context().variable(Domain::Any)?;
    let tuple = probe.context().construct(Constructor::Tuple, &[result])?;
    probe.context().equate(&store, copied[0], tuple)?;
    let expression = projection(probe.context(), &store, copied[0])?;
    assert_eq!(
        probe.alias_fresh_results(&store, [(expression, result)]),
        Err(Error::InvalidIdentity)
    );
    assert_eq!(
        probe.context().unresolved_domain(result)?,
        Some(Domain::Any)
    );
    assert_eq!(source.unresolved_domain(input)?, Some(Domain::Any));
    Ok(())
}
