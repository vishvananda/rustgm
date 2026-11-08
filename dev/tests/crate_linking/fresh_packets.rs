//! Fresh-process source packets: one provider export, many independent consumers.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Packet {
    pub root: PathBuf,
    pub folder: PathBuf,
    spec: &'static str,
    provider: &'static str,
    pub metadata: PathBuf,
}
impl Packet {
    /// `group` names the retained output tree under `obj/parallel`.
    pub fn new(
        group: &str,
        spec: &'static str,
        label: &str,
        provider: &'static str,
    ) -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("root")?
            .to_owned();
        let folder = root.join(format!(
            "obj/parallel/{group}/{label}-{provider}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let metadata = folder.join(format!("{provider}.rgcrate"));
        Ok(Self {
            root,
            folder,
            spec,
            provider,
            metadata,
        })
    }
    pub fn publish(
        group: &str,
        spec: &'static str,
        label: &str,
        provider: &'static str,
        profile: &str,
    ) -> Result<Self> {
        let packet = Self::new(group, spec, label, provider)?;
        let source = packet.fixture(provider);
        super::artifacts::checked(
            "provider",
            &packet.folder,
            &mut packet.export(profile, &source, &packet.metadata),
        )?;
        Ok(packet)
    }
    pub fn fixture(&self, name: &str) -> PathBuf {
        self.root.join(format!("{}/{name}.rs", self.spec))
    }
    pub fn owner(&self) -> String {
        format!("base@{}_v12", self.provider)
    }
    pub fn export(&self, profile: &str, source: &Path, metadata: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command
            .args(["--emit-lowir", "--crate-type=lib", "--crate-name", "base"])
            .arg("--crate-disambiguator")
            .arg(format!("{}_v12", self.provider))
            .arg("--source-core")
            .arg(self.owner())
            .args(["--source-profile", profile, "--metadata-output"])
            .arg(metadata)
            .arg("-o")
            .arg(metadata.with_extension("lowir"))
            .arg(source);
        command
    }
    pub fn consumer(&self, source: &Path, label: &str, metadata: &Path, owner: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command
            .args(["--emit-lowir", "--crate-type=lib", "--crate-name", "client"])
            .args([
                "--crate-disambiguator",
                "branch_client",
                "--source-client",
                owner,
            ])
            .arg("--extern")
            .arg(format!("base={}", metadata.display()))
            .arg("-o")
            .arg(self.folder.join(format!("{label}.lowir")))
            .arg(source);
        command
    }
    pub fn client(&self, fixture: &str) -> Command {
        self.consumer(
            &self.fixture(fixture),
            fixture,
            &self.metadata,
            &self.owner(),
        )
    }
    pub fn native(&self, label: &str) -> Result<Output> {
        let object = self.folder.join(format!("{label}.o"));
        super::artifacts::checked(
            "backend",
            &self.folder,
            Command::new("python3")
                .arg(self.root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(self.folder.join(format!("{label}.lowir")))
                .arg("-o")
                .arg(&object),
        )?;
        let observer = self.folder.join("observer.c");
        std::fs::write(
            &observer,
            "int entry(void); int main(void) { return entry(); }\n",
        )?;
        let executable = self.folder.join(label);
        super::artifacts::checked(
            "link",
            &self.folder,
            Command::new("cc")
                .arg(&observer)
                .arg(&object)
                .arg("-o")
                .arg(&executable),
        )?;
        // An abnormal exit must not leave a core image in the output tree.
        Ok(Command::new("sh")
            .args(["-c", "ulimit -c 0; exec \"$0\""])
            .arg(&executable)
            .output()?)
    }
    pub fn rejected(&self, label: &str, command: &mut Command, diagnostic: &str) -> Result {
        let output = command.output()?;
        std::fs::write(self.folder.join(format!("{label}.stderr")), &output.stderr)?;
        let text = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{label}: {text}");
        assert!(text.contains(diagnostic), "{label}: {text}");
        let emitted = self.folder.join(format!("{label}.lowir"));
        assert!(
            !emitted.exists() || std::fs::metadata(emitted)?.len() == 0,
            "{label} emitted executable LowIR after rejection"
        );
        Ok(())
    }
}
pub fn unique(bytes: &[u8], pattern: &[u8]) -> Result<usize> {
    let mut found = bytes
        .windows(pattern.len())
        .enumerate()
        .filter(|(_, window)| *window == pattern)
        .map(|(offset, _)| offset);
    let first = found.next().ok_or("anchored row is absent")?;
    if found.next().is_some() {
        return Err("anchored row is ambiguous".into());
    }
    Ok(first)
}
/// The one offset at which two otherwise identical packets differ.
pub fn difference(left: &[u8], right: &[u8]) -> Result<usize> {
    if left.len() != right.len() {
        return Err("packet lengths differ".into());
    }
    let found: Vec<usize> = left
        .iter()
        .zip(right)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(offset, _)| offset)
        .collect();
    let [offset] = *found.as_slice() else {
        return Err(format!("packet differences {found:?}").into());
    };
    Ok(offset)
}
pub fn relabel(bytes: &[u8], version: u8) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[13] = version;
    changed
}
pub fn with_id(bytes: &[u8], offset: usize, value: u32) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    changed
}
