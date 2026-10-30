//! Source inputs remain borrowed, with full qualifiers and distinct declared prefix.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_input_views_keep_owner_prefix_binders_and_exact_locations() -> Result {
    let report = crate::test_source::resolve(
        "trait A{} trait B:A{} fn f<T:B>(){} fn g<'a,T:Copy+'a>(_:&'a T){} fn main(){}",
        true,
    )?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut observed = 0;
    for function in &facts.functions {
        let owner = function.definition;
        let expected = facts
            .generics
            .predicates_in_scope(owner)
            .collect::<Vec<_>>();
        let views = all(&facts.generics, facts.crates.as_deref(), owner)?.collect::<Vec<_>>();
        assert_eq!(expected.len(), views.len());
        for (actual, source) in views.iter().zip(expected) {
            assert_eq!(actual.source.source(), Some(source.source));
            assert_eq!(actual.value, source.value);
        }
        let expected = facts
            .generics
            .declared_predicates_in_scope(owner)
            .collect::<Vec<_>>();
        let views = declared(&facts.generics, facts.crates.as_deref(), owner)?.collect::<Vec<_>>();
        assert_eq!(expected.len(), views.len());
        for (actual, source) in views.iter().zip(expected) {
            assert_eq!(actual.source.source(), Some(source.source));
            assert_eq!(actual.value, source.value);
        }
        observed += views.len();
    }
    assert!(observed >= 3);
    assert_eq!(size_of::<crate::sema::generics::Predicate>(), 20);
    assert_eq!(size_of::<Input>(), 24);
    assert!(
        all(&facts.generics, facts.crates.as_deref(), DefId(u32::MAX))?
            .next()
            .is_none()
    );
    Ok(())
}

#[test]
fn collected_scope_inputs_preserve_own_prefixes_and_pay_before_growth_or_cycles() -> Result {
    let report = crate::test_source::resolve(
        "trait Parent<T>{fn member(self)->T;}fn absent(){}fn main(){}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let member = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "member"
        })
        .ok_or("actual trait member")?
        .definition;
    assert_eq!(own_view(&facts.generics, None, member)?.len(), 0);
    let mut steps = 0;
    let mut bytes = 0;
    let inputs = collect_inherited(&facts.generics, None, member, |work, retained| {
        steps += work;
        bytes += retained;
        Ok(())
    })?;
    assert!(!inputs.is_empty());
    assert!(steps > inputs.len());
    assert!(bytes >= inputs.len() * size_of::<Input>());
    let expected = all(&facts.generics, None, member)?.collect::<Vec<_>>();
    assert_eq!(inputs.len(), expected.len());
    for (actual, expected) in inputs.iter().zip(expected) {
        assert_eq!(actual.source, expected.source);
        assert_eq!(actual.value, expected.value);
    }
    let mut attempted = 0;
    assert!(
        collect_inherited(&facts.generics, None, member, |work, bytes| {
            attempted += work;
            if bytes == 0 {
                Ok(())
            } else {
                Err("denied input growth".into())
            }
        })
        .is_err()
    );
    assert!(attempted > 0);
    assert_eq!(own_view(&facts.generics, None, member)?.len(), 0);
    let absent = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "absent"
        })
        .ok_or("ordinary empty declaration")?
        .definition;
    let empty = collect_inherited(&facts.generics, None, absent, |_, bytes| {
        assert_eq!(bytes, 0);
        Ok(())
    })?;
    assert_eq!(empty.capacity(), 0);
    facts.generics.parents.insert(member, member);
    let mut visits = 0;
    let error = collect_inherited(&facts.generics, None, member, |_, _| {
        visits += 1;
        Ok(())
    })
    .err()
    .ok_or("cyclic parent chain accepted")?;
    assert!(error.contains("parent cycle"));
    assert_eq!(visits, facts.generics.scope_limit() + 1);
    Ok(())
}
