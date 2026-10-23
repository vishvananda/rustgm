use super::{Binding, BindingTable, ContextId, Namespace, ScopeId};
use crate::resolve::{
    model::{Access, DefId, Origin},
    names::NameId,
};

fn binding(id: u32) -> Binding {
    Binding {
        target: DefId(id),
        access: Access::Public,
        origin: Origin::Direct,
        ambiguous: false,
    }
}

#[test]
fn single_locals_use_the_original_entry_without_allocating_a_context_index() {
    let mut table = BindingTable::default();
    for n in 0..1024 {
        let key = (ScopeId(0), NameId(n), Namespace::Value);
        table
            .define_local(key, ContextId(n % 4), binding(n), false)
            .unwrap();
        let entry = table.probe(key.0, key.1, key.2).unwrap();
        assert_eq!(
            table.local(entry, ContextId(n % 4), n as usize + 1),
            Some(binding(n))
        );
        assert_eq!(table.local(entry, ContextId(n % 4), n as usize), None);
        assert_eq!(table.local(entry, ContextId(7), usize::MAX), None);
    }
    assert!(table.histories.is_none());
}

#[test]
fn local_predecessors_ignore_unrelated_contexts_and_preserve_every_shadow() {
    let mut table = BindingTable::default();
    let key = (ScopeId(0), NameId(0), Namespace::Value);
    const CONTEXTS: u32 = 128;
    const SHADOWS: u32 = 32;
    for generation in 0..SHADOWS {
        for context in 0..CONTEXTS {
            let id = generation * CONTEXTS + context;
            table
                .define_local(key, ContextId(context), binding(id), generation != 0)
                .unwrap();
        }
    }
    let entry = table.probe(key.0, key.1, key.2).unwrap();
    let index = table.histories.as_ref().unwrap();
    for context in 0..CONTEXTS {
        for generation in 0..=SHADOWS {
            let visible = (generation * CONTEXTS + context) as usize;
            let expected = generation
                .checked_sub(1)
                .map(|g| binding(g * CONTEXTS + context));
            assert_eq!(table.local(entry, ContextId(context), visible), expected);
            assert!(index.take_comparisons() <= 7);
        }
        assert_eq!(
            table.local(entry, ContextId(context), usize::MAX),
            Some(binding((SHADOWS - 1) * CONTEXTS + context))
        );
        assert_eq!(index.take_comparisons(), 1);
    }
    assert_eq!(table.local(entry, ContextId(CONTEXTS), usize::MAX), None);
    assert_eq!(index.take_comparisons(), 0);
}

#[test]
fn duplicate_and_unordered_definitions_do_not_publish_partial_histories() {
    let mut table = BindingTable::default();
    let key = (ScopeId(0), NameId(0), Namespace::Value);
    table
        .define_local(key, ContextId(0), binding(4), false)
        .unwrap();
    assert_eq!(
        table.define_local(key, ContextId(0), binding(5), false),
        Err("duplicate hygienic local binding")
    );
    assert_eq!(
        table.define_local(key, ContextId(1), binding(3), false),
        Err("hygienic local definitions are not in creation order")
    );
    assert!(table.histories.is_none());
    assert_eq!(table.locals.len(), 1);
    table
        .define_local(key, ContextId(1), binding(6), false)
        .unwrap();
    assert_eq!(
        table.define_local(key, ContextId(0), binding(7), false),
        Err("duplicate hygienic local binding")
    );
    assert_eq!(
        table.define_local(key, ContextId(0), binding(5), true),
        Err("hygienic local definitions are not in creation order")
    );
    assert_eq!(table.locals.len(), 2);
    let entry = table.probe(key.0, key.1, key.2).unwrap();
    assert_eq!(
        table.local(entry, ContextId(0), usize::MAX),
        Some(binding(4))
    );
    assert_eq!(
        table.local(entry, ContextId(1), usize::MAX),
        Some(binding(6))
    );
    assert_eq!(table.local(entry, ContextId(1), 6), None);
    assert_eq!(table.scope(ScopeId(0)).count(), 0);
}

#[test]
fn scope_and_namespace_are_part_of_each_local_history_identity() {
    let mut table = BindingTable::default();
    for (i, (scope, namespace)) in [
        (ScopeId(0), Namespace::Value),
        (ScopeId(1), Namespace::Value),
        (ScopeId(0), Namespace::Label),
    ]
    .into_iter()
    .enumerate()
    {
        let id = u32::try_from(i * 2).unwrap();
        let key = (scope, NameId(0), namespace);
        table
            .define_local(key, ContextId(0), binding(id), false)
            .unwrap();
        table
            .define_local(key, ContextId(0), binding(id + 1), true)
            .unwrap();
        let entry = table.probe(scope, key.1, namespace).unwrap();
        assert_eq!(
            table.local(entry, ContextId(0), id as usize + 1),
            Some(binding(id))
        );
        assert_eq!(
            table.local(entry, ContextId(0), usize::MAX),
            Some(binding(id + 1))
        );
    }
}
