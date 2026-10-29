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
    // The cold provider adds 112 bytes of macro-contract state to its 72-byte base.
    assert_eq!(size_of::<crate::resolve::bindings::SourceCore>(), 184);
    assert_eq!(size_of::<Option<usize>>(), size_of::<Option<Choice>>());
    assert_eq!(
        size_of::<Option<usize>>(),
        size_of::<Option<frames::Chosen>>()
    );
    assert_eq!(size_of::<crate::sema::traits::selection::Selection>(), 136);
    assert_eq!(size_of::<Node>(), 160);
    assert_eq!(size_of::<Frame>(), 1312);
}

#[test]
fn nominal_formation_uses_only_the_existing_cold_owner() {
    #[allow(dead_code)]
    struct Before {
        application: crate::types::projection::Application,
        requirements: crate::sema::projections::families::Requirements,
        parent: Option<frames::Chosen>,
        cursor: usize,
        children: Vec<usize>,
        universes: crate::support::hash::HashMap<u32, crate::types::lifetime_proof::ScopeId>,
        stopped: Option<Decision>,
        registered: bool,
        finished: bool,
    }
    println!(
        "FormationBefore={} FormationAfter={} FormationKey={} FormationEntry={}",
        size_of::<Before>(),
        size_of::<member_conditions::Formation>(),
        size_of::<member_conditions::Key>(),
        size_of::<member_conditions::Entry>()
    );
    assert!(size_of::<member_conditions::Formation>() <= size_of::<Before>() + 32);
    assert_eq!(size_of::<Frame>(), 1312);
    assert_eq!(size_of::<Node>(), 160);
}
