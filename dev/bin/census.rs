//! The analysis-only census file is a driver diagnostic, never a build artifact.
use rustgm::{
    resolve::{expansion::Report, package::Error},
    sema::census::{Census, Location, Phase},
};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::PathBuf,
};

pub fn select(slot: &mut Option<String>, path: Option<String>) -> Result<(), String> {
    if slot.is_some() {
        return Err("duplicate --census-output".into());
    }
    *slot = Some(path.ok_or("--census-output requires a path")?);
    Ok(())
}

/// Run the census, write every record and always fail: a census never passes a
/// build and produces no view output, `LowIR`, artifact or metadata.
pub fn run(report: Result<Report, Error>, path: &str, semantics: bool) -> Result<(), String> {
    let (census, files) = match report {
        Err(error) => {
            let mut census = Census::default();
            census.stop(Phase::Frontend, &error);
            (census, Vec::new())
        }
        Ok(report) => {
            let (mut census, analysis) = rustgm::pipeline::census_source(&report);
            if semantics
                && let Some(analysis) = &analysis
                && let Err(error) =
                    rustgm::sema::semantic_view::check_ordinary_profile(&report, &analysis.facts)
            {
                census.stop(Phase::Completion, &error);
            }
            drop(analysis);
            (census, report.package.file_paths.clone())
        }
    };
    write(&census, &files, path).map_err(|error| format!("census output {path}: {error}"))?;
    summarize(&census).map_err(|error| error.to_string())?;
    Err(format!(
        "analysis-only census written to {path}; a census is never a passing build"
    ))
}

fn write(census: &Census, files: &[PathBuf], path: &str) -> io::Result<()> {
    let mut output = io::BufWriter::new(fs::File::create(path)?);
    writeln!(
        output,
        "rustgm-census\t1\tanalysis-only\tnot-a-qualification"
    )?;
    let mut referenced = BTreeSet::new();
    for (index, record) in census.records().iter().enumerate() {
        writeln!(
            output,
            "record\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            index + 1,
            record.phase.name(),
            record.category.name(),
            field(record.item.as_deref().unwrap_or("-")),
            location(&record.location),
            record
                .blocked_by
                .map_or_else(|| "-".to_owned(), |cause| (cause + 1).to_string()),
            field(&record.message)
        )?;
        if let Location::Source { file, .. } = record.location {
            referenced.insert(file);
        }
        for frame in &record.expansions {
            writeln!(
                output,
                "expansion\t{}\t{}\tfile {}:{}",
                index + 1,
                field(&frame.macro_name),
                frame.file,
                frame.offset
            )?;
            referenced.insert(frame.file);
        }
    }
    for file in referenced {
        if let Some(path) = usize::try_from(file).ok().and_then(|file| files.get(file)) {
            writeln!(
                output,
                "file\t{file}\t{}",
                field(&path.display().to_string())
            )?;
        }
    }
    for phase in Phase::ALL {
        let counts = census.counts(phase);
        writeln!(
            output,
            "summary\t{}\tchecked\t{}\tfailed\t{}\tblocked\t{}",
            phase.name(),
            counts.checked,
            counts.failed,
            counts.blocked
        )?;
    }
    match census.terminal() {
        Some((index, record)) => writeln!(
            output,
            "terminal\t{}\t{}\t{}",
            index + 1,
            record.phase.name(),
            record.category.name()
        )?,
        None => writeln!(output, "terminal\tnone")?,
    }
    writeln!(output, "end\t{}", census.records().len())?;
    output.flush()
}

fn summarize(census: &Census) -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    for phase in Phase::ALL {
        let counts = census.counts(phase);
        writeln!(
            stderr,
            "census {}: checked {} failed {} blocked {}",
            phase.name(),
            counts.checked,
            counts.failed,
            counts.blocked
        )?;
    }
    match census.terminal() {
        Some((index, record)) => writeln!(
            stderr,
            "census ended by record {} ({} {})",
            index + 1,
            record.category.name(),
            record.phase.name()
        ),
        None => writeln!(stderr, "census reached the end of analysis"),
    }
}

fn location(location: &Location) -> String {
    match location {
        Location::Source { file, offset } => format!("file {file}:{offset}"),
        Location::Imported { label, offset } => format!("imported {}:{offset}", field(label)),
        Location::Unlocated => "unlocated".into(),
    }
}

/// Fields stay on one tab-separated line; escapes are reversible.
fn field(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(character),
        }
    }
    escaped
}
