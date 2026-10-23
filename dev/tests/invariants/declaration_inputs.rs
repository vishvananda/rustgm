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
