use rustgm::support::hash::HashMap;
use rustgm::{
    expand::config::Config,
    resolve::{
        self, expansion,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Files {
    sources: HashMap<PathBuf, String>,
    reads: Vec<PathBuf>,
    identities: HashMap<PathBuf, PathBuf>,
    resources: HashMap<PathBuf, Vec<u8>>,
    resource_reads: Vec<PathBuf>,
    environment: HashMap<String, Result<Option<String>, String>>,
    environment_reads: Vec<String>,
}
impl Sources for Files {
    fn resource(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        self.resource_reads.push(path.to_owned());
        Ok(self.resources.get(path).cloned())
    }
    fn environment(&mut self, name: &str) -> Result<Option<String>, String> {
        self.environment_reads.push(name.to_owned());
        self.environment.get(name).cloned().unwrap_or(Ok(None))
    }

    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        self.reads.push(path.to_owned());
        self.sources
            .get(path)
            .map(|text| {
                Ok(InputFile {
                    identity: self
                        .identities
                        .get(path)
                        .cloned()
                        .unwrap_or_else(|| path.to_owned()),
                    path: path.to_owned(),
                    source: SourceFile::new(text.as_bytes().to_vec())?,
                })
            })
            .transpose()
    }
}
fn analyze(source: &str, files: &mut Files) -> Result<expansion::Report, String> {
    expansion::analyze(
        InputFile {
            identity: "/crate/lib.rs".into(),
            path: "/crate/lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        files,
        &Config::default(),
    )
    .map_err(|e| format!("{e:?}"))
}

#[test]
fn actual_builtin_and_declarative_invocations_share_one_ancestry_index() -> Result<(), String> {
    use rustgm::expand::marks::ExpansionId;
    let source = "macro_rules! inner {()=>{line!()}} macro_rules! outer {()=>{inner!()}} fn f()->u32{outer!()} fn g()->u32{outer!()} const X:&str=stringify!(other);";
    let report = analyze(source, &mut Files::default())?;
    assert_eq!(report.invocations.len(), 7);
    assert_eq!(
        report
            .invocations
            .iter()
            .filter(|call| call.rule.is_some())
            .count(),
        4
    );
    let mut parents = vec![ExpansionId(0)];
    for call in &report.invocations {
        assert_eq!(call.context.0 as usize, parents.len());
        assert!(call.parent.0 < call.context.0);
        parents.push(call.parent);
    }
    for ancestor in 0..=7 {
        for context in 0..=7 {
            let mut current = ExpansionId(context);
            while current != ExpansionId(ancestor) && current.0 != 0 {
                current = parents[current.0 as usize];
            }
            assert_eq!(
                report
                    .resolution
                    .introductions
                    .ancestor(ExpansionId(ancestor), ExpansionId(context)),
                current == ExpansionId(ancestor),
            );
        }
    }
    assert!(
        !report
            .resolution
            .introductions
            .ancestor(ExpansionId(8), ExpansionId(8))
    );
    Ok(())
}

#[test]
fn whole_crate_worklist_expands_generated_definitions_and_resolves_their_items()
-> Result<(), String> {
    let source = "macro_rules! make {()=>{macro_rules! value {()=>{42}} fn answer()->i32{value!()}}} make!(); fn entry()->i32{answer()}";
    let mut report = analyze(source, &mut Files::default())?;
    assert_eq!(report.invocations.len(), 2);
    assert_eq!(report.definitions.len(), 2);
    let facts = resolve::analyze(&mut report.package).map_err(|e| format!("{e:?}"))?;
    assert!(facts.expansions.is_empty());
    assert!(
        facts
            .definitions
            .iter()
            .any(|d| report.package.names.get(d.name) == "answer")
    );
    assert_eq!(report.package.units[0].source.original(), source);
    Ok(())
}

#[test]
fn whole_crate_nested_captures_are_parsed_once_and_scopes_are_restored() -> Result<(), String> {
    let source = "macro_rules! inner {($x:expr)=>{$x*3}} macro_rules! outer {($x:expr)=>{inner!($x)}} fn f()->i32{outer!(1+2)}";
    let mut report = analyze(source, &mut Files::default())?;
    assert_eq!(report.invocations.len(), 2);
    assert_eq!(
        report
            .invocations
            .iter()
            .map(|i| i.parser_calls)
            .collect::<Vec<_>>(),
        [1, 0]
    );
    assert_eq!(report.captures.len(), 1);
    resolve::analyze(&mut report.package).map_err(|e| format!("{e:?}"))?;
    let source = "macro_rules! m {()=>{1}} fn f(){m!();{macro_rules! m {()=>{2}}m!();}m!();}";
    let report = analyze(source, &mut Files::default())?;
    assert_eq!(
        report
            .invocations
            .iter()
            .map(|i| i.definition)
            .collect::<Vec<_>>(),
        [0, 1, 0]
    );
    Ok(())
}

#[test]
fn generated_file_modules_use_the_declaring_directory_and_textual_scope() -> Result<(), String> {
    let source = "macro_rules! value {()=>{42}} macro_rules! make {()=>{mod child;}} make!(); fn f()->i32{child::answer()}";
    let mut files = Files::default();
    files.sources.insert(
        "/crate/child.rs".into(),
        "pub fn answer()->i32{value!()}".into(),
    );
    let mut report = analyze(source, &mut files)?;
    assert_eq!(report.package.modules.len(), 2);
    assert_eq!(report.package.units.len(), 2);
    assert_eq!(
        files.reads,
        [
            PathBuf::from("/crate/child.rs"),
            PathBuf::from("/crate/child/mod.rs")
        ]
    );
    assert_eq!(report.invocations.len(), 2);
    resolve::analyze(&mut report.package).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

#[test]
fn repeated_physical_modules_expand_from_original_typed_facts() -> Result<(), String> {
    use rustgm::literal::Value;
    let source = "macro_rules! value {()=>{1}} #[path=\"shared.rs\"] mod first; macro_rules! value {()=>{2}} #[path=\"shared.rs\"] mod second;";
    let shared = "#[cfg(any())] fn hidden(){missing!();}\r\npub fn answer()->i32{let _=\"base\\n\";value!()}\r\n";
    let mut files = Files::default();
    files
        .sources
        .insert("/crate/shared.rs".into(), shared.into());
    let report = analyze(source, &mut files)?;
    assert_eq!(
        report
            .invocations
            .iter()
            .map(|i| i.definition)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(report.package.units.len(), 3);
    assert_ne!(
        report.package.modules[1].file,
        report.package.modules[2].file
    );
    for (index, expected) in [(1, 1), (2, 2)] {
        let unit = &report.package.units[index];
        assert_eq!(unit.source.original(), shared);
        assert!(!unit.source.text().contains('\r'));
        assert_eq!(unit.literals.decoded_bytes(), 5);
        assert!(!unit.configured.owners[0].enabled);
        let Value::Integer(value) = unit.literal(1).map_err(|e| format!("{e:?}"))? else {
            return Err("missing instance-specific integer".into());
        };
        assert_eq!(value.magnitude, expected);
        let original = unit.fresh_instance().map_err(|e| format!("{e:?}"))?;
        assert_eq!(original.source.text(), shared.replace("\r\n", "\n"));
        assert_eq!(original.configured.ast.literals.len(), 1);
        assert_eq!(original.literals.decoded_bytes(), 5);
        assert!(original.origins.is_empty());
        assert!(
            original
                .configured
                .ast
                .nodes
                .iter()
                .any(|n| matches!(n.kind, rustgm::syntax::ast::Kind::Macro { .. }))
        );
    }
    Ok(())
}

#[test]
fn disabled_generated_syntax_never_demands_expansion_literals_or_files() -> Result<(), String> {
    let source = "macro_rules! make {()=>{#[cfg(any())] mod absent; #[cfg(any())] fn hidden(){unknown!();1bad;} #[cfg(any())] macro_rules! invalid{bad}}} make!();";
    let mut files = Files::default();
    let report = analyze(source, &mut files)?;
    assert!(files.reads.is_empty());
    assert_eq!(report.invocations.len(), 1);
    assert_eq!(report.definitions.len(), 1);
    Ok(())
}

#[test]
fn generated_documentation_attributes_and_empty_visibilities_remain_typed() -> Result<(), String> {
    let source =
        "macro_rules! make {($v:vis,$a:meta)=>{#[$a] $v fn answer(){}}} make!(,doc=\"hello\");";
    let mut report = analyze(source, &mut Files::default())?;
    assert_eq!(report.captures.len(), 2);
    resolve::analyze(&mut report.package).map_err(|e| format!("{e:?}"))?;
    let source = "macro_rules! make {($i:item)=>{$i}} make!(/// generated\nfn answer(){});";
    let mut report = analyze(source, &mut Files::default())?;
    assert_eq!(report.invocations.len(), 1);
    resolve::analyze(&mut report.package).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

#[test]
fn recursive_or_out_of_scope_macros_fail_without_stack_overflow() {
    for (source, reason) in [
        (
            "macro_rules! m {()=>{m!()}} fn f(){m!();}",
            "recursion exceeds 128",
        ),
        (
            "fn f(){{macro_rules! m {()=>{1}}}m!();}",
            "not in textual scope",
        ),
        ("m!();macro_rules! m {()=>{}}", "not in textual scope"),
        (
            "macro_rules! m {()=>{1 2}} fn f(){let x=m!();}",
            "unexpected token after macro expansion",
        ),
    ] {
        assert!(
            analyze(source, &mut Files::default()).is_err_and(|e| e.contains(reason)),
            "{source}"
        );
    }
}

#[test]
fn exports_and_output_dependencies_allow_independent_calls_to_progress() -> Result<(), String> {
    for source in [
        "crate::m!();mod inside {#[macro_export] macro_rules! m {()=>{}}}",
        "m!();mod inside {#[macro_export] macro_rules! m {()=>{}}}",
        "late!();macro_rules! ready {()=>{#[macro_export] macro_rules! late {()=>{}}}} ready!();",
        "#[macro_use] mod inside {macro_rules! m {()=>{}}} m!();",
        "macro_rules! make {()=>{macro_rules! m {()=>{}}}} make!();m!();",
        "macro_rules! m {()=>{}} macro_rules! make {()=>{macro_rules! m {()=>{}} m!();}} make!();",
        "macro_rules! m {()=>{}} macro_rules! make {()=>{macro_rules! m {()=>{}}}} make!();macro_rules! m {()=>{}} m!();",
    ] {
        analyze(source, &mut Files::default()).map_err(|e| format!("{source}: {e}"))?;
    }
    Ok(())
}

#[test]
fn generated_textual_ambiguity_and_module_scope_errors_match_rustc() {
    for source in [
        "macro_rules! m {()=>{}} macro_rules! make {()=>{macro_rules! m {()=>{}}}} make!();m!();",
        "macro_rules! first {()=>{macro_rules! m {()=>{}}}} macro_rules! second {()=>{macro_rules! m {()=>{}}}} first!();second!();m!();",
        "macro_rules! m {()=>{}} late!();m!();macro_rules! setup {()=>{#[macro_export] macro_rules! late {()=>{macro_rules! m {()=>{}}}}}} setup!();",
        "mod a {#[macro_export] macro_rules! m {()=>{}}} mod b {m!();}",
        "mod inside {macro_rules! m {()=>{}}} m!();",
        "#[macro_use(m)] mod inside {macro_rules! m {()=>{}}} m!();",
        "macro_rules! m {()=>{}} self::m!();",
    ] {
        assert!(analyze(source, &mut Files::default()).is_err(), "{source}");
    }
}

const PATH_IMPORTS_CASES: &[(&str, &str, bool)] = &[
    (
        "use_before_local",
        "use m as alias; macro_rules! m {()=>{}} alias!();",
        false,
    ),
    (
        "use_after_local",
        "macro_rules! m {()=>{}} use m as alias; alias!();",
        true,
    ),
    (
        "qualified_local",
        "macro_rules! m {()=>{}} self::m!();",
        false,
    ),
    (
        "qualified_reexport",
        "macro_rules! m {()=>{}} pub(crate) use m as alias; self::alias!();",
        true,
    ),
    (
        "qualified_original_import",
        "macro_rules! m {()=>{}} pub(crate) use m; self::m!();",
        true,
    ),
    (
        "export_forward",
        "crate::m!(); mod inside {#[macro_export] macro_rules! m {()=>{}}}",
        true,
    ),
    (
        "export_unqualified_forward",
        "m!(); mod inside {#[macro_export] macro_rules! m {()=>{}}}",
        true,
    ),
    ("local_forward", "m!(); macro_rules! m {()=>{}}", false),
    (
        "exported_later_expansion",
        "late!(); macro_rules! ready {()=>{#[macro_export] macro_rules! late {()=>{}}}} ready!();",
        true,
    ),
    (
        "pending_shadow",
        "macro_rules! m {()=>{compile_error!(\"wrong binding\");}} late!(); m!(); macro_rules! setup {()=>{#[macro_export] macro_rules! late {()=>{macro_rules! m {()=>{}}}}}} setup!();",
        false,
    ),
    (
        "import_forward_export",
        "use crate::m as alias; alias!(); mod inside {#[macro_export] macro_rules! m {()=>{}}}",
        true,
    ),
    (
        "import_module_alias",
        "use crate::inside as renamed; renamed::alias!(); mod inside {macro_rules! m {()=>{}} pub(crate) use m as alias;}",
        true,
    ),
    (
        "import_local_child",
        "macro_rules! m {()=>{}} mod inner {use m as alias; alias!();}",
        true,
    ),
    (
        "macro_use_module",
        "#[macro_use] mod inner {macro_rules! m {()=>{}}} m!();",
        true,
    ),
    (
        "macro_use_no_module",
        "mod inner {macro_rules! m {()=>{}}} m!();",
        false,
    ),
    (
        "macro_use_list_module",
        "#[macro_use(m)] mod inner {macro_rules! m {()=>{}}} m!();",
        false,
    ),
    (
        "private_reexport",
        "mod inner {macro_rules! m {()=>{}} use m as alias;} inner::alias!();",
        false,
    ),
    (
        "public_unexported",
        "macro_rules! m {()=>{}} pub use m as alias; alias!();",
        false,
    ),
    (
        "glob_ambiguity",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; use b::*; m!();",
        false,
    ),
    (
        "glob_same_target",
        "#[macro_export]macro_rules! m {()=>{}} mod a {pub use crate::m;} mod b {pub use crate::m;} mod c {use crate::a::*;use crate::b::*;m!();}",
        true,
    ),
];

#[test]
fn path_imports_match_pinned_rustc() {
    for &(name, source, success) in PATH_IMPORTS_CASES {
        let result = analyze(source, &mut Files::default()).and_then(|report| {
            resolve::analyze_expanded(report).map_err(|error| format!("{error:?}"))
        });
        assert_eq!(
            result.is_ok(),
            success,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}

const MODULE_BARRIERS_CASES: &[(&str, &str, bool)] = &[
    (
        "export_other_child",
        "mod a {#[macro_export] macro_rules! m {()=>{}}} mod b {m!();}",
        false,
    ),
    (
        "export_before_child",
        "mod b {m!();} mod a {#[macro_export] macro_rules! m {()=>{}}}",
        false,
    ),
    (
        "alias_parent_child",
        "macro_rules! m {()=>{}} use m as alias; mod b {alias!();}",
        false,
    ),
    (
        "alias_after_child",
        "macro_rules! m {()=>{}} mod b {alias!();} use m as alias;",
        false,
    ),
    (
        "export_bad_attr",
        "#[macro_export(unknown)] macro_rules! m {()=>{}}",
        false,
    ),
    ("macro_use_function", "#[macro_use] fn f(){}", true),
    (
        "use_late_generated",
        "use m as alias; macro_rules! make {()=>{macro_rules! m {()=>{}}}} make!();alias!();",
        false,
    ),
];

#[test]
fn module_barriers_match_pinned_rustc() {
    for &(name, source, success) in MODULE_BARRIERS_CASES {
        let result = analyze(source, &mut Files::default()).and_then(|report| {
            resolve::analyze_expanded(report).map_err(|error| format!("{error:?}"))
        });
        assert_eq!(
            result.is_ok(),
            success,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}

const GENERATED_IMPORTS_CASES: &[(&str, &str, bool)] = &[
    (
        "generated_strong_after_glob",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; m!(); macro_rules! setup {()=>{use b::m;}} setup!();",
        false,
    ),
    (
        "generated_ambiguous_after_glob",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; m!(); macro_rules! setup {()=>{use b::*;}} setup!();",
        false,
    ),
    (
        "generated_strong_selects_later",
        "mod a {macro_rules! m {()=>{compile_error!(\"old\");}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; m!(); macro_rules! setup {()=>{use b::m;}} setup!();",
        false,
    ),
    (
        "generated_import_through_glob",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} mod generator {macro_rules! setup {()=>{use crate::b::m;}} pub(crate) use setup;} use a::*; use generator::*; m!(); setup!();",
        false,
    ),
    (
        "generated_two_glob_calls",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} mod generator {macro_rules! setup {()=>{}} pub(crate) use setup;} use a::*; use generator::*; m!(); setup!();",
        true,
    ),
    (
        "generated_cycle_changes_target",
        "mod a {macro_rules! m {()=>{use crate::b::m;}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; m!();",
        false,
    ),
    (
        "import_generated_ambiguous",
        "macro_rules! m {()=>{}} macro_rules! make {()=>{macro_rules! m {()=>{}}}} make!();use m as alias;alias!();",
        false,
    ),
    (
        "root_alias",
        "use crate as root; root::m!(); #[macro_export] macro_rules! m {()=>{}}",
        true,
    ),
    (
        "block_alias",
        "macro_rules! m {()=>{}} fn f(){use m as alias; alias!();}",
        true,
    ),
    (
        "child_reexport_wider",
        "mod inner {macro_rules! m {()=>{}} pub(crate) use m as alias;} inner::alias!();",
        true,
    ),
];

#[test]
fn generated_imports_match_pinned_rustc() {
    for &(name, source, success) in GENERATED_IMPORTS_CASES {
        let result = analyze(source, &mut Files::default()).and_then(|report| {
            resolve::analyze_expanded(report).map_err(|error| format!("{error:?}"))
        });
        assert_eq!(
            result.is_ok(),
            success,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}

const PATH_BINDING_ANCESTRY_CASES: &[(&str, &str, bool)] = &[
    (
        "generated_strong_before_call",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; macro_rules! setup {()=>{use b::m;}} setup!();m!();",
        false,
    ),
    (
        "generated_strong_same_target",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; macro_rules! setup {()=>{use a::m;}} setup!();m!();",
        true,
    ),
    (
        "generated_strong_inside_expansion",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; macro_rules! setup {()=>{use b::m;m!();}} setup!();",
        false,
    ),
    (
        "generated_qualified_inside_expansion",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; macro_rules! setup {()=>{use b::m;self::m!();}} setup!();",
        false,
    ),
    (
        "generated_qualified_after_expansion",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; macro_rules! setup {()=>{use b::m;}} setup!();self::m!();",
        false,
    ),
    (
        "ordinary_strong_beats_glob",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; use b::m;m!();",
        true,
    ),
    (
        "generated_imports_same_expansion",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} macro_rules! setup {()=>{use a::*;use b::m;m!();}} setup!();",
        false,
    ),
    (
        "generated_glob_source_strong",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use b::m; macro_rules! setup {()=>{use a::*;}} setup!();m!();",
        true,
    ),
    (
        "generated_strong_unused",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; macro_rules! setup {()=>{use b::m;}} setup!();",
        true,
    ),
    (
        "generated_strong_alias",
        "mod a {macro_rules! m {()=>{}} pub(crate) use m;} mod b {macro_rules! m {()=>{}} pub(crate) use m;} use a::*; macro_rules! setup {()=>{use b::m;}} setup!();use m as alias;alias!();",
        false,
    ),
];

#[test]
fn path_binding_ancestry_match_pinned_rustc() {
    for &(name, source, success) in PATH_BINDING_ANCESTRY_CASES {
        let result = analyze(source, &mut Files::default()).and_then(|report| {
            resolve::analyze_expanded(report).map_err(|error| format!("{error:?}"))
        });
        assert_eq!(
            result.is_ok(),
            success,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}

const PATTERN_CONSUMER_CASES: &[(&str, &str, bool)] = &[
    (
        "expr_literal_pattern",
        "macro_rules! p {($e:expr)=>{match 1 {$e=>1,_=>0}}} fn f()->i32{p!(1)}",
        true,
    ),
    (
        "expr_negative_pattern",
        "macro_rules! p {($e:expr)=>{match 1 {$e=>1,_=>0}}} fn f()->i32{p!(-1)}",
        true,
    ),
    (
        "expr_const_path_pattern",
        "const C:i32=1;macro_rules! p {($e:expr)=>{match 1 {$e=>1,_=>0}}} fn f()->i32{p!(C)}",
        false,
    ),
    (
        "expr_binary_pattern",
        "macro_rules! p {($e:expr)=>{match 1 {$e=>1,_=>0}}} fn f()->i32{p!(1+2)}",
        false,
    ),
    (
        "expr_group_pattern",
        "macro_rules! p {($e:expr)=>{match 1 {$e=>1,_=>0}}} fn f()->i32{p!((1))}",
        false,
    ),
    (
        "expr_block_pattern",
        "macro_rules! p {($e:expr)=>{match 1 {$e=>1,_=>0}}} fn f()->i32{p!({1})}",
        false,
    ),
    (
        "expr_inline_const_pattern",
        "macro_rules! p {($e:expr)=>{match 1 {$e=>1,_=>0}}} fn f()->i32{p!(const{1})}",
        false,
    ),
    (
        "path_variant_pattern",
        "enum E{A(i32)}macro_rules! p {($p:path)=>{match E::A(1) {$p(x)=>x}}} fn f()->i32{p!(E::A)}",
        true,
    ),
    (
        "path_struct_pattern",
        "struct S{x:i32}macro_rules! p {($p:path)=>{match (S{x:1}) {$p{x}=>x}}} fn f()->i32{p!(S)}",
        true,
    ),
    (
        "path_struct_expression",
        "struct S{x:i32}macro_rules! p {($p:path)=>{$p{x:1}}} fn f()->S{p!(S)}",
        false,
    ),
    (
        "path_call_expression",
        "fn id(x:i32)->i32{x}macro_rules! p {($p:path)=>{$p(1)}}fn f()->i32{p!(id)}",
        true,
    ),
    (
        "path_suffix_expression",
        "mod m{pub fn f()->i32{1}} macro_rules! p {($p:path)=>{$p::f()}}fn f()->i32{p!(m)}",
        false,
    ),
    (
        "path_suffix_type",
        "mod m{pub struct T;}macro_rules! p {($p:path)=>{type T=$p::T;}}p!(m);",
        false,
    ),
    (
        "path_turbofish_expression",
        "fn id<T>(x:T)->T{x}macro_rules! p {($p:path)=>{$p::<i32>(1)}}fn f()->i32{p!(id)}",
        false,
    ),
    (
        "path_trait_bound",
        "trait T{}macro_rules! p {($p:path)=>{fn f<X:$p>(){}}}p!(T);",
        true,
    ),
    (
        "expr_range_pattern",
        "macro_rules! p {($a:expr,$b:expr)=>{match 1 {$a..=$b=>1,_=>0}}}fn f()->i32{p!(0,2)}",
        true,
    ),
    (
        "path_range_pattern",
        "const A:i32=0;const B:i32=2;macro_rules! p {($a:path,$b:path)=>{match 1 {$a..=$b=>1,_=>0}}}fn f()->i32{p!(A,B)}",
        true,
    ),
];

#[test]
fn opaque_pattern_and_path_consumers_match_pinned_rustc() {
    for &(name, source, success) in PATTERN_CONSUMER_CASES {
        let result = analyze(source, &mut Files::default());
        assert_eq!(
            result.is_ok(),
            success,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[test]
fn opaque_attribute_paths_are_consumed_without_source_reparsing() -> Result<(), String> {
    for source in [
        "macro_rules! p {($p:path)=>{#[$p(unused)] fn f(){}}}p!(allow);",
        "macro_rules! p {($p:ty)=>{#[$p(unused)] fn f(){}}}p!(allow);",
        "macro_rules! q {($m:meta)=>{#[$m] fn f(){}}}macro_rules! p {($p:path)=>{q!($p(unused));}}p!(allow);",
        "macro_rules! p {($p:path)=>{#[cfg($p)] fn f(){unknown!();}}}p!(unavailable);",
    ] {
        analyze(source, &mut Files::default()).map_err(|e| format!("{source}: {e}"))?;
    }
    Ok(())
}

#[test]
fn forwarded_metadata_and_type_path_boundaries_match_pinned_rustc() {
    for (name, source, success) in [
        (
            "type_forwarded_path_bound",
            "trait T{}macro_rules! q {($p:path)=>{fn f<X:$p>(){}}}macro_rules! p {($t:ty)=>{q!($t);}}p!(T);",
            true,
        ),
        (
            "type_nonpath_metadata",
            "macro_rules! p {($t:ty)=>{#[$t(unused)] fn f(){}}}p!(&allow);",
            false,
        ),
        (
            "generic_path_metadata",
            "macro_rules! p {($t:path)=>{#[$t(unused)] fn f(){}}}p!(allow<u8>);",
            false,
        ),
        (
            "path_metadata_opaque_forward",
            "macro_rules! q {($m:meta)=>{#[$m]fn f(){}}}macro_rules! p {($t:path)=>{q!($t);}}p!(test);",
            true,
        ),
        (
            "metadata_after_opaque",
            "macro_rules! q {($m:meta(unused))=>{}}macro_rules! p {($m:meta)=>{q!($m(unused));}}p!(allow);",
            true,
        ),
    ] {
        let result = analyze(source, &mut Files::default());
        assert_eq!(
            result.is_ok(),
            success,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[test]
fn captured_imports_keep_the_expansion_that_introduced_them() -> Result<(), String> {
    for (setup, source_import) in [
        ("use b::m;", true),
        ("macro_rules! setup {()=>{use b::m;}} setup!();", false),
        (
            "macro_rules! setup {($p:path)=>{use $p;}} setup!(b::m);",
            false,
        ),
        (
            "macro_rules! setup {($i:item)=>{$i}} setup!(use b::m;);",
            false,
        ),
        (
            "macro_rules! setup {($($t:tt)*)=>{$($t)*}} setup!(use b::m;);",
            false,
        ),
        (
            "macro_rules! setup {($p:ident)=>{use b::$p;}} setup!(m);",
            false,
        ),
    ] {
        let source = format!(
            "mod a {{macro_rules! m {{()=>{{1}}}} pub(crate) use m;}} mod b {{macro_rules! m {{()=>{{2}}}} pub(crate) use m;}} use a::*; {setup} fn f(){{let _=m!();}}"
        );
        let result = analyze(&source, &mut Files::default());
        assert_eq!(result.is_ok(), source_import, "{setup}");
        let source = format!(
            "mod a {{pub fn m()->i32{{1}}}} mod b {{pub fn m()->i32{{2}}}} use a::*; {setup} fn f(){{let _=m();}}"
        );
        resolve::analyze_expanded(analyze(&source, &mut Files::default())?)
            .map_err(|e| format!("{setup}: {e:?}"))?;
    }
    Ok(())
}

#[test]
fn captured_calls_use_introduction_ancestry_for_textual_shadows() -> Result<(), String> {
    for source in [
        "macro_rules! m {()=>{unknown}} macro_rules! setup {($i:item)=>{macro_rules! m {()=>{2}} $i}} setup!(fn answer()->i32{m!()});",
        "macro_rules! m {()=>{unknown}} macro_rules! setup {($($i:item)*)=>{$($i)*}} setup!(macro_rules! m {()=>{2}} fn answer()->i32{m!()});",
        "macro_rules! m {()=>{unknown}} macro_rules! setup {($e:expr)=>{{macro_rules! m {()=>{2}} $e}}}fn f()->i32{setup!(m!())}",
        "macro_rules! m {()=>{unknown}}macro_rules! forward {($e:expr)=>{$e}}macro_rules! setup {($e:expr)=>{{macro_rules! m {()=>{2}} forward!($e)}}}fn f()->i32{setup!(m!())}",
    ] {
        let report = analyze(source, &mut Files::default())?;
        assert_eq!(
            report.invocations.last().map(|i| i.definition),
            Some(report.definitions.len() - 1)
        );
        resolve::analyze_expanded(report).map_err(|e| format!("{source}: {e:?}"))?;
    }
    assert!(analyze("macro_rules! m {()=>{1}} macro_rules! setup {($i:item)=>{$i}} setup!(macro_rules! m {()=>{2}});fn f()->i32{m!()}", &mut Files::default()).is_err());
    Ok(())
}

#[test]
fn opaque_use_paths_allow_groups_globs_and_aliases_without_reparsing() -> Result<(), String> {
    for fragment in ["path", "ty"] {
        for suffix in ["::*", "::{m}", " as alias"] {
            let source = format!(
                "mod a {{pub fn m(){{}}}} macro_rules! setup {{($p:{fragment})=>{{use $p{suffix};}}}}setup!(a);"
            );
            let report = analyze(&source, &mut Files::default())?;
            assert_eq!(report.invocations[0].parser_calls, 1);
            resolve::analyze_expanded(report).map_err(|e| format!("{source}: {e:?}"))?;
        }
        for (value, suffix) in [("a", "::m"), ("a::<i32>", " as alias")] {
            let source = format!(
                "mod a {{pub fn m(){{}}}} macro_rules! setup {{($p:{fragment})=>{{use $p{suffix};}}}}setup!({value});"
            );
            assert!(analyze(&source, &mut Files::default()).is_err(), "{source}");
        }
    }
    assert!(
        analyze(
            "macro_rules! setup {($p:ty)=>{use $p as alias;}}setup!(&i32);",
            &mut Files::default()
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn captured_modules_keep_macro_path_ambiguity_marks() {
    for setup in [
        "macro_rules! setup {()=>{pub mod m {macro_rules! f {()=>{2}}pub(crate) use f;}}}setup!();",
        "macro_rules! setup {($i:item)=>{$i}}setup!(pub mod m {macro_rules! f {()=>{2}}pub(crate) use f;});",
    ] {
        let source = format!(
            "mod a {{pub mod m {{macro_rules! f {{()=>{{1}}}}pub(crate) use f;}}}}use a::*;{setup}fn f()->i32{{m::f!()}}"
        );
        let error = analyze(&source, &mut Files::default()).err();
        assert!(
            error.is_some_and(|e| e.contains("ambiguous macro-expanded path")),
            "{source}"
        );
    }
}

#[test]
fn local_inner_macros_rewrite_literal_names_but_preserve_captured_names() -> Result<(), String> {
    let root = "#[macro_export]macro_rules! helper {()=>{1}}";
    for (pattern, body, argument, root_helper) in [
        ("", "helper!()", "", true),
        ("$e:expr", "$e", "helper!()", false),
        ("$i:ident", "$i!()", "helper", false),
        ("$($i:tt)*", "$($i)*", "helper!()", false),
        ("", "self::helper!()", "", false),
    ] {
        let source = format!(
            "{root} #[macro_export(local_inner_macros)]macro_rules! outer {{({pattern})=>{{{body}}}}}mod x{{macro_rules! helper{{()=>{{2}}}}pub(crate) use helper;pub fn value()->i32{{crate::outer!({argument})}}}}"
        );
        let report = analyze(&source, &mut Files::default())?;
        assert_eq!(
            report.invocations.last().map(|i| i.definition),
            Some(if root_helper { 0 } else { 2 }),
            "{source}"
        );
        resolve::analyze_expanded(report).map_err(|e| format!("{source}: {e:?}"))?;
    }
    assert!(analyze("macro_rules! helper {()=>{1}}#[macro_export(local_inner_macros)]macro_rules! outer {()=>{helper!()}}fn f()->i32{outer!()}", &mut Files::default()).is_err());
    Ok(())
}

#[test]
fn dollar_crate_uses_typed_root_identity_in_paths_and_imports() -> Result<(), String> {
    for body in [
        "$crate::helper!()",
        "{use $crate::helper as alias; alias!()}",
        "{use $crate::{helper as alias}; alias!()}",
    ] {
        let source = format!(
            "#[macro_export]macro_rules! helper {{()=>{{1}}}}macro_rules! outer {{()=>{{{body}}}}}mod x{{macro_rules! helper{{()=>{{2}}}}pub fn value()->i32{{outer!()}}}}"
        );
        let report = analyze(&source, &mut Files::default())?;
        assert_eq!(
            report.invocations.last().map(|i| i.definition),
            Some(0),
            "{source}"
        );
        resolve::analyze_expanded(report).map_err(|e| format!("{source}: {e:?}"))?;
    }
    for body in [
        "{use $crate::helper; helper!()}",
        "{use $crate::{helper}; helper!()}",
    ] {
        let source = format!(
            "#[macro_export]macro_rules! helper {{()=>{{1}}}}macro_rules! outer {{()=>{{{body}}}}}mod x{{macro_rules! helper{{()=>{{2}}}}pub fn value()->i32{{outer!()}}}}"
        );
        assert!(analyze(&source, &mut Files::default()).is_err(), "{source}");
    }
    Ok(())
}

#[test]
fn crate_recursion_limits_follow_configured_first_attribute_and_exact_depth() -> Result<(), String>
{
    for (attributes, success) in [
        (r#"#![recursion_limit="0"]"#, false),
        (r#"#![recursion_limit="1"]"#, false),
        (r#"#![recursion_limit="2"]"#, true),
        (r#"#![recursion_limit="+3"]"#, true),
        (r#"#![recursion_limit="-1"]"#, false),
        (r#"#![recursion_limit="abc"]"#, false),
        (r"#![recursion_limit=3]", false),
        (r#"#![recursion_limit="0"]#![recursion_limit="3"]"#, false),
        (r#"#![recursion_limit="3"]#![recursion_limit="0"]"#, true),
        (r#"#![cfg_attr(unix,recursion_limit="1")]"#, false),
    ] {
        let source = format!(
            "{attributes}macro_rules! a {{()=>{{b!()}}}}macro_rules! b {{()=>{{1}}}}fn f()->i32{{a!()}}"
        );
        assert_eq!(
            analyze(&source, &mut Files::default()).is_ok(),
            success,
            "{attributes}"
        );
    }
    let mut source = String::from(r#"#![recursion_limit="256"]"#);
    for i in 0..200 {
        use std::fmt::Write as _;
        write!(source, "macro_rules! m{i} {{()=>{{m{}!()}}}}", i + 1).map_err(|e| e.to_string())?;
    }
    source.push_str("macro_rules! m200 {()=>{1}}fn f()->i32{m0!()}");
    let report = analyze(&source, &mut Files::default())?;
    assert_eq!(report.invocations.len(), 201);
    resolve::analyze_expanded(report).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

#[test]
fn textual_path_conflicts_follow_lexical_scope_and_late_imports() {
    for (source, success) in [
        (
            "macro_rules! a {()=>{1}}macro_rules! b{()=>{2}}use b as a;fn main(){let _=a!();}",
            true,
        ),
        (
            "macro_rules! a{()=>{1}}macro_rules! b{()=>{2}}fn main(){use b as a;let _=a!();}",
            false,
        ),
        (
            "macro_rules! a{()=>{1}}macro_rules! b{()=>{2}}fn main(){use b as a;use a as alias;let _=alias!();}",
            false,
        ),
        (
            "macro_rules! a{()=>{1}}mod x{macro_rules! a{()=>{2}}pub(crate) use a;}fn main(){use x::*;let _=a!();}",
            false,
        ),
        (
            "macro_rules! a{()=>{1}}mod x{macro_rules! a{()=>{2}}pub(crate) use a;}use x::*;fn main(){let _=a!();}",
            true,
        ),
        (
            "macro_rules! b{()=>{2}}use b as a;fn main(){macro_rules! a{()=>{1}}let _=a!();}",
            true,
        ),
        ("macro_rules! a{()=>{1}}fn main(){use a;let _=a!();}", true),
        (
            "macro_rules! b{()=>{2}}use b as a;#[macro_use]mod x{macro_rules! a{()=>{1}}}fn main(){let _=a!();}",
            false,
        ),
        (
            "macro_rules! a{()=>{1}}macro_rules! b{()=>{2}}macro_rules! setup{()=>{use b as a;}}fn main(){let _=a!();setup!();}",
            false,
        ),
    ] {
        let result = analyze(source, &mut Files::default())
            .and_then(|report| resolve::analyze_expanded(report).map_err(|e| format!("{e:?}")));
        assert_eq!(
            result.is_ok(),
            success,
            "{source}: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[test]
fn expanded_textual_candidates_compare_path_candidate_ancestry() {
    for (source, success) in [
        (
            "macro_rules! b{()=>{1}}use b as m;macro_rules! setup{()=>{macro_rules! m{()=>{2}}}}setup!();fn main(){let _=m!();}",
            false,
        ),
        (
            "macro_rules! b{()=>{1}}use b as m;macro_rules! setup{()=>{macro_rules! m{()=>{2}}fn f()->i32{m!()}}}setup!();fn main(){}",
            true,
        ),
        (
            "macro_rules! b{()=>{1}}use b as m;macro_rules! setup{()=>{macro_rules! m{()=>{2}}}}setup!();use m as alias;fn main(){let _=alias!();}",
            false,
        ),
        (
            "macro_rules! b{()=>{1}}macro_rules! setup{()=>{macro_rules! m{()=>{2}}}}setup!();fn main(){let _=m!();}use b as m;",
            false,
        ),
        (
            "macro_rules! b{()=>{1}}macro_rules! setup{()=>{macro_rules! m{()=>{2}}use b as m;}}setup!();fn main(){let _=m!();}",
            true,
        ),
        (
            "macro_rules! b{()=>{1}}macro_rules! setup{()=>{use b as m;macro_rules! m{()=>{2}}}}setup!();fn main(){let _=m!();}",
            true,
        ),
        (
            "macro_rules! b{()=>{1}}macro_rules! import{()=>{use b as m;}}macro_rules! setup{()=>{macro_rules! m{()=>{2}}import!();}}setup!();fn main(){let _=m!();}",
            true,
        ),
    ] {
        let result = analyze(source, &mut Files::default())
            .and_then(|report| resolve::analyze_expanded(report).map_err(|e| format!("{e:?}")));
        assert_eq!(
            result.is_ok(),
            success,
            "{source}: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[test]
fn early_paths_check_outer_candidates_without_changing_ordinary_lookup() {
    for (source, success) in [
        (
            "mod glob {pub mod m {pub struct S;}}mod m {pub struct S;} fn f(){use glob::*;use m::S;}fn main(){}",
            false,
        ),
        (
            "mod glob {pub mod m {pub struct S;}}mod m {pub struct S;} fn f(){use glob::*;let _=m::S;}fn main(){}",
            true,
        ),
        (
            "mod m{pub struct S;}mod glob{pub(crate) use crate::m;}fn f(){use glob::*;use m::S;}fn main(){}",
            true,
        ),
        (
            "mod glob {pub mod m {pub struct S;}}mod m {pub struct S;} fn f(){use glob::*;use crate::glob::m::S;}fn main(){}",
            true,
        ),
        (
            "mod a {macro_rules! m{()=>{1}}pub(crate) use m;}mod b {macro_rules! m{()=>{2}}pub(crate) use m;}use a::m;fn main(){use b::*;let _=m!();}",
            false,
        ),
        (
            "mod a {macro_rules! m{()=>{1}}pub(crate) use m;}mod b {macro_rules! m{()=>{2}}pub(crate) use m;}use a::m;fn main(){use b::m;let _=m!();}",
            true,
        ),
        (
            "mod a {macro_rules! m{()=>{1}}pub(crate) use m;}mod b {macro_rules! m{()=>{2}}pub(crate) use m;}use a::m;macro_rules! setup{()=>{use b::m;}}fn main(){setup!();let _=m!();}",
            false,
        ),
        (
            "mod a {macro_rules! m{()=>{1}}pub(crate) use m;}mod b {macro_rules! m{()=>{2}}pub(crate) use m;}use a::m;macro_rules! setup{()=>{use b::m;let _=m!();}}fn main(){setup!();}",
            true,
        ),
        (
            "mod glob {pub mod m {pub struct S;}}mod m {pub struct S;} macro_rules! setup {()=>{mod m{pub struct S;}}}fn f(){setup!();use m::S;}fn main(){}",
            false,
        ),
        (
            "mod a {pub mod m{pub struct S;}}mod b {pub mod m{pub struct S;}}use a::*;macro_rules! setup{()=>{use b::m;}}setup!();use m::S;fn main(){}",
            false,
        ),
        (
            "mod a {pub mod m{pub struct S;}}mod b {pub mod m{pub struct S;}}use a::*;macro_rules! setup{()=>{use b::m;}}setup!();fn f(){let _=m::S;}fn main(){}",
            true,
        ),
    ] {
        let result = analyze(source, &mut Files::default())
            .and_then(|report| resolve::analyze_expanded(report).map_err(|e| format!("{e:?}")));
        assert_eq!(
            result.is_ok(),
            success,
            "{source}: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[test]
fn cfg_builtin_emits_typed_booleans_and_consumes_opaque_metadata() -> Result<(), String> {
    use rustgm::{literal::Value, syntax::ast::Kind};
    for (source, expected, calls) in [
        ("fn f()->bool{cfg!(unix)}", true, 1),
        ("fn f()->bool{cfg!(windows,)}", false, 1),
        (
            "fn f()->bool{cfg!(all(unix, target_os=\"linux\"))}",
            true,
            1,
        ),
        (
            "macro_rules! query {($m:meta)=>{cfg!($m)}}fn f()->bool{query!(all(unix,target_os=\"linux\"))}",
            true,
            2,
        ),
        ("#![no_implicit_prelude]fn f()->bool{cfg!(unix)}", true, 1),
    ] {
        let report = analyze(source, &mut Files::default())?;
        assert_eq!(report.invocations.len(), calls);
        let call = report
            .invocations
            .last()
            .ok_or("missing builtin invocation")?;
        assert_eq!(call.rule, None);
        assert_eq!(call.parser_calls, 0);
        let unit = report.package.unit(call.node.file);
        let Kind::Literal(index) = unit.configured.ast.node(call.node.node).kind else {
            return Err("builtin did not construct a literal node".into());
        };
        assert!(
            matches!(unit.literal(index).map_err(|e| format!("{e:?}"))?, Value::Boolean(value) if value == expected)
        );
        assert_eq!(unit.source.original(), source);
        resolve::analyze_expanded(report).map_err(|e| format!("{e:?}"))?;
    }
    Ok(())
}

#[test]
fn builtin_scope_can_be_shadowed_but_is_not_the_crate_root() -> Result<(), String> {
    for source in [
        "macro_rules! cfg {()=>{7}}fn f()->i32{cfg!()}",
        "macro_rules! setup {()=>{macro_rules! cfg {()=>{7}}fn f()->i32{cfg!()}}}setup!();",
        "#[cfg(any())] fn hidden(){cfg!(broken input);}fn f(){cfg!(unix);}",
    ] {
        resolve::analyze_expanded(analyze(source, &mut Files::default())?)
            .map_err(|e| format!("{e:?}"))?;
    }
    for source in [
        "fn f()->bool{crate::cfg!(unix)}",
        "use cfg as chosen;fn f()->bool{chosen!(unix)}",
        "macro_rules! setup {()=>{macro_rules! cfg {()=>{7}}}}setup!();fn f()->i32{cfg!()}",
    ] {
        assert!(analyze(source, &mut Files::default()).is_err(), "{source}");
    }
    Ok(())
}

#[test]
fn cfg_builtin_rejects_invalid_arguments_and_output_contexts() {
    for source in [
        "fn f(){cfg!();}",
        "fn f(){cfg!(unix,windows);}",
        "fn f(){cfg!(unix,,);}",
        "fn f(){cfg!(target_os=concat!(\"lin\",\"ux\"));}",
        "fn f(){cfg!(target_os=3);}",
        "type T=cfg!(unix);",
        "cfg!(unix);",
    ] {
        assert!(analyze(source, &mut Files::default()).is_err(), "{source}");
    }
}

#[test]
fn builtin_materialization_does_not_renumber_later_generated_macro_targets() -> Result<(), String> {
    let source = "fn early()->bool{cfg!(unix)}macro_rules! make {()=>{macro_rules! late{()=>{3}}use late as alias;}}make!();fn f()->i32{alias!()}";
    let report = analyze(source, &mut Files::default())?;
    let last = report.invocations.last().ok_or("missing generated call")?;
    assert_eq!(
        report
            .package
            .names
            .get(report.definitions[last.definition].name),
        "late"
    );
    resolve::analyze_expanded(report).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

#[test]
fn location_builtins_distinguish_templates_from_whole_captured_calls() -> Result<(), String> {
    use rustgm::{
        expand::builtin::Builtin,
        literal::{Value, value::IntegerType},
        resolve::expansion::Body,
        syntax::ast::Kind,
    };
    for (source, expected) in [
        (
            "fn main(){\nlet result=(line!(),column!());\n}\n",
            &[2, 21][..],
        ),
        (
            "macro_rules! m {()=>{(line!(),column!())}}\nfn main(){\n let result=m!();\n}\n",
            &[3, 13][..],
        ),
        (
            "macro_rules! m {($e:expr)=>{$e}}\nfn main(){\n let result=m!((line!(),column!()));\n}\n",
            &[3, 25][..],
        ),
        (
            "macro_rules! m {($i:ident)=>{$i!()}}\nfn main(){\n let result=(m!(line),m!(column));\n}\n",
            &[3, 23][..],
        ),
        (
            "macro_rules! b {($e:expr)=>{$e}}\nmacro_rules! a {()=>{b!((line!(),column!()))}}\nfn main(){\n let result=a!();\n}\n",
            &[4, 13][..],
        ),
        (
            "fn main(){\n let é=0; let result=(line!(),column!());\n}\n",
            &[2, 31][..],
        ),
        ("﻿const C:u32=column!();\n", &[13][..]),
    ] {
        for source in [source.to_owned(), source.replace('\n', "\r\n")] {
            let report = analyze(&source, &mut Files::default())?;
            let mut actual = Vec::new();
            for call in &report.invocations {
                if !matches!(
                    report.definitions[call.definition].body,
                    Body::Builtin(Builtin::Line | Builtin::Column)
                ) {
                    continue;
                }
                let unit = report.package.unit(call.node.file);
                let Kind::Literal(index) = unit.configured.ast.node(call.node.node).kind else {
                    return Err("location result is not a literal".into());
                };
                let Value::Integer(value) = unit.literal(index).map_err(|e| format!("{e:?}"))?
                else {
                    return Err("location result is not an integer".into());
                };
                assert_eq!(value.suffix, Some(IntegerType::U32));
                actual.push(value.magnitude);
            }
            assert_eq!(actual, expected, "{source}");
            resolve::analyze_expanded(report).map_err(|e| format!("{e:?}"))?;
        }
    }
    Ok(())
}

#[test]
fn location_builtins_reject_arguments() {
    for source in ["fn f(){line!(1);}", "fn f(){column!(,);}"] {
        assert!(analyze(source, &mut Files::default()).is_err(), "{source}");
    }
}

#[test]
fn ordinary_type_items_shadow_modules_but_generics_do_not_in_macro_lookup() -> Result<(), String> {
    let prelude = "#[macro_export] macro_rules! make {()=>{1}} mod a{pub use crate::make;} ";
    for (body, accepted) in [
        ("fn f(){struct a;let _=a::make!();}", false),
        ("fn f<a>(){let _=a::make!();}", true),
        ("fn f<a>(_: [u8; a::make!()]){}", true),
        ("fn f(){enum a{B}let _=a::make!();}", false),
        ("fn f(){type a=u8;let _=a::make!();}", false),
        ("fn f(){let _=a::make!();struct a;}", false),
        ("fn f<a>(){let _=crate::a::make!();}", true),
        ("fn f<a>(){fn g(){let _=a::make!();}}", true),
        ("fn f(){let a=2;let _=a::make!();}", true),
        (
            "macro_rules! shadow{()=>{struct a;}} fn f(){shadow!();let _=a::make!();}",
            false,
        ),
        (
            "macro_rules! f{()=>{fn f<a>(_: [u8;a::make!()]){}}}f!();",
            true,
        ),
        (
            "macro_rules! f{($e:expr)=>{fn f<a>(){let _=$e;}}}f!(a::make!());",
            true,
        ),
    ] {
        let source = format!("{prelude}{body}");
        let result = analyze(&source, &mut Files::default());
        assert_eq!(
            result.is_ok(),
            accepted,
            "{source}: {:?}",
            result.as_ref().err()
        );
        if let Ok(report) = result {
            resolve::analyze_expanded(report).map_err(|e| format!("{source}: {e:?}"))?;
        }
    }
    Ok(())
}

#[test]
fn early_item_imports_and_late_shadows_reuse_final_definition_identities() -> Result<(), String> {
    let prelude = "#[macro_export] macro_rules! make {()=>{1}} mod a{pub use crate::make;} ";
    for body in [
        "enum E{V} fn f(){use E::V as a;let _=a::make!();}",
        "mod types{pub struct a;}fn f(){use types::*;let _=a::make!();}",
        "macro_rules! shadow{()=>{struct a;}}fn f(){let _=a::make!();shadow!();}",
        "fn f<a>(){use a::make;let _=make!();}",
    ] {
        let source = format!("{prelude}{body}");
        let result = analyze(&source, &mut Files::default())
            .and_then(|r| resolve::analyze_expanded(r).map_err(|e| format!("{e:?}")));
        assert!(result.is_err(), "{source}");
    }
    for source in [
        format!("{prelude}fn f(){{struct a;let _=crate::a::make!();}}"),
        "macro_rules! setup{()=>{fn g()->i32{3}}}setup!();use g as f;fn h()->i32{f()}".into(),
        "enum E{V}use E::V as a;fn f(){let _=a;}".into(),
    ] {
        let report = analyze(&source, &mut Files::default())?;
        let definitions: Vec<_> = report
            .resolution
            .definitions
            .iter()
            .map(|d| (d.name, d.namespace, d.kind, d.scope, d.node, d.members))
            .collect();
        let report = resolve::analyze_expanded(report).map_err(|e| format!("{source}: {e:?}"))?;
        for (index, expected) in definitions.into_iter().enumerate() {
            let d = &report.resolution.definitions[index];
            assert_eq!(
                (d.name, d.namespace, d.kind, d.scope, d.node, d.members),
                expected
            );
        }
    }
    Ok(())
}

fn diagnosed(source: &str, files: &mut Files) -> Result<resolve::package::Error, String> {
    let result = expansion::analyze(
        InputFile {
            identity: "/crate/lib.rs".into(),
            path: "/crate/lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        files,
        &Config::default(),
    )
    .and_then(resolve::analyze_expanded);
    match result {
        Ok(_) => Err("expected a source diagnostic".into()),
        Err(error) => Ok(error),
    }
}

#[test]
fn expansion_diagnostics_project_physical_tokens_and_ordered_call_definition_traces()
-> Result<(), String> {
    for (source, offset, names, calls, definitions) in [
        (
            "macro_rules! inner{()=>{missing}} macro_rules! outer{()=>{inner!()}} fn f(){outer!();}",
            24,
            vec!["inner", "outer"],
            vec![58, 76],
            vec![Some(0), Some(34)],
        ),
        (
            "macro_rules! identity{($x:expr)=>{$x}}fn f(){identity!(missing);}",
            55,
            vec![],
            vec![],
            vec![],
        ),
        (
            "macro_rules! maker{()=>{macro_rules! inner{()=>{missing}} inner!();}}fn f(){maker!();}",
            48,
            vec!["inner", "maker"],
            vec![58, 76],
            vec![Some(24), Some(0)],
        ),
        (
            "macro_rules! m{()=>{1 2}}fn f(){m!();}",
            22,
            vec!["m"],
            vec![32],
            vec![Some(0)],
        ),
        (
            "macro_rules! m{()=>{line!(1)}}fn f(){m!();}",
            26,
            vec!["line", "m"],
            vec![20, 37],
            vec![None, Some(0)],
        ),
    ] {
        let error = diagnosed(source, &mut Files::default())?;
        assert_eq!(error.file.source().map(|file| file.0), Some(0));
        assert_eq!(error.span.start, offset, "{source}");
        assert_eq!(
            error
                .expansions
                .iter()
                .map(|e| e.macro_name.as_str())
                .collect::<Vec<_>>(),
            names
        );
        assert_eq!(
            error
                .expansions
                .iter()
                .map(|e| e.call.span.start)
                .collect::<Vec<_>>(),
            calls
        );
        assert_eq!(
            error
                .expansions
                .iter()
                .map(|e| e.definition.map(|d| d.span.start))
                .collect::<Vec<_>>(),
            definitions
        );
    }
    Ok(())
}

#[test]
fn expansion_diagnostics_cross_files_and_handle_uninstalled_failed_modules() -> Result<(), String> {
    let mut files = Files::default();
    let child = "macro_rules! m {()=>{missing}}";
    files.sources.insert("/crate/child.rs".into(), child.into());
    let error = diagnosed("#[macro_use] mod child; fn f(){m!();}", &mut files)?;
    assert_eq!(error.file.source().map(|file| file.0), Some(1));
    assert_eq!(error.span.start, 21);
    assert_eq!(error.expansions.len(), 1);
    let trace = &error.expansions[0];
    assert_eq!(trace.macro_name, "m");
    assert_eq!(trace.call.file.0, 0);
    assert_eq!(trace.call.span.start, 31);
    assert!(
        trace
            .definition
            .is_some_and(|d| d.file.0 == 1 && d.span.start == 0)
    );
    files
        .sources
        .insert("/crate/child.rs".into(), "fn invalid(".into());
    let error = diagnosed("macro_rules! m{()=>{mod child;}}m!();", &mut files)?;
    assert_eq!(error.file.source().map(|file| file.0), Some(1));
    assert_eq!(error.expansions.len(), 1);
    assert_eq!(error.expansions[0].macro_name, "m");
    assert_eq!(error.expansions[0].call.file.0, 0);
    Ok(())
}

fn builtin_strings(report: &expansion::Report) -> Result<Vec<String>, String> {
    use rustgm::{expand::builtin::Builtin, literal::Value, syntax::ast::Kind};
    let mut values = Vec::new();
    for call in &report.invocations {
        if !matches!(
            report.definitions[call.definition].body,
            expansion::Body::Builtin(
                Builtin::File
                    | Builtin::ModulePath
                    | Builtin::Stringify
                    | Builtin::Concat
                    | Builtin::Env
                    | Builtin::IncludeStr
            )
        ) {
            continue;
        }
        let unit = report.package.unit(call.node.file);
        let Kind::Literal(index) = unit.configured.ast.node(call.node.node).kind else {
            return Err("expected a typed string literal".into());
        };
        let Value::String(bytes) = unit.literal(index).map_err(|e| e.message)? else {
            return Err("expected a string value".into());
        };
        values.push(String::from_utf8(bytes.into_owned()).map_err(|e| e.to_string())?);
    }
    Ok(values)
}

#[test]
fn file_and_module_builtins_keep_requested_names_and_current_module_identity() -> Result<(), String>
{
    struct Aliases;
    impl Sources for Aliases {
        fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
            if !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("shared.rs" | "alias.rs")
            ) {
                return Ok(None);
            }
            Ok(Some(InputFile {
                identity: "/physical/shared.rs".into(),
                path: path.to_owned(),
                source: SourceFile::new(
                    "pub fn f()->&'static str{file!()}pub fn m()->&'static str{module_path!()}"
                        .as_bytes()
                        .to_vec(),
                )?,
            }))
        }
    }
    let source = "#[path=\"shared.rs\"]mod a;#[path=\"./shared.rs\"]mod b;#[path=\"alias.rs\"]mod c;const F:&str=file!();const M:&str=module_path!();";
    let report = expansion::analyze_with_crate_name(
        InputFile {
            identity: "/physical/main.rs".into(),
            path: "main.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut Aliases,
        &Config::default(),
        Some("probe"),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|e| e.to_string())?;
    assert_eq!(report.package.units.len(), 4);
    assert_eq!(
        builtin_strings(&report)?,
        [
            "shared.rs",
            "probe::a",
            "./shared.rs",
            "probe::b",
            "alias.rs",
            "probe::c",
            "main.rs",
            "probe"
        ]
    );
    let captured = "macro_rules! make{($e:expr)=>{mod child{pub fn f()->&'static str{$e}}}}make!(module_path!());fn f(){mod inner{pub fn m()->&'static str{module_path!()}}}";
    let report = analyze(captured, &mut Files::default())?;
    assert_eq!(builtin_strings(&report)?, ["lib::inner", "lib::child"]);
    resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    let mut files = Files::default();
    files.sources.insert(
        "/crate/defs.rs".into(),
        "macro_rules! name{()=>{file!()}}".into(),
    );
    let report = analyze("#[macro_use] mod defs; const F:&str=name!();", &mut files)?;
    assert_eq!(builtin_strings(&report)?, ["/crate/lib.rs"]);
    Ok(())
}

#[test]
fn crate_names_follow_driver_attributes_and_the_pinned_first_value_rules() -> Result<(), String> {
    for (prefix, requested, expected) in [
        ("", None, Some("lib")),
        ("", Some("probe"), Some("probe")),
        ("#![crate_name=\"selected\"]", None, Some("selected")),
        ("#![crate_name=\"probe\"]", Some("probe"), Some("probe")),
        ("#![crate_name=\"inner\"]", Some("probe"), None),
        (
            "#![crate_name=\"first\"]#![crate_name=\"second\"]",
            None,
            Some("first"),
        ),
        ("#![crate_name=\"first\"]#![crate_name=1]", None, None),
        ("#![cfg_attr(unix,crate_name=\"active\")]", None, None),
        ("", Some("dash-name"), None),
        ("", Some(""), None),
        ("", Some("123"), Some("123")),
        ("", Some("héllo"), Some("héllo")),
    ] {
        let source = format!("{prefix}const M:&str=module_path!();");
        let result = expansion::analyze_with_crate_name(
            InputFile {
                identity: "/crate/lib.rs".into(),
                path: "/crate/lib.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut Files::default(),
            &Config::default(),
            requested,
        );
        assert_eq!(
            result.is_ok(),
            expected.is_some(),
            "{source}: {:?}",
            result.as_ref().err()
        );
        if let (Ok(report), Some(expected)) = (result, expected) {
            assert_eq!(builtin_strings(&report)?, [expected]);
        }
    }
    for name in ["file", "module_path"] {
        for source in [
            format!("const S:&str={name}!(1);"),
            format!("{name}!();"),
            format!("type T={name}!();"),
        ] {
            assert!(analyze(&source, &mut Files::default()).is_err(), "{source}");
        }
    }
    Ok(())
}

#[test]
fn stringify_preserves_source_spacing_and_delimiter_rules() -> Result<(), String> {
    for (tokens, expected) in [
        ("", ""),
        ("a+b", "a+b"),
        ("a /* comment */ + b", "a + b"),
        (
            "a ( b,c ) {d} [ e ] f ! () x :: y",
            "a(b,c) {d} [e] f! () x :: y",
        ),
        (
            "{ a} {b } { c } {} { /*c*/ } (( a ))",
            "{ a } {b} { c } {} {} ((a))",
        ),
        (
            "r#fn ! (x) fn (x) let (x) Self () pub (crate) if ! x",
            "r#fn! (x) fn(x) let (x) Self() pub(crate) if ! x",
        ),
        ("e\u{301} r#e\u{301} 'e\u{301}", "é r#é 'é"),
        ("a\u{200e}+\u{200f}b", "a + b"),
        ("missing!(arbitrary tokens)", "missing!(arbitrary tokens)"),
    ] {
        let report = analyze(
            &format!("fn f(){{let _=stringify!({tokens});}}"),
            &mut Files::default(),
        )?;
        assert_eq!(builtin_strings(&report)?, [expected], "{tokens}");
        assert_eq!(report.invocations.len(), 1);
        assert_eq!(report.invocations[0].parser_calls, 0);
        rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[test]
fn stringify_uses_template_spacing_and_original_opaque_tokens() -> Result<(), String> {
    for (definition, invocation, expected, retained) in [
        (
            "macro_rules! m{()=>{stringify!(a+b f!() &mut x x::y x.foo {d})}}",
            "m!()",
            "a + b f! () & mut x x :: y x.foo {d}",
            0,
        ),
        (
            "macro_rules! m{($e:expr)=>{stringify!(a $e b $e; ($e) $e + x)}}",
            "m!(1+2)",
            "a 1+2 b 1+2; (1+2) 1+2 + x",
            3,
        ),
        (
            "macro_rules! m{($e:expr)=>{n!($e)}} macro_rules! n{($e:expr)=>{stringify!($e $e)}}",
            "m!(1+2)",
            "1+2 1+2",
            3,
        ),
        (
            "macro_rules! m{($v:vis struct $i:ident)=>{stringify!(a $v b $v struct $i)}}",
            "m!(struct X)",
            "a  b  struct X",
            0,
        ),
        (
            "macro_rules! m{(A<$t:ty>)=>{stringify!($t)}}",
            "m!(A<B<u32>>)",
            "B<u32>",
            4,
        ),
        (
            "macro_rules! m{($i:item)=>{stringify!($i)}}",
            "m!(fn f(){let x=1;})",
            "fn f(){let x=1;}",
            11,
        ),
        (
            "macro_rules! m{()=>{stringify!($crate::a)}}",
            "m!()",
            "$crate :: a",
            0,
        ),
        (
            "macro_rules! m{($x:ident)=>{stringify!($x where)}}",
            "m!(name)",
            "name where",
            0,
        ),
    ] {
        let report = analyze(
            &format!("{definition} fn f(){{let _={invocation};}}"),
            &mut Files::default(),
        )?;
        assert_eq!(builtin_strings(&report)?, [expected], "{invocation}");
        assert_eq!(report.captures.token_count(), retained, "{invocation}");
    }
    Ok(())
}

#[test]
fn stringify_distinguishes_direct_and_captured_documentation() -> Result<(), String> {
    let docs = "/// hello\n/** block */ /*! inner */ //! line\nx";
    let direct = analyze(
        &format!("fn f(){{let _=stringify!({docs});}}"),
        &mut Files::default(),
    )?;
    assert_eq!(
        builtin_strings(&direct)?,
        ["/// hello\n/** block */\n /*! inner */\n //! line\nx"]
    );
    let template = analyze(
        &format!("macro_rules! m{{()=>{{stringify!({docs})}}}} fn f(){{let _=m!();}}"),
        &mut Files::default(),
    )?;
    assert_eq!(builtin_strings(&template)?, builtin_strings(&direct)?);
    let captured = analyze(
        &format!(
            "macro_rules! m{{($($x:tt)*)=>{{stringify!($($x)*)}}}} fn f(){{let _=m!({docs});}}"
        ),
        &mut Files::default(),
    )?;
    assert_eq!(
        builtin_strings(&captured)?,
        [
            "#[doc = r\" hello\"] #[doc = r\" block \"] #![doc = r\" inner \"] #![doc = r\" line\"]\nx"
        ]
    );
    Ok(())
}

#[test]
fn stringify_wraps_long_groups_with_bounded_iterative_layout() -> Result<(), String> {
    let words = (0..20)
        .map(|n| format!("name{n}"))
        .collect::<Vec<_>>()
        .join(" ");
    let report = analyze(
        &format!("fn f(){{let _=stringify!({{ {words} }});}}"),
        &mut Files::default(),
    )?;
    assert_eq!(
        builtin_strings(&report)?,
        [
            "{\n    name0 name1 name2 name3 name4 name5 name6 name7 name8 name9 name10 name11\n    name12 name13 name14 name15 name16 name17 name18 name19\n}"
        ]
    );
    let nested = format!("{}x{}", "(".repeat(512), ")".repeat(512));
    let report = analyze(
        &format!("fn f(){{let _=stringify!({nested});}}"),
        &mut Files::default(),
    )?;
    assert_eq!(builtin_strings(&report)?, [nested]);
    for source in ["type T=stringify!(x);", "stringify!(x);"] {
        assert!(analyze(source, &mut Files::default()).is_err());
    }
    Ok(())
}

#[test]
fn eager_concat_consumes_typed_literals_and_rejects_other_expression_shapes() -> Result<(), String>
{
    for (arguments, expected) in [
        ("", ""),
        (
            "\"a\",42u8,0xff,-0x10,1_2.30e+0_2f64,-0.0,true,'é',",
            "a42255-1612.30e+02-0.0trueé",
        ),
        ("256u8, -256i8", "256-256"),
        ("cfg!(unix)", "true"),
        ("stringify!(x+y)", "x+y"),
    ] {
        let report = analyze(
            &format!("fn f(){{let _=concat!({arguments});}}"),
            &mut Files::default(),
        )?;
        assert_eq!(builtin_strings(&report)?[0], expected);
        assert_eq!(report.invocations[0].parser_calls, 1);
        rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    }
    for arguments in [
        "(\"a\")",
        "{\"a\"}",
        "1+2",
        "-(12)",
        "b\"a\"",
        "c\"a\"",
        "b'a'",
        "\"a\" \"b\"",
        "--1",
        "-true",
    ] {
        assert!(
            analyze(
                &format!("fn f(){{let _=concat!({arguments});}}"),
                &mut Files::default()
            )
            .is_err(),
            "{arguments}"
        );
    }
    Ok(())
}

#[test]
fn eager_arguments_wait_for_generated_macros_without_reparsing_or_leaking_nodes()
-> Result<(), String> {
    let source = "macro_rules! make{()=>{#[macro_export] macro_rules! a{()=>{concat!(\"a\",\"b\")}} #[macro_export] macro_rules! b{()=>{\"c\"}}}} fn f()-> &'static str{concat!(crate::a!(),crate::b!(),stringify!(x+y))} make!();";
    let report = analyze(source, &mut Files::default())?;
    assert_eq!(builtin_strings(&report)?[0], "abcx+y");
    let concats = report
        .invocations
        .iter()
        .filter(|call| {
            matches!(
                report.definitions[call.definition].body,
                expansion::Body::Builtin(rustgm::expand::builtin::Builtin::Concat)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(concats.len(), 2);
    assert!(concats.iter().all(|call| call.parser_calls == 1));
    let report = rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    assert!(report.resolution.expansions.is_empty());
    let active_containers = report
        .package
        .units
        .iter()
        .map(|unit| {
            unit.configured
                .ast
                .nodes
                .iter()
                .zip(&unit.configured.active)
                .filter(|(node, active)| {
                    **active && matches!(node.kind, rustgm::syntax::ast::Kind::Expansion { .. })
                })
                .count()
        })
        .sum::<usize>();
    // Only make!'s item-list expansion remains reachable; argument containers are retired.
    assert_eq!(active_containers, 1);
    Ok(())
}

#[test]
fn eager_arguments_preserve_capture_identity_and_location_causes() -> Result<(), String> {
    let source = "macro_rules! outer{($e:expr)=>{concat!($e,concat!(\"b\",\"c\"))}} fn f(){let _=outer!(\"a\");}";
    let report = analyze(source, &mut Files::default())?;
    assert_eq!(builtin_strings(&report)?, ["abc", "bc"]);
    assert_eq!(report.captures.len(), 1);
    assert_eq!(report.captures.token_count(), 1);
    let source = "
fn f()-> &'static str{concat!(line!(), \":\",
 line!(), \":\", column!())}";
    let report = analyze(source, &mut Files::default())?;
    assert_eq!(builtin_strings(&report)?, ["2:3:16"]);
    let source =
        "macro_rules! drop{($($t:tt)*)=>{\"kept\"}} fn f(){let _=concat!(drop!(missing!()));}";
    assert_eq!(
        builtin_strings(&analyze(source, &mut Files::default())?)?,
        ["kept"]
    );
    Ok(())
}

#[test]
fn eager_errors_report_the_failing_dependency_and_expansion_chain() -> Result<(), String> {
    let source = "fn f(){let _=concat!(missing!());}";
    let error = expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut Files::default(),
        &Config::default(),
    )
    .err()
    .ok_or("expected failure")?;
    assert_eq!(
        error.span.start as usize,
        source.find("missing").ok_or("missing source marker")?
    );
    assert_eq!(error.expansions[0].macro_name, "concat");
    let source = "compile_error!(concat!(\"stop \",stringify!(now)));";
    let error = analyze(source, &mut Files::default())
        .err()
        .ok_or("expected compile_error")?;
    assert!(error.contains("stop now"), "{error}");
    for source in [
        "compile_error!();",
        "compile_error!(42);",
        "compile_error!(\"a\",\"b\");",
        "#![recursion_limit=\"8\"] macro_rules! m{()=>{concat!(m!())}} fn f(){let _=m!();}",
    ] {
        assert!(analyze(source, &mut Files::default()).is_err(), "{source}");
    }
    Ok(())
}

#[test]
fn eager_arguments_borrow_opaque_facts_from_another_source_unit() -> Result<(), String> {
    let mut files = Files::default();
    files.sources.insert("/crate/defs.rs".into(), "macro_rules! make{($e:expr)=>{#[macro_export] macro_rules! invoke{()=>{concat!($e,\"!\")}}}} make!(\"cross\");".into());
    let source = "mod defs; fn f()-> &'static str{crate::invoke!()}";
    let report = analyze(source, &mut files)?;
    assert_eq!(builtin_strings(&report)?, ["cross!"]);
    assert_eq!(report.captures.len(), 1);
    assert_eq!(report.captures.token_count(), 1);
    assert_eq!(report.package.units[0].source.original(), source);
    rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn environment_builtins_are_explicit_cached_and_eager() -> Result<(), String> {
    let mut files = Files::default();
    files
        .environment
        .insert("RUSTGM_TEST".into(), Ok(Some("é\r\nvalue".into())));
    let report = analyze(
        "fn f(){let _=env!(concat!(\"RUSTGM_\",\"TEST\"));let _=env!(\"RUSTGM_TEST\",\"unused\");} #[cfg(any())] fn off(){let _=env!(\"MISSING\");}",
        &mut files,
    )?;
    let values = builtin_strings(&report)?;
    assert_eq!(
        values.iter().filter(|v| v.as_str() == "é\r\nvalue").count(),
        2
    );
    assert_eq!(files.environment_reads, ["RUSTGM_TEST"]);
    rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    let error = analyze(
        "fn f(){let _=env!(\"ABSENT\",concat!(\"custom \",\"message\"));}",
        &mut files,
    )
    .err()
    .ok_or("missing environment error")?;
    assert!(error.contains("custom message"), "{error}");
    files
        .environment
        .insert("BAD".into(), Err("value is not Unicode".into()));
    let error = analyze("fn f(){let _=env!(\"BAD\",\"custom\");}", &mut files)
        .err()
        .ok_or("missing provider error")?;
    assert!(error.contains("value is not Unicode"), "{error}");
    for arguments in [
        "",
        "42",
        "\"RUSTGM_TEST\",42",
        "\"RUSTGM_TEST\",\"a\",\"b\"",
        "(\"RUSTGM_TEST\")",
    ] {
        files.environment_reads.clear();
        assert!(analyze(&format!("fn f(){{let _=env!({arguments});}}"), &mut files).is_err());
        assert!(files.environment_reads.is_empty());
    }
    Ok(())
}

#[test]
fn included_resources_preserve_original_bytes_and_share_provider_reads() -> Result<(), String> {
    let mut files = Files::default();
    let text = "\u{feff}hello\r\nworld\r";
    files
        .resources
        .insert("/crate/text.txt".into(), text.as_bytes().to_vec());
    files
        .resources
        .insert("/crate/data.bin".into(), vec![0, 255, 13, 10, 128, 65]);
    let report = analyze(
        "fn f(){let _=include_str!(concat!(\"text\",\".txt\"));let _=include_str!(\"./text.txt\");let _=include_bytes!(\"data.bin\");} #[cfg(any())] fn off(){let _=include_str!(\"missing\");}",
        &mut files,
    )?;
    assert_eq!(
        builtin_strings(&report)?
            .iter()
            .filter(|v| v.as_str() == text)
            .count(),
        2
    );
    assert_eq!(
        files.resource_reads,
        [
            PathBuf::from("/crate/text.txt"),
            PathBuf::from("/crate/data.bin")
        ]
    );
    let call = report
        .invocations
        .iter()
        .find(|call| {
            matches!(
                report.definitions[call.definition].body,
                expansion::Body::Builtin(rustgm::expand::builtin::Builtin::IncludeBytes)
            )
        })
        .ok_or("missing include_bytes")?;
    let unit = report.package.unit(call.node.file);
    let rustgm::syntax::ast::Kind::Literal(index) = unit.configured.ast.node(call.node.node).kind
    else {
        return Err("missing byte literal".into());
    };
    assert_eq!(
        unit.literal(index).map_err(|e| e.message)?,
        rustgm::literal::Value::ByteString(vec![0, 255, 13, 10, 128, 65].into())
    );
    assert_eq!(report.package.units.len(), 1);
    rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    let error = analyze("fn f(){let _=include_str!(\"data.bin\");}", &mut files)
        .err()
        .ok_or("expected UTF-8 failure")?;
    assert!(error.contains("not UTF-8"));
    assert!(analyze("fn f(){let _=include_bytes!(\"missing\");}", &mut files).is_err());
    Ok(())
}

#[test]
fn resource_paths_follow_physical_call_causes_and_requested_source_paths() -> Result<(), String> {
    let mut files = Files::default();
    files
        .resources
        .insert("/crate/text.txt".into(), b"root".to_vec());
    files
        .resources
        .insert("/crate/sub/text.txt".into(), b"child".to_vec());
    files.sources.insert("/crate/sub/defs.rs".into(), "#[macro_export] macro_rules! text{()=>{include_str!(\"text.txt\")}} macro_rules! make{($e:expr)=>{#[macro_export] macro_rules! captured{()=>{$e}}}} make!(include_str!(\"text.txt\"));".into());
    let source =
        "#[path=\"sub/defs.rs\"] mod defs;fn f(){let _=crate::text!();let _=crate::captured!();}";
    assert_eq!(
        builtin_strings(&analyze(source, &mut files)?)?,
        ["root", "root"]
    );
    let source = "fn f(){let _=include_str!(\"text.txt\");}";
    let result = expansion::analyze(
        InputFile {
            identity: "stdin.rs".into(),
            path: "<anon>".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut files,
        &Config::default(),
    );
    assert!(result.is_err());
    Ok(())
}

#[test]
fn source_includes_cache_syntax_but_expand_each_receiving_scope() -> Result<(), String> {
    let mut files = Files::default();
    files.sources.insert(
        "/crate/shared.rs".into(),
        "pub fn answer()->i32{value!()}".into(),
    );
    let report = analyze(
        "macro_rules! value{()=>{1}} mod first{include!(\"shared.rs\");} macro_rules! value{()=>{2}} mod second{include!(concat!(\"shared\",\".rs\"));}",
        &mut files,
    )?;
    assert_eq!(files.reads, [PathBuf::from("/crate/shared.rs")]);
    let calls = report
        .invocations
        .iter()
        .filter(|call| {
            matches!(
                report.definitions[call.definition].body,
                expansion::Body::Builtin(rustgm::expand::builtin::Builtin::Include)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.parser_calls)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    let values = report
        .invocations
        .iter()
        .filter(|call| {
            report
                .package
                .names
                .get(report.definitions[call.definition].name)
                == "value"
        })
        .map(|call| {
            let unit = report.package.unit(call.node.file);
            let rustgm::syntax::ast::Kind::Literal(index) =
                unit.configured.ast.node(call.node.node).kind
            else {
                return Err("expected integer syntax".to_owned());
            };
            let rustgm::literal::Value::Integer(value) =
                unit.literal(index).map_err(|e| e.message)?
            else {
                return Err("expected integer literal".to_owned());
            };
            Ok(value.magnitude)
        })
        .collect::<Result<Vec<_>, String>>()?;
    assert_eq!(values, [1, 2]);
    assert_eq!(report.package.units.len(), 2);
    assert!(report.package.units[1].origins.is_empty());
    let report = rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    assert!(report.resolution.expansions.is_empty());
    Ok(())
}

#[test]
fn included_modules_take_directory_ownership_and_restore_the_caller() -> Result<(), String> {
    let mut files = Files::default();
    for (path, source) in [
        (
            "/crate/generated/part.rs",
            "mod child; mod nested{mod leaf;} fn location()-> &'static str{concat!(file!(),\":\",line!(),\":\",module_path!())}",
        ),
        ("/crate/generated/child.rs", "pub fn child(){}"),
        ("/crate/generated/nested/leaf.rs", "pub fn leaf(){}"),
        ("/crate/after.rs", "pub fn after(){}"),
    ] {
        files.sources.insert(path.into(), source.into());
    }
    let report = analyze("include!(\"generated/part.rs\"); mod after;", &mut files)?;
    assert_eq!(report.package.modules.len(), 5);
    assert!(builtin_strings(&report)?.contains(&"/crate/generated/part.rs:1:lib".to_owned()));
    assert!(
        files
            .reads
            .contains(&PathBuf::from("/crate/generated/child.rs"))
    );
    assert!(
        files
            .reads
            .contains(&PathBuf::from("/crate/generated/nested/leaf.rs"))
    );
    assert!(files.reads.contains(&PathBuf::from("/crate/after.rs")));
    rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn included_expressions_are_unhygienic_and_preserve_decoded_literals() -> Result<(), String> {
    let mut files = Files::default();
    files.sources.insert(
        "/crate/expr.rs".into(),
        "{let _=\"a\\nb\";let _=b\"\\xff\";x + 1}".into(),
    );
    let report = analyze(
        "fn f(x:i32)->i32{include!(\"expr.rs\")} fn g(x:i32)->i32{include!(\"expr.rs\")}",
        &mut files,
    )?;
    assert_eq!(files.reads.len(), 1);
    let unit = &report.package.units[0];
    let strings = (0..unit.configured.ast.literals.len())
        .map(|index| {
            unit.literal(u32::try_from(index).map_err(|e| e.to_string())?)
                .map_err(|e| e.message.to_owned())
        })
        .collect::<Result<Vec<_>, String>>()?;
    assert_eq!(strings.iter().filter(|value| matches!(value, rustgm::literal::Value::String(bytes) if bytes.as_ref() == b"a\nb")).count(), 2);
    assert_eq!(strings.iter().filter(|value| matches!(value, rustgm::literal::Value::ByteString(bytes) if bytes.as_ref() == [255])).count(), 2);
    rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn include_context_restrictions_and_recursive_inclusion_are_diagnosed() -> Result<(), String> {
    for (part, source) in [
        ("42; 7", "fn f()->i32{include!(\"part.rs\")}"),
        ("let x=1;x;", "fn f(){include!(\"part.rs\");}"),
        ("fn made(){}", "fn f(){include!(\"part.rs\");}"),
        ("i32", "type T=include!(\"part.rs\");"),
        ("1", "fn f(){match 1{include!(\"part.rs\")=>(),_=>()}}"),
        ("#![allow(unused)] fn made(){}", "include!(\"part.rs\");"),
        ("#!/bin/rust\n42", "fn f()->i32{include!(\"part.rs\")}"),
        ("fn made(){}", "trait T{include!(\"part.rs\");}"),
        ("fn made(){}", "struct S;impl S{include!(\"part.rs\");}"),
        ("fn made();", "unsafe extern \"C\"{include!(\"part.rs\");}"),
    ] {
        let mut files = Files::default();
        files.sources.insert("/crate/part.rs".into(), part.into());
        assert!(analyze(source, &mut files).is_err(), "{part}: {source}");
    }
    let mut files = Files::default();
    files
        .sources
        .insert("/crate/part.rs".into(), "include!(\"part.rs\");".into());
    let error = analyze(
        "#![recursion_limit=\"8\"] include!(\"part.rs\");",
        &mut files,
    )
    .err()
    .ok_or("expected recursive include failure")?;
    assert!(error.contains("recursion exceeds 8"), "{error}");
    assert_eq!(files.reads.len(), 1);
    files
        .sources
        .insert("/crate/part.rs".into(), "#!/bin/rust\nfn made(){}".into());
    rustgm::resolve::analyze_expanded(analyze("include!(\"part.rs\");", &mut files)?)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn included_macros_wait_for_eager_dependencies_and_keep_physical_errors() -> Result<(), String> {
    let mut files = Files::default();
    files
        .sources
        .insert("/crate/path.rs".into(), "name!()".into());
    files
        .sources
        .insert("/crate/part.rs".into(), "fn f()->i32{missing!()}".into());
    let source = "macro_rules! name{()=>{\"part.rs\"}} include!(include!(\"path.rs\"));";
    let error = expansion::analyze(
        InputFile {
            identity: "/crate/lib.rs".into(),
            path: "/crate/lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut files,
        &Config::default(),
    )
    .err()
    .ok_or("expected missing macro failure")?;
    assert!(error.message.contains("unresolved macro"), "{error}");
    assert_eq!(error.file.source(), Some(resolve::unit::FileId(2)));
    assert_eq!(error.span.start, 12);
    assert_eq!(error.expansions.len(), 1);
    assert_eq!(error.expansions[0].macro_name, "include");
    files
        .sources
        .insert("/crate/part.rs".into(), "fn f()->i32{42}".into());
    rustgm::resolve::analyze_expanded(analyze(source, &mut files)?).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn canonical_include_aliases_reuse_parsing_and_keep_requested_file_names() -> Result<(), String> {
    let mut files = Files::default();
    for name in ["first", "second"] {
        let path = PathBuf::from(format!("/crate/{name}/part.rs"));
        files.sources.insert(path.clone(), "file!()".into());
        files.identities.insert(path, "/physical/part.rs".into());
    }
    let report = analyze(
        "fn f(){let _=include!(\"first/part.rs\");let _=include!(\"second/part.rs\");}",
        &mut files,
    )?;
    let calls = report
        .invocations
        .iter()
        .filter(|call| {
            matches!(
                report.definitions[call.definition].body,
                expansion::Body::Builtin(rustgm::expand::builtin::Builtin::Include)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.parser_calls)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    let values = builtin_strings(&report)?;
    assert!(values.contains(&"/crate/first/part.rs".to_owned()));
    assert!(values.contains(&"/crate/second/part.rs".to_owned()));
    assert_eq!(files.reads.len(), 2);
    rustgm::resolve::analyze_expanded(report).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn included_ordinary_name_errors_keep_the_include_invocation_trace() -> Result<(), String> {
    let mut files = Files::default();
    files
        .sources
        .insert("/crate/part.rs".into(), "missing".into());
    let report = analyze("fn f()->i32{include!(\"part.rs\")}", &mut files)?;
    let error = rustgm::resolve::analyze_expanded(report)
        .err()
        .ok_or("expected ordinary name failure")?;
    assert_eq!(error.file.source().map(|file| file.0), Some(1));
    assert_eq!(error.span.start, 0);
    assert_eq!(error.expansions.len(), 1);
    assert_eq!(error.expansions[0].macro_name, "include");
    assert_eq!(error.expansions[0].call.file.0, 0);
    assert_eq!(error.expansions[0].call.span.start, 12);
    Ok(())
}
