//! The actual driver publishes complete contracts before fresh CLI consumers run.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEPRECATED_PROVIDER: &str = r#"
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="1.0.0",note="use the replacement")]
pub fn removed() {}
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="1.0.0",note="use the current module")]
pub mod old {
    #[stable(feature="deprecation_contract",since="1.0.0")]
    pub fn inside() {}
}
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="TBD",note="scheduled")]
pub fn scheduled() {}
#[stable(feature="deprecation_contract",since="1.0.0")]
#[deprecated(since="1.99.0",note="next release")]
pub fn next_release() {}
"#;

const DIVISION_PROVIDER: &str = r#"
#[lang="div"]
pub const trait Div<Rhs = Self> {
    type Output;
    fn div(self, rhs: Rhs) -> Self::Output;
}
impl Div for u8 {
    type Output = u8;
    fn div(self, _rhs: u8) -> u8 { self }
}
"#;

const RESERVATION_PROVIDER: &str = r#"
#[stable(feature="reservation_transport",since="1.0.0")]
#[rustc_const_unstable(feature="reservation_effect",issue="none")]
#[lang="From"]
pub const trait From<T>: Sized {
    #[stable(feature="reservation_transport",since="1.0.0")]
    fn from(value: T) -> Self;
}
#[stable(feature="reservation_transport",since="1.0.0")]
#[rustc_const_unstable(feature="reservation_effect",issue="none")]
#[rustc_reservation_impl="preserve future implementation"]
const impl<T> From<!> for T {
    fn from(value: !) -> T { value }
}
"#;

fn qualify(native: bool) -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let lane = if native { "native" } else { "frontend" };
    let evidence = root
        .join("obj/parallel/pa31/source-cli-contracts")
        .join(format!("{lane}-{}", std::process::id()));
    std::fs::create_dir_all(evidence.parent().ok_or("evidence parent")?)?;
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/check_source_attributes.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--evidence-root")
        .arg(&evidence);
    if native {
        command.arg("--native");
    }
    let result = command.output()?;
    std::fs::write(evidence.with_extension("stdout"), &result.stdout)?;
    std::fs::write(evidence.with_extension("stderr"), &result.stderr)?;
    if !result.status.success() {
        return Err(format!(
            "source CLI contracts: {:?}\n{}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    if !result.stderr.is_empty() {
        return Err("source CLI contract harness emitted an undeclared diagnostic".into());
    }
    Ok(())
}

#[test]
fn cli_function_contracts_reach_fresh_consumers_with_exact_diagnostics() -> Result {
    qualify(false)
}

#[test]
#[ignore = "requires prebuilt locked object backend, pinned Rust and host C compiler"]
fn cli_function_contracts_reach_fresh_native_consumers_returning_seven() -> Result {
    qualify(true)
}

fn source_export(
    profile: &str,
    fixture: &Path,
    metadata: &Path,
    lowir: &Path,
    provider: bool,
) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command.args([
        "--emit-lowir",
        "--crate-type=lib",
        "--crate-name",
        "base",
        "--crate-disambiguator",
        "exact_cli",
        "--source-profile",
        profile,
    ]);
    if provider {
        command.args(["--source-core", "base@exact_cli"]);
    }
    command
        .arg("--metadata-output")
        .arg(metadata)
        .arg("-o")
        .arg(lowir)
        .arg(fixture);
    command
}

fn availability_foundation(root: &Path) -> Result<String> {
    let source = std::fs::read_to_string(
        root.join("pa31/tests/spec/source-availability-transport/provider.rs"),
    )?;
    Ok(source
        .split_once("\n#[unstable(feature=\"experimental_identity\"")
        .ok_or("source availability fixture lost its role-only prefix")?
        .0
        .to_owned())
}

fn macro_role_foundation(root: &Path) -> Result<String> {
    let source = std::fs::read_to_string(root.join("pa29/tests/spec/macro-contexts/provider.rs"))?;
    Ok(source
        .split_once("// Authored macro-context transport controls.")
        .ok_or("macro-context fixture lost its role foundation")?
        .0
        .to_owned())
}

fn source_client(source: &Path, metadata: &Path, lowir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args([
            "--emit-lowir",
            "--edition=2024",
            "--crate-name",
            "client",
            "--crate-disambiguator",
            "exact_cli",
            "--source-client",
            "base@exact_cli",
            "--extern",
        ])
        .arg(format!("base={}", metadata.display()))
        .arg("-o")
        .arg(lowir)
        .arg(source);
    command
}

#[test]
fn cli_version_fourteen_preserves_actual_deprecation_and_client_policy() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/deprecation-metadata-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let foundation = availability_foundation(root)?;
    let source = folder.join("provider.rs");
    std::fs::write(&source, format!("{foundation}\n{DEPRECATED_PROVIDER}"))?;
    let metadata = folder.join("provider.rgcrate");
    let mut command = source_export(
        "module-body-operations-v14",
        &source,
        &metadata,
        &folder.join("provider.lowir"),
        true,
    );
    command.arg("--cap-lints=allow");
    super::artifacts::checked("provider", &folder, &mut command)?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    for (case, policy, callee, diagnostic) in [
        ("active", "deny", "removed", Some("use the replacement")),
        (
            "inherited",
            "deny",
            "old::inside",
            Some("use the current module"),
        ),
        ("tbd", "deny", "scheduled", None),
        ("next", "deny", "next_release", None),
        ("allow", "allow", "removed", None),
        ("expect", "expect", "removed", None),
    ] {
        let source = folder.join(format!("{case}.rs"));
        let text = format!(
            "#![feature(no_core)]#![no_core]#![deny(unfulfilled_lint_expectations)]\n\
             extern crate base;#[{policy}(deprecated)]fn main(){{base::{callee}();}}"
        );
        std::fs::write(&source, text)?;
        let output =
            source_client(&source, &metadata, &folder.join(format!("{case}.lowir"))).output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(diagnostic.is_some())),
            "{case}: {stderr}"
        );
        if let Some(note) = diagnostic {
            assert!(
                stderr.contains(&format!("use of deprecated declaration: {note}")),
                "{case}: {stderr}"
            );
        } else {
            assert!(stderr.is_empty(), "{case}: {stderr}");
        }
    }
    Ok(())
}

#[test]
fn cli_version_fourteen_preserves_const_reservation_for_coherence() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/reservation-polarity-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let foundation = availability_foundation(root)?;
    let provider = folder.join("provider.rs");
    std::fs::write(
        &provider,
        format!("#![feature(never_type,const_trait_impl)]\n{foundation}\n{RESERVATION_PROVIDER}"),
    )?;
    let metadata = folder.join("provider.rgcrate");
    let mut command = source_export(
        "module-body-operations-v14",
        &provider,
        &metadata,
        &folder.join("provider.lowir"),
        true,
    );
    command.arg("--cap-lints=allow");
    super::artifacts::checked("provider", &folder, &mut command)?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    for (case, body, diagnostic) in [
        ("unused", "fn main() {}", None),
        (
            "use",
            "struct Local; fn require<T: base::From<!>>() {} fn main() { require::<Local>(); }",
            Some("trait obligation requires an unambiguous matching assumption or implementation"),
        ),
        (
            "overlap",
            "struct Local; trait Marker {} impl<T: base::From<!>> Marker for T {} impl Marker for Local {} fn main() {}",
            Some("overlapping trait implementation headers require coherence proof"),
        ),
    ] {
        let client = folder.join(format!("{case}.rs"));
        std::fs::write(
            &client,
            format!("#![feature(never_type,no_core)]#![no_core]\n{body}"),
        )?;
        let output =
            source_client(&client, &metadata, &folder.join(format!("{case}.lowir"))).output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(diagnostic.is_some())),
            "{case}: {stderr}"
        );
        if let Some(message) = diagnostic {
            assert!(stderr.contains(message), "{case}: {stderr}");
        } else {
            assert!(stderr.is_empty(), "{case}: {stderr}");
        }
    }
    Ok(())
}

#[test]
fn cli_version_fourteen_uses_exact_division_impl_effects() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-division-transport/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let foundation = macro_role_foundation(root)?;
    let provider = folder.join("provider.rs");
    std::fs::write(&provider, format!("{foundation}{DIVISION_PROVIDER}"))?;
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let mut producer = source_export(
        "module-body-operations-v14",
        &provider,
        &metadata,
        &lowir,
        true,
    );
    producer.arg("--cap-lints=allow");
    super::artifacts::checked("provider", &folder, &mut producer)?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let client = folder.join("client.rs");
    std::fs::write(
        &client,
        r#"#![feature(no_core)]#![no_core]
#[unsafe(no_mangle)] pub extern "C" fn observed() -> u8 { <u8 as base::Div>::div(42, 2) }
fn main() {}
"#,
    )?;
    let client_lowir = folder.join("client.lowir");
    super::artifacts::checked(
        "client",
        &folder,
        &mut source_client(&client, &metadata, &client_lowir),
    )?;
    let text = std::fs::read_to_string(client_lowir)?;
    assert!(text.contains("call u8 @rgc$"), "{text}");
    assert!(text.contains("(42, 2)"), "{text}");
    let constant = folder.join("constant.rs");
    std::fs::write(
        &constant,
        r#"#![feature(no_core,const_trait_impl)]#![no_core]
const VALUE: u8 = <u8 as base::Div>::div(42, 2);
#[unsafe(no_mangle)] pub extern "C" fn observed() -> u8 { VALUE }
fn main() {}
"#,
    )?;
    let rejected = source_client(&constant, &metadata, &folder.join("constant.lowir")).output()?;
    let stderr = String::from_utf8(rejected.stderr)?;
    assert_eq!(rejected.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("trait obligation"), "{stderr}");
    let const_provider = folder.join("const-provider.rs");
    let const_trait = DIVISION_PROVIDER.replace("impl Div for u8", "const impl Div for u8");
    std::fs::write(&const_provider, format!("{foundation}{const_trait}"))?;
    let const_metadata = folder.join("const-provider.rgcrate");
    super::artifacts::checked(
        "const-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &const_provider,
            &const_metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let const_lowir = folder.join("const-client.lowir");
    super::artifacts::checked(
        "const-client",
        &folder,
        &mut source_client(&constant, &const_metadata, &const_lowir),
    )?;
    let text = std::fs::read_to_string(const_lowir)?;
    assert!(text.contains("return u8 42"), "{text}");
    let legacy = folder.join("legacy.rgcrate");
    let rejected = source_export("module-bodies-v13", &provider, &legacy, &lowir, true)
        .arg("--cap-lints=allow")
        .output()?;
    let stderr = String::from_utf8(rejected.stderr)?;
    assert_eq!(rejected.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("source Div contract transport is not implemented"),
        "{stderr}"
    );
    assert!(!legacy.exists());
    Ok(())
}

#[test]
fn cli_version_fourteen_uses_exact_subtraction_impl_effect() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-subtraction-transport/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let provider = folder.join("provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    std::fs::write(
        &provider,
        macro_role_foundation(root)?
            + r#"
#[lang = "sub"]
pub const trait Sub<Rhs = Self> {
    type Output;
    fn sub(self, rhs: Rhs) -> Self::Output;
}
const impl Sub for f64 {
    type Output = f64;
    fn sub(self, other: f64) -> f64 { self - other }
}
"#,
    )?;
    super::artifacts::checked(
        "subtraction-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let client = folder.join("client.rs");
    let lowir = folder.join("client.lowir");
    std::fs::write(
        &client,
        r#"
#![feature(no_core, const_trait_impl)]
#![no_core]
extern crate base;
const VALUE: i64 = <f64 as base::Sub>::sub(5.0, 3.0) as i64;
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> i64 { VALUE }
"#,
    )?;
    super::artifacts::checked(
        "subtraction-client",
        &folder,
        source_client(&client, &metadata, &lowir).arg("--crate-type=lib"),
    )?;
    let output = std::fs::read_to_string(lowir)?;
    assert!(output.contains("return i64 2"), "{output}");
    Ok(())
}

#[test]
fn cli_version_fourteen_transports_required_range_declarations() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-range-transport/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/range-helper-roles/source-declarations.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    super::artifacts::checked(
        "range-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &fixture,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    required_range_headers(&metadata)?;
    for (name, body, status) in [("bound", "", 0), ("no-impl", "allowed::<u8>();", 1)] {
        let client = folder.join(format!("{name}.rs"));
        std::fs::write(
            &client,
            format!(
                r#"#![feature(no_core,pattern_type_range_trait,range_role_fixture)]
#![no_core]
fn allowed<T: base::RangePattern>() {{}}
#[unsafe(no_mangle)] pub extern "C" fn observed() {{ {body} }}
"#
            ),
        )?;
        let output = source_client(&client, &metadata, &folder.join(format!("{name}.lowir")))
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(output.status.code(), Some(status), "{name}: {stderr}");
        if status == 1 {
            assert!(stderr.contains("trait obligation"), "{name}: {stderr}");
        }
    }
    let rejected = source_export(
        "module-bodies-v13",
        &fixture,
        &folder.join("older.rgcrate"),
        &lowir,
        true,
    )
    .arg("--cap-lints=allow")
    .output()?;
    let stderr = String::from_utf8(rejected.stderr)?;
    assert_eq!(rejected.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("source range helper contract transport is not implemented"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn cli_version_fourteen_transports_required_generic_trait_constants() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-associated-constants/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let provider = folder.join("provider.rs");
    let metadata = folder.join("provider.rgm");
    let lowir = folder.join("provider.lowir");
    std::fs::write(
        &provider,
        availability_foundation(root)?
            + r#"
#[stable(feature="required_table",since="1.0.0")]
pub trait Table<T> {
    #[stable(feature="required_table",since="1.0.0")]
    const ROW: T;
}
"#,
    )?;
    super::artifacts::checked(
        "required-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    required_constant_artifact(&metadata)?;
    let usage = r#"
struct A;
struct B;
impl base::Table<u8> for A { const ROW: u8 = 19; }
impl base::Table<u16> for B { const ROW: u16 = 513; }
#[unsafe(no_mangle)] pub extern "C" fn first() -> u8 { <A as base::Table<u8>>::ROW }
#[unsafe(no_mangle)] pub extern "C" fn second() -> u16 { <B as base::Table<u16>>::ROW }
"#;
    for (name, text, diagnostic) in [
        ("client", usage, ""),
        (
            "missing",
            "struct A; impl base::Table<u8> for A {} fn main() {}",
            "missing a required associated constant",
        ),
        (
            "mismatch",
            "struct A; impl base::Table<u8> for A { const ROW: u16 = 19; }",
            "incompatible types",
        ),
    ] {
        let client = folder.join(format!("{name}.rs"));
        let output = folder.join(format!("{name}.lowir"));
        std::fs::write(
            &client,
            format!("#![feature(no_core)]\n#![no_core]\nextern crate base;\n{text}"),
        )?;
        let result = source_client(&client, &metadata, &output)
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(result.stderr)?;
        std::fs::write(folder.join(format!("{name}.stderr")), &stderr)?;
        let status = i32::from(!diagnostic.is_empty());
        assert_eq!(result.status.code(), Some(status), "{name}: {stderr}");
        if status == 0 {
            let written = std::fs::read_to_string(output)?;
            assert!(
                written.contains("return u8 19") && written.contains("return u16 513"),
                "{written}"
            );
        } else {
            assert!(stderr.contains(diagnostic), "{stderr}");
        }
    }
    let older = source_export(
        "module-bodies-v13",
        &provider,
        &folder.join("old.rgm"),
        &lowir,
        true,
    )
    .arg("--cap-lints=allow")
    .output()?;
    assert_eq!(older.status.code(), Some(1));
    assert!(
        String::from_utf8(older.stderr)?
            .contains("source associated constants require their wider profile")
    );
    Ok(())
}

fn required_constant_artifact(metadata: &Path) -> Result {
    use rustgm::support::crates::{self, generics::source::Profile};
    let bytes = std::fs::read(metadata)?;
    let packet = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&packet)?, bytes);
    let descriptor = packet.source_descriptor().ok_or("source descriptor")?;
    assert_eq!(
        descriptor.profile(),
        Profile::Source2024ModuleBodyOperations
    );
    Ok(())
}

#[test]
fn cli_version_fourteen_publishes_constant_initializer_original() -> Result {
    use rustgm::support::crates::{self, owner_catalog::BodyRef};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-associated-constants/provided-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let provider = folder.join("provider.rs");
    let metadata = folder.join("provider.rgm");
    let lowir = folder.join("provider.lowir");
    std::fs::write(
        &provider,
        availability_foundation(root)?
            + r#"
#[stable(feature="provided_value",since="1.0.0")]
pub fn before() -> u8 { 4 }
#[stable(feature="provided_value",since="1.0.0")]
pub trait Value<T> {
    #[stable(feature="provided_value",since="1.0.0")]
    const DEFAULT: u8 = { let value = 8u8; value };
}
struct Defaulted;
#[stable(feature="provided_value",since="1.0.0")]
impl Value<u8> for Defaulted {}
#[stable(feature="provided_value",since="1.0.0")]
pub struct Overridden {}
#[stable(feature="provided_value",since="1.0.0")]
impl Value<u8> for Overridden {
    const DEFAULT: u8 = { let value = 9u8; value };
}
#[stable(feature="provided_value",since="1.0.0")]
pub fn after() -> u8 { 12 }
"#,
    )?;
    super::artifacts::checked(
        "provided-constant-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    let packet = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&packet)?, bytes);
    let bodies = packet
        .source_descriptor()
        .ok_or("provided source descriptor")?
        .compiler_operations()?;
    // The shared foundation also supplies the generic identity original.
    assert_eq!(
        bodies.ordinary_bodies().collect::<Vec<_>>(),
        [BodyRef(0), BodyRef(1), BodyRef(2), BodyRef(3), BodyRef(4)]
    );
    assert_eq!(bodies.ordinary_input_count(), 5);
    provided_constant_clients(&folder, &metadata)
}

fn provided_constant_clients(folder: &Path, metadata: &Path) -> Result {
    for (name, source, expected) in [
        (
            "import-only",
            r#"#[unsafe(no_mangle)] pub extern "C" fn imported() -> u8 { 4 }"#,
            "return u8 4",
        ),
        (
            "inherited-default",
            r#"
struct Foreign;
impl base::Value<u8> for Foreign {}
#[unsafe(no_mangle)]
pub extern "C" fn selected() -> u8 { <Foreign as base::Value<u8>>::DEFAULT }
"#,
            "return u8 8",
        ),
        (
            "imported-override",
            r#"
#[unsafe(no_mangle)]
pub extern "C" fn selected() -> u8 { <base::Overridden as base::Value<u8>>::DEFAULT }
"#,
            "return u8 9",
        ),
        (
            "local-override",
            r#"
struct Foreign;
impl base::Value<u8> for Foreign {
    const DEFAULT: u8 = { let own = 21; own };
}
#[unsafe(no_mangle)]
pub extern "C" fn selected() -> u8 { <Foreign as base::Value<u8>>::DEFAULT }
"#,
            "return u8 21",
        ),
    ] {
        let client = folder.join(format!("{name}.rs"));
        let output = folder.join(format!("{name}.lowir"));
        std::fs::write(
            &client,
            format!("#![feature(no_core)]\n#![no_core]\nextern crate base;\n{source}"),
        )?;
        super::artifacts::checked(
            name,
            folder,
            source_client(&client, metadata, &output)
                .args(["--crate-type=lib", "--cap-lints=allow"]),
        )?;
        let written = std::fs::read_to_string(output)?;
        assert!(
            written.lines().any(|line| line.trim() == expected),
            "{written}"
        );
    }
    Ok(())
}

#[test]
fn cli_version_fourteen_replays_symbolic_supertrait_constants() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-symbolic-constants/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let provider = folder.join("provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let provider_lowir = folder.join("provider.lowir");
    std::fs::write(
        &provider,
        macro_role_foundation(root)?
            + r"
pub trait Part<T> {
    const PART: u8;
}
pub trait Total<T>: Part<T> {
    const TOTAL: u8 = { let value = Self::PART; value };
}
pub fn selected<T: Total<u8>>() -> u8 { T::TOTAL }
",
    )?;
    super::artifacts::checked(
        "symbolic-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &provider_lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let packet = std::fs::read(&metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&packet)?)?,
        packet
    );
    let client = folder.join("client.rs");
    let client_lowir = folder.join("client.lowir");
    std::fs::write(
        &client,
        r#"
#![feature(no_core)]
#![no_core]
extern crate base;
struct First;
impl base::Part<u8> for First { const PART: u8 = 17; }
impl base::Total<u8> for First {}
struct Second;
impl base::Part<u8> for Second { const PART: u8 = 23; }
impl base::Total<u8> for Second { const TOTAL: u8 = 29; }
#[unsafe(no_mangle)]
pub extern "C" fn first() -> u8 { <First as base::Total<u8>>::TOTAL }
#[unsafe(no_mangle)]
pub extern "C" fn second() -> u8 { <Second as base::Total<u8>>::TOTAL }
"#,
    )?;
    super::artifacts::checked(
        "symbolic-client",
        &folder,
        source_client(&client, &metadata, &client_lowir).arg("--crate-type=lib"),
    )?;
    let output = std::fs::read_to_string(client_lowir)?;
    assert!(
        output.contains("return u8 17") && output.contains("return u8 29"),
        "{output}"
    );
    rejects_missing_constant_premise(root, &folder)?;
    Ok(())
}

fn rejects_missing_constant_premise(root: &Path, folder: &Path) -> Result {
    use rustgm::support::crates::{self, owner_catalog::BodyKind};
    const ASSOCIATED_READ: [u8; 3] = [18, 2, 5];
    const ARGUMENT_RANGE: u8 = 9;
    let provider = folder.join("premise-provider.rs");
    let metadata = folder.join("premise-provider.rgcrate");
    let lowir = folder.join("premise-provider.lowir");
    std::fs::write(
        &provider,
        macro_role_foundation(root)?
            + r"
pub trait Part<T> { const PART: u8; }
pub trait Unrelated<T> { const UNRELATED: u8 = 13; }
pub fn symbolic_premise_value<T: Part<u8>>() -> u8 { T::PART }
",
    )?;
    super::artifacts::checked(
        "premise-provider",
        folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let source = folder.join("unused.rs");
    std::fs::write(
        &source,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n",
    )?;
    super::artifacts::checked(
        "premise-client",
        folder,
        source_client(&source, &metadata, &folder.join("premise-client.lowir"))
            .arg("--crate-type=lib"),
    )?;
    let mut packet = std::fs::read(metadata)?;
    let decoded = crates::decode(&packet)?;
    let owners = decoded
        .generic_payload()
        .ok_or("symbolic payload")?
        .owners();
    let mut defaults = owners
        .rows()
        .iter()
        .filter(|row| row.kind == BodyKind::Constant);
    let other = defaults
        .next()
        .ok_or("alternate constant initializer")?
        .declaration
        .0;
    assert!(
        defaults.next().is_none(),
        "sole default constant is Unrelated::UNRELATED"
    );
    // Preserve the result type and argument range. The alternate member has
    // the same declaration shape and arity, with a different trait obligation.
    let mut matches = packet
        .windows(8)
        .enumerate()
        .filter(|(_, row)| row.starts_with(&ASSOCIATED_READ) && row[7] == ARGUMENT_RANGE);
    let (offset, _) = matches.next().ok_or("encoded symbolic operand")?;
    assert!(matches.next().is_none(), "symbolic operand must be unique");
    let member = offset + ASSOCIATED_READ.len();
    packet[member..member + 4].copy_from_slice(&other.to_le_bytes());
    assert_eq!(crates::encode(&crates::decode(&packet)?)?, packet);

    let metadata = folder.join("missing-premise.rgcrate");
    let lowir = folder.join("unused.lowir");
    std::fs::write(&metadata, packet)?;
    let output = source_client(&source, &metadata, &lowir)
        .arg("--crate-type=lib")
        .output()?;
    std::fs::write(folder.join("missing-premise.stderr"), &output.stderr)?;
    let error = String::from_utf8(output.stderr)?;
    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(
        error.contains("body trait requirement is not proved"),
        "{error}"
    );
    Ok(())
}

fn required_range_headers(metadata: &Path) -> Result {
    use rustgm::support::crates::{self, generics::source::Profile};
    let bytes = std::fs::read(metadata)?;
    let packet = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&packet)?, bytes);
    let source = packet
        .source_descriptor()
        .ok_or("range source descriptor")?;
    assert_eq!(source.profile(), Profile::Source2024ModuleBodyOperations);
    let bodies = source.compiler_operations()?;
    assert_eq!(bodies.ordinary_bodies().len(), 0);
    assert!(bodies.operations().is_empty());
    Ok(())
}

#[test]
fn cli_version_fourteen_binds_private_self_alias_to_actual_root() -> Result {
    use rustgm::support::crates::{self, generics::source::BindingSource};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-root-self-alias/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let foundation = macro_role_foundation(root)?;
    let provider = folder.join("provider.rs");
    std::fs::write(
        &provider,
        format!(
            "{foundation}
extern crate self as local;
pub mod values {{ pub fn seven() -> u8 {{ 7 }} }}
pub fn observed() -> u8 {{ local::values::seven() }}
"
        ),
    )?;
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    super::artifacts::checked(
        "provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    let packet = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&packet)?, bytes);
    let source = packet.source_descriptor().ok_or("source descriptor")?;
    let rows = source.module_contracts()?.rows();
    let alias = rows
        .namespace
        .bindings
        .iter()
        .find(|binding| binding.name == "local")
        .ok_or("root alias binding")?;
    let root = rows.namespace.root;
    assert_eq!(alias.module, root);
    assert_eq!(alias.target.declaration, root);
    assert_eq!(alias.source, BindingSource::Import);
    assert_eq!(
        alias.visibility,
        crates::generics::source::ModuleVisibility::Within(root)
    );
    for (name, call, status) in [
        ("allowed", "base::observed", 0),
        ("private", "base::local::values::seven", 1),
    ] {
        let client = folder.join(format!("{name}.rs"));
        std::fs::write(
            &client,
            format!("#![feature(no_core)]#![no_core]\npub fn read() -> u8 {{ {call}() }}\n"),
        )?;
        let output = source_client(&client, &metadata, &folder.join(format!("{name}.lowir")))
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(output.status.code(), Some(status), "{name}: {stderr}");
        if status == 1 {
            assert!(stderr.contains("private"), "{stderr}");
        }
    }
    let legacy = folder.join("legacy.rgcrate");
    let output = source_export("module-bodies-v13", &provider, &legacy, &lowir, true)
        .arg("--cap-lints=allow")
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("source declaration kind is outside the initial source profile"));
    assert!(!legacy.exists());
    Ok(())
}

#[test]
fn cli_version_fourteen_preserves_f16_and_f128_type_identity() -> Result {
    use rustgm::support::crates::{self, generics::source::Profile};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-float-types/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let provider = folder.join("provider.rs");
    let foundation = macro_role_foundation(root)?;
    std::fs::write(
        &provider,
        format!(
            "#![feature(f16,f128)]\n{foundation}\npub struct FloatParts {{ pub half: f16, pub quad: f128 }}\n"
        ),
    )?;
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    super::artifacts::checked(
        "provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    let packet = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&packet)?, bytes);
    let descriptor = packet.source_descriptor().ok_or("source descriptor")?;
    assert_eq!(
        descriptor.profile(),
        Profile::Source2024ModuleBodyOperations
    );
    for (name, ty, status) in [("allowed", "f128", 0), ("wrong-type", "f64", 1)] {
        let client = folder.join(format!("{name}.rs"));
        std::fs::write(
            &client,
            format!(
                "#![feature(no_core,f16,f128)]#![no_core]\nfn half(v: &base::FloatParts) -> &f16 {{ &v.half }}\nfn quad(v: &base::FloatParts) -> &{ty} {{ &v.quad }}\n"
            ),
        )?;
        let output = source_client(&client, &metadata, &folder.join(format!("{name}.lowir")))
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(output.status.code(), Some(status), "{name}: {stderr}");
        if status == 1 {
            assert!(stderr.contains("incompatible types"), "{stderr}");
        }
    }
    let legacy = folder.join("legacy.rgcrate");
    let output = source_export("module-bodies-v13", &provider, &legacy, &lowir, true)
        .arg("--cap-lints=allow")
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("source scalar type is outside the supported profile"));
    assert!(!legacy.exists());
    Ok(())
}

#[test]
fn cli_version_fourteen_transports_exact_reflection_nominal_roles() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/reflection-metadata-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let foundation = macro_role_foundation(root)?;
    let provider = folder.join("provider.rs");
    std::fs::write(
        &provider,
        format!(
            r#"{foundation}
pub mod reflected {{
    #[lang = "type_info"]
    #[non_exhaustive]
    pub struct Info {{ pub kind: u8 }}
    #[lang = "type_id"]
    pub struct Identity {{ pub data: [*const (); 2] }}
}}
pub fn describe() -> reflected::Info {{ reflected::Info {{ kind: 42 }} }}
"#
        ),
    )?;
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let mut producer = source_export(
        "module-body-operations-v14",
        &provider,
        &metadata,
        &lowir,
        true,
    );
    producer.arg("--cap-lints=allow");
    super::artifacts::checked("provider", &folder, &mut producer)?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let client = folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]#![no_core]\n\
         #[unsafe(no_mangle)] pub extern \"C\" fn observed() -> u8 { base::describe().kind }\n\
         #[unsafe(no_mangle)] pub extern \"C\" fn words(raw: usize) -> usize {\n\
             let value = raw as *const base::reflected::Identity;\n\
             unsafe { (&raw const (*value).data) as usize }\n\
         }\nfn main() {}\n",
    )?;
    super::artifacts::checked(
        "client",
        &folder,
        &mut source_client(&client, &metadata, &folder.join("client.lowir")),
    )?;
    let legacy = source_export(
        "module-bodies-v13",
        &provider,
        &folder.join("legacy.rgcrate"),
        &folder.join("legacy.lowir"),
        true,
    )
    .arg("--cap-lints=allow")
    .output()?;
    assert_eq!(legacy.status.code(), Some(1));
    let stderr = String::from_utf8(legacy.stderr)?;
    assert!(
        stderr.contains("source type reflection contract transport is not implemented"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn cli_version_five_keeps_exact_c_publication_and_source_provider_boundaries() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/exact-source-linkage-v5/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/exact-source-linkage/provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let output = super::artifacts::checked(
        "version-five",
        &folder,
        &mut source_export("module-definitions-v5", &fixture, &metadata, &lowir, true),
    )?;
    assert!(output.stderr.is_empty());
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(bytes.get(13..15), Some([5, 0].as_slice()));
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    for (profile, provider, expected) in [
        ("module-definitions-v5", false, 2),
        ("module-availability-v3", true, 1),
    ] {
        let rejected_metadata = folder.join(format!("rejected-{expected}.rgcrate"));
        let output = source_export(
            profile,
            &fixture,
            &rejected_metadata,
            &folder.join(format!("rejected-{expected}.lowir")),
            provider,
        )
        .output()?;
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!rejected_metadata.exists());
    }
    Ok(())
}

#[test]
fn cli_version_seven_keeps_written_effects_and_old_profile_boundaries() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-callable-effects-v1/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/source-callable-effects/provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let output = super::artifacts::checked(
        "version-seven",
        &folder,
        &mut source_export("module-callables-v7", &fixture, &metadata, &lowir, true),
    )?;
    assert!(output.stderr.is_empty());
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(bytes.get(13..15), Some([7, 0].as_slice()));
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let rejected = folder.join("older.rgcrate");
    let output = source_export(
        "module-compatibility-v6",
        &fixture,
        &rejected,
        &folder.join("older.lowir"),
        true,
    )
    .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!rejected.exists());
    Ok(())
}

#[test]
fn cli_source_client_feature_requests_keep_plain_library_exports() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-callable-effects-v1/consumer-cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixtures = root.join("pa31/tests/spec/source-callable-effects");
    let provider = folder.join("provider.rgcrate");
    let output = source_export(
        "module-callables-v7",
        &fixtures.join("combined-provider.rs"),
        &provider,
        &folder.join("provider.lowir"),
        true,
    )
    .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = folder.join("client.rgcrate");
    let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-lowir",
            "--crate-type=lib",
            "--crate-name",
            "observer",
            "--crate-disambiguator",
            "native",
            "--source-client",
            "base@exact_cli",
            "--extern",
        ])
        .arg(format!("base={}", provider.display()))
        .arg("--metadata-output")
        .arg(&metadata)
        .arg("-o")
        .arg(folder.join("client.lowir"))
        .arg(fixtures.join("native-client.rs"))
        .output()?;
    std::fs::write(folder.join("client.stderr"), &output.stderr)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let warnings = String::from_utf8(output.stderr)?;
    assert_eq!(warnings.matches("warning[deprecated]:").count(), 2);
    let bytes = std::fs::read(metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    Ok(())
}

#[test]
fn cli_version_eight_retains_actual_unsafe_calls_and_legacy_denials() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/source-callable-effects-v1/safety-cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/source-call-safety/preserved-provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    let output = super::artifacts::checked(
        "version-eight",
        &folder,
        &mut source_export("module-safety-v8", &fixture, &metadata, &lowir, true),
    )?;
    assert!(output.stderr.is_empty());
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(bytes.get(13..15), Some([8, 0].as_slice()));
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let rejected = folder.join("older.rgcrate");
    let output = source_export(
        "module-callables-v7",
        &fixture,
        &rejected,
        &folder.join("older.lowir"),
        true,
    )
    .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!rejected.exists());
    Ok(())
}

#[test]
fn cli_version_fourteen_transports_closed_module_type_aliases() -> Result {
    use rustgm::support::crates;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-type-aliases/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let provider = folder.join("provider.rs");
    let mut source = macro_role_foundation(root)?;
    source.push_str(
        r"
pub mod digits {
    pub type Digit32 = u32;
    pub type Quad = (Digit32, u16);
    type Hiddentype = u64;
}
pub use digits::Digit32 as Word;
pub struct Wrapper { pub value: u32 }
pub type Wrapped = Wrapper;
mod nested { pub type Reexported = u32; }
pub use nested::Reexported;
",
    );
    std::fs::write(&provider, source)?;
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    super::artifacts::checked(
        "provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for (name, source, status) in [
        (
            "allowed",
            "pub fn value() -> base::digits::Digit32 { 42 }\npub fn word() -> base::Word { 7 }\nfn pair(_: &base::digits::Quad) {}\nfn wrapped(v: &base::Wrapped) -> &base::Wrapper { v }\nfn reexported() -> base::Reexported { 7 }",
            0,
        ),
        ("private", "fn hidden(_: &base::digits::Hiddentype) {}", 1),
        (
            "wrong",
            "fn value(x: u64) -> base::digits::Digit32 { x }",
            1,
        ),
        (
            "generic-arguments",
            "fn applied(_: base::digits::Digit32<u8>) {}",
            1,
        ),
    ] {
        let client = folder.join(format!("{name}.rs"));
        std::fs::write(
            &client,
            format!("#![feature(no_core)]#![no_core]\n{source}\n"),
        )?;
        let output = source_client(&client, &metadata, &folder.join(format!("{name}.lowir")))
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(output.status.code(), Some(status), "{name}: {stderr}");
        if name == "wrong" {
            assert!(stderr.contains("incompatible types"), "{stderr}");
        }
    }
    let legacy = folder.join("legacy.rgcrate");
    let output = source_export("module-bodies-v13", &provider, &legacy, &lowir, true)
        .arg("--cap-lints=allow")
        .output()?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("named, associated or generic alias transport"),
        "{stderr}"
    );
    assert!(!legacy.exists());
    Ok(())
}

#[test]
fn cli_version_fourteen_reproves_generic_alias_projections_at_use() -> Result {
    use rustgm::support::crates;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/v14-generic-aliases/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let provider = folder.join("provider.rs");
    let metadata = folder.join("provider.rgcrate");
    let lowir = folder.join("provider.lowir");
    std::fs::write(
        &provider,
        macro_role_foundation(root)?
            + r"
pub type Identity<T> = T;
pub struct Wrapped<T>(pub T);
pub type WrappedValue<T> = Wrapped<T>;
pub trait NotAllOnesHelper { type Type; }
pub type NotAllOnes<T> = <T as NotAllOnesHelper>::Type;
impl NotAllOnesHelper for u32 { type Type = u16; }
",
    )?;
    super::artifacts::checked(
        "generic-alias-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &lowir,
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for (name, source, expected) in [
        (
            "concrete",
            "pub fn value() -> base::Identity<base::NotAllOnes<u32>> { 37 }",
            None,
        ),
        (
            "nominal",
            "fn wrapped(value: &base::WrappedValue<u32>) -> &base::Wrapped<u32> { value }",
            None,
        ),
        ("unused", "type Unused = base::NotAllOnes<bool>;", None),
        (
            "demanded",
            "pub fn demand(_: base::NotAllOnes<bool>) {}",
            Some("associated type"),
        ),
        (
            "missing-argument",
            "pub fn demand(_: base::Identity) {}",
            Some("argument"),
        ),
    ] {
        let client = folder.join(format!("{name}.rs"));
        std::fs::write(
            &client,
            format!("#![feature(no_core)]#![no_core]\n{source}\n"),
        )?;
        let target = folder.join(format!("{name}.lowir"));
        let output = source_client(&client, &metadata, &target)
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(output.stderr)?;
        if let Some(diagnostic) = expected {
            assert_eq!(output.status.code(), Some(1), "{name}: {stderr}");
            assert!(stderr.contains(diagnostic), "{name}: {stderr}");
        } else {
            assert!(output.status.success(), "{name}: {stderr}");
            if name == "concrete" {
                let written = std::fs::read_to_string(target)?;
                assert!(written.contains("return u16 37"), "{written}");
            }
        }
    }
    Ok(())
}

#[test]
fn cli_version_fourteen_reproves_imported_pointee_metadata() -> Result {
    use rustgm::support::crates;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/pointee-metadata-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let source = std::fs::read_to_string(
        root.join("pa31/tests/spec/trait-aliases/source-thin/native-provider.rs"),
    )?;
    let (foundation, _) = source
        .split_once("pub fn scalar")
        .ok_or("Pointee role fixture")?;
    let provider = folder.join("provider.rs");
    std::fs::write(&provider, foundation)?;
    let metadata = folder.join("provider.rgcrate");
    super::artifacts::checked(
        "pointee-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &folder.join("provider.lowir"),
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);

    let client = folder.join("client.rs");
    std::fs::write(
        &client,
        r"#![feature(no_core)]#![no_core]
pub fn scalar() -> <u8 as base::Pointee>::Metadata { () }
pub fn slice(value: <[u8] as base::Pointee>::Metadata) -> usize { value }
",
    )?;
    super::artifacts::checked(
        "pointee-client",
        &folder,
        source_client(&client, &metadata, &folder.join("client.lowir"))
            .args(["--crate-type=lib", "--cap-lints=allow"]),
    )?;
    let wrong = folder.join("wrong.rs");
    std::fs::write(
        &wrong,
        r"#![feature(no_core)]#![no_core]
pub fn scalar() -> <u8 as base::Pointee>::Metadata { 4_usize }
",
    )?;
    let failed = source_client(&wrong, &metadata, &folder.join("wrong.lowir"))
        .args(["--crate-type=lib", "--cap-lints=allow"])
        .output()?;
    assert_eq!(failed.status.code(), Some(1));
    let stderr = String::from_utf8(failed.stderr)?;
    assert!(stderr.contains("incompatible types"), "{stderr}");
    Ok(())
}

#[test]
fn cli_version_fourteen_imports_thin_trait_alias_clauses() -> Result {
    use rustgm::support::crates;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/thin-metadata-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let source = std::fs::read_to_string(
        root.join("pa31/tests/spec/trait-aliases/source-thin/native-provider.rs"),
    )?;
    let (foundation, _) = source
        .split_once("pub fn scalar")
        .ok_or("Pointee role fixture")?;
    let thin = source
        .lines()
        .find(|line| line.starts_with("pub trait Thin ="))
        .ok_or("Thin alias fixture")?;
    let provider = folder.join("provider.rs");
    std::fs::write(&provider, format!("{foundation}{thin}\n"))?;
    let metadata = folder.join("provider.rgcrate");
    super::artifacts::checked(
        "thin-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &folder.join("provider.lowir"),
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for (name, use_alias) in [
        (
            "thin",
            "pub fn observed(address: usize) -> u8 { narrow(address as *const u8); 7 }",
        ),
        (
            "wide",
            "fn observed(pointer: *const [u8]) { narrow(pointer); }",
        ),
    ] {
        let client = folder.join(format!("{name}.rs"));
        std::fs::write(
            &client,
            format!(
                r"#![feature(no_core,trait_alias)]#![no_core]
fn narrow<T: base::Thin + ?base::Sized>(_: *const T) -> <T as base::Pointee>::Metadata {{ () }}
{use_alias}
"
            ),
        )?;
        let output = folder.join(format!("{name}.lowir"));
        let result = source_client(&client, &metadata, &output)
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(result.stderr)?;
        if name == "thin" {
            assert!(result.status.success(), "{stderr}");
            let lowir = std::fs::read_to_string(output)?;
            assert!(lowir.contains("return u8 7"), "{lowir}");
        } else {
            assert_eq!(result.status.code(), Some(1), "{stderr}");
            assert!(stderr.contains("incompatible types"), "{stderr}");
        }
    }
    Ok(())
}

#[test]
fn cli_version_fourteen_keeps_imported_foreign_types_opaque() -> Result {
    use rustgm::support::crates;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/foreign-metadata-v14/cli-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let source = std::fs::read_to_string(
        root.join("pa31/tests/spec/foreign-types/identity-and-pointee.rs"),
    )?;
    let (foundation, _) = source
        .split_once("unsafe extern")
        .ok_or("foreign role fixture")?;
    let provider = folder.join("provider.rs");
    std::fs::write(
        &provider,
        format!(
            r#"{foundation}
unsafe extern "C" {{ pub type VTable; pub type Other; }}
pub struct Holder<T: PointeeSized> {{ pub raw: *const T }}
pub struct Carrier {{ pub table: Holder<VTable> }}
pub type TablePtr = *const VTable;
"#
        ),
    )?;
    let metadata = folder.join("provider.rgcrate");
    super::artifacts::checked(
        "foreign-provider",
        &folder,
        source_export(
            "module-body-operations-v14",
            &provider,
            &metadata,
            &folder.join("provider.lowir"),
            true,
        )
        .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for (name, body, diagnostic) in [
        (
            "pointer",
            "fn table(value: base::Carrier) -> base::TablePtr { value.table.raw }\n\
            pub fn opaque(address: usize) -> usize { (address as base::TablePtr) as usize }\n\
            pub fn metadata() -> <base::VTable as base::Pointee>::Metadata { () }",
            None,
        ),
        (
            "different",
            "fn wrong(input: *const base::Other) -> base::TablePtr { input }",
            Some("incompatible types"),
        ),
        (
            "by-value",
            "fn wrong(_: base::VTable) {}",
            Some("function parameters and result must be sized"),
        ),
    ] {
        let client = folder.join(format!("{name}.rs"));
        std::fs::write(
            &client,
            format!("#![feature(no_core)]#![no_core]\n{body}\n"),
        )?;
        let result = source_client(&client, &metadata, &folder.join(format!("{name}.lowir")))
            .args(["--crate-type=lib", "--cap-lints=allow"])
            .output()?;
        let stderr = String::from_utf8(result.stderr)?;
        if let Some(diagnostic) = diagnostic {
            assert_eq!(result.status.code(), Some(1), "{name}: {stderr}");
            assert!(stderr.contains(diagnostic), "{name}: {stderr}");
        } else {
            assert!(result.status.success(), "{name}: {stderr}");
        }
    }
    Ok(())
}
