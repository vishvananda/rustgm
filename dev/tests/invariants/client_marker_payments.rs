//! A source client pays each imported auto marker's application from its
//! installed header budget, exactly the work and bytes that the observed
//! interning attempted, and a repeated application pays only its lookup.
use super::{AutoTrait, Bound, Declarations, bound, imported_application, role};
use crate::{
    lower::instances::SourcePublication,
    resolve::body_origin::DeclarationOrigin,
    sema::crates::Linkage,
    support::crates,
    types::{GenericArgs, ItemId, Primitive, TypeId, arguments::Argument, traits::Application},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const MARKERS: [AutoTrait; 5] = [
    AutoTrait::Freeze,
    AutoTrait::Send,
    AutoTrait::Sync,
    AutoTrait::Unpin,
    AutoTrait::UnsafeUnpin,
];
const SPEC: &str = "pa29/tests/spec/auto-declarations";
/// The installed header budget's work and retained bytes.
fn budget(facts: &Declarations) -> Result<(usize, usize)> {
    let (work, bytes, _, _) = facts
        .crates
        .as_deref()
        .and_then(Linkage::original_work)
        .ok_or("installed header budget")?;
    Ok((work, bytes))
}
/// The `answer` client's completed analysis over the honest packet.
fn client() -> Result<Declarations> {
    let provider = crate::test_source::fixture(&format!("{SPEC}/provider.rs"))?;
    let client = crate::test_source::fixture(&format!("{SPEC}/answer.rs"))?;
    let interface = crate::module_packets::exported_profile(
        &provider,
        SourcePublication::ModuleBodies,
        "auto_markers",
    )?;
    let interface = crates::decode(&crates::encode(&interface)?)?;
    let (_, analysis, _) = crate::source_packets::client_analysis(interface, &client)?;
    Ok(analysis.facts)
}
/// The lookup, then the row's arguments and growth when it is absent.
fn attempted(facts: &Declarations, application: Application) -> Result<(usize, usize)> {
    if facts.types.find_trait_application(application).is_some() {
        return Ok((1, 0));
    }
    let count = facts
        .types
        .arguments(application.arguments)
        .ok_or("application arguments")?
        .len();
    let growth = facts.types.trait_application_growth();
    Ok((1 + count + growth.work, growth.bytes))
}
#[test]
fn imported_marker_bounds_pay_the_header_budget_once_per_marker() -> Result {
    let mut facts = client()?;
    let mut first: Vec<Bound> = Vec::new();
    let mut interned = 0;
    for round in 0..3 {
        for (index, marker) in MARKERS.into_iter().enumerate() {
            let declaration = facts
                .source_bindings
                .as_deref()
                .ok_or("source client catalog")?
                .auto_marker(role(marker).ok_or("marker role")?, true)?
                .ok_or("imported marker")?;
            assert!(
                matches!(declaration.origin, DeclarationOrigin::Imported(_)),
                "{marker:?}"
            );
            let application = Application {
                item: ItemId(declaration.definition.0),
                arguments: GenericArgs::EMPTY,
            };
            let present = facts.types.find_trait_application(application).is_some();
            let expected = attempted(&facts, application)?;
            let before = budget(&facts)?;
            let charged = bound(&mut facts, marker)?;
            let after = budget(&facts)?;
            assert_eq!(
                (after.0 - before.0, after.1 - before.1),
                expected,
                "{marker:?} in round {round}"
            );
            assert!(facts.types.find_trait_application(application).is_some());
            if round == 0 {
                interned += usize::from(!present);
                first.push(charged);
            } else {
                assert!(present, "{marker:?} was interned twice");
                assert_eq!(charged, first[index], "{marker:?}");
            }
        }
    }
    // Each marker's own trait header already interned its zero-argument
    // application as its own-trait premise, so a client's marker bounds only
    // look their rows up: none of the at most five rows is interned here.
    assert_eq!(interned, 0);
    Ok(())
}
/// The growth branch, which an honest client's marker bounds never reach:
/// an absent row pays its arguments and growth once, then only its lookup.
#[test]
fn imported_applications_pay_their_attempted_growth_once() -> Result {
    let mut facts = client()?;
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source client catalog")?
        .auto_marker(role(AutoTrait::Send).ok_or("marker role")?, true)?
        .ok_or("imported Send")?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U8))])?;
    let application = Application {
        item: ItemId(declaration.definition.0),
        arguments,
    };
    assert!(facts.types.find_trait_application(application).is_none());
    let mut first = None;
    for round in 0..3 {
        let expected = attempted(&facts, application)?;
        assert_eq!(expected.0 == 1, round > 0, "round {round}");
        let before = budget(&facts)?;
        let interned = imported_application(&mut facts, application)?;
        let after = budget(&facts)?;
        assert_eq!(
            (after.0 - before.0, after.1 - before.1),
            expected,
            "round {round}"
        );
        assert_eq!(*first.get_or_insert(interned), interned);
    }
    Ok(())
}
