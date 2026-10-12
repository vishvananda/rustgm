use rustgm::{
    expand::textual::{Lookup, Observer, Scope, Scopes},
    support::names::Names,
};
#[test]
fn unresolved_predecessors_block_shadowable_names_but_not_nearer_definitions() -> Result<(), String>
{
    let mut names = Names::default();
    let m = names.intern("m").map_err(str::to_owned)?;
    let ready = names.intern("ready").map_err(str::to_owned)?;
    let mut scopes = Scopes::default();
    let prior = scopes
        .define(Scope::ROOT, m, 0)
        .map_err(|e| format!("{e:?}"))?;
    let output = scopes.output().map_err(|e| format!("{e:?}"))?;
    let later = scopes
        .define(output, ready, 1)
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        scopes.lookup(later, ready).map_err(|e| format!("{e:?}"))?,
        Lookup::Found(1)
    );
    assert_eq!(
        scopes.lookup(later, m).map_err(|e| format!("{e:?}"))?,
        Lookup::Pending(output)
    );
    scopes.watch(output, Observer::Invocation(9));
    let generated = scopes.define(prior, m, 2).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        scopes
            .complete(output, generated)
            .map_err(|e| format!("{e:?}"))?,
        [Observer::Invocation(9)]
    );
    assert_eq!(
        scopes.lookup(later, m).map_err(|e| format!("{e:?}"))?,
        Lookup::Found(2)
    );
    assert_eq!(
        scopes.lookup(prior, m).map_err(|e| format!("{e:?}"))?,
        Lookup::Found(0)
    );
    Ok(())
}
#[test]
fn repeated_output_links_and_queries_have_linear_observed_work() -> Result<(), String> {
    let mut names = Names::default();
    let m = names.intern("m").map_err(str::to_owned)?;
    let mut scopes = Scopes::default();
    let mut head = scopes
        .define(Scope::ROOT, m, 7)
        .map_err(|e| format!("{e:?}"))?;
    for _ in 0..10_000 {
        let output = scopes.output().map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            scopes.lookup(output, m).map_err(|e| format!("{e:?}"))?,
            Lookup::Pending(output)
        );
        scopes
            .complete(output, head)
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            scopes.lookup(output, m).map_err(|e| format!("{e:?}"))?,
            Lookup::Found(7)
        );
        head = output;
    }
    assert!(scopes.lookup_steps < 40_000);
    assert!(scopes.complete(head, head).is_err());
    Ok(())
}
