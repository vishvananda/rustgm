//! Opaque use handles cannot cross catalogs or collapse complete enclosing contexts.
use super::*;
use crate::{
    support::source::FileId,
    syntax::ast::NodeId,
    types::{GenericArgs, Primitive},
};

fn key() -> CallerKey {
    CallerKey {
        source: NodeKey {
            file: FileId(0),
            node: NodeId::new(3),
        },
        declaration: Some(DefId(2)),
        universals: 1,
        enclosing: Enclosing::Original,
    }
}
fn value(caller: CallerId) -> Use {
    let instance = Instance {
        definition: DefId(4),
        arguments: GenericArgs::EMPTY,
    };
    Use {
        caller,
        input: instance,
        selected: instance,
        original: NodeKey {
            file: FileId(0),
            node: NodeId::new(7),
        },
        actual_result: TypeId::primitive(Primitive::U8),
        requested: TypeId::primitive(Primitive::U8),
        seal: None,
    }
}

#[test]
fn catalog_epochs_and_result_seals_do_not_cross_declaration_owners() -> Result<(), &'static str> {
    eprintln!(
        "source catalog layouts caller={} catalog={} witness={} target={} request={} code={}",
        size_of::<Caller>(),
        size_of::<Catalog>(),
        size_of::<Handle>(),
        size_of::<crate::sema::values::execution::Target>(),
        size_of::<crate::sema::values::execution::Request>(),
        size_of::<crate::const_eval::code::Code>(),
    );
    let publication = Publication::new([0..2, 4..5])?;
    let mut first = Catalog::new()?;
    let first_caller = first.caller(key(), publication)?;
    let handle = first.retain_use(value(first_caller))?;
    let mut second = Catalog::new()?;
    let second_caller = second.caller(key(), publication)?;
    let other = second.retain_use(value(second_caller))?;
    assert_eq!(handle.row, other.row);
    assert_ne!(handle.epoch, other.epoch);
    assert!(first.lookup(other).is_err());
    assert!(second.lookup(handle).is_err());
    assert_eq!(format!("{handle:?}"), "SourceComputation(..)");
    let mut changed = value(first_caller);
    changed.requested = TypeId::primitive(Primitive::U16);
    let changed_handle = first.retain_use(changed)?;
    assert_ne!(handle, changed_handle);
    assert_eq!(
        first.lookup(handle)?.0.requested,
        TypeId::primitive(Primitive::U8)
    );
    assert_eq!(
        first.lookup(changed_handle)?.0.requested,
        TypeId::primitive(Primitive::U16)
    );
    Ok(())
}

#[test]
fn full_contexts_and_publications_survive_reuse_without_recycling_rows() -> Result<(), &'static str>
{
    let mut catalog = Catalog::new()?;
    let publication = Publication::new([0..2, 4..5])?;
    let caller = catalog.caller(key(), publication)?;
    let handle = catalog.retain_use(value(caller))?;
    for _ in 0..128 {
        assert!(catalog.caller(key(), publication)? == caller);
        assert_eq!(catalog.retain_use(value(caller))?, handle);
    }
    assert_eq!(catalog.callers.len(), 1);
    assert_eq!(catalog.uses.len(), 1);
    for argument in [GenericArgs(1), GenericArgs(2)] {
        let context = CallerKey {
            enclosing: Enclosing::Initializer(Instance {
                definition: DefId(2),
                arguments: argument,
            }),
            ..key()
        };
        let caller = catalog.caller(context, publication)?;
        let use_ = catalog.retain_use(value(caller))?;
        assert_ne!(use_, handle);
        assert_eq!(catalog.lookup(use_)?.1, context);
    }
    assert_eq!(catalog.callers.len(), 3);
    assert_eq!(catalog.uses.len(), 3);
    assert!(
        catalog
            .caller(key(), Publication::new([0..3, 4..5])?)
            .is_err()
    );
    assert_eq!(catalog.callers.len(), 3);
    Ok(())
}

#[test]
fn catalog_growth_is_preflighted_for_both_independent_axes() -> Result<(), &'static str> {
    for count in [8usize, 32, 128] {
        let publication = Publication::new([0..0, 0..0])?;
        for many_callers in [false, true] {
            let mut catalog = Catalog::new()?;
            let initial = catalog.bytes;
            for n in 0..count {
                let mut identity = key();
                if many_callers {
                    identity.source.node =
                        NodeId::new(u32::try_from(n).map_err(|_| "node overflow")?);
                }
                let caller = catalog.caller(identity, publication)?;
                let mut entry = value(caller);
                entry.input.definition =
                    DefId(u32::try_from(n).map_err(|_| "definition overflow")?);
                catalog.retain_use(entry)?;
            }
            assert_eq!(catalog.callers.len(), if many_callers { count } else { 1 });
            assert_eq!(catalog.uses.len(), count);
            assert!(catalog.bytes - initial <= count * 2048);
            catalog.bytes = LIMIT;
            let before = (catalog.callers.len(), catalog.uses.len());
            let mut entry = value(CallerId(NonZeroU32::MIN));
            entry.original.node = NodeId::new(900);
            assert!(catalog.retain_use(entry).is_err());
            assert!(
                catalog
                    .caller(
                        CallerKey {
                            universals: 2,
                            ..key()
                        },
                        publication
                    )
                    .is_err()
            );
            assert_eq!((catalog.callers.len(), catalog.uses.len()), before);
        }
    }
    assert!(Publication::new([std::ops::Range { start: 3, end: 1 }, 4..5]).is_err());
    assert!(Publication::new([0..4, 3..5]).is_err());
    Ok(())
}
