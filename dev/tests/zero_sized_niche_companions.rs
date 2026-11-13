//! A two-variant enum whose one variant holds a zero-sized field niche-fills
//! the other's carrier, as rustc lays out `Result<(), NonNull<T>>`. Core's
//! `io/error/repr_bitpacked.rs` asserts `size_of::<Result<(), Repr>>() == 8`.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/zero-sized-niche-companions";
const TAG: &str = "zero_sized_niche_companions";

/// The CLI's typing status and its stderr for one packet file.
fn cli_types(path: &std::path::Path) -> Result<(bool, String)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
        .args(["--source-core", &format!("minicore@{TAG}")])
        .args(["--crate-name", "minicore", "--crate-disambiguator", TAG])
        .arg(path)
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok((output.status.success(), stderr))
}

/// Lowers `source` with the named functions exported.
fn lowered(source: &str, exports: &[&str]) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, TAG)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let mut roots = Vec::new();
    for name in exports {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == *name)
            .ok_or_else(|| format!("missing exported {name}"))?;
        roots.push(rustgm::resolve::model::DefId(u32::try_from(index)?));
    }
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?)
}

/// Every accepted case types and lowers, and each rejection reports its
/// message at the last occurrence of its site.
#[test]
fn zero_sized_companions_niche_fill_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    let cases = crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?;
    for line in cases.lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        let [path, outcome, rest @ ..] = columns.as_slice() else {
            return Err(format!("malformed case row {line}").into());
        };
        let (types, stderr) = cli_types(&root.join(path))?;
        let source = crate::resolve::fixture(path)?;
        if *outcome == "0" {
            if let Err(error) = lowered(&source, &[]) {
                failures.push(format!("{path}: lowering {error}"));
            }
            if !types {
                failures.push(format!("{path}: {stderr}"));
            }
            continue;
        }
        let site = rest.first().ok_or("rejection site")?;
        let offset = source.rfind(site).ok_or("rejection site text")?;
        if types || !stderr.starts_with(&format!("ERROR: file 0: {offset}: {outcome}\n")) {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
    }
    assert_eq!(cases.lines().count(), 11);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Runtime construction and matching use the niche in either orientation: a
/// niche-filled result is one pointer, while `Result<usize, &u8>` keeps its
/// tagged aggregate ABI.
#[test]
fn zero_sized_companion_values_lower_through_the_carrier() -> Result {
    let mut source = crate::resolve::fixture(&format!("{PACKET}/result-unit-match.rs"))?;
    source.push_str(concat!(
        "#[unsafe(no_mangle)]\n",
        "pub fn make(b:bool,r:&'static u8)->Result<(),&'static u8>{if b{Ok(())}else{Err(r)}}\n",
        "#[unsafe(no_mangle)]\n",
        "pub fn swapped(r:&'static u8,b:bool)->Result<&'static u8,()>{if b{Err(())}else{Ok(r)}}\n",
        "#[unsafe(no_mangle)]\n",
        "pub fn test(r:Result<(),&'static u8>)->bool{match r{Ok(())=>true,Err(_)=>false}}\n",
        "#[unsafe(no_mangle)]\n",
        "pub fn wide(n:usize)->Result<usize,&'static u8>{Ok(n)}\n",
    ));
    let program = lowered(&source, &["make", "swapped", "test", "wide"])?;
    // Each exported function's scalar result and parameter types.
    let signature = |name: &str| -> Result<(rustgm::lowir::vocabulary::Type, Vec<_>)> {
        let symbol = program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name && symbol.body.is_some())
            .ok_or_else(|| format!("missing lowered {name}"))?;
        let rustgm::lowir::program::Signature::Scalar(signature) = program
            .signatures
            .get(symbol.signature.0 as usize)
            .ok_or("missing signature")?
        else {
            return Err(format!("{name} used another ABI profile").into());
        };
        Ok((
            signature.result,
            program.parameters(symbol.signature)?.to_vec(),
        ))
    };
    let pointer = rustgm::lowir::vocabulary::Type::Ptr;
    assert_eq!(signature("make")?.0, pointer);
    assert_eq!(signature("swapped")?.0, pointer);
    assert_eq!(signature("test")?.1, [pointer]);
    let (result, parameters) = signature("wide")?;
    assert_eq!(result, rustgm::lowir::vocabulary::Type::Void);
    assert_eq!(parameters.first(), Some(&pointer));
    Ok(())
}
