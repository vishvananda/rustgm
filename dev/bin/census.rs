//! The analysis-only census file is a driver diagnostic, never a build artifact.
use rustgm::{
    resolve::{expansion::Report, package::Error},
    sema::census::{Census, ItemKind, Location, Phase, Progress, Record},
};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub fn select(slot: &mut Option<String>, path: Option<String>) -> Result<(), String> {
    select_path(slot, path, "--census-output")
}
pub fn select_progress(slot: &mut Option<String>, path: Option<String>) -> Result<(), String> {
    select_path(slot, path, "--census-progress")
}
fn select_path(slot: &mut Option<String>, path: Option<String>, flag: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("duplicate {flag}"));
    }
    *slot = Some(path.ok_or_else(|| format!("{flag} requires a path"))?);
    Ok(())
}

/// Run the census, write every record and always fail: a census never passes a
/// build and produces no view output, `LowIR`, artifact or metadata.
pub fn run(
    report: Result<Report, Error>,
    options: &crate::crates::Options,
    semantics: bool,
) -> Result<(), String> {
    let path = options
        .census
        .as_deref()
        .ok_or("census output path is absent")?;
    let progress = options.census_progress.as_deref();
    let mut initial = observing(progress, path, report.as_ref().ok())?;
    let (census, files) = match report {
        Err(error) => {
            initial.stop(Phase::Frontend, &error);
            (initial, Vec::new())
        }
        Ok(report) => {
            let (mut census, analysis) = rustgm::pipeline::census_source_observed(&report, initial);
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
    if let Some(error) = census.progress_error() {
        return Err(error.to_owned());
    }
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
        write_record(&mut output, index + 1, record)?;
        if let Location::Source { file, .. } = record.location {
            referenced.insert(file);
        }
        for frame in &record.expansions {
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

fn destination(path: &str) -> io::Result<PathBuf> {
    let path = Path::new(path);
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "output path lacks a file name")
    })?;
    Ok(fs::canonicalize(parent)?.join(name))
}
fn observing(path: Option<&str>, target: &str, report: Option<&Report>) -> Result<Census, String> {
    let Some(path) = path else {
        return Ok(Census::default());
    };
    let prepare = || -> io::Result<_> {
        if destination(path)? == destination(target)? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "census progress and final output need distinct paths",
            ));
        }
        let mut output = io::BufWriter::new(
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)?,
        );
        writeln!(
            output,
            "rustgm-census-progress\t1\tpartial-analysis-only\tnot-a-qualification"
        )?;
        if let Some(report) = report {
            for (index, path) in report.package.file_paths.iter().enumerate() {
                writeln!(
                    output,
                    "file\t{index}\t{}",
                    field(&path.display().to_string())
                )?;
            }
        }
        output.flush()?;
        Ok(output)
    };
    let mut output = prepare().map_err(|error| format!("census progress {path}: {error}"))?;
    let label = path.to_owned();
    Ok(Census::with_observer(Box::new(move |event| {
        write_progress(&mut output, event)
            .and_then(|()| output.flush())
            .map_err(|error| format!("census progress {label}: {error}"))
    })))
}
fn write_progress(output: &mut impl Write, event: Progress<'_>) -> io::Result<()> {
    match event {
        Progress::Record { index, record } => write_record(output, index + 1, record),
        Progress::Item {
            kind,
            definition,
            key,
            offset,
        } => {
            let kind = match kind {
                ItemKind::Implementation => "implementation",
                ItemKind::Function => "function",
                ItemKind::Initializer => "initializer",
            };
            writeln!(
                output,
                "item\t{kind}\t{}\tfile {}:{offset}\tnode {}",
                definition.0,
                key.file.0,
                key.node.get()
            )
        }
    }
}
fn write_record(output: &mut impl Write, index: usize, record: &Record) -> io::Result<()> {
    writeln!(
        output,
        "record\t{index}\t{}\t{}\t{}\t{}\t{}\t{}",
        record.phase.name(),
        record.category.name(),
        field(record.item.as_deref().unwrap_or("-")),
        location(&record.location),
        record
            .blocked_by
            .map_or_else(|| "-".to_owned(), |cause| (cause + 1).to_string()),
        field(&record.message)
    )?;
    for frame in &record.expansions {
        writeln!(
            output,
            "expansion\t{index}\t{}\tfile {}:{}",
            field(&frame.macro_name),
            frame.file,
            frame.offset
        )?;
    }
    Ok(())
}
