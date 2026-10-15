//! Suspension preserves typed inputs, nested binders and shared graph work.
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    projection::{Application, Schema},
    substitute::{Normalizer, Resolution, Step},
    traits,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn projected(store: &mut Store, subject: TypeId) -> Result<TypeId, &'static str> {
    store.declare_projection(
        ItemId(10),
        Schema {
            trait_item: ItemId(1),
            sized: true,
        },
    )?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    store.intern(Kind::Projection(Application {
        item: ItemId(10),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))
}
fn request(step: Step) -> Result<rustgm::types::substitute::Request, &'static str> {
    match step {
        Step::Resolve(value) => Ok(value),
        Step::Complete(_) => Err("expected a projection request"),
    }
}

#[test]
fn dependencies_precede_selection_and_completed_roots_reuse_results() -> TestResult {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let inner = projected(&mut store, byte)?;
    let outer = projected(&mut store, inner)?;
    let list = store.intern_list(&[outer, outer, inner])?;
    let root = store.intern(Kind::Tuple(list))?;
    let mut fold = Normalizer::default();
    let first = request(fold.start(&mut store, root)?)?;
    assert_eq!(first.application.subject, byte);
    assert!(fold.start(&mut store, root).is_err());
    let word = TypeId::primitive(Primitive::U16);
    let second = request(fold.resume(&mut store, first, Resolution::Type(word))?)?;
    assert_eq!(second.application.subject, word);
    assert!(fold.resume(&mut store, first, Resolution::Opaque).is_err());
    let result = fold.resume(&mut store, second, Resolution::Type(byte))?;
    let list = store.intern_list(&[byte, byte, word])?;
    assert_eq!(result, Step::Complete(store.intern(Kind::Tuple(list))?));
    let steps = fold.steps();
    assert_eq!(fold.start(&mut store, root)?, result);
    assert_eq!(fold.steps(), steps + 1);
    Ok(())
}

#[test]
fn replacements_are_normalized_and_recursive_results_fail() -> TestResult {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let first = projected(&mut store, byte)?;
    let second = projected(&mut store, TypeId::primitive(Primitive::U16))?;
    let mut fold = Normalizer::default();
    let query = request(fold.start(&mut store, first)?)?;
    let next = request(fold.resume(&mut store, query, Resolution::Type(second))?)?;
    assert_eq!(
        fold.resume(&mut store, next, Resolution::Type(byte))?,
        Step::Complete(byte)
    );
    let mut cyclic = Normalizer::default();
    let query = request(cyclic.start(&mut store, first)?)?;
    let next = request(cyclic.resume(&mut store, query, Resolution::Type(second))?)?;
    assert_eq!(
        cyclic.resume(&mut store, next, Resolution::Type(first)),
        Err("cycle in associated type normalization")
    );
    assert!(cyclic.start(&mut store, byte).is_err());
    Ok(())
}

#[test]
fn opaque_results_keep_normalized_inputs_and_binder_depths_distinct() -> TestResult {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let subject = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: byte,
    })?;
    let projection = projected(&mut store, subject)?;
    let mut fold = Normalizer::default();
    let query = request(fold.start(&mut store, projection)?)?;
    assert_eq!(query.depth, 0);
    assert_eq!(
        fold.resume(&mut store, query, Resolution::Opaque)?,
        Step::Complete(projection)
    );
    let parameters = store.intern_list(&[projection])?;
    let callable = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let query = request(fold.start(&mut store, callable)?)?;
    assert_eq!(query.depth, 1);
    assert_eq!(query.application.subject, subject);
    let result = fold.resume(&mut store, query, Resolution::Type(subject))?;
    let parameters = store.intern_list(&[subject])?;
    let expected = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    assert_eq!(result, Step::Complete(expected));
    Ok(())
}

#[test]
fn generated_shared_results_use_bounded_iterative_work() -> TestResult {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let mut inputs = Vec::new();
    for length in 0..10_000 {
        let subject = store.intern(Kind::Array {
            element: byte,
            length,
        })?;
        inputs.push(projected(&mut store, subject)?);
    }
    let mut fold = Normalizer::default();
    let mut step = fold.start(&mut store, inputs[9999])?;
    let mut requests = 0;
    while let Step::Resolve(query) = step {
        let Some(Kind::Array { length, .. }) = store.kind(query.application.subject) else {
            return Err("missing ordinal".into());
        };
        let ty = if length == 0 {
            byte
        } else {
            let child = inputs[usize::try_from(length - 1)?];
            let list = store.intern_list(&[child, child])?;
            store.intern(Kind::Tuple(list))?
        };
        requests += 1;
        step = fold.resume(&mut store, query, Resolution::Type(ty))?;
    }
    let Step::Complete(ty) = step else {
        return Err("incomplete fold".into());
    };
    assert_eq!(store.contains_projections(ty), Some(false));
    assert_eq!(requests, 10_000);
    assert!(fold.steps() < 150_000, "{}", fold.steps());
    Ok(())
}
