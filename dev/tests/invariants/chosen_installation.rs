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
