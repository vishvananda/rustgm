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
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        input: None,
        output: None,
        count: false,
        mode: String::new(),
        config: Config::default(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--emit-tokens"
            | "--emit-literals"
            | "--emit-token-trees"
            | "--emit-ast"
            | "--emit-crate-ast"
            | "--emit-names"
            | "--emit-expanded"
            | "--emit-expanded-crate"
            | "--emit-lowir" => {
                options.mode = arg;
                options.count = false;
            }
            "--count-names" => {
                options.mode = "--emit-names".into();
                options.count = true;
            }
            "--count-expanded-crate" => {
                options.mode = "--emit-expanded-crate".into();
                options.count = true;
            }
            "--count-expanded" => {
                options.mode = "--emit-expanded".into();
                options.count = true;
            }
            "--count-tokens" => {
                options.mode = "--emit-tokens".into();
                options.count = true;
            }
            "--count-literals" => {
                options.mode = "--emit-literals".into();
                options.count = true;
            }
            "--count-token-trees" => {
                options.mode = "--emit-token-trees".into();
                options.count = true;
            }
            "--count-ast" => {
                options.mode = "--emit-ast".into();
                options.count = true;
            }
            "--count-crate-ast" => {
                options.mode = "--emit-crate-ast".into();
                options.count = true;
            }
            "--cfg" => {
                let setting = args.next().ok_or("--cfg requires an option")?;
                options
                    .config
                    .add(&setting)
                    .map_err(|e| format!("--cfg: {}: {}", e.offset, e.message))?;
            }
            "--edition=2024" => {}
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
        "--emit-names" | "--emit-expanded-crate"
    ) {
        modules::run(
            source,
            options.input.as_deref(),
            &options.config,
            options.count,
            if options.mode == "--emit-names" {
                modules::Mode::Names
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
        Ok(options) if options.mode == "--emit-lowir" => {
            let _ = writeln!(io::stderr().lock(), "NOT_IMPLEMENTED: {}", options.mode);
            ExitCode::from(86)
        }
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
