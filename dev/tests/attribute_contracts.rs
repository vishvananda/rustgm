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

#[test]
fn source_weak_statics_keep_actual_imports_and_the_callback_signature() -> TestResult {
    use rustgm::{
        resolve::model::DefId,
        sema::foreign::WeakStatic,
        support::linkage::{Linkage, Naming},
        syntax::item::Safety,
        types::{Kind, Primitive, TypeId, callable::Convention},
    };
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    let source = format!(
        r#"#![feature(linkage)]
{prefix}
#[lang = "Option"]
pub enum Option<T> {{ #[lang = "None"] None, #[lang = "Some"] Some(T) }}
unsafe extern "C" {{
    #[linkage = "extern_weak"]
    #[link_name = "host_weak_data"]
    static WEAK_DATA: *mut u8;
    #[cfg_attr(all(), linkage = "extern_weak")]
    static WEAK_CALL: Option<extern "C" fn(i32) -> i32>;
    #[cfg_attr(any(), linkage = "extern_weak")]
    static STRONG_DATA: *mut u8;
}}"#
    );
    let report = crate::resolve::source_provider(&source, "weak_static_headers")?;
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    let registry = facts.foreign.as_deref().ok_or("actual foreign registry")?;
    let global = |name| -> TestResult<_> {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|definition| report.package.names.get(definition.name) == name)
            .ok_or("missing actual foreign declaration")?;
        let id = DefId(u32::try_from(index)?);
        assert!(report.resolution.foreign_owners.contains_key(&id));
        assert!(registry.imports_global(id));
        let record = registry.global(id).ok_or("missing canonical global")?;
        assert_eq!(record.linkage, Linkage::Import);
        assert_eq!(record.safety, Safety::Default);
        assert!(!record.mutable);
        Ok(record)
    };
    let data = global("WEAK_DATA")?;
    assert_eq!(data.weak, Some(WeakStatic::Data));
    let Naming::Exact(symbol) = data.naming else {
        return Err("expected the exact native symbol".into());
    };
    assert_eq!(registry.names().get(symbol), Some("host_weak_data"));
    let Some(WeakStatic::Function(signature)) = global("WEAK_CALL")?.weak else {
        return Err("expected the canonical weak callback".into());
    };
    let Some(Kind::Function {
        convention: Convention::C,
        parameters,
        result,
        unsafe_: false,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("expected the actual safe C function pointer signature".into());
    };
    let i32_type = TypeId::primitive(Primitive::I32);
    assert_eq!(facts.types.list(parameters), Some([i32_type].as_slice()));
    assert_eq!(result, i32_type);
    assert!(global("STRONG_DATA")?.weak.is_none());
    Ok(())
}

#[test]
fn source_weak_statics_require_own_feature_shape_and_attribute_provenance() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    let feature = "#![feature(linkage)]";
    for (flags, input, cause) in [
        (
            "",
            r#"unsafe extern "C" { #[linkage = "extern_weak"] static W: *mut u8; }"#,
            "this crate's linkage feature",
        ),
        (
            feature,
            r#"unsafe extern "C" { #[linkage = "extern_weak"] static mut W: *mut u8; }"#,
            "extern mutable statics are not allowed",
        ),
        (
            feature,
            r#"unsafe extern "C" { #[linkage = "extern_weak"] static W: i32; }"#,
            "requires a raw pointer or the actual Option",
        ),
        (
            feature,
            r#"#[linkage = "extern_weak"] static W: *mut u8 = 0 as *mut u8;"#,
            "requires its actual foreign static owner",
        ),
        (
            feature,
            r#"unsafe extern "C" { #[linkage = "external_weak"] static W: *mut u8; }"#,
            "supports only extern_weak foreign statics",
        ),
    ] {
        let source = format!("{flags}\n{prefix}\n{input}");
        let report = crate::resolve::source_provider(&source, "weak_static_opponent")?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("accepted invalid weak static")?;
        assert!(error.to_string().contains(cause), "{error}");
    }
    for (matcher, attribute, invocation, accepted) in [
        ("", r#"#[linkage = "extern_weak"]"#, "", true),
        (
            "($($attribute:tt)*)",
            "$($attribute)*",
            r#"(#[linkage = "extern_weak"])"#,
            false,
        ),
    ] {
        let source = format!(
            "#![feature(allow_internal_unstable)]\n{prefix}
             #[allow_internal_unstable(linkage)]
             macro_rules! weak_static {{
                 ({matcher}) => {{
                     unsafe extern \"C\" {{ {attribute} static W: *mut u8; }}
                 }};
             }}
             weak_static!({invocation});"
        );
        let report = crate::resolve::source_provider(&source, "weak_static_macro_site")?;
        let result = rustgm::pipeline::analyze_source(&report);
        if accepted {
            let facts = result?.facts;
            let index = report
                .resolution
                .definitions
                .iter()
                .position(|definition| report.package.names.get(definition.name) == "W")
                .ok_or("macro grant did not produce its static")?;
            let record = facts
                .foreign
                .as_deref()
                .and_then(|registry| {
                    registry.global(rustgm::resolve::model::DefId(u32::try_from(index).ok()?))
                })
                .ok_or("macro grant did not bind the actual import")?;
            assert_eq!(record.weak, Some(rustgm::sema::foreign::WeakStatic::Data));
        } else {
            let error = result
                .err()
                .ok_or("caller-written weak marker acquired a macro allowance")?;
            assert!(
                error.to_string().contains("this crate's linkage feature"),
                "{error}"
            );
        }
    }
    Ok(())
}

#[test]
fn source_startup_demands_the_actual_specialization() -> TestResult {
    use rustgm::{
        lower::instances::{SourcePublication, build_source_library_profile},
        lowir::vocabulary::Type,
        pipeline::Analysis,
        support::bindings::LangItem,
    };
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    let source = format!(
        r#"{prefix}
#[lang = "termination"]
pub trait Termination {{
    fn report(self) -> i32;
}}
#[lang = "start"]
fn actual_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
    _sigpipe: u8,
) -> isize {{ main().report() as isize }}
pub struct Exit;
impl Termination for Exit {{ fn report(self) -> i32 {{ 23 }} }}
fn main() -> Exit {{ Exit }}"#
    );
    let mut report = crate::resolve::source_provider(&source, "source_startup_roles")?;
    let reserved = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("Source role inventory")?;
    for (role, name) in [
        (LangItem::Start, "actual_start"),
        (LangItem::Termination, "Termination"),
    ] {
        let declaration = reserved
            .language(role)
            .ok_or("actual reserved startup role")?;
        let definition = &report.resolution.definitions[declaration.definition.0 as usize];
        assert_eq!(definition.node, Some(declaration.origin));
        assert_eq!(report.package.names.get(definition.name), name);
    }
    let identity = report
        .package
        .source_core()
        .ok_or("Source owner")?
        .provider
        .clone();
    let Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let program = rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let main = program
        .symbols
        .iter()
        .find(|symbol| symbol.name == "main")
        .ok_or("missing native entry")?;
    assert_eq!(program.parameters(main.signature)?, &[Type::I32, Type::Ptr]);
    assert!(program.symbols.iter().any(|symbol| {
        symbol.body.is_some()
            && program.parameters(symbol.signature).ok()
                == Some(&[Type::Ptr, Type::I64, Type::Ptr, Type::U8])
    }));
    let Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let publication = build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )
    .err()
    .ok_or("startup roles silently escaped unsupported metadata")?;
    assert!(
        publication
            .to_string()
            .contains("selected Source startup roles require their checked artifact transport"),
        "{publication}"
    );
    Ok(())
}

#[test]
fn source_start_header_rejects_wrong_callback_pointee_and_formal() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    for (signature, cause) in [
        (
            "fn start<T>(_: fn() -> T, _: isize, _: *const *const u8, _: u8) -> i32",
            "source start requires fn(fn() -> T",
        ),
        (
            "fn start<T>(_: fn() -> T, _: isize, _: *const *mut u8, _: u8) -> isize",
            "source start requires fn(fn() -> T",
        ),
        (
            "fn start<T>(_: unsafe fn() -> T, _: isize, _: *const *const u8, _: u8) -> isize",
            "source start requires fn(fn() -> T",
        ),
        (
            "fn start<const N: usize>(_: fn() -> (), _: isize, _: *const *const u8, _: u8) -> isize",
            "exactly one owned type parameter",
        ),
    ] {
        let source = format!("{prefix}\n#[lang = \"start\"] {signature} {{ 0 }}");
        let report = crate::resolve::source_provider(&source, "source_start_header_opponents")?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("accepted wrong startup shape")?;
        assert!(error.to_string().contains(cause), "{error}");
    }
    Ok(())
}

fn minimal_runtime_source(tail: &str) -> TestResult<String> {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    Ok(format!("{prefix}\n{tail}"))
}

#[test]
fn source_begin_panic_runtime_demands_the_authored_generic_body() -> TestResult {
    use rustgm::lowir::program::{Operand, Operation};
    let body = r#"
pub trait Payload: Sized { fn byte(self) -> u8; }
impl Payload for u8 { fn byte(self) -> u8 { self } }
unsafe extern "C" { safe fn panic_observe(value: u8); }
#[lang = "begin_panic"]
#[rustc_do_not_const_check]
pub const fn renamed_begin<T: Payload>(payload: T) -> ! {
    panic_observe(payload.byte());
    loop {}
}
"#;
    let source = minimal_runtime_source(&format!("{body}\nfn main() {{ renamed_begin(17u8); }}"))?;
    let program = lower_used(&source)?;
    assert!(
        program
            .symbols
            .iter()
            .filter_map(|symbol| symbol.body.as_ref())
            .flat_map(|body| &body.instructions)
            .any(|instruction| matches!(instruction.operation,
                Operation::Call { callee: Operand::Symbol(target), .. }
                    if program.symbols[target.0 as usize].name == "panic_observe")),
        "runtime specialization lost the authored call to panic_observe"
    );

    let source = minimal_runtime_source(&format!("{body}\nfn main() {{ renamed_begin(17u16); }}"))?;
    let error = lower_used(&source)
        .err()
        .ok_or("runtime begin_panic did not require its authored payload bound")?;
    let message = error.to_string();
    assert!(
        message.contains("trait") || message.contains("bound"),
        "{message}"
    );
    Ok(())
}

#[test]
fn source_begin_panic_const_calls_preserve_payload_shape_reach_and_identity() -> TestResult {
    let header = r#"
fn runtime_only() {}
#[lang = "begin_panic"]
#[rustc_do_not_const_check]
pub const fn renamed_begin<T>(_payload: T) -> ! { runtime_only(); loop {} }
"#;
    for (tail, cause) in [
        (
            r#"const VALUE: u8 = if false { renamed_begin("message") } else { 19 };"#,
            None,
        ),
        (
            r#"const VALUE: u8 = renamed_begin("message");"#,
            Some("constant execution reached source begin_panic"),
        ),
        (
            "const fn checked_only() -> u8 { if false { renamed_begin(7u8) } else { 19 } }",
            Some("source begin_panic requires a string slice payload in const code"),
        ),
        (
            r#"const fn checked_only() -> u8 { if false { renamed_begin(&"message") } else { 19 } }"#,
            Some("source begin_panic requires a string slice payload in const code"),
        ),
        (
            "const fn begin_panic(value: u8) -> u8 { value } const VALUE: u8 = begin_panic(19);",
            None,
        ),
    ] {
        let source = minimal_runtime_source(&format!("{header}\n{tail}"))?;
        let report = crate::resolve::source_provider(&source, "begin_panic_const")?;
        let result = rustgm::pipeline::analyze_source(&report);
        if let Some(cause) = cause {
            let error = result
                .err()
                .ok_or("invalid const begin_panic was accepted")?;
            assert!(error.to_string().contains(cause), "{tail}: {error}");
        } else {
            result?;
        }
    }
    Ok(())
}

#[test]
fn selected_source_begin_panic_fences_both_writers() -> TestResult {
    use rustgm::{
        lower::instances::{SourcePublication, build_source_library_profile},
        pipeline::Analysis,
    };
    let source = minimal_runtime_source(
        r#"#[lang = "begin_panic"]
        #[rustc_do_not_const_check]
        const fn renamed_begin<T>(_payload: T) -> ! { loop {} }"#,
    )?;
    let report = crate::resolve::source_provider(&source, "begin_panic_transport")?;
    let identity = report
        .package
        .source_core()
        .ok_or("Source owner")?
        .provider
        .clone();
    let Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let scalar = rustgm::sema::crates::export(&report, &mut facts, identity.clone())
        .err()
        .ok_or("scalar writer would erase the selected Source begin_panic role")?;
    let source = build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )
    .err()
    .ok_or("Source writer would erase the selected begin_panic role")?;
    for error in [scalar, source] {
        assert!(
            error
                .to_string()
                .contains("selected Source begin_panic requires its checked artifact transport"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn source_eh_personality_retains_its_authored_native_body_and_name() -> TestResult {
    let source = minimal_runtime_source(
        r#"
#[repr(C)]
enum Reason { Fatal = 3, Continue = 8 }

unsafe extern "C" { safe fn observe_personality(actions: i32); }

#[lang = "eh_personality"]
#[unsafe(export_name = "discarded_personality_name")]
unsafe extern "C" fn renamed_personality(
    _version: i32, actions: i32, _class: u64, _exception: *mut i32, _context: *mut i32,
) -> Reason {
    observe_personality(19);
    match actions { 1 => Reason::Continue, _ => Reason::Fatal }
}

fn main() {}
"#,
    )?;
    let program = lower_used(&source)?;
    let [personality]: [_; 1] = program
        .symbols
        .iter()
        .filter(|symbol| symbol.name == "rust_eh_personality")
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| "expected one canonical personality symbol")?;
    assert!(personality.body.is_some());
    assert!(
        !program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "discarded_personality_name")
    );
    let mut lowir = Vec::new();
    rustgm::lowir::writer::write(&program, &mut lowir)?;
    let lowir = String::from_utf8(lowir)?;
    assert!(
        lowir
            .lines()
            .any(|line| { line.contains("call") && line.contains("@observe_personality(") }),
        "{lowir}"
    );
    Ok(())
}

#[test]
fn source_eh_personality_checks_role_identity_and_symbol_conflicts() -> TestResult {
    let plain = minimal_runtime_source("fn rust_eh_personality() {} fn main() {}")?;
    assert!(
        !lower_used(&plain)?
            .symbols
            .iter()
            .any(|symbol| symbol.name == "rust_eh_personality")
    );
    for (tail, cause) in [
        (
            r#"#[lang = "eh_personality"] #[unsafe(no_mangle)]
unsafe extern "C" fn role() {}"#,
            "no_mangle cannot be used on internal language items",
        ),
        (
            r#"#[lang = "eh_personality"] unsafe extern "C" fn role() {}
#[unsafe(export_name = "rust_eh_personality")] extern "C" fn duplicate() {}"#,
            "multiple definitions export the actual eh_personality symbol",
        ),
        (
            r#"#[lang = "eh_personality"] unsafe extern "C" fn role() -> i32 { false }"#,
            "incompatible types",
        ),
        (
            r#"#[lang = "eh_personality"] struct Wrong;"#,
            "required declaration target or owner",
        ),
    ] {
        let source = minimal_runtime_source(&format!("{tail}\nfn main() {{}}"))?;
        let error = lower_used(&source)
            .err()
            .ok_or("accepted invalid Source eh_personality")?;
        assert!(error.to_string().contains(cause), "{tail}: {error}");
    }
    Ok(())
}

#[test]
fn selected_source_eh_personality_fences_both_writers() -> TestResult {
    use rustgm::{
        lower::instances::{SourcePublication, build_source_library_profile},
        pipeline::Analysis,
    };
    let source = minimal_runtime_source(
        r#"#[lang = "eh_personality"] unsafe extern "C" fn renamed_personality() {}"#,
    )?;
    let report = crate::resolve::source_provider(&source, "personality_transport")?;
    let identity = report
        .package
        .source_core()
        .ok_or("Source owner")?
        .provider
        .clone();
    let Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let scalar = rustgm::sema::crates::export(&report, &mut facts, identity.clone())
        .err()
        .ok_or("scalar writer erased the selected personality role")?;
    let source = build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )
    .err()
    .ok_or("Source writer erased the selected personality role")?;
    for error in [scalar, source] {
        assert!(
            error
                .to_string()
                .contains("selected Source eh_personality requires its checked artifact transport"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn configured_doc_masked_keeps_prior_guest_alias_and_usable_reexport() -> TestResult {
    use std::{fs, process::Command};

    const NO_CORE: &str = "#![feature(no_core)]\n#![no_core]\n";
    const MASK: &str = "#[cfg_attr(all(), doc(masked))]";
    const OBSERVATION: &str = "function @observe_masked_dependency(";
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/doc-masked-guest")
        .join(std::process::id().to_string());
    fs::create_dir_all(&folder)?;
    let root = folder.join("core.rs");
    let dependency = folder.join("dependency.rs");
    let dependency_user = folder.join("masked.rs");
    let app = folder.join("app.rs");
    let skip = folder.join("skip.tsv");
    fs::write(
        &root,
        crate::resolve::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?,
    )?;
    fs::write(&skip, "")?;
    fs::write(
        &app,
        format!(
            "{NO_CORE}extern crate masked_guest;
             #[unsafe(no_mangle)]
             pub extern \"C\" fn observe_masked_dependency() -> u8 {{
                 masked_guest::reexported()
             }}
             fn main() {{}}"
        ),
    )?;
    let run = |attribute: &str, own: bool| -> TestResult<std::process::Output> {
        let grant = "#![feature(doc_masked)]\n";
        let (own_grant, prior_grant) = if own { (grant, "") } else { ("", grant) };
        fs::write(
            &dependency,
            format!("{NO_CORE}{prior_grant}pub fn origin() -> u8 {{ 19 }}"),
        )?;
        fs::write(
            &dependency_user,
            format!(
                "{NO_CORE}{own_grant}{attribute}
                 extern crate libc_guest as actual_dependency;
                 pub use actual_dependency::origin as reexported;"
            ),
        )?;
        Ok(Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-lowir",
                "--cap-lints=allow",
                "--edition=2024",
                "--crate-name",
                "core",
                "--crate-disambiguator",
                "masked_guest_control",
                "--source-core",
                "core@masked_guest_control",
            ])
            .arg("--bringup-skip-list")
            .arg(&skip)
            .arg("--bringup-guest")
            .arg(format!("libc_guest={}", dependency.display()))
            .arg("--bringup-guest")
            .arg(format!("masked_guest={}", dependency_user.display()))
            .arg("--bringup-guest")
            .arg(format!("app={}", app.display()))
            .arg(&root)
            .output()?)
    };
    let plain = run("", true)?;
    let marked = run(MASK, true)?;
    for output in [&plain, &marked] {
        let stderr = std::str::from_utf8(&output.stderr)?;
        assert!(output.status.success(), "{stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        let lowir = std::str::from_utf8(&output.stdout)?;
        assert!(lowir.contains(OBSERVATION), "{lowir}");
    }
    assert_eq!(plain.stdout, marked.stdout);

    let missing = run(MASK, false)?;
    let stderr = std::str::from_utf8(&missing.stderr)?;
    assert_eq!(missing.status.code(), Some(1), "{stderr}");
    assert!(missing.stdout.is_empty(), "{stderr}");
    assert!(
        stderr.contains("doc(masked) requires this crate's doc_masked feature"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn doc_masked_uses_its_own_lexical_invalid_owner_lint() -> TestResult {
    const PREFIX: &str = "#![feature(doc_masked)]\n";
    const INVALID: &str = "#[doc(masked)] pub struct Wrong;";
    const SELF: &str = "#[doc(masked)] pub extern crate self as root_alias;";
    const WRONG_ITEM: &str = "this attribute can only be applied to an `extern crate` item";
    const SELF_ITEM: &str = "this attribute cannot be applied to an `extern crate self` item";
    for (source, message) in [(INVALID, WRONG_ITEM), (SELF, SELF_ITEM)] {
        let input = format!("{PREFIX}{source}");
        let (_, facts) = analyze(&input)?;
        let warnings = warning_text(&facts)?;
        assert_eq!(facts.warnings().count(), 1, "{warnings}");
        assert!(
            warnings.contains("warning[invalid_doc_attributes]"),
            "{warnings}"
        );
        assert!(warnings.contains(message), "{warnings}");
    }

    for setting in [
        "#![allow(invalid_doc_attributes)]",
        "#![deny(unfulfilled_lint_expectations)]\n#[expect(invalid_doc_attributes)]",
    ] {
        let (_, facts) = analyze(&format!("{PREFIX}{setting}\n{INVALID}"))?;
        assert!(facts.warnings().next().is_none());
    }
    let (_, facts) = analyze("#[cfg(any())]\n#[doc(masked)] pub struct Removed;")?;
    assert!(facts.warnings().next().is_none());

    let denied = format!("{PREFIX}#![deny(invalid_doc_attributes)]\n{INVALID}");
    let error = analyze(&denied)
        .err()
        .ok_or("invalid doc(masked) owner escaped its own lint")?;
    assert!(error.to_string().contains(WRONG_ITEM), "{error}");
    let mut capped = Config::default();
    capped.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let (_, facts) = analyze_with_config(&denied, &capped)?;
    assert!(facts.warnings().next().is_none());
    assert_eq!(std::mem::size_of::<rustgm::sema::model::NodeFacts>(), 80);
    Ok(())
}

const GNU_UNWIND_IMPORTS: &str = r#"
#[lang = "Option"]
pub enum Option<T> { #[lang = "None"] None, #[lang = "Some"] Some(T) }

#[repr(C)]
pub enum _Unwind_Reason_Code {
    _URC_NO_REASON = 0,
    _URC_FOREIGN_EXCEPTION_CAUGHT = 1,
    _URC_FATAL_PHASE2_ERROR = 2,
    _URC_FATAL_PHASE1_ERROR = 3,
    _URC_NORMAL_STOP = 4,
    _URC_END_OF_STACK = 5,
    _URC_HANDLER_FOUND = 6,
    _URC_INSTALL_CONTEXT = 7,
    _URC_CONTINUE_UNWIND = 8,
    _URC_FAILURE = 9,
}

pub type _Unwind_Exception_Cleanup_Fn =
    Option<extern "C" fn(unwind_code: _Unwind_Reason_Code, exception: *mut _Unwind_Exception)>;

#[repr(C)]
pub struct _Unwind_Exception {
    pub exception_class: u64,
    pub exception_cleanup: _Unwind_Exception_Cleanup_Fn,
    pub private: [*const u8; 2],
}

unsafe extern "C-unwind" {
    pub fn _Unwind_Resume(exception: *mut _Unwind_Exception) -> !;
}
        unsafe extern "C-unwind" {
            pub fn _Unwind_RaiseException(exception: *mut _Unwind_Exception) -> _Unwind_Reason_Code;
        }
"#;

#[test]
fn source_c_unwind_imports_keep_typed_abi_without_native_demand() -> TestResult {
    use rustgm::{
        resolve::model::DefId,
        support::linkage::{Linkage, Naming},
        types::{Kind, TypeId, callable::Abi},
    };
    let source = minimal_runtime_source(&format!(
        r#"{GNU_UNWIND_IMPORTS}
#[unsafe(no_mangle)]
pub extern "C" fn ordinary_unwind_control() -> i32 {{ 19 }}
fn main() {{}}"#
    ))?;
    let mut report = crate::resolve::source_provider(&source, "source_unwind_imports")?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let foreign = facts.foreign.as_deref().ok_or("foreign registry")?;
    let definition = |name: &str| -> TestResult<u32> {
        let position = report
            .resolution
            .definitions
            .iter()
            .position(|definition| report.package.names.get(definition.name) == name)
            .ok_or("selected declaration")?;
        Ok(u32::try_from(position)?)
    };
    let exception = definition("_Unwind_Exception")?;
    let reason = definition("_Unwind_Reason_Code")?;
    for (name, never) in [("_Unwind_Resume", true), ("_Unwind_RaiseException", false)] {
        let record = foreign
            .function(DefId(definition(name)?))
            .ok_or("canonical foreign unwind import")?;
        assert_eq!(record.linkage, Linkage::Import);
        let Naming::Exact(id) = record.naming else {
            return Err("foreign import lost native name".into());
        };
        assert_eq!(foreign.names().get(id), Some(name));
        let Some(Kind::Function {
            unsafe_: true,
            convention,
            parameters,
            result,
            ..
        }) = facts.types.kind(record.signature)
        else {
            return Err("foreign unwind import lost its checked callable".into());
        };
        assert_eq!(convention.abi, Abi::CUnwind);
        assert!(!convention.variadic);
        let Some([parameter]) = facts.types.list(parameters) else {
            return Err("unwind import lost its exception pointer parameter".into());
        };
        assert!(
            matches!(facts.types.kind(*parameter), Some(Kind::Pointer { mutable: true, pointee })
            if matches!(facts.types.kind(pointee), Some(Kind::Adt(owner, _)) if owner.0 == exception))
        );
        if never {
            assert_eq!(result, TypeId::NEVER);
        } else {
            assert!(
                matches!(facts.types.kind(result), Some(Kind::Adt(owner, _)) if owner.0 == reason)
            );
        }
    }
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let program =
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| { symbol.name == "ordinary_unwind_control" && symbol.body.is_some() })
    );
    assert!(!program.symbols.iter().any(|symbol| matches!(
        symbol.name.as_str(),
        "_Unwind_Resume" | "_Unwind_RaiseException"
    )));
    Ok(())
}

#[test]
fn source_c_unwind_keeps_pointer_abi_and_fences_actual_native_calls() -> TestResult {
    let mismatch = minimal_runtime_source(&format!(
        r#"{GNU_UNWIND_IMPORTS}
fn incompatible() -> unsafe extern "C" fn(*mut _Unwind_Exception) -> ! {{ _Unwind_Resume }}"#
    ))?;
    let report = crate::resolve::source_provider(&mismatch, "source_unwind_pointer_mismatch")?;
    let error = rustgm::pipeline::analyze_source(&report)
        .err()
        .ok_or("C pointer accepted a C-unwind function item")?;
    assert!(
        error
            .to_string()
            .contains("incompatible callable convention or safety"),
        "{error}"
    );

    for tail in [
        r#"#[unsafe(no_mangle)]
            pub unsafe extern "C" fn reached(exception: *mut _Unwind_Exception) -> ! {
                unsafe { _Unwind_Resume(exception) }
            }"#,
        r#"#[unsafe(no_mangle)]
            pub unsafe extern "C" fn reached(
                callback: unsafe extern "C-unwind" fn(*mut _Unwind_Exception) -> !,
                exception: *mut _Unwind_Exception,
            ) -> ! {
                unsafe { callback(exception) }
            }"#,
    ] {
        let source =
            minimal_runtime_source(&format!("{GNU_UNWIND_IMPORTS}\n{tail}\nfn main() {{}}"))?;
        let mut report = crate::resolve::source_provider(&source, "source_unwind_native_gate")?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let mut facts = analysis.facts;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let error =
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)
                .err()
                .ok_or("native call erased its C-unwind contract")?;
        assert!(
            error
                .to_string()
                .contains("C-unwind native call or definition requires its unwind ABI lowering"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn selected_source_c_unwind_definition_fences_both_writers() -> TestResult {
    use rustgm::lower::instances::{SourcePublication, build_source_library_profile};
    let source = minimal_runtime_source(
        r#"pub extern "C-unwind" fn source_unwind(value: i32) -> i32 { value }"#,
    )?;
    let report = crate::resolve::source_provider(&source, "source_unwind_wire_gate")?;
    let identity = report
        .package
        .source_core()
        .ok_or("Source owner")?
        .provider
        .clone();
    let scalar_source = r#"#![feature(no_core, lang_items)]
        #![no_core]
        #[lang = "pointee_sized"] trait PointeeSized {}
        #[lang = "meta_sized"] trait MetaSized: PointeeSized {}
        #[lang = "sized"] trait Sized: MetaSized {}
        pub extern "C-unwind" fn scalar_unwind(value: i32) -> i32 { value }"#;
    let scalar_report = crate::resolve::source_provider(scalar_source, "scalar_unwind_wire_gate")?;
    let scalar_identity = scalar_report
        .package
        .source_core()
        .ok_or("scalar Source owner")?
        .provider
        .clone();
    let mut scalar_facts = rustgm::pipeline::analyze_source(&scalar_report)?.facts;
    let scalar = rustgm::sema::crates::export(&scalar_report, &mut scalar_facts, scalar_identity)
        .err()
        .ok_or("scalar metadata erased C-unwind")?;
    assert!(
        scalar
            .to_string()
            .contains("C-unwind export requires its versioned unwind ABI"),
        "{scalar}"
    );
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let source = build_source_library_profile(
        &report,
        &mut facts,
        identity,
        analysis.prepared,
        SourcePublication::ModuleBodyOperations,
    )
    .err()
    .ok_or("Source metadata erased C-unwind")?;
    assert!(
        source
            .to_string()
            .contains("C-unwind source metadata requires its versioned unwind ABI"),
        "{source}"
    );
    Ok(())
}
