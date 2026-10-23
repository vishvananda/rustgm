//! Exact Rust layout precedes negative-result API changes; DWARF enum zero is unusable.
use super::*;
#[test]
fn source_auto_and_refutation_layouts_are_measured() {
    enum Choice {
        Positive(usize),
        Negative(usize),
    }
    let sum: usize = [Choice::Positive(1), Choice::Negative(2)]
        .into_iter()
        .map(|choice| match choice {
            Choice::Positive(n) | Choice::Negative(n) => n,
        })
        .sum();
    assert_eq!(sum, 3);
    println!(
        "AstNode={} ItemKind={} Item={} OwnerKind={} Owner={} SourceCore={} Package={} Resolution={} Declarations={} Selection={} Evidence={} GraphNode={} Frame={} Trial={} Query={} OldSelected={} ProposedSelected={}",
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::syntax::item::ItemKind>(),
        size_of::<crate::syntax::item::Item>(),
        size_of::<crate::sema::associated::OwnerKind>(),
        size_of::<crate::sema::associated::Owner>(),
        size_of::<crate::resolve::bindings::SourceCore>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<Declarations>(),
        size_of::<crate::sema::traits::selection::Selection>(),
        size_of::<Evidence>(),
        size_of::<Node>(),
        size_of::<Frame>(),
        size_of::<Trial>(),
        size_of::<Query>(),
        size_of::<Option<usize>>(),
        size_of::<Option<Choice>>()
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::syntax::item::ItemKind>(), 28);
    assert_eq!(size_of::<crate::syntax::item::Item>(), 64);
    assert_eq!(size_of::<crate::sema::associated::OwnerKind>(), 20);
    assert_eq!(size_of::<crate::sema::associated::Owner>(), 32);
    assert_eq!(size_of::<crate::resolve::bindings::SourceCore>(), 72);
    assert_eq!(size_of::<Option<usize>>(), size_of::<Option<Choice>>());
    assert_eq!(
        size_of::<Option<usize>>(),
        size_of::<Option<frames::Chosen>>()
    );
    assert_eq!(size_of::<crate::sema::traits::selection::Selection>(), 136);
    assert_eq!(size_of::<Node>(), 160);
    assert_eq!(size_of::<Frame>(), 1312);
}
