use super::{Binding, ItemContext, Namespace, ScopeId, Work};
use crate::{
    expand::marks::{ExpansionId, LocalContext, Marks, SyntaxContextId, Transparency},
    resolve::{
        model::{Access, DefId, Origin, Resolution},
        names::NameId,
    },
};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn context() -> std::result::Result<ItemContext, &'static str> {
    let mut marks = Marks::default();
    let raw = marks.apply(
        SyntaxContextId::ROOT,
        ExpansionId(1),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    marks.item(raw)
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
fn a_preexisting_local_first_publishes_exactly_one_item_delta() -> Result {
    let context = context()?;
    let mut facts = Resolution::default();
    let root = (ScopeId(0), NameId(1), Namespace::Value, ItemContext::ROOT);
    let opaque = (root.0, root.1, root.2, context);
    facts.bindings.define_local(
        (root.0, root.1, root.2),
        LocalContext::ROOT,
        binding(0),
        false,
    )?;
    facts.bindings.insert_key(opaque, binding(1))?;
    let mut work = Work::new(&facts);
    work.observe();
    facts.bindings.insert_key(root, binding(2))?;
    work.declared(&facts.bindings, root);
    assert_eq!(
        work.changes[0],
        vec![(root.1, root.2, context), (root.1, root.2, root.3)]
    );
    assert_eq!(work.events, vec![root]);
    // A later mutation is a real second delta; it must not be deduplicated away.
    facts.bindings.insert_key(root, binding(3))?;
    work.declared(&facts.bindings, root);
    assert_eq!(work.changes[0].len(), 3);
    assert_eq!(work.events, vec![root, root]);
    Ok(())
}

#[test]
fn lazy_journals_distinguish_root_and_opaque_keys_across_the_snapshot() -> Result {
    let context = context()?;
    let mut facts = Resolution::default();
    let root = (ScopeId(0), NameId(1), Namespace::Type, ItemContext::ROOT);
    let opaque = (root.0, root.1, root.2, context);
    facts.bindings.insert_key(root, binding(0))?;
    let mut work = Work::new(&facts);
    facts.bindings.insert_key(opaque, binding(1))?;
    work.declared(&facts.bindings, opaque);
    assert_eq!(
        work.changes[0],
        vec![(root.1, root.2, root.3), (root.1, root.2, context)]
    );
    work.journal(&facts.bindings, root.0);
    assert_eq!(work.changes[0].len(), 2);
    Ok(())
}

#[test]
fn a_batch_of_old_local_entries_is_not_seeded_before_its_declared_events() -> Result {
    let context = context()?;
    for baseline in [false, true] {
        let mut facts = Resolution::default();
        let prior = (ScopeId(0), NameId(0), Namespace::Value, ItemContext::ROOT);
        if baseline {
            facts.bindings.insert_key(prior, binding(0))?;
            facts.bindings.remove_key(prior);
        }
        let a = (ScopeId(0), NameId(1), Namespace::Value, ItemContext::ROOT);
        let b = (ScopeId(0), NameId(2), Namespace::Value, ItemContext::ROOT);
        for (index, key) in [a, b].iter().enumerate() {
            facts.bindings.define_local(
                (key.0, key.1, key.2),
                LocalContext::ROOT,
                binding(u32::try_from(index)?),
                false,
            )?;
        }
        let mut work = Work::new(&facts);
        let e = (a.0, a.1, a.2, context);
        let batch = [e, b, a];
        for (index, key) in batch.iter().enumerate() {
            facts
                .bindings
                .insert_key(*key, binding(u32::try_from(index)? + 2))?;
        }
        // The expansion indexer finishes the entire declaration batch first.
        for key in batch {
            work.declared(&facts.bindings, key);
        }
        let mut expected = Vec::new();
        if baseline {
            expected.push((prior.1, prior.2, prior.3));
        }
        expected.extend(batch.map(|key| (key.1, key.2, key.3)));
        assert_eq!(work.changes[0], expected);
        let order = facts.bindings.keys(b.0).collect::<Vec<_>>();
        facts.bindings.remove_key(b);
        work.declared(&facts.bindings, b);
        facts.bindings.insert_key(b, binding(9))?;
        work.declared(&facts.bindings, b);
        expected.extend([(b.1, b.2, b.3); 2]);
        assert_eq!(work.changes[0], expected);
        assert_eq!(facts.bindings.keys(b.0).collect::<Vec<_>>(), order);
        let later = (ScopeId(8), a.1, a.2, context);
        facts.bindings.insert_key(later, binding(10))?;
        work.extend(&facts);
        work.declared(&facts.bindings, later);
        assert_eq!(work.changes[8], vec![(later.1, later.2, later.3)]);
    }
    Ok(())
}

#[test]
fn waiting_paths_wake_only_for_their_complete_context_key() -> Result {
    let context = context()?;
    let facts = Resolution::default();
    let root = (ScopeId(0), NameId(1), Namespace::Macro, ItemContext::ROOT);
    let opaque = (root.0, root.1, root.2, context);
    let mut work = Work::new(&facts);
    work.queued = vec![false; 3];
    work.watch(0, vec![root]);
    work.watch(1, vec![opaque]);
    work.watch(2, vec![root, opaque]);
    work.changed(opaque);
    assert_eq!(work.queue.iter().copied().collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(work.waiting.len(), 1);
    work.changed(root);
    assert_eq!(
        work.queue.iter().copied().collect::<Vec<_>>(),
        vec![1, 2, 0]
    );
    assert!(work.waiting.is_empty());
    Ok(())
}
