use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{declarations, model::Declarations},
    support::source::SourceFile,
    types::{Kind, Primitive, TypeId},
};
use std::{fmt::Write as _, path::Path};
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn names(source: &str) -> Result<Report, String> {
    resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|e| e.to_string())
}
fn analyze(source: &str) -> Result<(Report, Declarations), String> {
    let report = names(source)?;
    let facts = declarations::analyze(&report).map_err(|e| e.to_string())?;
    Ok((report, facts))
}
fn definition(report: &Report, facts: &Declarations, name: &str) -> Result<TypeId, String> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == name)
        .and_then(|(index, _)| facts.definitions[index])
        .ok_or_else(|| format!("missing type for {name}"))
}
#[test]
fn aliases_forward_signatures_and_function_item_identity_share_canonical_facts()
-> Result<(), String> {
    let (report, facts) = analyze(
        "type First=Second; type Second=u64; fn first(x:First)->Second{x} fn second(x:u64)->u64{x}",
    )?;
    assert_eq!(
        definition(&report, &facts, "First")?,
        TypeId::primitive(Primitive::U64)
    );
    assert_eq!(facts.functions.len(), 2);
    assert_eq!(facts.functions[0].signature, facts.functions[1].signature);
    assert_ne!(
        definition(&report, &facts, "first")?,
        definition(&report, &facts, "second")?
    );
    assert!(facts.functions.iter().all(|f| f.body.is_some()));
    Ok(())
}
#[test]
fn alias_cycles_and_signature_placeholders_are_rejected() -> Result<(), String> {
    for source in [
        "type A=A;",
        "type A=B; type B=(A,);",
        "fn f(x:_){let _=x;}",
        "fn f()->_{1}",
        "type A=_;",
    ] {
        assert!(analyze(source).is_err(), "{source}");
    }
    let mut source = String::new();
    for i in 0..10_000 {
        write!(source, "type T{i}=T{};", i + 1).map_err(|e| e.to_string())?;
    }
    source.push_str("type T10000=i32;");
    let (report, facts) = analyze(&source)?;
    assert_eq!(
        definition(&report, &facts, "T0")?,
        TypeId::primitive(Primitive::I32)
    );
    Ok(())
}
#[test]
fn structural_annotations_and_configured_parameters_consume_typed_syntax() -> Result<(), String> {
    let (report, facts) = analyze(
        "type Pair=(u8,bool); type Buffer=[Pair;4usize]; type Pointer=*const Buffer; type Callback=fn(u8)->bool; fn f(#[cfg(any())] x:Missing, value:u8)->bool{true}",
    )?;
    assert!(matches!(
        facts.types.kind(definition(&report, &facts, "Pointer")?),
        Some(Kind::Pointer { mutable: false, .. })
    ));
    assert!(matches!(
        facts.types.kind(definition(&report, &facts, "Callback")?),
        Some(Kind::Function { .. })
    ));
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(facts.functions[0].signature)
    else {
        return Err("expected signature".into());
    };
    assert_eq!(
        facts.types.list(parameters),
        Some([TypeId::primitive(Primitive::U8)].as_slice())
    );
    Ok(())
}
#[test]
fn aliases_created_by_expansion_keep_the_resolved_definition_identity() -> Result<(), String> {
    let (report, facts) = analyze(
        "mod original{pub type Number=i32;} use original::Number as Renamed; macro_rules! m{()=>{fn f(x:Renamed)->crate::original::Number{x}}} m!();",
    )?;
    assert_eq!(
        definition(&report, &facts, "Number")?,
        TypeId::primitive(Primitive::I32)
    );
    assert_eq!(facts.functions.len(), 1);
    let Some(Kind::Function { result, .. }) = facts.types.kind(facts.functions[0].signature) else {
        return Err("expected signature".into());
    };
    assert_eq!(result, TypeId::primitive(Primitive::I32));
    Ok(())
}

#[test]
fn reference_signatures_canonicalize_bound_names_and_elided_regions() -> Result<(), String> {
    let (report, facts) = analyze(
        "type A=fn(&i32)->&i32; type B=for<'a> fn(&'a i32)->&'a i32; type C=for<'different> fn(&'different i32)->&'different i32; fn f<'a>(x:&'a i32)->&'a i32{x} fn g(x:&i32)->&i32{x}",
    )?;
    let a = definition(&report, &facts, "A")?;
    assert_eq!(a, definition(&report, &facts, "B")?);
    assert_eq!(a, definition(&report, &facts, "C")?);
    assert_eq!(facts.functions[0].signature, a);
    assert_eq!(facts.functions[1].signature, a);
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        ..
    }) = facts.types.kind(a)
    else {
        return Err("missing function signature".into());
    };
    assert_eq!(bound_regions, 1);
    assert_eq!(facts.types.list(parameters), Some([result].as_slice()));
    let Some(Kind::Reference { region, .. }) = facts.types.kind(result) else {
        return Err("missing reference".into());
    };
    assert_eq!(region.bound_parts(), Some((0, 0)));
    Ok(())
}

#[test]
fn nested_binders_do_not_capture_outer_signature_lifetimes() -> Result<(), String> {
    let (report, facts) = analyze(
        "type A=for<'a> fn(&'a i32, for<'b> fn(&'a i32,&'b i32)); type B=for<'x> fn(&'x i32, for<'y> fn(&'x i32,&'y i32)); type C=for<'x> fn(&'x i32, for<'y> fn(&'y i32,&'y i32));",
    )?;
    let a = definition(&report, &facts, "A")?;
    assert_eq!(a, definition(&report, &facts, "B")?);
    assert_ne!(a, definition(&report, &facts, "C")?);
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(a) else {
        return Err("missing outer signature".into());
    };
    let parameters = facts.types.list(parameters).ok_or("missing parameters")?;
    let Some(Kind::Function {
        parameters,
        bound_regions,
        ..
    }) = facts.types.kind(parameters[1])
    else {
        return Err("missing nested signature".into());
    };
    assert_eq!(bound_regions, 1);
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("missing nested parameters")?;
    let mut parts = Vec::new();
    for ty in parameters {
        let Some(Kind::Reference { region, .. }) = facts.types.kind(*ty) else {
            return Err("missing nested reference".into());
        };
        parts.push(region.bound_parts());
    }
    assert_eq!(parts, [Some((1, 0)), Some((0, 0))]);
    Ok(())
}

#[test]
fn signature_elision_acceptance_matches_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let version = Command::new("rustc").arg("--version").output()?;
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("rustc 1.98.0 "));
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa9-work/signature-rustc")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory)?;
    let cases = [
        ("type A=fn(&i32)->&i32;", true),
        ("type A=fn(&'_ i32)->&'_ i32;", true),
        ("type A=fn()->&i32;", false),
        ("type A=fn(&i32,&i32)->&i32;", false),
        ("type A=fn(&'static i32,u8)->&i32;", true),
        ("type A=for<'a> fn(&'a i32,&'a i32)->&i32;", false),
        ("type A=for<'a,'b> fn(&'a i32,&'b i32)->&i32;", false),
        ("type A=for<'a> fn()->&'a i32;", false),
        ("type A=for<'a> fn();", true),
        ("type A=fn(&&i32)->&i32;", false),
        ("type A=fn(fn(&i32)->&i32)->&i32;", false),
        ("type A=fn(fn(&'static i32))->&i32;", false),
        ("type A=for<'a> fn(fn(&'a i32))->&i32;", false),
        ("type A=fn(*const &i32)->&i32;", true),
        ("type A=fn((&i32,u8))->&i32;", true),
        ("type A=for<'a> fn((&'a i32,&'a i32))->&i32;", true),
        ("type R=&'static i32; type A=fn(R)->&i32;", false),
        ("type A=&i32;", false),
        ("type A=&'static i32;", true),
        ("fn f<'a>(x:&'a i32)->&i32{x}", true),
        ("fn f(x:&i32,y:&i32)->&i32{x}", false),
        ("fn f()->&'static i32{loop{}}", true),
    ];
    for (index, (source, accepted)) in cases.into_iter().enumerate() {
        let mut child = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "--cap-lints=allow",
                "--crate-name=signature_check",
                "-",
            ])
            .arg("-o")
            .arg(directory.join(format!("{index}.rmeta")))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("missing compiler stdin")?
            .write_all(source.as_bytes())?;
        let result = child.wait_with_output()?;
        std::fs::write(directory.join(format!("{index}.rs")), source)?;
        std::fs::write(directory.join(format!("{index}.stderr")), &result.stderr)?;
        assert_eq!(
            result.status.success(),
            accepted,
            "rustc: {source}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual = analyze(source);
        assert_eq!(
            actual.is_ok(),
            accepted,
            "RustGM: {source}: {:?}",
            actual.err()
        );
    }
    Ok(())
}
