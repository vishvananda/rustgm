use super::{Binding, BindingTable, LocalContext, Namespace, ScopeId};
use crate::resolve::{
    model::{Access, DefId, Origin},
    names::NameId,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn contexts(count: u32) -> Result<Vec<LocalContext>, &'static str> {
    use crate::expand::marks::{ExpansionId, Marks, SyntaxContextId, Transparency};
    let mut marks = Marks::default();
    (0..count)
        .map(|index| {
            if index == 0 {
                return Ok(LocalContext::ROOT);
            }
            let context = marks.apply(
                SyntaxContextId::ROOT,
                ExpansionId(index),
                Transparency::SemiOpaque,
                SyntaxContextId::ROOT,
            )?;
            marks.local(context)
        })
        .collect()
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
fn single_locals_use_the_original_entry_without_allocating_a_context_index() -> TestResult {
    let contexts = contexts(129)?;
    let mut table = BindingTable::default();
    for n in 0..1024 {
        let key = (ScopeId(0), NameId(n), Namespace::Value);
        table.define_local(key, contexts[(n % 4) as usize], binding(n), false)?;
        let entry = table
            .probe(key.0, key.1, key.2)
            .ok_or("missing local entry")?;
        assert_eq!(
            table.local(entry, contexts[(n % 4) as usize], n as usize + 1),
            Some(binding(n))
        );
        assert_eq!(
            table.local(entry, contexts[(n % 4) as usize], n as usize),
            None
        );
        assert_eq!(table.local(entry, contexts[7], usize::MAX), None);
    }
    assert!(table.histories.is_none());
    Ok(())
}

#[test]
fn local_predecessors_ignore_unrelated_contexts_and_preserve_every_shadow() -> TestResult {
    const CONTEXTS: u32 = 128;
    const SHADOWS: u32 = 32;
    let contexts = contexts(129)?;
    let mut table = BindingTable::default();
    let key = (ScopeId(0), NameId(0), Namespace::Value);
    for generation in 0..SHADOWS {
        for context in 0..CONTEXTS {
            let id = generation * CONTEXTS + context;
            table.define_local(
                key,
                contexts[(context) as usize],
                binding(id),
                generation != 0,
            )?;
        }
    }
    let entry = table
        .probe(key.0, key.1, key.2)
        .ok_or("missing local entry")?;
    let index = table
        .histories
        .as_ref()
        .ok_or("missing context histories")?;
    for context in 0..CONTEXTS {
        for generation in 0..=SHADOWS {
            let visible = (generation * CONTEXTS + context) as usize;
            let expected = generation
                .checked_sub(1)
                .map(|g| binding(g * CONTEXTS + context));
            assert_eq!(
                table.local(entry, contexts[(context) as usize], visible),
                expected
            );
            assert!(index.take_comparisons() <= 7);
        }
        assert_eq!(
            table.local(entry, contexts[(context) as usize], usize::MAX),
            Some(binding((SHADOWS - 1) * CONTEXTS + context))
        );
        assert_eq!(index.take_comparisons(), 1);
    }
    assert_eq!(
        table.local(entry, contexts[(CONTEXTS) as usize], usize::MAX),
        None
    );
    assert_eq!(index.take_comparisons(), 0);
    Ok(())
}

#[test]
fn duplicate_and_unordered_definitions_do_not_publish_partial_histories() -> TestResult {
    let contexts = contexts(129)?;
    let mut table = BindingTable::default();
    let key = (ScopeId(0), NameId(0), Namespace::Value);
    table.define_local(key, contexts[0], binding(4), false)?;
    assert_eq!(
        table.define_local(key, contexts[0], binding(5), false),
        Err("duplicate hygienic local binding")
    );
    assert_eq!(
        table.define_local(key, contexts[1], binding(3), false),
        Err("hygienic local definitions are not in creation order")
    );
    assert!(table.histories.is_none());
    assert_eq!(table.locals.len(), 1);
    table.define_local(key, contexts[1], binding(6), false)?;
    assert_eq!(
        table.define_local(key, contexts[0], binding(7), false),
        Err("duplicate hygienic local binding")
    );
    assert_eq!(
        table.define_local(key, contexts[0], binding(5), true),
        Err("hygienic local definitions are not in creation order")
    );
    assert_eq!(table.locals.len(), 2);
    let entry = table
        .probe(key.0, key.1, key.2)
        .ok_or("missing local entry")?;
    assert_eq!(
        table.local(entry, contexts[0], usize::MAX),
        Some(binding(4))
    );
    assert_eq!(
        table.local(entry, contexts[1], usize::MAX),
        Some(binding(6))
    );
    assert_eq!(table.local(entry, contexts[1], 6), None);
    assert_eq!(table.scope(ScopeId(0)).count(), 0);
    Ok(())
}

#[test]
fn scope_and_namespace_are_part_of_each_local_history_identity() -> TestResult {
    let contexts = contexts(129)?;
    let mut table = BindingTable::default();
    for (i, (scope, namespace)) in [
        (ScopeId(0), Namespace::Value),
        (ScopeId(1), Namespace::Value),
        (ScopeId(0), Namespace::Label),
    ]
    .into_iter()
    .enumerate()
    {
        let id = u32::try_from(i * 2)?;
        let key = (scope, NameId(0), namespace);
        table.define_local(key, contexts[0], binding(id), false)?;
        table.define_local(key, contexts[0], binding(id + 1), true)?;
        let entry = table
            .probe(scope, key.1, namespace)
            .ok_or("missing local entry")?;
        assert_eq!(
            table.local(entry, contexts[0], id as usize + 1),
            Some(binding(id))
        );
        assert_eq!(
            table.local(entry, contexts[0], usize::MAX),
            Some(binding(id + 1))
        );
    }
    Ok(())
}
