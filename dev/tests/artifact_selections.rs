//! PA34 artifact declarations retain the caller's phase and supported output.
use rustgm::{
    expand::config::Config,
    resolve::{
        bindings::Context as SourceContext,
        expansion,
        load::{InputFile, Sources},
    },
    support::{crates::ArtifactContext, source::SourceFile},
};
use std::{
    path::Path,
    process::{Command, Output},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "../pa34/tests/spec/crate-artifact-attributes";
const UNSUPPORTED: &str = "source-selected library artifacts require an explicit";
const TRANSPORT: &str = "source scalar pool requires a wider transport profile";
const FORM: &str = "crate_type requires a string literal";
const FROZEN: &str = "source crate_type artifact selection requires source compiler mode";

#[derive(Clone, Copy)]
enum Profile {
    Source,
    SourceLibrary,
    Guest(&'static str),
    Ordinary,
    Inspection,
}
struct Case {
    name: &'static str,
    source: &'static str,
    profile: Profile,
    output: Option<&'static str>,
    failure: Option<&'static str>,
}
const fn case(
    name: &'static str,
    source: &'static str,
    profile: Profile,
    output: Option<&'static str>,
    failure: Option<&'static str>,
) -> Case {
    Case {
        name,
        source,
        profile,
        output,
        failure,
    }
}
const CASES: &[Case] = &[
    case(
        "source-bringup-lib-overrides-bin",
        "bin.rs",
        Profile::SourceLibrary,
        Some("lib"),
        None,
    ),
    case(
        "source-bringup-lib-overrides-dylib",
        "dylib.rs",
        Profile::SourceLibrary,
        Some("lib"),
        None,
    ),
    case("source-default-bin", "bin.rs", Profile::Source, None, None),
    case(
        "source-omitted-rlib",
        "rlib.rs",
        Profile::Source,
        None,
        Some(UNSUPPORTED),
    ),
    case(
        "source-bin-overrides-rlib",
        "rlib.rs",
        Profile::Source,
        Some("bin"),
        None,
    ),
    case(
        "source-lib-overrides-bin",
        "bin.rs",
        Profile::Source,
        Some("lib"),
        Some(TRANSPORT),
    ),
    case(
        "source-bin-overrides-dylib",
        "dylib.rs",
        Profile::Source,
        Some("bin"),
        None,
    ),
    case(
        "source-lib-overrides-dylib",
        "dylib.rs",
        Profile::Source,
        Some("lib"),
        Some(TRANSPORT),
    ),
    case(
        "source-omitted-dylib",
        "dylib.rs",
        Profile::Source,
        None,
        Some(UNSUPPORTED),
    ),
    case(
        "source-non-string",
        "non-string.rs",
        Profile::Source,
        None,
        Some(FORM),
    ),
    case(
        "source-malformed-list",
        "list.rs",
        Profile::Source,
        Some("bin"),
        Some(FORM),
    ),
    case(
        "source-unknown",
        "unknown.rs",
        Profile::Source,
        None,
        Some("crate_type value is outside the supported artifact profile"),
    ),
    case(
        "guest-rlib",
        "host.rs",
        Profile::Guest("guest-rlib.rs"),
        None,
        None,
    ),
    case(
        "guest-non-string",
        "host.rs",
        Profile::Guest("guest-non-string.rs"),
        None,
        Some(FORM),
    ),
    case(
        "inspection-rlib",
        "inspection.rs",
        Profile::Inspection,
        None,
        None,
    ),
    case(
        "ordinary-omitted",
        "ordinary.rs",
        Profile::Ordinary,
        None,
        Some(FROZEN),
    ),
    case(
        "ordinary-explicit-bin",
        "ordinary.rs",
        Profile::Ordinary,
        Some("bin"),
        Some(FROZEN),
    ),
    case(
        "ordinary-non-root",
        "non-root.rs",
        Profile::Ordinary,
        None,
        Some(FROZEN),
    ),
    case(
        "inspection-non-root",
        "non-root.rs",
        Profile::Inspection,
        None,
        None,
    ),
];

fn run(case: &Case) -> Result<Output> {
    let (folder, source_command) = crate::guest_crates::prepare(case.name, &[])?;
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET);
    std::fs::copy(inputs.join(case.source), folder.join("core.rs"))?;
    let mut command = match case.profile {
        Profile::Source | Profile::SourceLibrary | Profile::Guest(_) => source_command,
        Profile::Ordinary | Profile::Inspection => {
            let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
            let mode = if matches!(case.profile, Profile::Inspection) {
                "--emit-expanded-crate"
            } else {
                "--emit-lowir"
            };
            command.args([mode, "--cap-lints=allow", "--edition=2024"]);
            command
        }
    };
    if let Some(output) = case.output {
        command.arg(format!("--crate-type={output}"));
    }
    if matches!(case.profile, Profile::Guest(_) | Profile::SourceLibrary) {
        let skip = folder.join("skip.tsv");
        std::fs::write(&skip, "")?;
        command.arg("--bringup-skip-list").arg(skip);
    }
    if let Profile::Guest(guest) = case.profile {
        std::fs::copy(inputs.join(guest), folder.join("guest.rs"))?;
        command
            .arg("--bringup-guest")
            .arg(format!("libc={}", folder.join("guest.rs").display()));
    }
    Ok(command.arg(folder.join("core.rs")).output()?)
}

#[test]
fn source_and_guest_requests_keep_explicit_output_overrides_and_profile_gaps() -> Result {
    assert_eq!(CASES.len(), 19);
    for case in CASES {
        let name = case.name;
        let expected = i32::from(case.failure.is_some());
        let output = run(case)?;
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(expected), "{name}: {diagnostic}");
        if let Some(message) = case.failure {
            assert!(diagnostic.contains(message), "{name}: {diagnostic}");
            assert!(output.stdout.is_empty(), "{name}");
        }
        if matches!(case.profile, Profile::SourceLibrary) {
            assert!(
                diagnostic.contains("source metadata is withheld"),
                "{name}: {diagnostic}"
            );
            assert!(
                !diagnostic.contains("bringup-blocker\t")
                    && !diagnostic.contains("bringup-unproven\t"),
                "{name}: {diagnostic}"
            );
        }
    }
    Ok(())
}

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn a_source_artifact_policy_alone_cannot_open_the_ordinary_compiler_profile() -> Result {
    let source = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(PACKET)
            .join("ordinary.rs"),
    )?;
    let result = expansion::analyze_with_context(
        InputFile {
            identity: "ordinary.rs".into(),
            path: "ordinary.rs".into(),
            source: SourceFile::new(source)?,
        },
        &mut NoFiles,
        &Config::default(),
        &SourceContext {
            artifacts: ArtifactContext::Source { selected: None },
            ..SourceContext::default()
        },
    );
    let error = result.err().ok_or("artifact policy opened source mode")?;
    assert!(
        error
            .to_string()
            .contains("source artifact policy requires source compiler bindings"),
        "{error}"
    );
    Ok(())
}
