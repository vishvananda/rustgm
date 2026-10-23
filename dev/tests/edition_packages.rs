//! Whole-package entrypoints retain physical and generated lexical editions.
use rustgm::{
    expand::{config::Config, macros::session},
    resolve::{
        expansion,
        load::{InputFile, Sources},
    },
    support::{edition::Edition, source::SourceFile},
};
use std::path::{Path, PathBuf};

#[test]
fn cfg_admission_uses_only_demanded_token_provenance() -> Result<(), rustgm::lex::token::LexError> {
    use rustgm::syntax::{
        edition_context::WithEditions,
        interpolation::EMPTY_INPUTS,
        token_tree::{Forest, error},
    };
    let queries = std::cell::Cell::new(0);
    let missing = WithEditions::new(&EMPTY_INPUTS, |token| {
        queries.set(queries.get() + 1);
        Err(error(token.start, "missing test provenance"))
    });
    let config = Config::default();
    for source in ["ordinary", "r#gen"] {
        let forest = Forest::parse(source)?;
        assert!(!rustgm::expand::config::predicate_with_inputs(
            &forest,
            source,
            forest.roots(),
            &config,
            &missing
        )?);
    }
    assert_eq!(queries.get(), 0);
    let forest = Forest::parse("gen")?;
    assert!(
        rustgm::expand::config::predicate_with_inputs(
            &forest,
            "gen",
            forest.roots(),
            &config,
            &missing
        )
        .is_err()
    );
    assert_eq!(queries.get(), 1);
    assert!(!rustgm::expand::config::predicate_with_inputs(
        &forest,
        "gen",
        forest.roots(),
        &config,
        &Edition::Rust2021
    )?);
    assert!(Config::in_edition(Edition::Rust2021).add("gen").is_ok());
    assert!(Config::default().add("gen").is_err());
    Ok(())
}

#[derive(Default)]
struct Files(Vec<(PathBuf, String)>);
impl Sources for Files {
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        self.0
            .iter()
            .find(|(name, _)| name == path)
            .map(|(_, text)| input(path, text))
            .transpose()
    }
}
fn input(path: &Path, text: &str) -> Result<InputFile, String> {
    Ok(InputFile {
        identity: path.into(),
        path: path.into(),
        source: SourceFile::new(text.as_bytes().to_vec())?,
    })
}
fn analyze(source: &str, edition: Edition, files: &mut Files) -> Result<expansion::Report, String> {
    let report = expansion::analyze(
        input(Path::new("/crate/main.rs"), source)?,
        files,
        &Config::in_edition(edition),
    )
    .map_err(|error| format!("{error:?}"))?;
    rustgm::resolve::analyze_expanded(report).map_err(|error| format!("{error:?}"))
}

fn lowir(source: &str, edition: Edition) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut report = analyze(source, edition, &mut Files::default())?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let program = rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let mut bytes = Vec::new();
    rustgm::lowir::writer::write(&program, &mut bytes)?;
    Ok(bytes)
}

const EXECUTABLES: &[(&str, &str, Edition)] = &[
    (
        "physical",
        "fn gen()->u8{7}fn main(){check(gen()==7);}",
        Edition::Rust2021,
    ),
    (
        "nested",
        "fn gen()->u8{7}macro_rules! id{($e:expr)=>{$e}}macro_rules! run{()=>{id!(gen())}}fn main(){check(run!()==7);}",
        Edition::Rust2021,
    ),
    (
        "generated-2021",
        "macro_rules! make{($name:ident,$body:tt)=>{macro_rules! $name $body}}make!(m,{($e:expr)=>{1};(const {$e:expr})=>{2}});fn main(){check(m!(const {3})==2);}",
        Edition::Rust2021,
    ),
    (
        "generated-2024",
        "macro_rules! make{($name:ident,$body:tt)=>{macro_rules! $name $body}}make!(m,{($e:expr)=>{1};(const {$e:expr})=>{2}});fn main(){check(m!(const {3})==1);}",
        Edition::Rust2024,
    ),
];

#[test]
fn explicit_editions_reach_ordinary_proof_and_lowering() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, edition) in EXECUTABLES {
        let source = format!("{}{body}", crate::native_references::CHECK);
        assert!(!lowir(&source, edition)?.is_empty(), "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc and CPPGM native backend"]
fn native_answers_match_the_selected_lexical_edition() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root
        .join("obj/parallel/pa34-package-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, body, edition) in EXECUTABLES {
        let source = format!("{}{body}", crate::native_references::CHECK);
        let input = folder.join(format!("{name}.rs"));
        let ir = folder.join(format!("{name}.lowir"));
        let candidate = folder.join(format!("{name}.candidate"));
        let reference = folder.join(format!("{name}.rustc"));
        std::fs::write(&input, &source)?;
        let flag = match edition {
            Edition::Rust2021 => "--edition=2021",
            Edition::Rust2024 => "--edition=2024",
        };
        run(
            &folder,
            &format!("{name}-reference"),
            Command::new("rustc")
                .args([
                    "+1.98.0",
                    flag,
                    "--crate-name=probe",
                    "--cap-lints=allow",
                    "-C",
                    "panic=abort",
                    "-C",
                    "overflow-checks=yes",
                ])
                .arg(&input)
                .arg("-o")
                .arg(&reference),
        )?;
        run(
            &folder,
            &format!("{name}-reference-run"),
            &mut Command::new(reference),
        )?;
        std::fs::write(&ir, lowir(&source, edition)?)?;
        run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(ir)
                .arg("-o")
                .arg(&candidate),
        )?;
        run(
            &folder,
            &format!("{name}-candidate-run"),
            &mut Command::new(candidate),
        )?;
    }
    Ok(())
}

fn run(
    folder: &Path,
    name: &str,
    command: &mut std::process::Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{name}.stdout")), &output.stdout)?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    std::fs::write(
        folder.join(format!("{name}.status")),
        format!("{:?}\n", output.status),
    )?;
    assert!(
        output.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn physical_captured_and_transcribed_names_keep_the_package_edition() {
    for source in [
        "fn gen()->u8{7} fn main(){let _=gen();}",
        "macro_rules! id {($e:expr)=>{$e}} fn gen()->u8{7} fn main(){let _=id!(gen());}",
        "macro_rules! make {()=>{fn gen()->u8{7} fn main(){let _=gen();}}} make!();",
        "fn gen()->u8{7} macro_rules! id {($e:expr)=>{$e}} macro_rules! run {()=>{id!(gen())}} fn main(){let _=run!();}",
        "macro_rules! name {($gen:expr)=>{stringify!($gen)}} fn main(){let _=concat!(name!(gen()),\"!\");}",
        "#[cfg(gen)] fn f(){} fn main(){}",
        "#[cfg_attr(any(),gen)] fn f(){} fn main(){}",
        "#[cfg_attr(all(),cfg_attr(any(),gen))] fn f(){} fn main(){}",
        "macro_rules! make {()=>{#[cfg_attr(any(),gen)] fn f(){}}} make!(); fn main(){}",
        "macro_rules! make {($p:path)=>{#[cfg_attr(any(),$p)] fn f(){}}} make!(gen); fn main(){}",
    ] {
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let result = analyze(source, edition, &mut Files::default());
            assert_eq!(
                result.is_ok(),
                edition == Edition::Rust2021,
                "{edition:?}: {source}: {:?}",
                result.err()
            );
        }
    }
}

#[test]
fn raw_names_and_undemanded_nested_configuration_remain_edition_independent() -> Result<(), String>
{
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for source in [
            "fn r#gen()->u8{7} fn main(){let _=r#gen();}",
            "#[cfg(r#gen)] fn f(){} fn main(){}",
            "#[cfg_attr(any(),cfg_attr(any(),gen))] fn f(){} fn main(){}",
        ] {
            analyze(source, edition, &mut Files::default())?;
        }
    }
    Ok(())
}

#[test]
fn generated_definitions_and_standalone_sessions_select_defining_fragment_rules()
-> Result<(), String> {
    let generated = "macro_rules! make {($name:ident,$body:tt)=>{macro_rules! $name $body}} make!(m,{($e:expr)=>{1};(const {$e:expr})=>{2}}); fn main(){let _=m!(const {3});}";
    let standalone =
        "macro_rules! m {($e:expr)=>{1};(const {$e:expr})=>{2}} fn main(){let _=m!(const {3});}";
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        let expected = usize::from(edition == Edition::Rust2021);
        let report = analyze(generated, edition, &mut Files::default())?;
        assert_eq!(report.invocations.len(), 2);
        assert_eq!(report.invocations[1].rule, Some(expected));
        let report = session::analyze(standalone, &Config::in_edition(edition))
            .map_err(|error| format!("{error:?}"))?;
        assert_eq!(report.invocations[0].matched.rule, expected);
    }
    Ok(())
}

#[test]
fn module_instances_and_physical_includes_inherit_compilation_unit_edition() {
    let mut files = Files(vec![
        ("/crate/child.rs".into(), "pub fn gen()->u8{7}".into()),
        ("/crate/items.rs".into(), "fn gen()->u8{7}".into()),
        ("/crate/expr.rs".into(), "{let gen=7;gen}".into()),
    ]);
    for source in [
        "mod child; fn main(){let _=child::gen();}",
        "#[path=\"child.rs\"] mod first; #[path=\"child.rs\"] mod second; fn main(){let _=first::r#gen()+second::r#gen();}",
        "include!(\"items.rs\"); fn main(){let _=gen();}",
        "fn main(){let _=include!(\"expr.rs\");}",
    ] {
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let result = analyze(source, edition, &mut files);
            assert_eq!(
                result.is_ok(),
                edition == Edition::Rust2021,
                "{edition:?}: {source}: {:?}",
                result.err()
            );
        }
    }
}
