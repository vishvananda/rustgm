//! Fresh CLI processes consume serialized source nominal constructors and ownership.
use rustgm::support::crates;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Packet {
    root: PathBuf,
    folder: PathBuf,
    metadata: PathBuf,
}
impl Packet {
    fn publish(label: &str) -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("root")?
            .to_owned();
        let folder = root.join(format!(
            "obj/parallel/source-transport/{label}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let metadata = folder.join("base.rgcrate");
        super::artifacts::checked(
            "provider",
            &folder,
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .args([
                    "--emit-lowir",
                    "--crate-type=lib",
                    "--crate-name",
                    "base",
                    "--crate-disambiguator",
                    "nominal_v10",
                    "--source-core",
                    "base@nominal_v10",
                    "--source-profile",
                    "module-nominals-v10",
                    "--metadata-output",
                ])
                .arg(&metadata)
                .arg("-o")
                .arg(folder.join("provider.lowir"))
                .arg(root.join("pa29/tests/spec/source-nominals/provider.rs")),
        )?;
        Ok(Self {
            root,
            folder,
            metadata,
        })
    }
    fn consumer(&self, name: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command
            .args([
                "--emit-lowir",
                "--crate-type=lib",
                "--crate-name",
                "client",
                "--crate-disambiguator",
                "nominal_client",
                "--source-client",
                "base@nominal_v10",
                "--extern",
            ])
            .arg(format!("base={}", self.metadata.display()))
            .arg("-o")
            .arg(self.folder.join(format!("{name}.lowir")))
            .arg(
                self.root
                    .join(format!("pa29/tests/spec/source-nominals/{name}.rs")),
            );
        command
    }
}
#[test]
fn source_nominal_constructors_survive_wire_and_fresh_ownership_consumers() -> Result {
    let packet = Packet::publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([10, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let descriptor = interface.source_descriptor().ok_or("source descriptor")?;
    assert_eq!(
        descriptor.profile(),
        crates::SourceProfile::Source2024ModuleNominals
    );
    super::artifacts::checked("client", &packet.folder, &mut packet.consumer("client"))?;
    for (name, diagnostic) in [
        ("moved", "uninitialized or moved"),
        ("private", "aggregate field is private"),
        ("wrong", "incompatible types"),
        (
            "escape",
            "generic nominals are outside the metadata profile",
        ),
        ("wrong-owner", "incompatible types"),
        (
            "escape-body",
            "required lifetime does not outlive its target",
        ),
    ] {
        let output = packet.consumer(name).output()?;
        std::fs::write(packet.folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(1),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let rejected_output = packet.folder.join(format!("{name}.lowir"));
        assert!(
            !rejected_output.exists() || std::fs::metadata(rejected_output)?.len() == 0,
            "{name} emitted executable LowIR after rejection"
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn source_nominal_constructor_executes_imported_originals_natively() -> Result {
    let packet = Packet::publish("native")?;
    super::artifacts::checked("client", &packet.folder, &mut packet.consumer("client"))?;
    let object = packet.folder.join("client.o");
    super::artifacts::checked(
        "backend",
        &packet.folder,
        Command::new("python3")
            .arg(packet.root.join("scripts/backend.py"))
            .arg("--emit-object")
            .arg(packet.folder.join("client.lowir"))
            .arg("-o")
            .arg(&object),
    )?;
    let observer = packet.folder.join("observer.c");
    std::fs::write(
        &observer,
        "int entry(void); int main(void) { return entry(); }\n",
    )?;
    let executable = packet.folder.join("native");
    super::artifacts::checked(
        "link",
        &packet.folder,
        Command::new("cc")
            .arg(&observer)
            .arg(&object)
            .arg("-o")
            .arg(&executable),
    )?;
    let output = Command::new(executable).output()?;
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    Ok(())
}
