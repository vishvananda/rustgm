//! The `rustgm` command line with one more output: `--emit-object` compiles
//! the lowered program in process through the pinned CPPGM `LowIR` library
//! and writes the relocatable object to `-o`.
//!
//! A program holding a form the bridge does not state takes the text path
//! instead, visibly: the forms are reported on standard error, the program's
//! `LowIR` text is written beside the output and compiled by the object
//! bridge (`scripts/backend.py --emit-object`), and that object is written.
//! Every other command and output is the `rustgm` binary's own.
//!
//! `--library-view PATH` also writes the library's text view of the module;
//! `--object-pic` compiles position-independent code. `--emission-timing`
//! reports on standard error the wall time of the emission step alone (the
//! writer, or the bridge and the library's compile), for measurement.
#[path = "../../bin/bringup.rs"]
mod bringup;
#[path = "../../bin/budget_options.rs"]
mod budget_options;
#[path = "../../bin/census.rs"]
mod census;
#[path = "../../bin/crates.rs"]
mod crates;
#[path = "../../bin/driver.rs"]
mod driver;
#[path = "../../bin/export_census.rs"]
mod export_census;
#[path = "../../bin/link_manifest.rs"]
mod link_manifest;
#[path = "../../bin/modules.rs"]
mod modules;

/// Where a lowered program leaves this binary: an object, or the text.
mod emission {
    use crate::crates::ExecutableMetadata;
    use rustgm::lowir::{program::Program, writer};
    use rustgm_backend_bridge::{Failure, Request};
    use std::{
        fs,
        io::{self, BufWriter, Write},
        path::{Path, PathBuf},
        process::Command,
        sync::OnceLock,
        time::Instant,
    };

    /// What the command line asked of the object output.
    #[derive(Default)]
    pub struct Settings {
        pub object: bool,
        pub pic: bool,
        pub view: Option<PathBuf>,
        pub output: Option<PathBuf>,
        pub timing: bool,
    }
    static SETTINGS: OnceLock<Settings> = OnceLock::new();

    /// Record the settings once, before the driver runs.
    pub fn configure(settings: Settings) {
        let _ = SETTINGS.set(settings);
    }

    /// Write the object of the program, or its text or record counts.
    pub fn emit(
        program: &Program,
        count: bool,
        out: &mut impl Write,
        metadata: Option<ExecutableMetadata<'_>>,
    ) -> Result<(), String> {
        let start = Instant::now();
        let settings = SETTINGS.get();
        let emitted = match settings {
            Some(settings) if settings.object && !count => object(program, settings, out),
            _ => if count {
                writer::count(program, out)
            } else {
                writer::write(program, out)
            }
            .map_err(|error| error.to_string()),
        };
        if settings.is_some_and(|settings| settings.timing) {
            let _ = writeln!(
                io::stderr().lock(),
                "rustgm-inprocess: emission {} ns",
                start.elapsed().as_nanos()
            );
        }
        emitted?;
        if let Some(metadata) = metadata {
            metadata.publish(out)?;
        }
        Ok(())
    }

    fn object(program: &Program, settings: &Settings, out: &mut impl Write) -> Result<(), String> {
        let request = Request {
            level: 0,
            pic: settings.pic,
            view: settings.view.as_deref(),
        };
        match rustgm_backend_bridge::compile(program, &request) {
            Ok(object) => {
                let bytes = object.bytes().map_err(|error| error.to_string())?;
                out.write_all(bytes).map_err(|error| error.to_string())
            }
            Err(Failure::Unsupported(forms)) => {
                let _ = writeln!(
                    io::stderr().lock(),
                    "rustgm-inprocess: taking the text path; forms the bridge does not state: {forms}"
                );
                fallback(program, settings, out)
            }
            Err(failure) => Err(failure.to_string()),
        }
    }

    /// The text path: the program's text, compiled by the object bridge.
    fn fallback(
        program: &Program,
        settings: &Settings,
        out: &mut impl Write,
    ) -> Result<(), String> {
        if settings.pic {
            return Err("the text path compiles fixed code only".into());
        }
        let output = settings
            .output
            .as_deref()
            .ok_or("--emit-object requires -o")?;
        let beside = |suffix: &str| {
            let mut name = output.as_os_str().to_owned();
            name.push(suffix);
            PathBuf::from(name)
        };
        let (text, object) = (beside(".fallback.lowir"), beside(".fallback.o"));
        let mut file = BufWriter::new(fs::File::create(&text).map_err(|error| error.to_string())?);
        writer::write(program, &mut file).map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())?;
        drop(file);
        let script = Path::new(env!("RUSTGM_BRIDGE_ROOT")).join("scripts/backend.py");
        let status = Command::new("python3")
            .arg(script)
            .arg("--emit-object")
            .arg(&text)
            .arg("-o")
            .arg(&object)
            .status()
            .map_err(|error| format!("cannot run the object bridge: {error}"))?;
        if !status.success() {
            return Err(format!(
                "the object bridge failed on the text path ({status}); its input is {}",
                text.display()
            ));
        }
        let bytes = fs::read(&object).map_err(|error| error.to_string())?;
        out.write_all(&bytes).map_err(|error| error.to_string())?;
        fs::remove_file(&object).map_err(|error| error.to_string())?;
        fs::remove_file(&text).map_err(|error| error.to_string())
    }
}

use std::{path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let mut settings = emission::Settings::default();
    let mut arguments = Vec::new();
    let mut given = std::env::args().skip(1);
    while let Some(argument) = given.next() {
        match argument.as_str() {
            "--emit-object" => {
                settings.object = true;
                arguments.push("--emit-lowir".to_owned());
            }
            "--object-pic" => settings.pic = true,
            "--emission-timing" => settings.timing = true,
            "--library-view" => {
                let Some(path) = given.next() else {
                    eprintln!("USAGE: --library-view requires a path");
                    return ExitCode::from(2);
                };
                settings.view = Some(PathBuf::from(path));
            }
            "-o" => {
                let path = given.next();
                settings.output = path.as_ref().map(PathBuf::from);
                arguments.push(argument);
                arguments.extend(path);
            }
            _ => arguments.push(argument),
        }
    }
    emission::configure(settings);
    driver::run(arguments.into_iter())
}
