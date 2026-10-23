use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
struct Files;
impl Sources for Files {
    fn resource(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
    fn environment(&mut self, name: &str) -> Result<Option<String>, String> {
        match std::env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(_)) => {
                Err(format!("environment variable {name:?} is not Unicode"))
            }
        }
    }

    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        Ok(Some(InputFile {
            identity: fs::canonicalize(path).map_err(|e| e.to_string())?,
            path: path.to_owned(),
            source: SourceFile::new(bytes)?,
        }))
    }
}
#[derive(Clone, Copy)]
pub enum Mode {
    Names,
    Expanded,
    Types,
    Semantics,
    Relationships,
    Generics,
    Patterns,
    Layouts,
    Bodies,
    Ownership,
    Regions,
    Loans,
    Lowir,
    Constants,
}
pub fn run(
    source: SourceFile,
    path: Option<&str>,
    config: &Config,
    crates: &crate::crates::Options,
    count: bool,
    mode: Mode,
    out: &mut impl Write,
) -> Result<(), String> {
    let identity = path.filter(|p| *p != "-").map_or_else(
        || Ok(PathBuf::from("stdin.rs")),
        |p| fs::canonicalize(p).map_err(|e| e.to_string()),
    )?;
    let root = InputFile {
        identity,
        path: path.filter(|p| *p != "-").unwrap_or("<anon>").into(),
        source,
    };
    if matches!(
        mode,
        Mode::Expanded
            | Mode::Types
            | Mode::Semantics
            | Mode::Relationships
            | Mode::Generics
            | Mode::Patterns
            | Mode::Layouts
            | Mode::Bodies
            | Mode::Ownership
            | Mode::Regions
            | Mode::Loans
            | Mode::Constants
            | Mode::Lowir
    ) {
        let report =
            resolve::expansion::analyze_with_context(root, &mut Files, config, &crates.context()?)
                .and_then(resolve::analyze_expanded)
                .map_err(|e| e.to_string())?;
        if matches!(
            mode,
            Mode::Types
                | Mode::Semantics
                | Mode::Relationships
                | Mode::Generics
                | Mode::Patterns
                | Mode::Layouts
                | Mode::Bodies
                | Mode::Ownership
                | Mode::Regions
                | Mode::Loans
                | Mode::Constants
                | Mode::Lowir
        ) {
            return semantic(report, count, mode, crates, out);
        }
        return if count {
            resolve::render::count_expanded(&report, out)
        } else {
            resolve::render::expanded(&report, out)
        }
        .map_err(|e| e.to_string());
    }
    let mut package =
        resolve::load::load_with_context(root, &mut Files, config, &crates.context()?)
            .map_err(|e| e.to_string())?;
    let facts = resolve::analyze(&mut package).map_err(|e| e.to_string())?;
    if count {
        writeln!(
            out,
            "files {} modules {} names {} scopes {} definitions {} references {}",
            package.units.len(),
            package.modules.len(),
            package.names.len(),
            facts.scopes.len(),
            facts.definitions.len(),
            facts.references.len()
        )
        .map_err(|e| e.to_string())
    } else {
        resolve::render::write(&package, &facts, out).map_err(|e| e.to_string())
    }
}

fn phase_error(
    report: &resolve::expansion::Report,
    node: resolve::unit::NodeKey,
    message: &str,
) -> String {
    let offset = report
        .package
        .unit(node.file)
        .configured
        .ast
        .node(node.node)
        .span
        .start;
    report
        .diagnostic(resolve::package::Error::at(node.file, offset, message))
        .to_string()
}
fn region_proofs(
    report: &resolve::expansion::Report,
    facts: &mut rustgm::sema::model::Declarations,
    program: &rustgm::body::ir::Program,
) -> Result<rustgm::borrow::regions::Analysis, String> {
    let mut bodies = rustgm::body::regions::collect(facts)
        .map_err(str::to_owned)?
        .bodies;
    if facts.uses_source_copy() {
        let mut rows = rustgm::support::hash::HashMap::default();
        for (index, contract) in bodies.iter().enumerate() {
            if rows.insert(contract.key, index).is_some() {
                return Err("source Copy inspection has duplicate contracts".into());
            }
        }
        for body in &program.bodies {
            let index = *rows
                .get(&body.key())
                .ok_or("source Copy inspection body has no contract")?;
            rustgm::body::prepare_source_copy(facts, body, &mut bodies[index])?;
        }
    }
    let contracts = rustgm::body::regions::Contracts {
        bodies,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    rustgm::borrow::regions::check_bodies(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        program,
    )
    .map_err(|e| {
        borrow_error(
            report,
            rustgm::borrow::Error {
                node: e.source,
                message: e.message,
            },
        )
    })
}
fn borrow_error(report: &resolve::expansion::Report, error: rustgm::borrow::Error) -> String {
    match error.node {
        Some(node) => phase_error(report, node, &error.message),
        None => error.message,
    }
}
fn ownership_view(
    report: &resolve::expansion::Report,
    facts: &mut rustgm::sema::model::Declarations,
    program: &rustgm::body::ir::Program,
    mode: Mode,
    count: bool,
    out: &mut impl Write,
) -> Result<(), String> {
    let contracts = region_proofs(report, facts, program)?;
    let check = if matches!(mode, Mode::Loans) {
        rustgm::borrow::inspect
    } else {
        rustgm::borrow::check
    };
    let checked = check(
        &mut rustgm::borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: facts.uses_source_copy(),
            copies: &mut facts.copies,
        },
        &contracts,
        program,
    )
    .map_err(|e| borrow_error(report, e))?;
    rustgm::lower::alignment::check(facts, program)
        .map_err(|e| phase_error(report, e.node, e.message))?;
    if matches!(mode, Mode::Loans) {
        return if count {
            rustgm::borrow::loans::view::count(&checked, out)
        } else {
            rustgm::borrow::loans::view::write(&checked, out)
        }
        .map_err(|e| e.to_string());
    }
    if count {
        rustgm::borrow::ownership::view::count(checked.owned(), out)
    } else {
        rustgm::borrow::ownership::view::write(checked.owned(), out)
    }
    .map_err(|e| e.to_string())
}
fn semantic(
    report: resolve::expansion::Report,
    count: bool,
    mode: Mode,
    crates: &crate::crates::Options,
    out: &mut impl Write,
) -> Result<(), String> {
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    if matches!(mode, Mode::Semantics | Mode::Relationships) {
        rustgm::sema::semantic_view::check_ordinary_profile(&report, &facts)
            .map_err(|e| e.to_string())?;
    }
    if matches!(mode, Mode::Semantics) {
        return if count {
            rustgm::sema::semantic_view::count(&facts, out)
        } else {
            rustgm::sema::semantic_view::write(&report, &facts, out)
        }
        .map_err(|e| e.to_string());
    }
    if matches!(mode, Mode::Relationships) {
        return if count {
            rustgm::sema::semantic_view::relationships::count(&report, &facts, out)
        } else {
            rustgm::sema::semantic_view::relationships::write(&report, &facts, out)
        }
        .map_err(|e| e.to_string());
    }
    if matches!(mode, Mode::Generics) {
        return if count {
            rustgm::sema::semantic_view::generics::count(&report, &facts, out)
        } else {
            rustgm::sema::semantic_view::generics::write(&report, &facts, out)
        }
        .map_err(|e| e.to_string());
    }
    if matches!(mode, Mode::Patterns) {
        return if count {
            rustgm::sema::pattern_view::count(&facts, out)
        } else {
            rustgm::sema::pattern_view::write(&facts, out)
        }
        .map_err(|e| e.to_string());
    }
    if matches!(mode, Mode::Layouts) {
        rustgm::sema::layout_view::analyze(&mut facts).map_err(|e| e.to_string())?;
        return if count {
            rustgm::sema::layout_view::count(&facts, out)
        } else {
            rustgm::sema::layout_view::write(&report, &facts, out)
        }
        .map_err(|e| e.to_string());
    }
    if matches!(mode, Mode::Constants) {
        return emit_constants(&report, &mut facts, prepared, count, out);
    }
    if matches!(mode, Mode::Lowir) {
        return emit_lowir(report, facts, prepared, crates, count, out);
    }
    if matches!(
        mode,
        Mode::Bodies | Mode::Ownership | Mode::Regions | Mode::Loans
    ) {
        let mut program = rustgm::body::build(&report, &facts).map_err(|e| e.to_string())?;
        if matches!(mode, Mode::Ownership | Mode::Regions | Mode::Loans) {
            rustgm::body::promote::analyze(&report, &mut facts, &mut program)
                .map_err(|e| e.to_string())?;
        }
        // Selected patterns have become explicit places and binding actions.
        // Ownership and lowering consume those actions, so release this arena.
        drop(std::mem::take(&mut facts.patterns));
        if matches!(mode, Mode::Regions) {
            let contracts = region_proofs(&report, &mut facts, &program)?;
            return if count {
                rustgm::borrow::regions::view::count(&contracts, out)
            } else {
                rustgm::borrow::regions::view::write(&contracts, out)
            }
            .map_err(|e| e.to_string());
        }
        if matches!(mode, Mode::Ownership | Mode::Loans) {
            return ownership_view(&report, &mut facts, &program, mode, count, out);
        }
        return if count {
            rustgm::body::body_view::count(&program, out)
        } else {
            rustgm::body::body_view::write(&program, &facts.types, out)
        }
        .map_err(|e| e.to_string());
    }
    if count {
        rustgm::sema::type_view::count(&facts, out)
    } else {
        rustgm::sema::type_view::write(&report, &facts, out)
    }
    .map_err(|e| e.to_string())
}

fn emit_constants(
    report: &resolve::expansion::Report,
    facts: &mut rustgm::sema::model::Declarations,
    prepared: rustgm::pipeline::Prepared,
    count: bool,
    out: &mut impl Write,
) -> Result<(), String> {
    let inventory = rustgm::lower::instances::inspect_prepared(
        report,
        facts,
        prepared,
        rustgm::lower::constant_view::inspect,
    )
    .map_err(|e| e.to_string())?
    .map_err(str::to_owned)?;
    if count {
        inventory.count(out)
    } else {
        inventory.write(out)
    }
    .map_err(|e| e.to_string())
}

fn emit_lowir(
    mut report: resolve::expansion::Report,
    mut facts: rustgm::sema::model::Declarations,
    prepared: rustgm::pipeline::Prepared,
    crates: &crate::crates::Options,
    count: bool,
    out: &mut impl Write,
) -> Result<(), String> {
    let (lowir, interface) = if crates.library {
        let (lowir, interface) = rustgm::lower::instances::build_library_prepared(
            &report,
            &mut facts,
            crates.identity()?.ok_or("missing library identity")?,
            prepared,
        )
        .map_err(|error| error.to_string())?;
        (lowir, Some(interface))
    } else {
        if let Some(identity) = crates.identity()? {
            facts
                .crates
                .get_or_insert_default()
                .set_identity(identity)
                .map_err(str::to_owned)?;
        }
        let entry = rustgm::sema::entry::select(&mut report, &facts)
            .map_err(|e| report.diagnostic(e).to_string())?;
        let lowir = rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
            .map_err(|e| e.to_string())?;
        (lowir, None)
    };
    if let Some(path) = &crates.metadata {
        let interface = interface
            .as_ref()
            .ok_or("metadata requires library exports")?;
        fs::write(
            path,
            rustgm::support::crates::encode(interface).map_err(str::to_owned)?,
        )
        .map_err(|error| error.to_string())?;
    }
    drop(facts);
    drop(report);
    if count {
        rustgm::lowir::writer::count(&lowir, out)
    } else {
        rustgm::lowir::writer::write(&lowir, out)
    }
    .map_err(|error| error.to_string())
}
