//! Textual lookup preserves exact item identities through immutable scope links.
use super::*;
use crate::expand::marks::{ExpansionId, Marks, SyntaxContextId, Transparency};
type TestResult<T = ()> = Result<T, &'static str>;

fn lex<T>(value: Result<T, LexError>) -> TestResult<T> {
    value.map_err(|error| error.message)
}
fn context(marks: &mut Marks, id: u32) -> TestResult<ItemContext> {
    let raw = marks.apply(
        SyntaxContextId::ROOT,
        ExpansionId(id),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    marks.item(raw)
}

#[test]
fn ordinary_definitions_and_queries_keep_the_context_owner_absent() -> TestResult {
    let mut scopes = Scopes::default();
    let name = NameId(1);
    let mut head = Scope::ROOT;
    for definition in 0..512 {
        head = lex(scopes.define_context(head, name, ItemContext::ROOT, definition))?;
        assert_eq!(lex(scopes.lookup(head, name))?, Lookup::Found(definition));
    }
    assert!(scopes.contexts.is_none());
    assert_eq!(size_of::<Entry>(), 24);
    assert_eq!(size_of::<(Scope, NameId)>(), 8);
    eprintln!(
        "textual sizes scopes={} entry={} cold_owner={}",
        size_of::<Scopes>(),
        size_of::<Entry>(),
        size_of::<Contexts>()
    );
    Ok(())
}

#[test]
fn identical_spellings_and_historical_branches_keep_exact_contexts() -> TestResult {
    let mut marks = Marks::default();
    let a = context(&mut marks, 1)?;
    let b = context(&mut marks, 2)?;
    let mut scopes = Scopes::default();
    let name = NameId(1);
    let root = lex(scopes.define(Scope::ROOT, name, 0))?;
    let left = lex(scopes.define_context(root, name, a, 1))?;
    let right = lex(scopes.define_context(root, name, b, 2))?;
    let shadow = lex(scopes.define_context(left, name, a, 3))?;
    for (head, identity, expected) in [
        (left, a, Lookup::Found(1)),
        (right, b, Lookup::Found(2)),
        (shadow, a, Lookup::Found(3)),
        (left, b, Lookup::Missing),
        (right, a, Lookup::Missing),
        (shadow, ItemContext::ROOT, Lookup::Found(0)),
        (right, ItemContext::ROOT, Lookup::Found(0)),
        (root, a, Lookup::Missing),
    ] {
        for _ in 0..2 {
            assert_eq!(lex(scopes.lookup_context(head, name, identity))?, expected);
        }
    }
    assert_eq!(
        scopes
            .contexts
            .as_ref()
            .ok_or("missing contexts")?
            .definitions
            .len(),
        3
    );
    Ok(())
}

#[test]
fn pending_context_queries_resume_without_reusing_a_sibling_answer() -> TestResult {
    let mut marks = Marks::default();
    let a = context(&mut marks, 1)?;
    let b = context(&mut marks, 2)?;
    let mut scopes = Scopes::default();
    assert!(scopes.define_context(Scope(9), NameId(1), a, 0).is_err());
    assert!(scopes.contexts.is_none());
    let name = NameId(1);
    let root = lex(scopes.define(Scope::ROOT, name, 0))?;
    let output = lex(scopes.output())?;
    for identity in [ItemContext::ROOT, a, b] {
        assert_eq!(
            lex(scopes.lookup_context(output, name, identity))?,
            Lookup::Pending(output)
        );
    }
    scopes.watch(output, Observer::Invocation(2));
    scopes.watch(output, Observer::Import(3));
    let produced = lex(scopes.define_context(root, name, a, 1))?;
    assert_eq!(
        lex(scopes.complete(output, produced))?,
        [Observer::Invocation(2), Observer::Import(3)]
    );
    for (identity, expected) in [
        (ItemContext::ROOT, Lookup::Found(0)),
        (a, Lookup::Found(1)),
        (b, Lookup::Missing),
    ] {
        assert_eq!(
            lex(scopes.lookup_context(output, name, identity))?,
            expected
        );
    }
    assert!(scopes.complete(output, root).is_err());
    assert_eq!(
        lex(scopes.lookup_context(output, name, a))?,
        Lookup::Found(1)
    );
    Ok(())
}

#[test]
fn repeated_context_queries_share_the_reached_scope_suffix() -> TestResult {
    for depth in [8, 32, 128, 512] {
        for uses in [8, 32, 128] {
            let mut marks = Marks::default();
            let identity = context(&mut marks, 1)?;
            let absent = context(&mut marks, 2)?;
            let mut scopes = Scopes::default();
            let name = NameId(1);
            let root = lex(scopes.define(Scope::ROOT, name, 0))?;
            let mut head = lex(scopes.define_context(root, name, identity, 1))?;
            for index in 0..depth {
                head = lex(scopes.define(head, NameId(2), index))?;
            }
            let before = scopes.lookup_steps;
            for _ in 0..uses {
                assert_eq!(
                    lex(scopes.lookup_context(head, name, identity))?,
                    Lookup::Found(1)
                );
                assert_eq!(
                    lex(scopes.lookup_context(head, name, absent))?,
                    Lookup::Missing
                );
                assert_eq!(lex(scopes.lookup(head, name))?, Lookup::Found(0));
            }
            let work = scopes.lookup_steps - before;
            let cold = scopes.contexts.as_ref().ok_or("missing contexts")?;
            assert_eq!(cold.definitions.len(), 1);
            assert_eq!(cold.cache.len(), 2 * depth + 4);
            assert_eq!(scopes.cache.len(), depth + 2);
            assert_eq!(work, 3 * depth + 3 * uses + 3);
            eprintln!(
                "textual suffix depth={depth} uses={uses} steps={work} contextual_cache={}",
                cold.cache.len()
            );
        }
    }
    Ok(())
}
