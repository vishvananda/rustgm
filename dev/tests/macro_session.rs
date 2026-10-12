use rustgm::{
    expand::{
        config::Config,
        macros::{prepare, session, transcribe::Piece},
    },
    syntax::token_tree::Forest,
};
#[test]
fn textual_shadowing_restores_outer_definition_identity() -> Result<(), String> {
    let source =
        "macro_rules! m {(outer)=>{}} fn f(){macro_rules! m {(inner)=>{}} m!(inner);} m!(outer);";
    let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(report.definitions.len(), 2);
    assert_eq!(report.invocations[0].definition, 1);
    assert_eq!(report.invocations[1].definition, 0);
    assert!(report.invocations.iter().all(|i| i.prepared.is_none()));
    Ok(())
}
#[test]
fn docs_create_attribute_tokens_with_a_mapping_to_the_original_comment() -> Result<(), String> {
    let source = "/** a \"## b */";
    let forest = Forest::parse(source).map_err(|e| format!("{e:?}"))?;
    let prepared = prepare::if_needed(source, &forest)
        .map_err(|e| format!("{e:?}"))?
        .ok_or("expected a documentation spelling image")?;
    assert_eq!(prepared.source, "# [ doc = r###\" a \"## b \"### ] ");
    assert_eq!(prepared.origins.len(), 6);
    assert!(
        prepared
            .origins
            .iter()
            .all(|o| o.original.start == 0 && o.original.end as usize == source.len())
    );
    assert!(
        prepared
            .origins
            .iter()
            .all(|o| prepared.original_offset(o.generated.start) == 0)
    );
    let source = "macro_rules! m {(#[doc=$s:literal])=>{const S:&str=$s;}} m!{/// hello\n}";
    let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert!(report.invocations[0].prepared.is_some());
    assert_eq!(report.invocations[0].matched.captures.parser_calls, 1);
    assert!(
        report.invocations[0]
            .transcription
            .pieces
            .iter()
            .any(|p| matches!(p, Piece::Opaque(_)))
    );
    Ok(())
}
#[test]
fn disabled_definitions_do_not_compile_or_publish_matchers() -> Result<(), String> {
    let source = "#[cfg(any())] macro_rules! m {($x:expr bad)=>{}} #[cfg(any())] fn f(){m!();}";
    let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert!(report.definitions.is_empty());
    assert!(report.invocations.is_empty());
    Ok(())
}
