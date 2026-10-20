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
            copies: rustgm::types::copy::Properties::default(),
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
    assert_eq!(
        first
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        byte
    );
    assert!(fold.start(&mut store, root).is_err());
    let word = TypeId::primitive(Primitive::U16);
    let second = request(fold.resume(&mut store, first, Resolution::Type(word))?)?;
    assert_eq!(
        second
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        word
    );
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
fn checked_children_make_appended_projection_work_linear() -> TestResult {
    for depth in [8, 32, 128, 512] {
        let mut store = Store::default();
        let mut child = TypeId::primitive(Primitive::U8);
        let mut steps = 0;
        for _ in 0..depth {
            let root = projected(&mut store, child)?;
            let mut fold = Normalizer::default();
            let query = request(fold.start_with_input(&mut store, root, child)?)?;
            assert_eq!(
                query
                    .alias
                    .associated()
                    .ok_or("expected associated projection")?
                    .subject,
                child
            );
            assert!(fold.start_with_input(&mut store, root, child).is_err());
            assert_eq!(
                fold.resume(&mut store, query, Resolution::Opaque)?,
                Step::Complete(root)
            );
            steps += fold.steps();
            child = root;
        }
        assert!(steps < 16 * depth, "depth {depth}: {steps} steps");
    }
    Ok(())
}

#[test]
fn checked_subjects_do_not_skip_new_arguments_or_replacement_dependencies() -> TestResult {
    use rustgm::types::arguments::Argument;
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U16);
    let checked = projected(&mut store, byte)?;
    let introduced = projected(&mut store, word)?;
    let arguments = store.intern_arguments(&[Argument::Type(introduced)])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments,
    })?;
    let root = store.intern(Kind::Projection(Application {
        item: ItemId(10),
        subject: checked,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let mut fold = Normalizer::default();
    let argument = request(fold.start_with_input(&mut store, root, checked)?)?;
    assert_eq!(
        argument
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        word
    );
    let outer = request(fold.resume(&mut store, argument, Resolution::Type(byte))?)?;
    assert_eq!(
        outer
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        checked
    );
    let arguments = store
        .trait_application(
            outer
                .alias
                .associated()
                .ok_or("expected associated projection")?
                .trait_application,
        )
        .ok_or("missing trait")?
        .arguments;
    assert_eq!(
        store
            .arguments(arguments)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>(),
        vec![Argument::Type(byte)]
    );
    let replacement = projected(&mut store, TypeId::UNIT)?;
    let next = request(fold.resume(&mut store, outer, Resolution::Type(replacement))?)?;
    assert_eq!(
        next.alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        TypeId::UNIT
    );
    assert_eq!(
        fold.resume(&mut store, next, Resolution::Type(word))?,
        Step::Complete(word)
    );
    assert!(fold.start_with_input(&mut store, root, introduced).is_err());
    Ok(())
}

#[test]
fn checked_root_inputs_do_not_authorize_a_nested_callable_scope() -> TestResult {
    let mut store = Store::default();
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let checked = projected(&mut store, reference)?;
    let parameters = store.intern_list(&[checked])?;
    let callable = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let mut fold = Normalizer::default();
    let query = request(fold.start_with_input(&mut store, callable, checked)?)?;
    assert_eq!(query.depth, 1);
    assert_eq!(
        query
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        reference
    );
    assert_eq!(
        fold.resume(&mut store, query, Resolution::Opaque)?,
        Step::Complete(callable)
    );
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
    assert_eq!(
        query
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        subject
    );
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
        let Some(Kind::Array { length, .. }) = store.kind(
            query
                .alias
                .associated()
                .ok_or("expected associated projection")?
                .subject,
        ) else {
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

#[test]
fn sibling_callable_requests_have_independent_scopes_and_caches() -> TestResult {
    use rustgm::types::lifetime_proof::ScopeId;
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U16);
    let projection = projected(&mut store, byte)?;
    let parameters = store.intern_list(&[projection])?;
    let mut callables = Vec::new();
    for count in [1, 2] {
        callables.push(store.intern(Kind::Function {
            bound_regions: count,
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
        })?);
    }
    let roots = store.intern_list(&callables)?;
    let root = store.intern(Kind::Tuple(roots))?;
    let mut fold = Normalizer::default();
    let first = request(fold.start(&mut store, root)?)?;
    assert_eq!(first.depth, 1);
    assert_eq!(fold.binder(first.scope), Some((ScopeId::ROOT, 2)));
    let second = request(fold.resume(&mut store, first, Resolution::Type(word))?)?;
    assert_eq!(second.depth, first.depth);
    assert_eq!(second.alias, first.alias);
    assert_ne!(second.scope, first.scope);
    assert_eq!(fold.binder(second.scope), Some((ScopeId::ROOT, 1)));
    assert!(
        fold.resume(&mut store, first, Resolution::Type(byte))
            .is_err()
    );
    let Step::Complete(result) = fold.resume(&mut store, second, Resolution::Type(byte))? else {
        return Err("unfinished sibling callable normalization".into());
    };
    let Some(Kind::Tuple(types)) = store.kind(result) else {
        return Err("missing normalized sibling tuple".into());
    };
    for (&ty, expected) in store
        .list(types)
        .ok_or("missing siblings")?
        .iter()
        .zip([byte, word])
    {
        let Some(Kind::Function { parameters, .. }) = store.kind(ty) else {
            return Err("missing normalized callable".into());
        };
        assert_eq!(store.list(parameters), Some([expected].as_slice()));
    }
    Ok(())
}

#[test]
fn empty_callable_binders_preserve_depth_with_linear_iterative_work() -> TestResult {
    use std::collections::HashSet;
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let projection = projected(&mut store, byte)?;
    let mut root = TypeId::UNIT;
    for _ in 0..1024 {
        let parameters = store.intern_list(&[projection, root])?;
        root = store.intern(Kind::Function {
            bound_regions: 0,
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
        })?;
    }
    let mut fold = Normalizer::default();
    let mut step = fold.start(&mut store, root)?;
    let mut scopes = HashSet::new();
    let mut expected_depth = 1024;
    while let Step::Resolve(query) = step {
        assert_eq!(query.depth, expected_depth);
        assert_eq!(fold.binder(query.scope).map(|(_, count)| count), Some(0));
        assert!(scopes.insert(query.scope));
        step = fold.resume(&mut store, query, Resolution::Opaque)?;
        expected_depth -= 1;
    }
    assert_eq!(step, Step::Complete(root));
    assert_eq!(scopes.len(), 1024);
    assert_eq!(expected_depth, 0);
    assert!(fold.steps() < 20_000, "{}", fold.steps());
    Ok(())
}

#[test]
fn selected_callable_results_open_a_scope_before_normalizing_their_children() -> TestResult {
    use rustgm::types::lifetime_proof::ScopeId;
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let outer = projected(&mut store, byte)?;
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: byte,
    })?;
    let inner = projected(&mut store, reference)?;
    let parameters = store.intern_list(&[inner])?;
    let callable = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let mut fold = Normalizer::default();
    let query = request(fold.start(&mut store, outer)?)?;
    assert_eq!(query.scope, ScopeId::ROOT);
    let nested = request(fold.resume(&mut store, query, Resolution::Type(callable))?)?;
    assert_eq!(fold.binder(nested.scope), Some((ScopeId::ROOT, 1)));
    assert_eq!(
        nested
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .subject,
        reference
    );
    assert_eq!(nested.depth, 1);
    let Step::Complete(result) = fold.resume(&mut store, nested, Resolution::Type(reference))?
    else {
        return Err("unfinished selected callable normalization".into());
    };
    assert_eq!(store.contains_projections(result), Some(false));
    Ok(())
}
