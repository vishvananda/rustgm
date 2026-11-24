//! Strict declaration export diagnostics live at the filesystem boundary.
use rustgm::{
    lower::instances::{DeclCensus, inspect_declaration_exports},
    pipeline::Prepared,
    resolve::{expansion::Report, package::DiagnosticFile},
    sema::model::Declarations,
};
use std::{
    fs::OpenOptions,
    io::{self, Write},
};

pub(super) fn run(
    report: &Report,
    facts: &mut Declarations,
    prepared: Prepared,
    options: &crate::crates::Options,
    path: &str,
) -> Result<(), String> {
    let identity = options
        .identity()?
        .ok_or("missing export census library identity")?;
    let census = inspect_declaration_exports(report, facts, &identity, prepared)
        .map_err(|error| error.to_string())?;
    write(report, path, &census).map_err(|error| format!("export census {path}: {error}"))?;
    Err(format!(
        "analysis-only declaration export census written to {path}; no artifacts produced"
    ))
}

fn write(report: &Report, path: &str, census: &DeclCensus) -> io::Result<()> {
    use crate::census::field;
    let mut output =
        io::BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    writeln!(
        output,
        "rustgm-export-census\t1\tdeclaration-admission\tanalysis-only"
    )?;
    for failure in &census.refused {
        let error = &failure.diagnostic;
        let location = match &error.file {
            DiagnosticFile::Source(file) => {
                report.package.file_paths.get(file.0 as usize).map(|path| {
                    let path = path.display();
                    report
                        .package
                        .unit(*file)
                        .source
                        .position(error.span.start)
                        .map_or_else(
                            |_| format!("{path}:byte-{}", error.span.start),
                            |position| format!("{path}:{}:{}", position.line, position.column),
                        )
                })
            }
            DiagnosticFile::Imported(label) => Some(format!("{label}:byte-{}", error.span.start)),
            DiagnosticFile::Unlocated => None,
        };
        writeln!(
            output,
            "failed\t{}\t{}\t{}\t{}\t{}",
            failure.declaration,
            failure.kind,
            field(&failure.path),
            field(location.as_deref().unwrap_or("unlocated")),
            field(&error.message)
        )?;
    }
    writeln!(
        output,
        "summary\tchecked\t{}\tfailed\t{}",
        census.checked,
        census.refused.len()
    )?;
    output.flush()
}
