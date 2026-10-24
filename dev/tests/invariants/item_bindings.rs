use super::{Binding, BindingTable, Entry, ItemContext, Local, LocalContext, Namespace, ScopeId};
use crate::{
    expand::marks::{ExpansionId, Marks, SyntaxContextId, Transparency},
    resolve::{
        model::{Access, DefId, Origin},
        names::NameId,
    },
};

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

std::thread_local! {
    static KEY_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
pub(super) fn visit_key() {
    KEY_VISITS.set(KEY_VISITS.get() + 1);
}

fn contexts(count: u32) -> std::result::Result<Vec<ItemContext>, &'static str> {
    let mut marks = Marks::default();
    let mut contexts = vec![ItemContext::ROOT];
    for index in 1..=count {
        let context = marks.apply(
            SyntaxContextId::ROOT,
            ExpansionId(index),
            Transparency::Opaque,
            SyntaxContextId::ROOT,
        )?;
        contexts.push(marks.item(context)?);
    }
    Ok(contexts)
}
fn binding(id: u32) -> Binding {
    Binding {
        target: DefId(id),
        access: Access::Public,
        origin: Origin::Direct,
        ambiguous: false,
    }
}

#[test]
fn root_bindings_keep_the_context_index_unallocated() -> Result {
    let mut table = BindingTable::default();
    for name in 0..1024 {
        table.insert(ScopeId(0), NameId(name), Namespace::Value, binding(name))?;
        assert_eq!(
            table.get(ScopeId(0), NameId(name), Namespace::Value),
            Some(&binding(name))
        );
    }
    assert!(!table.has_contextual_items());
    assert!(table.histories.is_none());
    assert_eq!(table.scope(ScopeId(0)).count(), 1024);
    assert_eq!(table.iter().count(), 1024);
    eprintln!(
        "item bindings table={} entry={} local={} index={} full_key={} journal_key={}",
        size_of::<BindingTable>(),
        size_of::<Entry>(),
        size_of::<Local>(),
        size_of::<super::contextual::Index>(),
        size_of::<super::BindingKey>(),
        size_of::<(NameId, Namespace, ItemContext)>()
    );
    assert_eq!(size_of::<BindingTable>(), 120);
    assert_eq!(size_of::<Entry>(), 44);
    assert_eq!(size_of::<Local>(), 32);
    Ok(())
}

#[test]
fn contextual_population_never_grows_the_root_slots() -> Result {
    let contexts = contexts(128)?;
    for root_width in [0, 8, 32, 128] {
        let mut table = BindingTable::default();
        for name in 0..root_width {
            table.insert(ScopeId(0), NameId(name), Namespace::Type, binding(name))?;
        }
        let capacity = table.slots.len();
        for context in contexts.iter().skip(1) {
            for name in 0..128 {
                let key = (ScopeId(0), NameId(name), Namespace::Type, *context);
                table.insert_key(key, binding(name))?;
                assert_eq!(table.get_key(key), Some(&binding(name)));
            }
            assert_eq!(table.slots.len(), capacity);
        }
        assert!(table.has_contextual_items());
        assert_eq!(
            table.get(ScopeId(0), NameId(root_width), Namespace::Type),
            None
        );
        assert!(
            table
                .probe(ScopeId(0), NameId(root_width), Namespace::Type)
                .is_none()
        );
        table.insert(
            ScopeId(0),
            NameId(root_width),
            Namespace::Type,
            binding(root_width),
        )?;
        assert!(table.slots.len() <= capacity.max(64) * 2);
        assert_eq!(table.iter().count(), root_width as usize + 1);
    }
    Ok(())
}

#[test]
fn exact_context_duplicates_preserve_alias_keys_and_global_order() -> Result {
    let contexts = contexts(2)?;
    let mut table = BindingTable::default();
    let mut expected = Vec::new();
    for (index, context) in contexts.iter().enumerate() {
        let key = (ScopeId(0), NameId(9), Namespace::Type, *context);
        // All three aliases target the same declaration; identity is the key's.
        assert!(table.declare_key(key, binding(7), false)?);
        assert!(!table.declare_key(key, binding(8), false)?);
        expected.push(((key.1, key.2, key.3), u32::try_from(index)?));
    }
    assert_eq!(table.keys(ScopeId(0)).collect::<Vec<_>>(), expected);
    let key = (ScopeId(0), NameId(9), Namespace::Type, contexts[1]);
    table.remove_key(key);
    assert_eq!(table.get_key(key), None);
    assert_eq!(table.created(), 3);
    assert!(table.has_contextual_items());
    assert!(table.declare_key(key, binding(10), false)?);
    assert_eq!(table.created(), 3);
    assert_eq!(table.keys(ScopeId(0)).collect::<Vec<_>>(), expected);
    assert_eq!(table.get_key(key), Some(&binding(10)));
    assert_eq!(
        table.get(ScopeId(0), NameId(9), Namespace::Type),
        Some(&binding(7))
    );
    Ok(())
}

#[test]
fn local_only_entries_share_ordinals_but_keep_local_and_item_projections_separate() -> Result {
    let contexts = contexts(1)?;
    let mut table = BindingTable::default();
    let local = (ScopeId(0), NameId(1), Namespace::Value);
    table.define_local(local, LocalContext::ROOT, binding(0), false)?;
    let key = (local.0, local.1, local.2, contexts[1]);
    table.insert_key(key, binding(1))?;
    assert_eq!(table.created(), 2);
    assert_eq!(
        table.keys(local.0).collect::<Vec<_>>(),
        vec![((local.1, local.2, contexts[1]), 1)]
    );
    table.insert(local.0, local.1, local.2, binding(2))?;
    let root = table
        .probe(local.0, local.1, local.2)
        .ok_or("missing local")?;
    assert_eq!(table.local(root, LocalContext::ROOT, 3), Some(binding(0)));
    assert_eq!(table.binding(root), Some(&binding(2)));
    assert_eq!(table.get_key(key), Some(&binding(1)));
    assert_eq!(table.created(), 2);
    assert_eq!(
        table
            .keys(local.0)
            .map(|(_, ordinal)| ordinal)
            .collect::<Vec<_>>(),
        vec![1, 0]
    );
    Ok(())
}

#[test]
fn snapshot_journal_work_ignores_the_later_scope_suffix() -> Result {
    let contexts = contexts(1)?;
    for scopes in [1, 8, 32] {
        for suffix in [8, 32, 128] {
            let mut table = BindingTable::default();
            for scope in 0..scopes {
                for name in 0..4 {
                    table.insert(
                        ScopeId(scope),
                        NameId(name),
                        Namespace::Value,
                        binding(name),
                    )?;
                }
            }
            let snapshot = table.snapshot();
            assert_eq!(snapshot.tails.len(), scopes as usize);
            for scope in 0..scopes {
                for name in 4..suffix + 4 {
                    let key = (
                        ScopeId(scope),
                        NameId(name),
                        Namespace::Value,
                        contexts[(name % 2) as usize],
                    );
                    table.insert_key(key, binding(name))?;
                }
            }
            for scope in 0..scopes {
                KEY_VISITS.set(0);
                let keys = table
                    .keys_before(ScopeId(scope), &snapshot)
                    .collect::<Vec<_>>();
                assert_eq!(
                    keys,
                    (0..4)
                        .map(|n| (NameId(n), Namespace::Value, ItemContext::ROOT))
                        .collect::<Vec<_>>()
                );
                // The inclusive iterator currently peeks one entry after the tail.
                assert!(KEY_VISITS.get() <= 5);
            }
        }
    }
    Ok(())
}
