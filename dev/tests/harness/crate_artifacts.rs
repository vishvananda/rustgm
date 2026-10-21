//! Cross-crate fixtures use the public CLI and independent pinned Rust compilation.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
pub struct Build {
    pub root: PathBuf,
    pub folder: PathBuf,
    pub first: PathBuf,
    pub second: PathBuf,
    family: &'static str,
    crate_name: &'static str,
}
pub fn checked(
    label: &str,
    folder: &Path,
    command: &mut Command,
) -> Result<Output, Box<dyn std::error::Error>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{label}.stderr")), &output.stderr)?;
    if !output.status.success() {
        return Err(format!(
            "{label}: {:?}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output)
}
pub fn build(tag: &str) -> Result<Build, Box<dyn std::error::Error>> {
    build_sources(tag, "scalar", "shared")
}
pub fn build_values(tag: &str) -> Result<Build, Box<dyn std::error::Error>> {
    build_sources(tag, "values", "values")
}
fn build_sources(
    tag: &str,
    family: &'static str,
    crate_name: &'static str,
) -> Result<Build, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?
        .to_owned();
    let folder = root.join(format!(
        "obj/parallel/pa29-crate-linking/{tag}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let first = folder.join("first.rgm");
    let second = folder.join("second.rgm");
    let build = Build {
        root,
        folder,
        first,
        second,
        family,
        crate_name,
    };
    for name in ["first", "second"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command
            .args([
                "--emit-lowir",
                "--crate-type=lib",
                "--crate-name",
                crate_name,
                "--crate-disambiguator",
                name,
                "--metadata-output",
            ])
            .arg(build.folder.join(format!("{name}.rgm")))
            .arg("-o")
            .arg(build.folder.join(format!("{name}.lowir")))
            .arg(build.fixture(name));
        if name == "second" {
            command
                .arg("--extern")
                .arg(format!("one={}", build.first.display()));
        }
        checked(name, &build.folder, &mut command)?;
    }
    Ok(build)
}
impl Build {
    pub fn fixture(&self, name: &str) -> PathBuf {
        self.root
            .join(format!("pa29/tests/spec/{}/{name}.rs", self.family))
    }
    pub fn client(&self, name: &str, reverse: bool) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command.arg("--emit-lowir").arg(self.fixture(name));
        let mut externs = [
            format!("one={}", self.first.display()),
            format!("two={}", self.second.display()),
        ];
        if reverse {
            externs.reverse();
        }
        for argument in externs {
            command.arg("--extern").arg(argument);
        }
        command
    }
    pub fn qualify(&self) -> Result<(), Box<dyn std::error::Error>> {
        for name in ["first", "second"] {
            let mut command = rustc();
            command
                .args(["--crate-type=rlib", "--crate-name"])
                .arg(self.crate_name)
                .arg("-C")
                .arg(format!("metadata={name}"))
                .arg(self.fixture(name))
                .arg("-o")
                .arg(self.folder.join(format!("lib{name}.rlib")));
            if name == "second" {
                command.arg("--extern").arg(format!(
                    "one={}",
                    self.folder.join("libfirst.rlib").display()
                ));
            }
            checked(&format!("{name}.rustc"), &self.folder, &mut command)?;
        }
        Ok(())
    }
    pub fn execute(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let independent = self.folder.join(format!("{name}.rustc"));
        let mut command = rustc();
        command
            .arg("--crate-name=client")
            .arg(self.fixture(name))
            .arg("-L")
            .arg(format!("dependency={}", self.folder.display()))
            .arg("--extern")
            .arg(format!(
                "one={}",
                self.folder.join("libfirst.rlib").display()
            ))
            .arg("--extern")
            .arg(format!(
                "two={}",
                self.folder.join("libsecond.rlib").display()
            ))
            .arg("-o")
            .arg(&independent);
        checked(&format!("{name}.rustc-build"), &self.folder, &mut command)?;
        let lowir = self.folder.join(format!("{name}.lowir"));
        let output = checked(
            &format!("{name}.frontend"),
            &self.folder,
            &mut self.client(name, false),
        )?;
        std::fs::write(&lowir, output.stdout)?;
        let native = self.folder.join(format!("{name}.native"));
        checked(
            &format!("{name}.backend"),
            &self.folder,
            Command::new("python3")
                .arg(self.root.join("scripts/backend.py"))
                .arg(self.folder.join("first.lowir"))
                .arg(self.folder.join("second.lowir"))
                .arg(lowir)
                .arg("-o")
                .arg(&native),
        )?;
        for (label, binary) in [("rustc", independent), ("native", native)] {
            let output = checked(
                &format!("{name}.{label}-run"),
                &self.folder,
                &mut Command::new(binary),
            )?;
            assert!(output.stdout.is_empty() && output.stderr.is_empty());
        }
        Ok(())
    }
}
fn rustc() -> Command {
    let mut command = Command::new("rustc");
    command.args([
        "+1.98.0",
        "--edition=2024",
        "--cap-lints=allow",
        "-C",
        "overflow-checks=yes",
        "-C",
        "panic=abort",
    ]);
    command
}
