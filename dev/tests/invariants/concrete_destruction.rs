//! Exact destruction queries visit only their closed reachable type graph.
use super::*;
use crate::types::{GenericArgs, Primitive, copy, opaque};

fn graph(types: &mut Store, width: usize, depth: usize) -> Result<TypeId, &'static str> {
    let mut ty = TypeId::primitive(Primitive::U8);
    for _ in 0..depth {
        let fields = types.intern_list(&vec![ty; width])?;
        ty = types.intern(Kind::Tuple(fields))?;
    }
    Ok(ty)
}

#[test]
fn concrete_destruction_storage_depends_on_reachable_graph_not_store() -> Result<(), &'static str> {
    for width in [8, 32, 128] {
        for depth in [1, 8, 32] {
            let mut baseline = None;
            for noise in [0, 1024, 8192] {
                let mut types = Store::default();
                for length in 0..noise {
                    types.intern(Kind::Array {
                        element: TypeId::primitive(Primitive::Bool),
                        length,
                    })?;
                }
                let ty = graph(&mut types, width, depth)?;
                let mut query = NeedsDrop::concrete_sparse([]);
                assert!(!query.demand(&types, &Definitions::default(), ty)?);
                assert!(query.states.is_empty());
                let scope = query.scope.as_deref().ok_or("sparse states")?;
                assert!(scope.environment.is_none());
                assert_eq!(scope.states.len(), depth + 1);
                let measured = (query.steps, scope.states.len(), scope.states.capacity());
                assert!(query.steps <= 4 * width * depth + 2);
                if let Some(expected) = baseline {
                    assert_eq!(measured, expected);
                } else {
                    baseline = Some(measured);
                }
                let steps = query.steps;
                for _ in 0..128 {
                    assert!(!query.demand(&types, &Definitions::default(), ty)?);
                }
                assert_eq!(query.steps, steps);
            }
        }
    }
    Ok(())
}

#[test]
fn concrete_destruction_does_not_turn_missing_heads_into_cleanup_answers()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let definitions = Definitions::default();
    for copy in [false, true] {
        let id = types.declare_opaque(opaque::Declaration {
            trait_result: false,
            sized: true,
            copies: copy::Properties { copy, clone_: copy },
        })?;
        let ty = types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
        let environment = Arc::new(Environment::new(&types, &[])?);
        let mut symbolic = NeedsDrop::in_environment([], environment);
        assert_eq!(symbolic.demand(&types, &definitions, ty)?, !copy);
        let mut incomplete = NeedsDrop::concrete_sparse([]);
        assert!(incomplete.demand(&types, &definitions, ty).is_err());
        types.publish_storage_head(ty, TypeId::primitive(Primitive::U8))?;
        assert!(incomplete.demand(&types, &definitions, ty).is_err());
        assert_eq!(symbolic.demand(&types, &definitions, ty)?, !copy);
        let mut complete = NeedsDrop::concrete_sparse([]);
        assert!(!complete.demand(&types, &definitions, ty)?);
        let steps = complete.steps;
        assert!(!complete.demand(&types, &definitions, ty)?);
        assert_eq!(complete.steps, steps);
        assert_eq!(complete.scope.as_deref().ok_or("states")?.states.len(), 2);
    }
    Ok(())
}

#[test]
fn concrete_destruction_requires_projection_normalization_even_with_copy_schema()
-> Result<(), &'static str> {
    use crate::types::{projection, traits};
    let mut types = Store::default();
    types.declare_projection(
        ItemId(2),
        projection::Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: copy::Properties {
                copy: true,
                clone_: true,
            },
        },
    )?;
    let trait_application = types.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    let ty = types.intern(Kind::Projection(projection::Application {
        item: ItemId(2),
        subject: TypeId::UNIT,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?;
    let definitions = Definitions::default();
    assert!(!NeedsDrop::new([]).demand(&types, &definitions, ty)?);
    assert_eq!(
        NeedsDrop::concrete_sparse([]).demand(&types, &definitions, ty),
        Err("associated type destruction checking requires normalization")
    );
    Ok(())
}

#[test]
fn concrete_destruction_retains_actual_ownership_cycles_and_exhaustion() -> Result<(), &'static str>
{
    use crate::types::{RegionId, aggregate};
    let mut types = Store::default();
    let item = ItemId(1);
    let nominal = types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    let fields = types.intern_list(&[])?;
    let mut definitions = Definitions::default();
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let mut query = NeedsDrop::concrete_sparse([item]);
    assert!(query.demand(&types, &definitions, nominal)?);
    for ty in [
        types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::STATIC,
            referent: nominal,
        })?,
        types.intern(Kind::Array {
            element: nominal,
            length: 0,
        })?,
    ] {
        assert!(!query.demand(&types, &definitions, ty)?);
    }
    let fields = types.intern_list(&[nominal])?;
    let mut cyclic = Definitions::default();
    cyclic.insert(
        item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let mut query = NeedsDrop::concrete_sparse([]);
    for _ in 0..2 {
        assert_eq!(
            query.demand(&types, &cyclic, nominal),
            Err("recursive by-value destruction requirement")
        );
    }
    let root = graph(&mut types, 8, 4)?;
    let mut query = NeedsDrop::concrete_sparse([]);
    query.steps = 999_999;
    for _ in 0..2 {
        assert_eq!(
            query.demand(&types, &definitions, root),
            Err("destruction query work budget exceeded")
        );
    }
    Ok(())
}

#[test]
fn sparse_concrete_scope_preserves_the_existing_query_layout() {
    struct PreviousScope {
        _environment: Arc<Environment>,
        _builtins: Option<Box<BuiltinQuery>>,
        _states: HashMap<TypeId, State>,
    }
    assert_eq!(size_of::<Scope>(), size_of::<PreviousScope>());
    assert_eq!(
        size_of::<Option<Arc<Environment>>>(),
        size_of::<Arc<Environment>>()
    );
    println!(
        "NeedsDrop={} Scope={}",
        size_of::<NeedsDrop>(),
        size_of::<Scope>()
    );
}
