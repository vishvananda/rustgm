use rustgm::{
    expand::config::Config,
    resolve::{
        self, expansion,
        load::{InputFile, Sources},
        model::{DefId, DefKind, Namespace, Target},
    },
    support::source::SourceFile,
};
use std::path::Path;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn analyze(source: &str) -> Result<expansion::Report, String> {
    let expanded = expansion::analyze(
        InputFile {
            identity: "/crate/lib.rs".into(),
            path: "/crate/lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .map_err(|e| format!("{e:?}"))?;
    resolve::analyze_expanded(expanded).map_err(|e| format!("{e:?}"))
}
fn definitions(report: &expansion::Report, name: &str, kind: DefKind) -> Vec<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, d)| report.package.names.get(d.name) == name && d.kind == kind)
        .map(|(index, _)| DefId(u32::try_from(index).unwrap_or(u32::MAX)))
        .collect()
}
fn references(report: &expansion::Report, name: &str, namespace: Namespace) -> Vec<DefId> {
    report
        .resolution
        .references
        .iter()
        .filter(|r| r.namespace == namespace)
        .filter_map(|r| {
            if let Target::Definition(id) = r.target {
                Some(id)
            } else {
                None
            }
        })
        .filter(|id| {
            report
                .package
                .names
                .get(report.resolution.definitions[id.0 as usize].name)
                == name
        })
        .collect()
}

#[test]
fn captured_pattern_alternatives_keep_the_captured_binding_identity() -> Result<(), String> {
    let report = analyze(
        "macro_rules! select{($p:pat,$x:ident)=>{match(0,1){$p=>$x,_=>0}}}fn f()->i32{select!((x,0)|(0,x),x)}",
    )?;
    let locals = definitions(&report, "x", DefKind::Local);
    assert_eq!(locals.len(), 1);
    assert_eq!(references(&report, "x", Namespace::Value), locals);
    assert_eq!(report.resolution.pattern_aliases.len(), 1);
    assert_eq!(report.resolution.pattern_aliases[0].1, locals[0]);
    let source = "macro_rules! p{($x:ident)=>{(x,0)|(0,$x)}}fn f(){match(0,0){p!(x)=>{},_=>{}}}";
    assert!(
        analyze(source).is_err(),
        "different hygiene identities cannot be joined by spelling"
    );
    Ok(())
}

#[test]
fn literal_macro_names_keep_definition_site_binding_versions() -> Result<(), String> {
    let report = analyze("fn f()->i32{let x=1;macro_rules! m {()=>{x}} let x=2;m!()}")?;
    let locals = definitions(&report, "x", DefKind::Local);
    assert_eq!(locals.len(), 2);
    assert_eq!(references(&report, "x", Namespace::Value), [locals[0]]);
    let report = analyze("fn f()->i32{let x=1;macro_rules! m {($v:ident)=>{$v}} let x=2;m!(x)}")?;
    let locals = definitions(&report, "x", DefKind::Local);
    assert_eq!(references(&report, "x", Namespace::Value), [locals[1]]);
    Ok(())
}

#[test]
fn generated_locals_are_private_but_captured_binders_can_shadow() -> Result<(), String> {
    for (source, selected) in [
        (
            "fn f()->i32{let x=1;macro_rules! m {()=>{let x=2;}} m!();x}",
            0,
        ),
        (
            "fn f()->i32{let x=1;macro_rules! m {($v:ident)=>{let $v=2;}} m!(x);x}",
            1,
        ),
        (
            "fn f()->i32{let x=1;macro_rules! m {()=>{{let x=2;x}}} m!()}",
            1,
        ),
        (
            "fn f()->i32{let x=1;macro_rules! m {($v:expr)=>{{let x=2;$v}}} m!(x)}",
            0,
        ),
    ] {
        let report = analyze(source)?;
        let locals = definitions(&report, "x", DefKind::Local);
        assert_eq!(
            references(&report, "x", Namespace::Value),
            [locals[selected]],
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn nested_generated_definitions_retain_their_own_local_instances() -> Result<(), String> {
    let report = analyze(
        "fn f()->i32{macro_rules! make {($get:ident,$v:expr)=>{let x=$v;macro_rules! $get {()=>{x}}}} make!(first,1);make!(second,2);first!()+second!()}",
    )?;
    let locals = definitions(&report, "x", DefKind::Local);
    assert_eq!(locals.len(), 2);
    assert_eq!(references(&report, "x", Namespace::Value), locals);
    Ok(())
}

#[test]
fn item_lookup_uses_the_call_site_and_nearer_item_scopes() -> Result<(), String> {
    let report = analyze(
        "mod a {fn value()->i32{1}macro_rules! m {()=>{value()}}pub(crate) use m;}fn value()->i32{2}fn f()->i32{a::m!()}",
    )?;
    let functions = definitions(&report, "value", DefKind::Function);
    assert_eq!(functions.len(), 2);
    let target = references(&report, "value", Namespace::Value)[0];
    let definition = &report.resolution.definitions[target.0 as usize];
    assert_eq!(
        report.resolution.scopes[definition.scope.0 as usize]
            .module
            .0,
        0
    );
    let report = analyze("fn f()->i32{let x=1;macro_rules! m {()=>{x}} {const x:i32=2;m!()}}")?;
    assert_eq!(
        references(&report, "x", Namespace::Value),
        definitions(&report, "x", DefKind::Const)
    );
    Ok(())
}

#[test]
fn labels_keep_definition_or_capture_contexts() -> Result<(), String> {
    for (source, selected) in [
        (
            "fn f(){'outer:loop {macro_rules! m {()=>{break 'outer}} 'outer:loop {m!();}}}",
            0,
        ),
        (
            "fn f(){'outer:loop {macro_rules! m {($l:lifetime)=>{break $l}} 'outer:loop {m!('outer);}}}",
            1,
        ),
    ] {
        let report = analyze(source)?;
        let labels = definitions(&report, "outer", DefKind::Label);
        assert_eq!(
            references(&report, "outer", Namespace::Label),
            [labels[selected]]
        );
    }
    Ok(())
}

#[test]
fn definition_site_and_function_barriers_reject_inaccessible_locals() {
    for source in [
        "fn f(){macro_rules! m {()=>{x}} let x=2;let _=m!();}",
        "fn f(){let x=1;macro_rules! m {()=>{x}} fn other()->i32{m!()}}",
        "fn f(){macro_rules! m {($e:expr)=>{'hidden:loop{$e;}}}m!(break 'hidden);}",
        "fn f(){{let x=7;#[macro_export]macro_rules! m {()=>{x}}}let _=crate::m!();}",
        "fn f(){macro_rules! m {()=>{let x=1;}}m!();let _=x;}",
    ] {
        assert!(analyze(source).is_err(), "{source}");
    }
}

#[test]
fn distinct_hygiene_marks_allow_same_spelling_in_one_pattern_or_parameter_list()
-> Result<(), String> {
    let report = analyze("fn f()->i32{macro_rules! m {($x:ident)=>{let (x,$x)=(1,2);}}m!(x);x}")?;
    let locals = definitions(&report, "x", DefKind::Local);
    assert_eq!(locals.len(), 2);
    assert_eq!(references(&report, "x", Namespace::Value), [locals[1]]);
    let report = analyze("macro_rules! m {($x:ident)=>{fn f(x:i32,$x:i32)->i32{x+$x}}}m!(x);")?;
    let parameters = definitions(&report, "x", DefKind::Parameter);
    assert_eq!(parameters.len(), 2);
    assert_eq!(references(&report, "x", Namespace::Value), parameters);
    Ok(())
}

#[test]
fn lifetime_receiver_uses_the_self_identifier_context() -> Result<(), String> {
    let report = analyze(
        "struct S;macro_rules! m {($self:ident)=>{fn f<'a>(&'a $self)->&'a S{$self}}}impl S{m!(self);}",
    )?;
    let receivers = definitions(&report, "self", DefKind::Parameter);
    assert_eq!(receivers.len(), 1);
    assert_eq!(references(&report, "self", Namespace::Value), receivers);
    Ok(())
}

#[test]
fn resolving_an_expanded_report_again_preserves_fact_identities() -> Result<(), String> {
    let report = analyze("macro_rules! m {()=>{}}use m as alias;alias!();")?;
    let count = report.resolution.definitions.len();
    let report = resolve::analyze_expanded(report).map_err(|e| format!("{e:?}"))?;
    assert_eq!(report.resolution.definitions.len(), count);
    Ok(())
}

#[test]
fn long_shadow_histories_preserve_an_early_macro_snapshot() -> Result<(), String> {
    let mut source = String::from("fn f(){let x=1;macro_rules! m {()=>{x}}");
    for _ in 0..2000 {
        source.push_str("let x=2;");
    }
    for _ in 0..2000 {
        source.push_str("m!();");
    }
    source.push('}');
    let report = analyze(&source)?;
    let locals = definitions(&report, "x", DefKind::Local);
    assert_eq!(locals.len(), 2001);
    assert_eq!(
        references(&report, "x", Namespace::Value),
        vec![locals[0]; 2000]
    );
    Ok(())
}
