//! Independently observed byte contracts retain actual source roles and native values.
use crate::provider_traits::TestResult;
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{bindings::Mode, crates::CrateIdentity, edition::Edition, source::SourceFile},
};
use std::path::Path;
const ACCEPT: &[&str] = &[
    "mixed-bytes",
    "empty",
    "trailing-comma",
    "array-literals",
    "repeat",
    "repeat-zero",
    "count-suffix",
    "nested-eager",
    "captured-expression",
    "captured-array",
    "renamed-role",
    "authored-shadow",
    "width-8",
    "width-32",
    "width-128",
    "uses-8",
    "uses-32",
    "uses-128",
    "included-complete-provider",
];
fn root() -> TestResult<&'static Path> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "root".into())
}
fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        root()?
            .join("pa31/tests/spec/source-concat-bytes")
            .join(format!("{name}.rs")),
    )?)
}
struct Files {
    raw: Vec<u8>,
}
impl Sources for Files {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
    fn resource(&mut self, path: &Path) -> std::result::Result<Option<Vec<u8>>, String> {
        Ok(
            (path.file_name() == Some(std::ffi::OsStr::new("bytes-input.bin")))
                .then(|| self.raw.clone()),
        )
    }
}
fn expanded(name: &str, edition: Edition) -> TestResult<Report> {
    let folder = root()?.join("pa31/tests/spec/source-concat-bytes");
    let path = folder.join(format!("{name}.rs"));
    let identity = CrateIdentity::new("minicore", "actual_concat_bytes")?;
    let context = if matches!(name, "ordinary-requested" | "ordinary-unrequested") {
        resolve::bindings::Context::default()
    } else {
        resolve::bindings::Context {
            crate_name: Some("minicore".into()),
            current: Some(identity.clone()),
            mode: Mode::Source { provider: identity },
            ..Default::default()
        }
    };
    Ok(resolve::expansion::analyze_with_context(
        InputFile {
            identity: path.clone(),
            path,
            source: SourceFile::new(fixture(name)?.into_bytes())?,
        },
        &mut Files {
            raw: std::fs::read(folder.join("bytes-input.bin"))?,
        },
        &Config::in_edition(edition),
        &context,
    )?)
}
fn lower(name: &str, edition: Edition) -> TestResult<rustgm::lowir::program::Program> {
    let report = resolve::analyze_expanded(expanded(name, edition)?)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_source_byte_literals_arrays_repeats_and_forwarding_lower_in_both_modern_editions()
-> TestResult {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for &name in ACCEPT {
            lower(name, edition).map_err(|error| format!("{name}/{edition:?}: {error}"))?;
        }
    }
    Ok(())
}
#[test]
fn byte_shape_and_count_opponents_reject_at_their_actual_typed_boundary() -> TestResult {
    for (name, message) in [
        ("top-level-integer", "byte or byte-string"),
        ("signed-array-element", "u8 integer"),
        ("array-element-overflow", "out of range"),
        ("repeat-zero-invalid-element", "out of range"),
        ("nested-array", "nested arrays"),
        ("nested-byte-string", "nested byte strings"),
        ("repeat-expression-count", "integer literal"),
        ("repeat-const-count", "integer literal"),
        ("negative-array-element", "byte literals"),
        ("ordinary-string", "byte or byte-string"),
        ("c-string", "byte or byte-string"),
        ("parenthesized-literal", "literal arguments"),
        ("malformed-separator", "comma"),
        ("nonliteral-operand", "literal arguments"),
        ("nested-included-complete-provider", "nested byte strings"),
    ] {
        let error = expanded(name, Edition::Rust2024)
            .err()
            .ok_or_else(|| format!("{name}: invalid input accepted"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn source_byte_claim_stability_and_ordinary_foreign_frontiers_remain_explicit() -> TestResult {
    let report = expanded("original-header", Edition::Rust2024)?;
    assert!(report.definitions.iter().any(|entry| matches!(
        entry.body,
        resolve::expansion::Body::Builtin(rustgm::expand::builtin::Builtin::ConcatBytes)
    )));
    let report = resolve::analyze_expanded(report)?;
    assert!(
        report
            .package
            .library_policy()
            .and_then(resolve::stability::Library::inventory)
            .is_some()
    );
    rustgm::pipeline::analyze_source(&report)?;
    for (name, message) in [
        (
            "scoped-role",
            "builtin macro binding requires its supported declaration kind",
        ),
        ("unknown-role", "builtin"),
        ("ordinary-requested", "unresolved"),
        ("ordinary-unrequested", "unresolved"),
    ] {
        let error = expanded(name, Edition::Rust2024)
            .err()
            .ok_or_else(|| format!("{name}: unsupported owner/profile admitted"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    // Wrong-owner attributes are checked by final source reservation, after
    // expansion has retained the actual configured function declaration.
    let error = resolve::analyze_expanded(expanded("wrong-owner", Edition::Rust2024)?)
        .err()
        .ok_or("builtin function owner admitted")?;
    assert!(
        error
            .message
            .contains("builtin macro binding requires its supported declaration kind"),
        "{error}"
    );
    Ok(())
}
fn expected(name: &str) -> TestResult<Vec<u8>> {
    let values = match name {
        "mixed-bytes" => b"AB\0\xffC\\D".to_vec(),
        "empty" | "repeat-zero" => vec![],
        "trailing-comma" => b"A".to_vec(),
        "array-literals" => vec![0, 65, 255],
        "repeat" => b"QQQ".to_vec(),
        "count-suffix" => b"XX".to_vec(),
        "captured-array" => vec![0, 65],
        "authored-shadow" => b"Z".to_vec(),
        "width-8" => (0..8).collect(),
        "width-32" => (0..32).collect(),
        "width-128" => (0..128).collect(),
        "included-complete-provider" => {
            let mut bytes =
                std::fs::read(root()?.join("pa31/tests/spec/source-concat-bytes/bytes-input.bin"))?;
            bytes.push(0);
            bytes
        }
        "nested-eager"
        | "captured-expression"
        | "renamed-role"
        | "uses-8"
        | "uses-32"
        | "uses-128" => b"AB".to_vec(),
        _ => return Err("unknown declared native byte contract".into()),
    };
    Ok(values)
}
fn consumer(bytes: &[u8]) -> String {
    let check = if bytes.is_empty() {
        String::new()
    } else {
        format!(
            "const unsigned char expected[]={{{}}};for(unsigned i=0;i<sizeof expected;i++)if(actual[i]!=expected[i])return 2;",
            bytes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    format!(
        "#include <stdio.h>\nextern const unsigned char*answer(void);int main(void){{const unsigned char*actual=answer();if(!actual)return 1;{check}puts(\"7\");return 0;}}\n"
    )
}
#[test]
#[ignore = "requires pinned CPPGM backend and host C linker"]
fn source_byte_results_preserve_independently_observed_native_bytes_in_both_editions() -> TestResult
{
    let folder = root()?
        .join("obj/parallel/pa31/concat-bytes-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (label, edition) in [("2021", Edition::Rust2021), ("2024", Edition::Rust2024)] {
        for &name in ACCEPT {
            let program = lower(name, edition)?;
            crate::ffi_program::check(
                root()?,
                &folder,
                &format!("{name}-{label}"),
                &program,
                &consumer(&expected(name)?),
                "7\n",
            )?;
        }
    }
    Ok(())
}
