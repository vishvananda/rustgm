use rustgm::{
    expand::{self, config::Config},
    lex::{
        Lexer,
        render::{self, RenderError},
    },
    literal,
    support::source::SourceFile,
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
            "--emit-tokens" | "--emit-literals" | "--emit-token-trees" | "--emit-lowir" => {
                options.mode = arg;
                options.count = false;
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
    let result = if options.count && options.mode == "--emit-token-trees" {
        let report = expand::render::analyze(source.text(), &options.config)
            .map_err(|e| format!("{}: {}", e.offset, e.message))?;
        writeln!(
            output,
            "trees {} attributes {} bytes {}",
            report.forest.nodes().len(),
            report.attributes.entries.len(),
            source.text().len()
        )
        .map_err(|e| e.to_string())
    } else if options.count && options.mode == "--emit-literals" {
        let count = literal::render::count(source.text())
            .map_err(|e| format!("{}: {}", e.offset, e.message))?;
        writeln!(output, "literals {count} bytes {}", source.text().len())
            .map_err(|e| e.to_string())
    } else if options.count {
        let mut lexer = Lexer::new(source.text()).map_err(|e| e.message.to_owned())?;
        let mut count = 0_u64;
        while lexer
            .next_token()
            .map_err(|e| format!("{}: {}", e.offset, e.message))?
            .is_some()
        {
            count += 1;
        }
        writeln!(output, "tokens {count} bytes {}", source.text().len()).map_err(|e| e.to_string())
    } else {
        let rendered = if options.mode == "--emit-token-trees" {
            expand::render::token_trees(source.text(), &options.config, &mut output)
        } else if options.mode == "--emit-literals" {
            literal::render::literals(source.text(), &mut output)
        } else {
            render::tokens(source.text(), &mut output)
        };
        rendered.map_err(|error| match error {
            RenderError::Lex(error) => format!("{}: {}", error.offset, error.message),
            RenderError::Io(error) => error.to_string(),
        })
    };
    output.flush().map_err(|e| e.to_string())?;
    result
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
