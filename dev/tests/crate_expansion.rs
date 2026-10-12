use rustgm::{
    expand::config::Config,
    resolve::{
        self, expansion,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
struct Files {
    files: HashMap<PathBuf, String>,
    reads: Vec<PathBuf>,
}
impl Sources for Files {
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        self.reads.push(path.to_owned());
        self.files
            .get(path)
            .map(|text| {
                Ok(InputFile {
                    identity: path.to_owned(),
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
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        files,
        &Config::default(),
    )
    .map_err(|e| format!("{e:?}"))
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
    files.files.insert(
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
    files.files.insert("/crate/shared.rs".into(), shared.into());
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
