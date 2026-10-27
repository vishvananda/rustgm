//! Failed Store growth leaves canonical identities intact and charges every retry.
use super::*;
fn snapshots(store: &Store) -> (usize, usize, usize) {
    (
        store.interned_extent(),
        store.node_table_allocation().bytes,
        store.trait_input_bytes(),
    )
}
fn rejected_growth() -> Result {
    let mut store = Store::default();
    let work = Cell::new(0_usize);
    let rejected = |steps: usize, bytes| {
        work.set(work.get().saturating_add(steps));
        if bytes == 0 {
            Ok(())
        } else {
            Err("canonical growth payment exhausted")
        }
    };
    let byte = TypeId::primitive(Primitive::U8);
    let args = [Argument::Type(byte)];
    for _ in 0..2 {
        let prior = (snapshots(&store), work.get());
        assert!(
            store
                .intern_arguments_observed::<true>(&args, &rejected)
                .is_err()
        );
        assert_eq!(snapshots(&store), prior.0);
        assert!(work.get() > prior.1);
    }
    let arguments = store.intern_arguments_observed::<true>(&args, &|_, _| Ok(()))?;
    let app = traits::Application {
        item: ItemId(2),
        arguments,
    };
    for _ in 0..2 {
        let prior = (snapshots(&store), work.get());
        assert!(
            store
                .intern_trait_application_observed::<true>(app, &rejected)
                .is_err()
        );
        assert_eq!(snapshots(&store), prior.0);
        assert!(work.get() > prior.1);
    }
    store.intern_trait_application_observed::<true>(app, &|_, _| Ok(()))?;
    store.intern_arguments_observed::<true>(&args, &rejected)?;
    store.intern_trait_application_observed::<true>(app, &rejected)?;
    let mut child = byte;
    while store.node_allocation_bound(1) == 0 {
        child = store.intern(Kind::Slice(child))?;
    }
    for _ in 0..2 {
        let prior = (snapshots(&store), work.get());
        assert!(
            store
                .intern_observed::<true>(Kind::Slice(child), &rejected)
                .is_err()
        );
        assert_eq!(snapshots(&store), prior.0);
        assert!(work.get() > prior.1);
    }
    let ty = store.intern_observed::<true>(Kind::Slice(child), &|_, _| Ok(()))?;
    assert_eq!(
        store.intern_observed::<true>(Kind::Slice(child), &rejected)?,
        ty
    );
    for _ in 0..2 {
        let prior = (snapshots(&store), work.get());
        assert!(
            store
                .intern_list_observed::<true>(&[byte], &rejected)
                .is_err()
        );
        assert!(
            store
                .intern_regions_observed::<true>(&[RegionId::STATIC], &rejected)
                .is_err()
        );
        assert_eq!(snapshots(&store), prior.0);
        assert!(work.get() > prior.1);
    }
    Ok(())
}
fn reconstructed_inputs() -> Result {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U16);
    let first = projected(&mut store, byte)?;
    store.declare_projection(
        ItemId(11),
        Schema {
            trait_item: ItemId(2),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(&[Argument::Type(first)])?;
    let parent = store.intern_trait_application(traits::Application {
        item: ItemId(2),
        arguments,
    })?;
    let outer = store.intern(Kind::Projection(Application {
        item: ItemId(11),
        subject: byte,
        trait_application: parent,
        arguments: GenericArgs::EMPTY,
    }))?;
    let mut map = HashMap::default();
    map.insert(first, Some(word));
    let paid = Cell::new(0_usize);
    let charge = |_: usize, bytes| {
        paid.set(paid.get().saturating_add(bytes));
        Ok(())
    };
    let mut fold = Normalizer::default();
    let before = store.interned_extent();
    let result = rewrite::<true>(&mut store, &mut fold, outer, &map, &charge)
        .map_err(NormalizationFailure::message)?;
    assert_ne!(result, outer);
    assert!(store.interned_extent() > before);
    assert!(paid.get() > 0);
    let Kind::Projection(actual) = store.kind(result).ok_or("actual rebuilt projection")? else {
        return Err("rebuilt projection lost identity".into());
    };
    let actual = store
        .trait_application(actual.trait_application)
        .ok_or("actual parent")?;
    assert_eq!(
        store
            .arguments(actual.arguments)
            .ok_or("actual inputs")?
            .collect::<Vec<_>>(),
        vec![Argument::Type(word)]
    );
    let before = (snapshots(&store), paid.get());
    assert_eq!(
        rewrite::<true>(&mut store, &mut fold, outer, &map, &charge)
            .map_err(NormalizationFailure::message)?,
        result
    );
    assert_eq!((snapshots(&store), paid.get()), before);
    let parameters = store.intern_list(&[first])?;
    let callable = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: byte,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let mut binder = Normalizer::default();
    let Step::Resolve(request) = binder
        .start_observed::<true>(&mut store, callable, &charge)
        .map_err(NormalizationFailure::message)?
    else {
        return Err("missing binder projection".into());
    };
    assert_eq!(request.depth, 1);
    let before = store.interned_extent();
    let Step::Complete(completed) = binder
        .resume_observed::<true>(&mut store, request, Resolution::Type(word), &charge)
        .map_err(NormalizationFailure::message)?
    else {
        return Err("binder failed to complete".into());
    };
    assert!(store.interned_extent() > before);
    let Kind::Function { parameters, .. } = store.kind(completed).ok_or("actual callable")? else {
        return Err("binder lost callable".into());
    };
    assert_eq!(store.list(parameters), Some(&[word][..]));
    println!(
        "Canonical consequence Store paid={} extent={} Normalizer={}",
        paid.get(),
        store.interned_extent(),
        size_of::<Normalizer>()
    );
    Ok(())
}
pub(super) fn controls() -> Result {
    rejected_growth()?;
    reconstructed_inputs()
}
