use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{bodies, declarations, model::Declarations},
    support::source::SourceFile,
};
use std::{fmt::Write, path::Path, process::Command};
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn analyze(source: &str) -> Result<(Report, Declarations), String> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "main.rs".into(),
            path: "main.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|e| e.to_string())?;
    let facts = declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    Ok((report, facts))
}
const ACCEPT: &[(&str, &[u128])] = &[
    (
        "enum A{X(),Y{}}enum E{V=((A::X))() as isize,W=A::Y{} as isize}",
        &[0, 1, 0, 1],
    ),
    ("static N:isize=3;enum E{A=N}", &[3]),
    (
        "#[derive(Clone,Copy)]enum A{X=3}static N:A=A::X;enum E{A=N as isize}",
        &[3, 3],
    ),
    (
        "enum A{X=3}macro_rules! value{()=>{A::X as isize}}enum E{A=value!()}",
        &[3, 3],
    ),
    ("enum E{A=1+2*3,B,C=(1<<6)|2}", &[7, 8, 66]),
    (
        "#[repr(i8)]enum E{A=-128,B=-7/2,C=-7%2,D=(!8i8)>>1}",
        &[128, 253, 255, 251],
    ),
    (
        "#[repr(u8)]enum E{A=(255u16+1) as u8,B=((1u16<<9)+3) as u8}",
        &[0, 3],
    ),
    ("enum E{A=E::B as isize+2,B=1,C}", &[3, 1, 2]),
    ("enum E{A=1,B=E::A as isize+2,C}", &[1, 3, 4]),
    (
        "const N:u8=M+1;const M:u8=4;#[repr(u8)]enum E{A=N+3,B}",
        &[8, 9],
    ),
    (
        "enum A{X=3}const N:A=A::X;enum E{A=N as isize+1,B}",
        &[3, 4, 5],
    ),
    (
        "enum E{A=if true{3}else{1/0},B=if false && 1/0>0{8}else{5}}",
        &[3, 5],
    ),
    (
        "enum E{A=if true || 1/0>0{8}else{4},B=('🦀' as u32 & 255) as isize}",
        &[8, 128],
    ),
    (
        "#[repr(i16)]enum E{A=(0.0f64/0.0) as i16,B=(1.0f64/0.0) as i16,C=(-1.0f32/0.0) as i16}",
        &[0, 32767, 32768],
    ),
    (
        "#[repr(u8)]enum E{A=(-1.0f64) as u8,B=300.0f32 as u8,C=(256u16 as f32 /2.0) as u8}",
        &[0, 255, 128],
    ),
    (
        "enum E{A=if (16777217u32 as f32)==16777216.0{1}else{0},B=if (255u8 as char)=='ÿ'{2}else{0}}",
        &[1, 2],
    ),
    (
        "#[repr(i128)]enum E{A=-170141183460469231731687303715884105728,B=AUX} const AUX:i128=170141183460469231731687303715884105727;",
        &[1 << 127, (1 << 127) - 1],
    ),
    (
        "#[repr(u128)]enum E{A=!0u128,B=(1u128<<127)+7}",
        &[u128::MAX, (1 << 127) + 7],
    ),
    (
        "#[repr(i16)]enum A{X=-2}#[repr(u128)]enum E{A=A::X as u128,B}",
        &[65534, u128::MAX - 1, u128::MAX],
    ),
    (
        "mod m{pub const N:isize=7;}use m::N;enum E{A={N*2},B}",
        &[14, 15],
    ),
    ("macro_rules! d{()=>{2+3}}enum E{A=d!(),B}", &[5, 6]),
    (
        "enum E{A=if (1.0f32/0.0)>(3.0f32){1}else{2},B=(2.75f64 as f32*2.0) as isize}",
        &[1, 5],
    ),
];
const REJECT: &[&str] = &[
    "enum E{A=E::A as isize}",
    "enum E{A=E::B as isize,B}",
    "const N:isize=E::A as isize;enum E{A=N}",
    "const N:isize=M;const M:isize=N;enum E{A=N}",
    "enum E{A=1+2,B=3}",
    "#[repr(u8)]enum E{A=255,B}",
    "#[repr(i8)]enum E{A=126+1,B}",
    "#[repr(u8)]enum E{A=255+1}",
    "#[repr(u8)]enum E{A=0-1}",
    "#[repr(i8)]enum E{A=64*2}",
    "#[repr(i8)]enum E{A=-128-1}",
    "#[repr(i8)]enum E{A=-(-128i8)}",
    "#[repr(i8)]enum E{A=-128/-1}",
    "#[repr(i8)]enum E{A=-128%-1}",
    "#[repr(u128)]enum E{A=(!0u128)+1}",
    "#[repr(i128)]enum E{A=(-170141183460469231731687303715884105728)/-1}",
    "enum E{A=1/0}",
    "enum E{A=1%0}",
    "enum E{A=if false{3}else{1/0}}",
    "enum E{A=(false || (1/0>0)) as isize}",
    "#[repr(u8)]enum E{A=1<<8}",
    "enum E{A=1<<-1i8}",
    "enum E{A=1<<128u128}",
    "#[repr(u8)]enum E{A=1u16+2}",
    "enum E{A=1.0+2.0}",
    "enum E{A=true}",
    "enum E{A=return 1}",
    "fn f()->isize{3}enum E{A=f()}",
    "fn f()->isize{3}enum E{A=if true{1}else{f()}}",
    "static mut N:isize=3;enum E{A=N}",
    "enum A{X=3}static N:A=A::X;enum E{A=N as isize}",
];
#[test]
fn constant_discriminant_values_and_rejections_agree_with_rustc()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa12-work/discriminant-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory)?;
    for (index, (source, expected)) in ACCEPT
        .iter()
        .map(|(s, v)| (*s, Some(*v)))
        .chain(REJECT.iter().map(|s| (*s, None)))
        .enumerate()
    {
        let source = format!("#![allow(dead_code)]\n{source}\nfn main(){{}}\n");
        let path = directory.join(format!("case{index}.rs"));
        std::fs::write(&path, &source)?;
        let rustc = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "discriminants",
                "--emit=metadata",
                "-o",
            ])
            .arg(path.with_extension("rmeta"))
            .arg(&path)
            .output()?;
        std::fs::write(path.with_extension("stderr"), &rustc.stderr)?;
        assert_eq!(
            rustc.status.success(),
            expected.is_some(),
            "rustc case {index}: {source}\n{}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let actual = analyze(&source);
        assert_eq!(
            actual.is_ok(),
            expected.is_some(),
            "RustGM case {index}: {source}\n{}",
            actual.as_ref().err().map_or("", String::as_str)
        );
        if let Some(expected) = expected {
            let (report, facts) = actual?;
            let values: Vec<_> = facts
                .aggregates
                .iter()
                .flat_map(|d| facts.aggregates.variants(d).iter().map(|v| v.discriminant))
                .collect();
            assert_eq!(values, expected, "case {index}: {source}");
            // Cached layout discriminants must observe completed query values.
            let layouts: Vec<_> = facts
                .aggregates
                .iter()
                .flat_map(|d| {
                    facts.definitions[d.item.0 as usize]
                        .and_then(|t| facts.layouts.get(t))
                        .into_iter()
                        .flat_map(|l| facts.layouts.variants(l).iter().map(|v| v.discriminant))
                })
                .collect();
            assert_eq!(layouts, expected);
            verify_values(&source, &report, &facts, expected, &path)?;
        }
    }
    Ok(())
}
fn verify_values(
    source: &str,
    report: &Report,
    facts: &Declarations,
    expected: &[u128],
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut main = String::from("fn main(){");
    for definition in &facts.aggregates {
        let name = report
            .package
            .names
            .get(report.resolution.definitions[definition.item.0 as usize].name);
        let tag = definition
            .repr
            .tag
            .unwrap_or(rustgm::types::Primitive::Isize);
        let (_, width) = tag.integer().ok_or("invalid discriminant representation")?;
        let mask = u128::MAX >> (128 - width);
        for member in &facts.members.variants {
            if member.owner != definition.item {
                continue;
            }
            let source = member.origin.diagnostic().ok_or("missing variant source")?;
            let unit = report.package.unit(source.file);
            let rustgm::syntax::ast::Kind::Variant(index) =
                unit.configured.ast.node(source.node).kind
            else {
                return Err("missing source variant name".into());
            };
            let variant = unit
                .spelling(unit.configured.ast.variants[index as usize].name)
                .map_err(|e| e.message)?;
            let suffix = match member.shape {
                rustgm::syntax::item::Shape::Unit => "",
                rustgm::syntax::item::Shape::Tuple => "()",
                rustgm::syntax::item::Shape::Record => "{}",
            };
            writeln!(
                main,
                "println!(\"{{}}\",({name}::{variant}{suffix} as u128)&{mask}u128);"
            )?;
        }
    }
    main.push('}');
    let probe = path.with_extension("values.rs");
    std::fs::write(&probe, source.replace("fn main(){}", &main))?;
    let binary = path.with_extension("values");
    let compiled = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name",
            "discriminant_values",
            "-o",
        ])
        .arg(&binary)
        .arg(&probe)
        .output()?;
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let output = Command::new(&binary).output()?;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    std::fs::write(path.with_extension("values.stdout"), &output.stdout)?;
    let actual = std::str::from_utf8(&output.stdout)?
        .lines()
        .map(str::parse::<u128>)
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(actual, expected, "{}", path.display());
    Ok(())
}
#[test]
fn deep_constant_and_variant_dependencies_use_bounded_worklists()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from("const N0:isize=1;\n");
    for i in 1..10_000 {
        writeln!(source, "const N{i}:isize=N{}+1;", i - 1)?;
    }
    source.push_str("enum E{A=N9999,B=AUX}const AUX:isize=E::A as isize+1;");
    let (_, facts) = analyze(&source)?;
    let definition = facts.aggregates.iter().next().ok_or("missing enum")?;
    assert_eq!(
        facts.aggregates.variants(definition)[1].discriminant,
        10_001
    );
    let mut source = String::from("enum E{");
    for i in 0..5_000 {
        write!(source, "V{i},")?;
    }
    source.push('}');
    let (_, facts) = analyze(&source)?;
    let definition = facts.aggregates.iter().next().ok_or("missing enum")?;
    assert_eq!(
        facts.aggregates.variants(definition)[4999].discriminant,
        4999
    );
    Ok(())
}
