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
        let result = analyze(source, &mut Files::default());
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
        let result = analyze(source, &mut Files::default());
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
        let result = analyze(source, &mut Files::default());
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
        let result = analyze(source, &mut Files::default());
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
