//! Environment expansion retains nominal identity and explicit provider snapshots.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema,
    support::source::SourceFile,
};
use std::path::Path;
const CASES: &[(&str, &str, bool)] = &[
    (
        "absent",
        "fn main(){let x:Option<&'static str>=option_env!(\"MISSING\");match x{Some(_)=>{},None=>{}}}",
        true,
    ),
    (
        "present",
        "fn main(){let x:Option<&'static str>=option_env!(\"KEY\");match x{Some(_)=>{},None=>{}}}",
        true,
    ),
    (
        "alias",
        "use core::option_env as optional;fn main(){let _:Option<&'static str>=optional!(concat!(\"K\",\"EY\"));}",
        true,
    ),
    (
        "no-prelude",
        "#![no_implicit_prelude]extern crate core;fn main(){let _:core::option::Option<&'static str>=core::option_env!(\"KEY\");}",
        true,
    ),
    (
        "nominal-shadow",
        "struct Option(u8);fn take(_:core::option::Option<&'static str>){}fn main(){let _=Option(7);take(option_env!(\"KEY\"));}",
        true,
    ),
    (
        "macro-shadow",
        "macro_rules! option_env{($x:expr)=>{7u8}}fn main(){let _:u8=option_env!(\"KEY\");}",
        true,
    ),
    (
        "mismatch",
        "fn main(){let _:Option<u8>=option_env!(\"MISSING\");}",
        false,
    ),
    ("no-argument", "fn main(){let _=option_env!();}", false),
    ("non-literal", "fn main(){let _=option_env!(42);}", false),
    (
        "extra-argument",
        "fn main(){let _=option_env!(\"KEY\",\"message\");}",
        false,
    ),
    (
        "type-position",
        "type X=option_env!(\"KEY\");fn main(){}",
        false,
    ),
    ("statement", "fn main(){option_env!(\"KEY\");}", true),
    (
        "constant-present",
        "const X:Option<&'static str>=option_env!(\"KEY\");fn main(){let _=X;}",
        true,
    ),
    (
        "constant-absent",
        "const X:Option<&'static str>=option_env!(\"MISSING\");fn main(){let _=X;}",
        true,
    ),
    (
        "pattern-position",
        "fn main(){let x:Option<&str>=None;match x{option_env!(\"MISSING\")=>{},_=>{}}}",
        false,
    ),
];
#[derive(Default)]
struct Provider {
    reads: Vec<String>,
    failed: bool,
}
impl Sources for Provider {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
    fn environment(&mut self, name: &str) -> Result<Option<String>, String> {
        self.reads.push(name.to_owned());
        if self.failed {
            return Err("environment value is not Unicode".into());
        }
        Ok((name == "KEY").then(|| "é\r\nvalue".to_owned()))
    }
}
fn expand(source: &str, provider: &mut Provider) -> Result<Report, String> {
    resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        provider,
        &Config::default(),
    )
    .map_err(|e| e.to_string())
}
fn checked(source: &str, provider: &mut Provider) -> Result<(), String> {
    let mut report =
        resolve::analyze_expanded(expand(source, provider)?).map_err(|e| e.to_string())?;
    let mut facts = sema::declarations::analyze(&report)
        .and_then(|facts| sema::bodies::analyze(&report, facts))
        .map_err(|e| e.to_string())?;
    let entry = sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
#[test]
fn option_environment_uses_canonical_nominal_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("option-environment-{name}"), source, accepted)?;
        let result = checked(source, &mut Provider::default());
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn option_environment_shares_eager_snapshots_and_skips_inactive_input()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn main(){let _=option_env!(concat!(\"K\",\"EY\"));let _=option_env!(\"KEY\");let _=env!(\"KEY\");}#[cfg(any())]fn off(){let _=option_env!(\"OFF\");}";
    let mut provider = Provider::default();
    checked(source, &mut provider)?;
    assert_eq!(provider.reads, ["KEY"]);
    let mut provider = Provider {
        failed: true,
        ..Provider::default()
    };
    let error = expand("fn main(){let _=option_env!(\"KEY\");}", &mut provider)
        .err()
        .ok_or("provider error was accepted")?;
    assert!(
        error.contains("environment value is not Unicode"),
        "{error}"
    );
    for &(name, source, _) in CASES {
        if matches!(
            name,
            "no-argument" | "extra-argument" | "non-literal" | "macro-shadow"
        ) {
            let mut provider = Provider::default();
            let _ = checked(source, &mut provider);
            assert!(provider.reads.is_empty(), "{name}: {:?}", provider.reads);
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn option_environment_executes_ordinary_option_shapes() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "option-environment",
        &[
            (
                "constant",
                "const X:Option<&'static str>=option_env!(\"CARGO_PKG_NAME\");fn main(){check(match X{Some(\"rustgm\")=>true,_=>false});}",
                false,
            ),
            (
                "present",
                "fn main(){check(match option_env!(\"CARGO_PKG_NAME\"){Some(\"rustgm\")=>true,_=>false});}",
                false,
            ),
            (
                "absent",
                "fn main(){check(match option_env!(\"RUSTGM_PA26_EXPECTED_ABSENT_72690\"){None=>true,Some(_)=>false});}",
                false,
            ),
            (
                "shadow",
                "struct Option(u8);fn main(){let _=Option(7);check(match option_env!(\"CARGO_PKG_NAME\"){core::option::Option::Some(\"rustgm\")=>true,_=>false});}",
                false,
            ),
        ],
    )
}
