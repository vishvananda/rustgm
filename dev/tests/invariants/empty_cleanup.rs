//! Empty-registry queries remain lazy, exact and shared within one emission.
use super::*;
use crate::types::{GenericArgs, Kind, Primitive, copy, opaque};
const LIMIT: usize = 1024 * 1024;

#[test]
fn empty_cleanup_shares_reachable_work_without_store_or_permission_fallback()
-> Result<(), &'static str> {
    let mut expected = None;
    for noise in [0, 8192] {
        let mut types = Store::default();
        let definitions = Definitions::default();
        for length in 0..noise {
            types.intern(Kind::Array {
                element: TypeId::UNIT,
                length,
            })?;
        }
        let mut shared = TypeId::primitive(Primitive::U8);
        for _ in 0..32 {
            let fields = types.intern_list(&[shared; 8])?;
            shared = types.intern(Kind::Tuple(fields))?;
        }
        let mut roots = Vec::new();
        for width in [8, 32, 128] {
            let fields = types.intern_list(&vec![shared; width])?;
            roots.push((width, types.intern(Kind::Tuple(fields))?));
        }
        let mut state = State {
            types: &types,
            definitions: &definitions,
            definition: None,
            empty: None,
        };
        assert!(state.empty.is_none());
        assert!(
            state
                .dropless(&types, &definitions, roots[0].1, false, LIMIT)
                .is_err()
        );
        assert!(state.empty.is_none());
        assert!(
            state
                .dropless(&types, &definitions, roots[0].1, true, 0)
                .is_err()
        );
        assert!(state.empty.is_none());
        let mut measured = Vec::new();
        for (width, root) in roots {
            let before = state.empty.as_deref().map_or(0, |query| query.steps);
            state.dropless(&types, &definitions, root, true, LIMIT)?;
            let query = state.empty.as_deref().ok_or("query")?;
            let counters = (query.steps - before, query.retained_bytes());
            if width != 8 {
                assert!(counters.0 <= 4 * width + 8);
            }
            let steps = query.steps;
            for _ in 0..128 {
                state.dropless(&types, &definitions, root, true, LIMIT)?;
            }
            let query = state.empty.as_deref().ok_or("query")?;
            assert_eq!(query.steps, steps + 128);
            assert_eq!(query.retained_bytes(), counters.1);
            println!("noise{noise}/width{width}: {counters:?}");
            measured.push(counters);
        }
        if let Some(expected) = &expected {
            assert_eq!(&measured, expected);
        } else {
            expected = Some(measured);
        }
        let steps = state.empty.as_deref().ok_or("query")?.steps;
        assert!(
            state
                .dropless(&Store::default(), &definitions, TypeId::UNIT, true, LIMIT)
                .is_err()
        );
        let unrelated = Box::new(Definitions::default());
        assert!(
            state
                .dropless(&types, &unrelated, TypeId::UNIT, true, LIMIT)
                .is_err()
        );
        assert!(
            state
                .dropless(&types, &definitions, TypeId::UNIT, false, LIMIT)
                .is_err()
        );
        assert_eq!(state.empty.as_deref().ok_or("query")?.steps, steps);
    }
    Ok(())
}
#[test]
fn empty_cleanup_never_publishes_missing_heads_or_copy_flags_as_false() -> Result<(), &'static str>
{
    let mut types = Store::default();
    let definitions = Definitions::default();
    let id = types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties {
            copy: true,
            clone_: true,
        },
    })?;
    let root = types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
    {
        let mut state = State {
            types: &types,
            definitions: &definitions,
            definition: None,
            empty: None,
        };
        for _ in 0..128 {
            assert!(
                state
                    .dropless(&types, &definitions, root, true, LIMIT)
                    .is_err()
            );
        }
    }
    types.publish_storage_head(root, TypeId::primitive(Primitive::U8))?;
    let mut fresh = State {
        types: &types,
        definitions: &definitions,
        definition: None,
        empty: None,
    };
    fresh.dropless(&types, &definitions, root, true, LIMIT)?;
    assert!(fresh.empty.is_some());
    println!(
        "State={} Context={} Body={}",
        size_of::<State<'_>>(),
        size_of::<crate::lower::context::Context<'_>>(),
        size_of::<crate::body::ir::Body>()
    );
    Ok(())
}

#[test]
fn empty_slice_cleanup_requires_fresh_exact_element_readiness() -> Result<(), &'static str> {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let byte = TypeId::primitive(Primitive::U8);
    let slice = types.intern(Kind::Slice(byte))?;
    let opaque = types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties {
            copy: true,
            clone_: true,
        },
    })?;
    let pending = types.intern(Kind::Opaque(opaque, GenericArgs::EMPTY))?;
    let pending_slice = types.intern(Kind::Slice(pending))?;
    {
        let mut state = State {
            types: &types,
            definitions: &definitions,
            definition: None,
            empty: None,
        };
        state.dropless(&types, &definitions, slice, true, LIMIT)?;
        let before = state.empty.as_deref().ok_or("query")?.steps;
        for _ in 0..128 {
            state.dropless(&types, &definitions, slice, true, LIMIT)?;
        }
        assert_eq!(state.empty.as_deref().ok_or("query")?.steps, before + 128);
        assert!(
            state
                .dropless(&types, &definitions, slice, false, LIMIT)
                .is_err()
        );
        for _ in 0..128 {
            assert!(
                state
                    .dropless(&types, &definitions, pending_slice, true, LIMIT)
                    .is_err()
            );
        }
    }
    types.publish_storage_head(pending, byte)?;
    let mut fresh = State {
        types: &types,
        definitions: &definitions,
        definition: None,
        empty: None,
    };
    fresh.dropless(&types, &definitions, pending_slice, true, LIMIT)?;
    Ok(())
}
