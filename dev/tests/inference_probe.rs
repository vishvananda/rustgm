//! Symbolic query results return through original holes without importing candidate constraints.
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Constant,
    infer::{
        Constructor, Context, Domain, Error, arguments::Argument, constants::ConstTermId,
        probe::Probe,
    },
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn exported_header_arguments_share_the_original_unknown_input() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[input, input])?;
    assert_eq!(copied[0], copied[1]);
    let parameter = probe.context().variable(Domain::Any)?;
    probe.context().equate(&store, parameter, copied[0])?;
    let result = probe
        .context()
        .construct(Constructor::Tuple, &[parameter, copied[0]])?;
    let output = probe
        .export(&store, &mut source, result)?
        .ok_or("lost symbolic result")?;
    assert_eq!(source.resolved(&mut store, input), Err(Error::Unresolved));
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    source.equate(&store, input, byte)?;
    let output = source.resolved(&mut store, output)?;
    let Some(Kind::Tuple(fields)) = store.kind(output) else {
        return Err("expected tuple".into());
    };
    assert_eq!(
        store.list(fields),
        Some(&[TypeId::primitive(Primitive::U8); 2][..])
    );
    Ok(())
}

#[test]
fn a_concrete_probe_result_does_not_choose_the_input_type() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Integer)?;
    let (mut probe, _) = Probe::new(&store, &mut source, &[input])?;
    let byte = probe
        .context()
        .known(&store, TypeId::primitive(Primitive::U8))?;
    let output = probe
        .export(&store, &mut source, byte)?
        .ok_or("lost closed result")?;
    assert_eq!(
        source.resolved(&mut store, output)?,
        TypeId::primitive(Primitive::U8)
    );
    assert_eq!(source.unresolved_domain(input)?, Some(Domain::Integer));
    assert_eq!(source.resolved(&mut store, input), Err(Error::Unresolved));
    Ok(())
}

#[test]
fn narrowed_bound_and_merged_inputs_reject_export_without_allocating_source_terms() -> TestResult {
    let store = Store::default();
    for mode in 0..3 {
        let mut source = Context::default();
        let a = source.variable(Domain::Any)?;
        let b = source.variable(Domain::Any)?;
        let (mut probe, copied) = Probe::new(&store, &mut source, &[a, b])?;
        let other = match mode {
            0 => probe.context().variable(Domain::Integer)?,
            1 => probe
                .context()
                .known(&store, TypeId::primitive(Primitive::U8))?,
            _ => copied[1],
        };
        probe.context().equate(&store, copied[0], other)?;
        let sentinel = source.variable(Domain::Any)?;
        assert_eq!(probe.export(&store, &mut source, copied[0])?, None);
        assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
        assert_eq!(source.unresolved_domain(a)?, Some(Domain::Any));
        assert_eq!(source.unresolved_domain(b)?, Some(Domain::Any));
    }
    Ok(())
}

fn length(
    context: &mut Context,
    store: &Store,
    term: rustgm::types::infer::TermId,
) -> Result<ConstTermId, &'static str> {
    let Some(Constructor::ArrayInfer(length)) =
        context.head(store, term).map_err(|_| "invalid array")?
    else {
        return Err("missing inferred length");
    };
    Ok(length)
}

#[test]
fn const_inputs_keep_their_identity_and_cannot_be_narrowed_or_merged() -> TestResult {
    let mut store = Store::default();
    let usize = TypeId::primitive(Primitive::Usize);
    let value = store.intern_constant(Constant { ty: usize, bits: 7 })?;
    for merge in [false, true] {
        let mut source = Context::default();
        let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
        let a = source.const_variable(&store, usize)?;
        let b = source.const_variable(&store, usize)?;
        let x = source.construct(Constructor::ArrayInfer(a), &[byte])?;
        let y = source.construct(Constructor::ArrayInfer(b), &[byte])?;
        let (mut probe, copied) = Probe::new(&store, &mut source, &[x, y])?;
        let first = length(probe.context(), &store, copied[0])?;
        let second = if merge {
            length(probe.context(), &store, copied[1])?
        } else {
            probe.context().known_const(&store, value)?
        };
        probe.context().equate_consts(first, second)?;
        assert_eq!(probe.export(&store, &mut source, copied[0])?, None);
        assert_eq!(source.resolved_const(&mut store, a), Err(Error::Unresolved));
        assert_eq!(source.resolved_const(&mut store, b), Err(Error::Unresolved));
    }
    Ok(())
}

#[test]
fn fresh_type_and_const_holes_cannot_escape_the_probe() -> TestResult {
    let store = Store::default();
    let mut source = Context::default();
    let (mut probe, _) = Probe::new(&store, &mut source, &[])?;
    let fresh = probe.context().variable(Domain::Any)?;
    let byte = probe
        .context()
        .known(&store, TypeId::primitive(Primitive::U8))?;
    let length = probe
        .context()
        .const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let array = probe
        .context()
        .construct(Constructor::ArrayInfer(length), &[byte])?;
    for result in [fresh, array] {
        let sentinel = source.variable(Domain::Any)?;
        assert_eq!(probe.export(&store, &mut source, result)?, None);
        assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    }
    probe.context().equate(&store, fresh, byte)?;
    assert!(probe.export(&store, &mut source, fresh)?.is_some());
    Ok(())
}

#[test]
fn exported_const_slots_reuse_original_variables_across_new_result_shapes() -> TestResult {
    let mut store = Store::default();
    let usize = TypeId::primitive(Primitive::Usize);
    let value = store.intern_constant(Constant { ty: usize, bits: 3 })?;
    let mut source = Context::default();
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    let input = source.const_variable(&store, usize)?;
    let array = source.construct(Constructor::ArrayInfer(input), &[byte])?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[array])?;
    let copied_length = length(probe.context(), &store, copied[0])?;
    let result = probe
        .context()
        .nominal(ItemId(42), &[Argument::Const(copied_length)])?;
    let output = probe
        .export(&store, &mut source, result)?
        .ok_or("lost symbolic const")?;
    assert_eq!(
        source.resolved_const(&mut store, input),
        Err(Error::Unresolved)
    );
    let known = source.known_const(&store, value)?;
    source.equate_consts(input, known)?;
    let output = source.resolved(&mut store, output)?;
    let Some(Kind::Adt(ItemId(42), arguments)) = store.kind(output) else {
        return Err("lost nominal result".into());
    };
    assert_eq!(
        store
            .arguments(arguments)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>(),
        [rustgm::types::arguments::Argument::Const(value)]
    );
    Ok(())
}

#[test]
fn altered_source_equations_make_the_probe_stale() -> TestResult {
    let store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[input])?;
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    source.equate(&store, input, byte)?;
    assert_eq!(
        probe.export(&store, &mut source, copied[0]),
        Err(Error::InvalidIdentity)
    );
    Ok(())
}

#[test]
fn shared_graph_export_is_bounded_and_leaves_region_obligations_in_the_probe() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut probe, copied) = Probe::new(&store, &mut source, &[input])?;
    let mut result = copied[0];
    for _ in 0..512 {
        result = probe
            .context()
            .construct(Constructor::Tuple, &[result, result])?;
    }
    probe
        .context()
        .region_equalities
        .push((RegionId(1), RegionId(2)));
    let output = probe
        .export(&store, &mut source, result)?
        .ok_or("lost shared result")?;
    assert!(source.region_equalities.is_empty());
    assert_eq!(
        probe.context().region_equalities,
        [(RegionId(1), RegionId(2))]
    );
    assert!(source.steps + probe.context().steps < 20_000);
    assert_eq!(source.resolved(&mut store, output), Err(Error::Unresolved));
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    source.equate(&store, input, byte)?;
    assert!(source.resolved(&mut store, output).is_ok());
    assert!(store.len() < 600);
    Ok(())
}
