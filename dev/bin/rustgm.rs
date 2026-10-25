mod crates;
mod modules;
use rustgm::{
    expand::{self, config::Config},
    lex::{
        Lexer,
        render::{self, RenderError},
    },
    literal,
    support::source::SourceFile,
    syntax::expression,
};
use std::{
    fs,
    io::{self, Read, Write},
    process::ExitCode,
};

struct Options {
    input: Option<String>,
    output: Option<String>,
    count: bool,
    mode: String,
    config: Config,
    crates: crates::Options,
}

fn selected_view(arg: &str) -> Option<(&str, bool)> {
    let (name, count) = arg
        .strip_prefix("--emit-")
        .map(|name| (name, false))
        .or_else(|| arg.strip_prefix("--count-").map(|name| (name, true)))?;
    matches!(
        name,
        "tokens"
            | "literals"
            | "token-trees"
            | "ast"
            | "crate-ast"
            | "names"
            | "expanded"
            | "expanded-crate"
            | "types"
            | "semantics"
            | "trait-semantics"
            | "generic-semantics"
            | "patterns"
            | "layouts"
            | "body-ir"
            | "ownership"
            | "regions"
            | "loans"
            | "constants"
            | "lowir"
    )
    .then_some((name, count))
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        input: None,
        output: None,
        count: false,
        mode: String::new(),
        config: Config::default(),
        crates: crates::Options::default(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some((name, count)) = selected_view(&arg) {
            options.mode = format!("--emit-{name}");
            options.count = count;
            continue;
        }
        match arg.as_str() {
            "--source-core" | "--source-client" => {
                options
                    .crates
                    .source_mode(arg == "--source-client", args.next())?;
            }
            "--crate-name" => {
                options.crates.name = Some(args.next().ok_or("--crate-name requires a name")?);
            }
            "--crate-disambiguator" => {
                options.crates.disambiguator =
                    Some(args.next().ok_or("--crate-disambiguator requires an ID")?);
            }
            "--metadata-output" => {
                options.crates.metadata =
                    Some(args.next().ok_or("--metadata-output requires a path")?);
            }
            "--crate-type=lib" => options.crates.library = true,
            "--crate-type=bin" => options.crates.library = false,
            "--extern" | "--dependency-metadata" => {
                if options.crates.dependency_count() >= 1024 {
                    return Err("external crate arguments exceed profile budget".into());
                }
                let value = args.next().ok_or(if arg == "--extern" {
                    "--extern requires ALIAS=METADATA"
                } else {
                    "--dependency-metadata requires a path"
                })?;
                if arg == "--extern" {
                    options.crates.externs.push(value);
                } else {
                    options.crates.dependency_metadata.push(value);
                }
            }
            "--cfg" => {
                let setting = args.next().ok_or("--cfg requires an option")?;
                options
                    .config
                    .add(&setting)
                    .map_err(|e| format!("--cfg: {}: {}", e.offset, e.message))?;
            }
            "--edition=2024" => {}
            "--ub-checks=yes" | "--ub-checks=no" => {
                options.config.set_runtime_checks(
                    rustgm::support::runtime_checks::RuntimeCheckConfig::with_ub_checks(
                        arg == "--ub-checks=yes",
                    ),
                );
            }
            "-o" => options.output = Some(args.next().ok_or("-o requires a path")?),
            "-" => options.input = Some(arg),
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
            _ if options.input.is_none() => options.input = Some(arg),
            _ => return Err("expected one source file".into()),
        }
    }
    if options.mode.is_empty() {
        return Err("usage: rustgm --emit-tokens|--emit-literals [-o OUTPUT] [SOURCE|-]".into());
    }
    options
        .crates
        .validate(&options.mode, options.count, options.output.as_deref())?;
    Ok(options)
}

fn compile(options: &Options) -> Result<(), String> {
    let mut bytes = Vec::new();
    if let Some(path) = options.input.as_deref().filter(|path| *path != "-") {
        bytes = fs::read(path).map_err(|e| e.to_string())?;
    } else {
        io::stdin()
            .lock()
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
    }
    let source = SourceFile::new(bytes)?;
    let stdout = io::stdout();
    let mut output: Box<dyn Write> = match &options.output {
        Some(path) => Box::new(io::BufWriter::new(
            fs::File::create(path).map_err(|e| e.to_string())?,
        )),
        None => Box::new(io::BufWriter::new(stdout.lock())),
    };
    let result = if matches!(
        options.mode.as_str(),
        "--emit-names"
            | "--emit-expanded-crate"
            | "--emit-types"
            | "--emit-semantics"
            | "--emit-trait-semantics"
            | "--emit-generic-semantics"
            | "--emit-patterns"
            | "--emit-layouts"
            | "--emit-body-ir"
            | "--emit-ownership"
            | "--emit-regions"
            | "--emit-loans"
            | "--emit-constants"
            | "--emit-lowir"
    ) {
        modules::run(
            source,
            options.input.as_deref(),
            &options.config,
            &options.crates,
            options.count,
            if options.mode == "--emit-names" {
                modules::Mode::Names
            } else if options.mode == "--emit-patterns" {
                modules::Mode::Patterns
            } else if options.mode == "--emit-semantics" {
                modules::Mode::Semantics
            } else if options.mode == "--emit-trait-semantics" {
                modules::Mode::Relationships
            } else if options.mode == "--emit-generic-semantics" {
                modules::Mode::Generics
            } else if options.mode == "--emit-types" {
                modules::Mode::Types
            } else if options.mode == "--emit-layouts" {
                modules::Mode::Layouts
            } else if options.mode == "--emit-body-ir" {
                modules::Mode::Bodies
            } else if options.mode == "--emit-ownership" {
                modules::Mode::Ownership
            } else if options.mode == "--emit-regions" {
                modules::Mode::Regions
            } else if options.mode == "--emit-loans" {
                modules::Mode::Loans
            } else if options.mode == "--emit-constants" {
                modules::Mode::Constants
            } else if options.mode == "--emit-lowir" {
                modules::Mode::Lowir
            } else {
                modules::Mode::Expanded
            },
            &mut output,
        )
    } else if options.count {
        count(options, source.text(), &mut output)
    } else {
        render_view(options, source.text(), &mut output)
    };
    output.flush().map_err(|e| e.to_string())?;
    result
}

fn count(options: &Options, source: &str, output: &mut impl Write) -> Result<(), String> {
    if options.mode == "--emit-expanded" {
        let report = expand::macros::session::analyze(source, &options.config)
            .map_err(|e| format!("{}: {}", e.offset, e.message))?;
        let captures: usize = report
            .invocations
            .iter()
            .map(|i| i.matched.captures.values.len())
            .sum();
        let pieces: usize = report
            .invocations
            .iter()
            .map(|i| i.transcription.pieces.len())
            .sum();
        writeln!(
            output,
            "definitions {} invocations {} captures {captures} pieces {pieces} bytes {}",
            report.definitions.len(),
            report.invocations.len(),
            source.len()
        )
        .map_err(|e| e.to_string())
    } else if options.mode == "--emit-crate-ast" {
        let report = expand::crate_syntax::analyze(source, &options.config)
            .map_err(|e| format!("{}: {}", e.offset, e.message))?;
        writeln!(
            output,
            "nodes {} items {} active {} attributes {} lookahead {} bytes {}",
            report.ast.nodes.len(),
            report.ast.items.len(),
            report.active.iter().filter(|active| **active).count(),
            report.ast.attributes.len(),
            report.ast.lookahead_high_water,
            source.len()
        )
        .map_err(|e| e.to_string())
    } else if options.mode == "--emit-ast" {
        let ast = expression::parse(source).map_err(|e| format!("{}: {}", e.offset, e.message))?;
        writeln!(
            output,
            "nodes {} edges {} captures {} lookahead {} bytes {}",
            ast.nodes.len(),
            ast.edges.len(),
            ast.captures.len(),
            ast.lookahead_high_water,
            source.len()
        )
        .map_err(|e| e.to_string())
    } else if options.mode == "--emit-token-trees" {
        let report = expand::render::analyze(source, &options.config)
            .map_err(|e| format!("{}: {}", e.offset, e.message))?;
        writeln!(
            output,
            "trees {} attributes {} bytes {}",
            report.forest.nodes().len(),
            report.attributes.entries.len(),
            source.len()
        )
        .map_err(|e| e.to_string())
    } else if options.mode == "--emit-literals" {
        let count =
            literal::render::count(source).map_err(|e| format!("{}: {}", e.offset, e.message))?;
        writeln!(output, "literals {count} bytes {}", source.len()).map_err(|e| e.to_string())
    } else {
        let mut lexer = Lexer::new(source).map_err(|e| e.message.to_owned())?;
        let mut count = 0_u64;
        while lexer
            .next_token()
            .map_err(|e| format!("{}: {}", e.offset, e.message))?
            .is_some()
        {
            count += 1;
        }
        writeln!(output, "tokens {count} bytes {}", source.len()).map_err(|e| e.to_string())
    }
}

fn render_view(options: &Options, source: &str, output: &mut impl Write) -> Result<(), String> {
    let rendered = if options.mode == "--emit-expanded" {
        expand::macros::render::expanded(source, &options.config, output)
    } else if options.mode == "--emit-crate-ast" {
        expand::crate_syntax::render(source, &options.config, output)
    } else if options.mode == "--emit-ast" {
        expression::render::expression(source, output)
    } else if options.mode == "--emit-token-trees" {
        expand::render::token_trees(source, &options.config, output)
    } else if options.mode == "--emit-literals" {
        literal::render::literals(source, output)
    } else {
        render::tokens(source, output)
    };
    rendered.map_err(|error| match error {
        RenderError::Lex(error) => format!("{}: {}", error.offset, error.message),
        RenderError::Io(error) => error.to_string(),
    })
}

fn main() -> ExitCode {
    match options() {
        Ok(options) => match compile(&options) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = writeln!(io::stderr().lock(), "ERROR: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "USAGE: {error}");
            ExitCode::from(2)
        }
    }
}
