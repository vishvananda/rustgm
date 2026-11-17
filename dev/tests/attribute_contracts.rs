//! Retained attribute contracts use independently declared source and warning outcomes.
use rustgm::{
    expand::config::Config,
    lowir::{
        data::{GlobalKind, Item, Target},
        program::Program,
    },
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::model::Declarations,
    support::source::SourceFile,
};
use std::{fmt::Write as _, path::Path};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Case {
    name: &'static str,
    path: &'static str,
    codes: &'static [&'static str],
    error: Option<&'static str>,
    stderr: &'static str,
    native_tail: Option<&'static str>,
}
const CASES: &[Case] = &[
    Case {
        name: "pa10-function-discard-default",
        path: "pa10/tests/spec/must-use/function-discard-default.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &["unused_must_use"],
        error: None,
        stderr: "warning[unused_must_use]: file 0: 110: unused must_use result\n",
    },
    Case {
        name: "pa10-function-discard-deny",
        path: "pa10/tests/spec/must-use/function-discard-deny.rs",
        native_tail: None,
        codes: &[],
        error: Some("unused must_use result"),
        stderr: "",
    },
    Case {
        name: "pa10-function-discard-warn",
        path: "pa10/tests/spec/must-use/function-discard-warn.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &["unused_must_use"],
        error: None,
        stderr: "warning[unused_must_use]: file 0: 152: unused must_use result: retain result\n",
    },
    Case {
        name: "pa10-function-discard-allow",
        path: "pa10/tests/spec/must-use/function-discard-allow.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-function-explicit-let",
        path: "pa10/tests/spec/must-use/function-explicit-let.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-function-pointer-discard",
        path: "pa10/tests/spec/must-use/function-pointer-discard.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-direct-item-variable-discard",
        path: "pa10/tests/spec/must-use/direct-item-variable-discard.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-outer-deny-inner-allow",
        path: "pa10/tests/spec/must-use/outer-deny-inner-allow.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-outer-forbid-inner-allow",
        path: "pa10/tests/spec/must-use/outer-forbid-inner-allow.rs",
        native_tail: None,
        codes: &[],
        error: Some("forbidden unused_must_use lint cannot be weakened"),
        stderr: "",
    },
    Case {
        name: "pa10-outer-forbid-inner-warn",
        path: "pa10/tests/spec/must-use/outer-forbid-inner-warn.rs",
        native_tail: None,
        codes: &[],
        error: Some("forbidden unused_must_use lint cannot be weakened"),
        stderr: "",
    },
    Case {
        name: "pa10-duplicate-default-warn",
        path: "pa10/tests/spec/must-use/duplicate-default-warn.rs",
        native_tail: Some("fn main(){check(f()==1u32);}"),
        codes: &["unused_attributes"],
        error: None,
        stderr: "warning[unused_attributes]: file 0: 32: duplicate must_use attribute\n",
    },
    Case {
        name: "pa10-duplicate-deny",
        path: "pa10/tests/spec/must-use/duplicate-deny.rs",
        native_tail: None,
        codes: &[],
        error: Some("duplicate must_use attribute"),
        stderr: "",
    },
    Case {
        name: "pa10-malformed-list",
        path: "pa10/tests/spec/must-use/malformed-list.rs",
        native_tail: None,
        codes: &[],
        error: Some("must_use attribute requires no arguments or one string literal message"),
        stderr: "",
    },
    Case {
        name: "pa10-nonstring-message",
        path: "pa10/tests/spec/must-use/nonstring-message.rs",
        native_tail: None,
        codes: &[],
        error: Some("must_use message requires a string literal"),
        stderr: "",
    },
    Case {
        name: "pa10-unsafe-wrapper",
        path: "pa10/tests/spec/must-use/unsafe-wrapper.rs",
        native_tail: None,
        codes: &[],
        error: Some("must_use attribute cannot use an unsafe wrapper"),
        stderr: "",
    },
    Case {
        name: "pa12-struct-discard-deny",
        path: "pa12/tests/spec/must-use/struct-discard-deny.rs",
        native_tail: None,
        codes: &[],
        error: Some("unused must_use result"),
        stderr: "",
    },
    Case {
        name: "pa12-struct-explicit-let",
        path: "pa12/tests/spec/must-use/struct-explicit-let.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa12-enum-discard-deny",
        path: "pa12/tests/spec/must-use/enum-discard-deny.rs",
        native_tail: None,
        codes: &[],
        error: Some("unused must_use result"),
        stderr: "",
    },
    Case {
        name: "pa12-enum-explicit-let",
        path: "pa12/tests/spec/must-use/enum-explicit-let.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa12-nominal-tuple-discard",
        path: "pa12/tests/spec/must-use/nominal-tuple-discard.rs",
        native_tail: None,
        codes: &[],
        error: Some("unused must_use result"),
        stderr: "",
    },
    Case {
        name: "pa12-nominal-empty-array-discard",
        path: "pa12/tests/spec/must-use/nominal-empty-array-discard.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa19-trait-concrete-discard",
        path: "pa19/tests/spec/must-use/trait-concrete-discard.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa19-trait-method-discard",
        path: "pa19/tests/spec/must-use/trait-method-discard.rs",
        native_tail: None,
        codes: &[],
        error: Some("unused must_use result"),
        stderr: "",
    },
    Case {
        name: "pa19-inherent-method-discard",
        path: "pa19/tests/spec/must-use/inherent-method-discard.rs",
        native_tail: None,
        codes: &[],
        error: Some("unused must_use result"),
        stderr: "",
    },
    Case {
        name: "pa21-trait-opaque-discard",
        path: "pa21/tests/spec/must-use/trait-opaque-discard.rs",
        native_tail: None,
        codes: &[],
        error: Some("unused must_use result"),
        stderr: "",
    },
    Case {
        name: "pa21-trait-opaque-explicit-let",
        path: "pa21/tests/spec/must-use/trait-opaque-explicit-let.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa24-closure-expression-warn",
        path: "pa24/tests/spec/must-use/closure-expression-warn.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &["unused_must_use"],
        error: None,
        stderr: "warning[unused_must_use]: file 0: 49: unused closure that must be used\n",
    },
    Case {
        name: "pa24-closure-variable-warn",
        path: "pa24/tests/spec/must-use/closure-variable-warn.rs",
        native_tail: Some("fn main(){probe();}"),
        codes: &["unused_must_use"],
        error: None,
        stderr: "warning[unused_must_use]: file 0: 66: unused closure that must be used\n",
    },
    Case {
        name: "pa10-inline-function-hint",
        path: "pa10/tests/spec/inline-attributes/function-hint.rs",
        native_tail: Some("fn main(){check(f()==1u32);}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-inline-function-always",
        path: "pa10/tests/spec/inline-attributes/function-always.rs",
        native_tail: Some("fn main(){check(f()==1u32);}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-inline-function-never",
        path: "pa10/tests/spec/inline-attributes/function-never.rs",
        native_tail: Some("fn main(){check(f()==1u32);}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-inline-duplicate-default",
        path: "pa10/tests/spec/inline-attributes/duplicate-default.rs",
        native_tail: Some("fn main(){check(f()==1u32);}"),
        codes: &["unused_attributes"],
        error: None,
        stderr: "warning[unused_attributes]: file 0: 47: duplicate inline attribute\n",
    },
    Case {
        name: "pa10-inline-duplicate-deny",
        path: "pa10/tests/spec/inline-attributes/duplicate-deny.rs",
        native_tail: None,
        codes: &[],
        error: Some("duplicate inline attribute"),
        stderr: "",
    },
    Case {
        name: "pa10-inline-empty-list",
        path: "pa10/tests/spec/inline-attributes/empty-list.rs",
        native_tail: None,
        codes: &[],
        error: Some("inline attribute requires exactly always or never"),
        stderr: "",
    },
    Case {
        name: "pa10-inline-unsafe-wrapper",
        path: "pa10/tests/spec/inline-attributes/unsafe-wrapper.rs",
        native_tail: None,
        codes: &[],
        error: Some("inline attribute cannot use an unsafe wrapper"),
        stderr: "",
    },
    Case {
        name: "pa19-inline-trait-required",
        path: "pa19/tests/spec/inline-attributes/trait-required.rs",
        native_tail: Some(
            "struct NativeS;impl T for NativeS{fn f(&self)->u32{7u32}}fn main(){check(NativeS.f()==7u32);}",
        ),
        codes: &["unused_attributes"],
        error: None,
        stderr: "warning[unused_attributes]: file 0: 46: inline attribute on a function without a body has no effect\n",
    },
    Case {
        name: "pa19-inline-trait-provided",
        path: "pa19/tests/spec/inline-attributes/trait-provided.rs",
        native_tail: Some(
            "struct NativeS;impl T for NativeS{}fn main(){check(NativeS.f()==1u32);}",
        ),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-bootstrap-import-alias",
        path: "pa10/tests/spec/test-removal/bootstrap-import-alias.rs",
        native_tail: Some("fn native_alive()->u32{7u32}fn main(){check(native_alive()==7u32);}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-bootstrap-qualified-reexport",
        path: "pa10/tests/spec/test-removal/bootstrap-qualified-reexport.rs",
        native_tail: Some("fn native_alive()->u32{7u32}fn main(){check(native_alive()==7u32);}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-bootstrap-forward-alias-chain",
        path: "pa10/tests/spec/test-removal/bootstrap-forward-alias-chain.rs",
        native_tail: Some("fn native_alive()->u32{7u32}fn main(){check(native_alive()==7u32);}"),
        codes: &[],
        error: None,
        stderr: "",
    },
    Case {
        name: "pa10-removed-name-does-not-reserve-binding",
        path: "pa10/tests/spec/test-removal/removed-name-does-not-reserve-binding.rs",
        native_tail: Some("fn main(){check(live()==7u32);}"),
        codes: &[],
        error: None,
        stderr: "",
    },
];
fn source(case: &Case) -> std::io::Result<String> {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(case.path),
    )
}
pub(crate) fn analyze(source: &str) -> Result<(Report, Declarations), resolve::package::Error> {
    analyze_with_config(source, &Config::default())
}
pub(crate) fn analyze_with_config(
    source: &str,
    config: &Config,
) -> Result<(Report, Declarations), resolve::package::Error> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "attributes.rs".into(),
            path: "attributes.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec()).map_err(|message| {
                resolve::package::Error::at(resolve::unit::FileId(0), 0, message)
            })?,
        },
        &mut NoFiles,
        config,
    )
    .and_then(resolve::analyze_expanded)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    Ok((report, analysis.facts))
}
fn warning_text(facts: &Declarations) -> TestResult<String> {
    let mut text = String::new();
    for warning in facts.warnings() {
        writeln!(
            text,
            "warning[{}]: {}",
            warning.code.name(),
            warning.diagnostic
        )?;
    }
    Ok(text)
}
#[test]
fn actual_owner_results_and_lexical_lint_controls_keep_persistent_diagnostics() -> TestResult {
    for case in CASES {
        let input = source(case)?;
        let result = analyze(&input);
        if let Some(expected) = case.error {
            let error = result
                .err()
                .ok_or_else(|| format!("{} accepted", case.name))?;
            assert!(
                error.to_string().contains(expected),
                "{}: {error}",
                case.name
            );
            continue;
        }
        let (report, facts) = result.map_err(|error| format!("{}: {error}", case.name))?;
        let codes: Vec<_> = facts
            .warnings()
            .map(|warning| warning.code.name())
            .collect();
        assert_eq!(codes, case.codes, "{}", case.name);
        assert_eq!(warning_text(&facts)?, case.stderr, "{}", case.name);
        assert_eq!(
            warning_text(&facts)?,
            case.stderr,
            "{} repeated borrowed view",
            case.name
        );
        assert_eq!(facts.must_use_work().warnings, codes.len(), "{}", case.name);
        // A removed Test body with missing names never creates an invocation.
        if case.path.contains("/test-removal/") {
            assert!(report.invocations.is_empty(), "{}", case.name);
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host linker"]
fn default_warning_and_explicit_discard_programs_keep_native_behavior() -> TestResult {
    let mut inputs = Vec::new();
    for case in CASES.iter().filter(|case| case.error.is_none()) {
        let semantic_source = source(case)?;
        let prefix = semantic_source
            .strip_suffix("fn main(){}\n")
            .ok_or("native fixture has no declared empty main tail")?;
        let tail = case
            .native_tail
            .ok_or("accepted attribute case has no declared reached native wrapper")?;
        let reached = format!("{prefix}{}{tail}\n", crate::native_references::CHECK);
        inputs.push((case, reached));
    }
    let cases: Vec<_> = inputs
        .iter()
        .map(|(case, source)| (case.name, source.as_str(), false, case.stderr.as_bytes()))
        .collect();
    crate::native_references::check_literal_cases_with_warnings(
        "parallel/pa31/attribute-contracts-native",
        &cases,
    )
}

fn query_instability_fixture(name: &str) -> TestResult<String> {
    Ok(crate::resolve::fixture(&format!(
        "pa31/tests/spec/query-instability/{name}.rs"
    ))?)
}

#[test]
fn query_instability_checks_source_owners_form_and_exact_spelling() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    for (name, cause) in [
        ("valid", None),
        ("cfg-disabled", None),
        ("arguments", Some("requires a word attribute")),
        ("equals", Some("requires a word attribute")),
        ("unsafe-wrapper", Some("requires a word attribute")),
        ("wrong-owner", Some("requires an actual function or method")),
        (
            "foreign",
            Some("duplicate or foreign rustc_lint_query_instability"),
        ),
        (
            "duplicate",
            Some("duplicate or foreign rustc_lint_query_instability"),
        ),
        ("lookalike", Some("unresolved attribute macro")),
        ("qualified", Some("unresolved attribute macro")),
    ] {
        let input = query_instability_fixture(name)?;
        let suffix = input
            .strip_prefix("#![no_std]\n#![feature(rustc_attrs)]\n")
            .ok_or("fixture lost standalone prefix")?;
        let report =
            crate::resolve::source_provider(&(prefix.clone() + suffix), "query_instability");
        let outcome = report.and_then(|report| {
            rustgm::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        if let Some(cause) = cause {
            let error = outcome.err().ok_or("expected source rejection")?;
            assert!(error.contains(cause), "{name}: {error}");
        } else {
            outcome?;
        }
    }
    let missing = query_instability_fixture("missing-request")?;
    let suffix = missing
        .strip_prefix("#![no_std]\n")
        .ok_or("standalone profile")?;
    let report = crate::resolve::source_provider(
        &format!("#![feature(no_core)]\n#![no_core]\n{suffix}"),
        "query_instability_missing",
    )?;
    let error = rustgm::pipeline::analyze_source(&report)
        .err()
        .ok_or("missing internal attribute request accepted")?;
    assert!(
        error
            .to_string()
            .contains("requires its rustc_attrs source profile"),
        "{error}"
    );

    let valid = query_instability_fixture("valid")?;
    let suffix = valid
        .strip_prefix("#![no_std]\n")
        .ok_or("standalone profile")?;
    let error = crate::resolve::resolve(suffix, true)
        .err()
        .ok_or("ordinary marker resolved")?;
    assert!(error.contains("unresolved attribute macro"), "{error}");
    Ok(())
}

fn used_provider() -> TestResult<String> {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    Ok(format!(
        "#![feature(auto_traits)]\n{prefix}\n#[lang = \"sync\"]pub unsafe auto trait Sync {{}}\n"
    ))
}

fn lower_used(source: &str) -> TestResult<Program> {
    let mut report = crate::resolve::source_provider(source, "used_retention")?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    Ok(rustgm::lower::instances::build_prepared(
        &report,
        &mut facts,
        entry,
        analysis.prepared,
    )?)
}

#[test]
fn used_compiler_retains_the_private_static_and_its_function_relocation() -> TestResult {
    let prefix = used_provider()?;
    let program = lower_used(&format!(
        "#![feature(used_with_arg)]\n{prefix}
         fn used_dependency()->u64{{37}}
         #[used(compiler)] static KEPT: fn()->u64=used_dependency;
         static UNMARKED: u64=29;
         fn main(){{}}"
    ))?;
    let [kept] = program.globals.as_slice() else {
        return Err("expected exactly one global from the actual used root".into());
    };
    assert_eq!(kept.kind, GlobalKind::Definition);
    let items = kept.items.ok_or("retained static has no initializer")?;
    let [
        Item::Address {
            target: Target::Function(dependency),
            addend: 0,
        },
    ] = &program.data[items.start as usize..items.end as usize]
    else {
        return Err("expected the retained static's function relocation".into());
    };
    let function = &program.symbols[dependency.0 as usize];
    assert!(function.body.is_some());
    assert!(!function.entry);
    Ok(())
}

#[test]
fn used_modes_check_source_owner_feature_and_linker_requirement() -> TestResult {
    let prefix = used_provider()?;
    let feature = "#![feature(used_with_arg)]";
    for (header, item, cause) in [
        (
            "",
            "#[used(compiler)]static ITEM:u64=7;",
            "requires this crate's used_with_arg",
        ),
        (
            feature,
            "#[used(compiler)]fn item(){}",
            "requires an actual static definition",
        ),
        (
            feature,
            "#[used(compiler, linker)]static ITEM:u64=7;",
            "malformed `used`",
        ),
        (
            "#![feature(used_with_arg)]#![deny(unused_attributes)]",
            "#[used(compiler)]#[used(compiler)]static ITEM:u64=7;",
            "duplicate `used`",
        ),
        (
            "",
            "#[used]static ITEM:u64=7;",
            "used linker retention is not implemented",
        ),
        (
            feature,
            "#[used(linker)]static ITEM:u64=7;",
            "used linker retention is not implemented",
        ),
        (
            feature,
            "#[used(compiler)]#[used(linker)]static ITEM:u64=7;",
            "used linker retention is not implemented",
        ),
    ] {
        let error = lower_used(&format!("{header}\n{prefix}\n{item}\nfn main(){{}}"))
            .err()
            .ok_or("expected retention rejection")?;
        assert!(error.to_string().contains(cause), "{item}: {error}");
    }
    let program = lower_used(&format!(
        "{prefix}\n#[cfg_attr(any(), used(compiler))]static ITEM:u64=7;\nfn main(){{}}"
    ))?;
    assert!(program.globals.is_empty());
    Ok(())
}

#[test]
fn gnu_init_arrays_keep_exact_names_and_typed_function_relocations() -> TestResult {
    let prefix = used_provider()?;
    let program = lower_used(&format!(
        "{prefix}
         #[used] #[unsafe(link_section = \".init_array.90\")]
         static FIRST: extern \"C\" fn() = first;
         extern \"C\" fn first() {{}}
         #[cfg(all(target_os = \"linux\", target_env = \"gnu\"))]
         #[used] #[unsafe(link_section = \".init_array.00099\")]
         static ARGV: extern \"C\" fn(i32,*const *const u8,*const *const u8) = argv;
         extern \"C\" fn argv(_:i32,_:*const *const u8,_:*const *const u8) {{}}
         #[cfg(target_arch = \"aarch64\")]
         #[used] #[unsafe(link_section = \".init_array.12\")]
         static INACTIVE: extern \"C\" fn() = first;
         fn main(){{}}"
    ))?;
    let mut observed = Vec::new();
    for (index, global) in program.globals.iter().enumerate() {
        let id = rustgm::lowir::data::GlobalId(u32::try_from(index)?);
        let section = program
            .init_array_section(id)
            .ok_or("lost init-array placement")?;
        let items = global.items.ok_or("missing constructor initializer")?;
        let [
            Item::Address {
                target: Target::Function(symbol),
                addend: 0,
            },
        ] = &program.data[items.start as usize..items.end as usize]
        else {
            return Err("constructor has no exact function relocation".into());
        };
        assert!(program.symbols[symbol.0 as usize].body.is_some());
        observed.push(section);
    }
    observed.sort_unstable();
    assert_eq!(observed, [".init_array.00099", ".init_array.90"]);
    let mut lowir = Vec::new();
    rustgm::lowir::writer::write(&program, &mut lowir)?;
    let lowir = String::from_utf8(lowir)?;
    assert!(lowir.contains("section=.init_array.00099]"), "{lowir}");
    assert!(lowir.contains("section=.init_array.90]"), "{lowir}");
    Ok(())
}

#[test]
fn gnu_init_arrays_require_unsafe_literal_owner_and_actual_c_callable() -> TestResult {
    let prefix = used_provider()?;
    for (attribute, item, cause) in [
        (
            "#[link_section = \".init_array.90\"]",
            "static INIT: extern \"C\" fn() = target;",
            "unsafe attribute used without unsafe",
        ),
        (
            "#[unsafe(link_section = 7)]",
            "static INIT: extern \"C\" fn() = target;",
            "link_section requires a string literal",
        ),
        (
            "#[unsafe(link_section = \".init_array.90\")]",
            "static INIT: u64 = 7;",
            "requires an actual function address",
        ),
        (
            "#[unsafe(link_section = \".init_array.90\")]",
            "static INIT: fn() = rust_fn;",
            "requires a C void() or glibc three-argument signature",
        ),
        (
            "#[unsafe(link_section = \".data.nonstartup\")]",
            "static INIT: u64 = 7;",
            "only GNU init-array global sections are supported",
        ),
    ] {
        let error = lower_used(&format!(
            "{prefix}\n#[used] {attribute} {item}\nextern \"C\" fn target(){{}}\nfn rust_fn(){{}}\nfn main(){{}}"
        ))
        .err()
        .ok_or("unsupported init-array declaration emitted")?;
        assert!(error.to_string().contains(cause), "{error}");
    }
    Ok(())
}

#[test]
fn source_foreign_effects_retain_distinct_canonical_function_assertions() -> TestResult {
    use rustgm::{resolve::model::DefId, sema::foreign::MemoryEffect};
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    let source = format!(
        "#![feature(ffi_const,ffi_pure)]\n{prefix}
         unsafe extern \"C\" {{
            #[link_name = \"__errno_location\"] #[unsafe(ffi_const)]
            pub safe fn errno_location() -> *mut i32;
            #[unsafe(ffi_pure)] pub fn observes(value:i32) -> i32;
            pub safe fn plain() -> i32;
         }}
         pub fn checked() -> *mut i32 {{ errno_location() }}"
    );
    let report = crate::resolve::source_provider(&source, "source_foreign_effects")?;
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    for (name, assertion) in [
        ("errno_location", Some(MemoryEffect::Const)),
        ("observes", Some(MemoryEffect::Pure)),
        ("plain", None),
        ("checked", None),
    ] {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|definition| report.package.names.get(definition.name) == name)
            .ok_or("missing exact function declaration")?;
        assert_eq!(
            facts.foreign_memory_effect(DefId(u32::try_from(index)?)),
            assertion
        );
    }
    Ok(())
}

#[test]
fn source_foreign_effects_require_exact_features_forms_owners_and_call_safety() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    let both = "#![feature(ffi_const,ffi_pure)]";
    let rows = [
        (
            "",
            "unsafe extern \"C\" { #[unsafe(ffi_const)] fn f(); }",
            "this crate's ffi_const feature",
        ),
        (
            "#![feature(ffi_const)]",
            "unsafe extern \"C\" { #[unsafe(ffi_pure)] fn f(); }",
            "this crate's ffi_pure feature",
        ),
        (
            both,
            "unsafe extern \"C\" { #[ffi_const] fn f(); }",
            "unsafe attribute used without unsafe",
        ),
        (
            both,
            "unsafe extern \"C\" { #[unsafe(ffi_const(0))] fn f(); }",
            "requires a word attribute",
        ),
        (
            both,
            "#[unsafe(ffi_const)] fn f() {}",
            "requires an actual foreign function",
        ),
        (
            both,
            "unsafe extern \"C\" { #[unsafe(ffi_const)] static F:i32; }",
            "requires an actual foreign function",
        ),
        (
            both,
            "unsafe extern \"C\" { #[unsafe(ffi_const)] #[unsafe(ffi_const)] fn f(); }",
            "duplicate foreign effect",
        ),
        (
            both,
            "unsafe extern \"C\" { #[unsafe(ffi_pure)] #[unsafe(ffi_const)] fn f(); }",
            "cannot be `#[ffi_pure]`",
        ),
        (
            both,
            "unsafe extern \"C\" { #[unsafe(ffi_const)] fn f(); } pub fn call() { f(); }",
            "call to unsafe function requires an unsafe context",
        ),
    ];
    for (header, suffix, cause) in rows {
        let report = crate::resolve::source_provider(
            &format!("{header}\n{prefix}\n{suffix}"),
            "source_foreign_effect_rejection",
        )?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("foreign effect accepted an opposing case")?;
        assert!(error.to_string().contains(cause), "{suffix}: {error}");
    }
    Ok(())
}

#[test]
fn source_cfi_encoding_retains_actual_local_type_without_changing_representation() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    let source = format!(
        "#![feature(cfi_encoding)]\n{prefix}
         pub fn register() {{
             #[cfi_encoding = \"i\"] #[repr(transparent)] pub struct Code(pub i32);
             let _ = Code(7);
         }}
         #[cfi_encoding = r#\" enum value \"#] pub enum Tag {{ Value }}
         #[cfi_encoding = \"U\"] pub union Data {{ value: i32 }}
         pub fn irrelevant() {{}}"
    );
    let report = crate::resolve::source_provider(&source, "cfi_encoding_retention")?;
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    for (name, expected) in [("Code", "i"), ("Tag", " enum value "), ("Data", "U")] {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|definition| {
                definition.namespace == rustgm::resolve::model::Namespace::Type
                    && report.package.names.get(definition.name) == name
            })
            .ok_or("missing canonical type")?;
        let id = rustgm::resolve::model::DefId(u32::try_from(index)?);
        assert_eq!(facts.cfi_encoding(id), Some(expected));
    }
    Ok(())
}

#[test]
fn source_cfi_encoding_requires_own_feature_payload_and_lint_target_policy() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    for (features, item, cause) in [
        (
            "",
            "#[cfi_encoding=\"i\"] struct Marked;",
            "this crate's cfi_encoding feature",
        ),
        (
            "#![feature(cfi_encoding)]",
            "#[cfi_encoding=7] struct Marked;",
            "requires a string literal",
        ),
        (
            "#![feature(cfi_encoding)]",
            "#[cfi_encoding=\" \"] struct Marked;",
            "requires a nonempty string literal",
        ),
        (
            "#![feature(cfi_encoding)]",
            "#[unsafe(cfi_encoding=\"i\")] struct Marked;",
            "requires a normal outer attribute",
        ),
        (
            "#![feature(cfi_encoding)]",
            "#[cfi_encoding=\"i\"] #[cfi_encoding=\"j\"] struct Marked;",
            "duplicate cfi_encoding",
        ),
        (
            "#![feature(cfi_encoding)]#![deny(unused_attributes)]",
            "#[cfi_encoding=\"i\"] fn marked(){}",
            "applies only to struct",
        ),
    ] {
        let report = crate::resolve::source_provider(
            &format!("{features}\n{prefix}\n{item}"),
            "cfi_encoding_negative",
        )?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("opposing encoding admitted")?;
        assert!(error.to_string().contains(cause), "{error}");
    }
    let report = crate::resolve::source_provider(
        &format!("{prefix}\n#[cfg_attr(any(),cfi_encoding=\"i\")] struct Marked;"),
        "cfi_encoding_disabled",
    )?;
    rustgm::pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
fn foreign_effect_macro_grants_follow_the_attribute_wrapper() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    for (matcher, attribute, invocation, accepted) in [
        ("$effect:ident", "#[unsafe($effect)]", "ffi_const", true),
        (
            "($($attribute:tt)*)",
            "$($attribute)*",
            "(#[unsafe(ffi_const)])",
            false,
        ),
    ] {
        let source = format!(
            "#![feature(allow_internal_unstable)]\n{prefix}
             #[allow_internal_unstable(ffi_const)]
             macro_rules! ffi {{
                 ({matcher}) => {{
                     unsafe extern \"C\" {{ {attribute} pub fn f(); }}
                 }};
             }}
             ffi!({invocation});"
        );
        let report = crate::resolve::source_provider(&source, "foreign_effect_macro_span")?;
        let result = rustgm::pipeline::analyze_source(&report);
        if accepted {
            result?;
        } else {
            let error = result
                .err()
                .ok_or("caller-written wrapper acquired an allowance")?;
            assert!(
                error.to_string().contains("this crate's ffi_const feature"),
                "{error}"
            );
        }
    }
    Ok(())
}
