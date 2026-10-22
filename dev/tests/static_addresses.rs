use crate::semantic;
use rustgm::{
    resolve::{expansion::Report, model::DefId},
    sema::values::{Evaluator, Origin, Reader, View},
};
use std::{path::Path, process::Command};

fn definition(report: &Report, name: &str) -> Result<DefId, &'static str> {
    report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == name)
        .and_then(|i| u32::try_from(i).ok())
        .map(DefId)
        .ok_or("missing definition")
}

#[test]
fn address_queries_do_not_read_initializers_but_storage_completion_does()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) =
        semantic::analyze("static A:u8=*P;static P:&u8=&A;const R:&u8=&A;const AGAIN:&u8=&A;")?;
    let a = definition(&report, "A")?;
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let r = evaluator.definition(definition(&report, "R")?)?;
    let again = evaluator.definition(definition(&report, "AGAIN")?)?;
    let reader = Reader::new(evaluator.facts());
    let (View::Reference(r), View::Reference(again)) = (reader.view(r)?, reader.view(again)?)
    else {
        return Err("missing symbolic references".into());
    };
    assert_eq!(r, again);
    assert_eq!(reader.location(r)?.origin, Origin::Static(a));
    assert!(reader.location(r)?.value.is_none());
    assert!(reader.definition(a).is_none());
    let Err(error) = evaluator.complete_static_storage() else {
        return Err("accepted an actual initializer value cycle".into());
    };
    assert!(
        error
            .to_string()
            .contains("cycle in enum discriminant or constant dependencies")
    );
    Ok(())
}

#[test]
fn deep_static_address_rings_complete_iteratively_and_reuse_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let count = 4096;
    let mut source = String::from("struct N{next:&'static N}\n");
    for i in 0..count {
        use std::fmt::Write;
        writeln!(source, "static N{i}:N=N{{next:&N{}}};", (i + 1) % count)?;
    }
    source.push_str("const ROOT:&N=&N0;");
    let (report, mut facts) = semantic::analyze(&source)?;
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let root = evaluator.definition(definition(&report, "ROOT")?)?;
    assert!(
        Reader::new(evaluator.facts())
            .definition(definition(&report, "N0")?)
            .is_none()
    );
    evaluator.complete_static_storage()?;
    let reader = Reader::new(evaluator.facts());
    let View::Reference(first) = reader.view(root)? else {
        return Err("root is not a reference".into());
    };
    let mut id = first;
    for i in 0..count {
        let Origin::Static(target) = reader.location(id)?.origin else {
            return Err("static address lost its identity".into());
        };
        assert_eq!(target, definition(&report, &format!("N{i}"))?);
        let value = reader.definition(target).ok_or("static backing")?;
        let View::Reference(next) = reader.view(reader.field(value, 0)?)? else {
            return Err("static link is not a reference".into());
        };
        id = next;
    }
    assert_eq!(id, first);
    Ok(())
}

#[test]
fn symbolic_addresses_preserve_value_read_bounds_and_alignment_rejections()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/static-address-errors-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for (name, source) in [
        (
            "direct-cycle",
            "static A:u8=B;static B:u8=A;fn main(){let _=A;}",
        ),
        (
            "reference-cycle",
            "static A:u8=*P;static P:&u8=&A;fn main(){let _=A;}",
        ),
        (
            "backing-cycle",
            "static A:u8=*P;static P:&u8=&A;const R:&u8=&A;fn main(){let _=R;}",
        ),
        (
            "bad-index",
            "static A:[u8;2]=[1,2];const R:&u8=&A[2];fn main(){let _=R;}",
        ),
        (
            "noncopy-read",
            "struct S(u8);static A:S=S(7);const R:&S=&A;const C:S=*R;fn main(){let _=C;}",
        ),
        (
            "unaligned-self",
            "#[repr(C,packed)]struct S{v:u64,r:&'static u64}static A:S=S{v:7,r:&A.v};fn main(){let _=&A;}",
        ),
    ] {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source)?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--cap-lints=allow",
                "--emit=metadata",
            ])
            .arg(&path)
            .arg("-o")
            .arg(path.with_extension("rmeta"))
            .output()?;
        std::fs::write(path.with_extension("rustc.stderr"), &independent.stderr)?;
        assert_eq!(independent.status.code(), Some(1), "{name}");
        for mode in ["--emit-lowir", "--count-lowir"] {
            let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(
                path.with_extension(format!("{mode}.stderr")),
                &candidate.stderr,
            )?;
            assert_eq!(candidate.status.code(), Some(1), "{name} {mode}");
            assert!(
                candidate.stdout.is_empty() && !candidate.stderr.is_empty(),
                "{name}"
            );
        }
    }
    Ok(())
}
