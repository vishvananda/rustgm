//! Only the selected checked syntax joins the existing source image atomically.
use super::*;
use crate::{
    expand::{builtin::choice, config::Config, macros::input::Input, registry::Registry},
    support::{names::Names, source::SourceFile},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn pending(source: &str) -> Result<(Unit, choice::Site, Input)> {
    let unit = Unit::parse(
        SourceFile::new(source.as_bytes().to_vec())?,
        &Config::default(),
        &mut Names::default(),
    )
    .map_err(|error| error.message)?;
    let (index, node, capture, grammar) = unit
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| match node.kind {
            Kind::Macro {
                capture, context, ..
            } => Some((index, *node, capture, context)),
            _ => None,
        })
        .ok_or("missing actual pending call")?;
    let forest = &unit.configured.ast.captures[capture as usize];
    let tokens = Input::from_forest(forest, forest.children(0).ok_or("missing input delimiter")?)
        .map_err(|error| error.message)?;
    let site = choice::Site {
        file: super::super::FileId(0),
        target: NodeId::new(u32::try_from(index)?),
        original: node.span,
        grammar,
    };
    Ok((unit, site, tokens))
}
fn state(unit: &Unit) -> (usize, usize, usize, usize, Vec<bool>) {
    (
        unit.configured.ast.nodes.len(),
        unit.configured.ast.edges.len(),
        unit.configured.effects.len(),
        unit.literals.decoded_bytes(),
        unit.configured.active.clone(),
    )
}
#[test]
fn malformed_or_unpaid_choice_never_changes_the_actual_unit() -> Result {
    for source in [
        "choice!{all()=>{fn valid(){}} _=>{fn broken(,){}}}",
        "choice!{any()=>{fn unused(){}}}",
    ] {
        let (mut unit, site, input) = pending(source)?;
        let before = state(&unit);
        assert!(
            unit.install_choice(
                site,
                &input,
                &Registry::default(),
                |_| None,
                &Config::default(),
                &|_, _| Ok(())
            )
            .is_err()
        );
        assert_eq!(state(&unit), before);
        assert_eq!(unit.source.text(), source);
        assert!(unit.origins.is_empty());
    }
    let source = "choice!{all()=>{fn selected(){}}}";
    let (mut unit, site, input) = pending(source)?;
    let before = state(&unit);
    assert!(
        unit.install_choice(
            site,
            &input,
            &Registry::default(),
            |_| None,
            &Config::default(),
            &|_, _| Err(error(0, "unpaid choice work"))
        )
        .is_err()
    );
    assert_eq!(state(&unit), before);
    let wrong = choice::Site {
        grammar: crate::syntax::ast::MacroContext::Expression,
        ..site
    };
    assert!(
        unit.install_choice(
            wrong,
            &input,
            &Registry::default(),
            |_| None,
            &Config::default(),
            &|_, _| Ok(())
        )
        .is_err()
    );
    assert_eq!(state(&unit), before);
    Ok(())
}
#[test]
fn only_selected_configuration_is_committed_with_original_source_and_origins() -> Result {
    // The inactive cfg is syntactically valid but invalid to evaluate. It must
    // never enter selected configuration, while its Rust item syntax is parsed.
    let source = "choice!{all()=>{fn selected(){}} _=>{#[cfg(not())] fn hidden(){}}}";
    let (mut unit, site, input) = pending(source)?;
    let installed = unit
        .install_choice(
            site,
            &input,
            &Registry::default(),
            |_| None,
            &Config::default(),
            &|_, _| Ok(()),
        )
        .map_err(|error| error.message)?;
    assert_eq!(installed.parser_calls, 2);
    assert_eq!(unit.source.text(), source);
    assert_eq!(unit.source.physical_len(), source.len());
    assert!(unit.origins.is_empty());
    assert!(!matches!(
        unit.configured.ast.node(site.target).kind,
        Kind::Macro { .. }
    ));
    let names = unit
        .configured
        .ast
        .items
        .iter()
        .filter_map(|item| {
            item.name
                .map(|span| &source[span.start as usize..span.end as usize])
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["selected"]);
    let before = state(&unit);
    assert!(
        unit.install_choice(
            site,
            &input,
            &Registry::default(),
            |_| None,
            &Config::default(),
            &|_, _| Ok(())
        )
        .is_err()
    );
    assert_eq!(state(&unit), before);
    Ok(())
}

#[test]
fn prepared_choice_keeps_owned_source_call_identity_and_publishes_nothing_before_gate() -> Result {
    let source = "choice!{all()=>{fn selected(){let _=\"retained\";}} _=>{fn hidden(){}}}";
    let (mut unit, site, input) = pending(source)?;
    let before = state(&unit);
    let prepared = unit
        .prepare_choice(
            site,
            &input,
            &Registry::default(),
            |_| None,
            &Config::default(),
            &|_, _| Ok(()),
        )
        .map_err(|e| e.message)?;
    assert_eq!(state(&unit), before);
    assert_eq!(unit.source.text(), source);
    let retained = std::sync::Arc::clone(unit.source.original_image());
    unit.source = SourceFile::new(source.as_bytes().to_vec())?;
    assert!(!std::sync::Arc::ptr_eq(
        &retained,
        unit.source.original_image()
    ));
    assert!(unit.install_prepared_choice(prepared).is_err());
    assert_eq!(state(&unit), before);
    let prepared = unit
        .prepare_choice(
            site,
            &input,
            &Registry::default(),
            |_| None,
            &Config::default(),
            &|_, _| Ok(()),
        )
        .map_err(|e| e.message)?;
    let (mut foreign, _, _) = pending(source)?;
    let foreign_before = state(&foreign);
    assert!(foreign.install_prepared_choice(prepared).is_err());
    assert_eq!(state(&foreign), foreign_before);
    let (mut gated, site, input) =
        pending("choice!{ub_checks=>{fn enabled(){}} _=>{fn disabled(){}}}")?;
    let before = state(&gated);
    let prepared = gated
        .prepare_choice(
            site,
            &input,
            &Registry::default(),
            |_| None,
            &Config::default(),
            &|_, _| Ok(()),
        )
        .map_err(|e| e.message)?;
    assert_eq!(prepared.cfg_uses().len(), 1);
    drop(prepared); // A failed Engine gate consumes no Unit publication.
    assert_eq!(state(&gated), before);
    assert!(
        gated
            .install_choice(
                site,
                &input,
                &Registry::default(),
                |_| None,
                &Config::default(),
                &|_, _| Ok(())
            )
            .is_err()
    );
    assert_eq!(state(&gated), before);
    Ok(())
}

#[test]
fn selected_false_owner_uses_graft_to_actual_effects_and_fresh_instances_keep_only_originals()
-> Result {
    let source = "#[cfg(ub_checks)]fn original_hidden(){}choice!{all()=>{#[cfg(ub_checks)]fn selected_hidden(){}}}";
    let (mut unit, site, input) = pending(source)?;
    let original = unit
        .configured
        .cfg_uses
        .as_ref()
        .ok_or("original false-owner demand")?
        .as_slice()[0];
    let before_effects = unit.configured.effects.len();
    unit.install_choice(
        site,
        &input,
        &Registry::default(),
        |_| None,
        &Config::default(),
        &|_, _| Ok(()),
    )
    .map_err(|e| e.message)?;
    let uses = unit.configured.cfg_uses.as_ref().ok_or("grafted demands")?;
    assert_eq!(uses.len(), 2);
    assert_eq!(uses.as_slice()[0].effect, original.effect);
    assert!((uses.as_slice()[1].effect as usize) >= before_effects);
    let effect = &unit.configured.effects[uses.as_slice()[1].effect as usize];
    assert!(matches!(
        effect.attribute.effect,
        crate::expand::configured::Effect::Cfg(false)
    ));
    assert_eq!(
        unit.source.text().get(
            uses.as_slice()[1].predicate.span.start as usize
                ..uses.as_slice()[1].predicate.span.end as usize
        ),
        Some("ub_checks")
    );
    let fresh = unit.fresh_instance().map_err(|e| e.message)?;
    let copied = fresh
        .configured
        .cfg_uses
        .as_ref()
        .ok_or("retained original demand")?;
    assert_eq!(copied.len(), 1);
    assert_eq!(copied.as_slice()[0].effect, original.effect);
    assert_eq!(copied.as_slice()[0].predicate.span, original.predicate.span);
    assert!(matches!(
        fresh.configured.ast.node(site.target).kind,
        Kind::Macro { .. }
    ));
    Ok(())
}
