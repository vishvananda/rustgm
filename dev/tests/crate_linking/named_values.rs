//! Imported scalar values remap directly to typed facts without invented source bodies.
use rustgm::{
    sema::{
        model::Scalar as Value,
        values::{Reader, View},
    },
    support::crates::{
        self, Constant, CrateIdentity, Dependencies, Dependency, ExternalDef, ExternalKind,
        Interface, Scalar, ScalarValue,
    },
};
use std::sync::Arc;

#[test]
fn repeated_scalar_imports_keep_one_value_per_identity_and_bounded_queries()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for width in [8usize, 32, 128] {
        let owner = CrateIdentity::new("library", "scaling")?;
        let mut constants = Vec::new();
        let mut text = String::new();
        for index in 0..width {
            constants.push(Constant::new(
                ExternalDef::new(
                    owner.clone(),
                    vec![format!("V{index}")],
                    ExternalKind::Constant,
                )?,
                ScalarValue::new(Scalar::Usize, 3)?,
            )?);
            write!(text, "const C{index}:usize=values::V{index};")?;
        }
        text.push_str("fn main(){}");
        let artifact = Interface::with_constants(owner.clone(), vec![], vec![], constants)?;
        let dependencies = Arc::new(Dependencies::new(
            vec![artifact],
            vec![Dependency::new("values".into(), owner)?],
        )?);
        let report = super::imports::resolve(&text, dependencies)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        assert_eq!(
            report
                .resolution
                .crates
                .as_ref()
                .ok_or("imports")?
                .constants()
                .count(),
            width
        );
        assert_eq!(analysis.facts.initializers.len(), width);
        assert_eq!(analysis.facts.functions.len(), 1);
        assert_eq!(analysis.facts.constant_work.initializer_visits, width);
        assert_eq!(analysis.facts.constant_work.initializer_validations, width);
        assert_eq!(analysis.facts.constant_work.expressions, 0);
        assert!(analysis.facts.constant_work.initializer_lookups.get() <= 3 * width + 4);
        for initializer in &analysis.facts.initializers {
            assert_eq!(
                Reader::new(&analysis.facts)
                    .definition(initializer.definition)
                    .ok_or("completed value")?
                    .as_scalar()?,
                Value::Integer(3)
            );
        }
    }
    Ok(())
}

#[test]
fn named_value_cli_exports_imports_and_dependency_order_are_reproducible()
-> Result<(), Box<dyn std::error::Error>> {
    let build = super::artifacts::build_values("values-portable")?;
    let bytes = std::fs::read(&build.first)?;
    let first = crates::decode(&bytes)?;
    assert_eq!(first.constants().len(), 20);
    assert_eq!(first.functions().len(), 1);
    assert_eq!(bytes[12], 1);
    let second = crates::decode(&std::fs::read(&build.second)?)?;
    assert_eq!(second.constants().len(), 3);
    assert_eq!(second.dependencies(), &[first.identity().clone()]);
    let original = super::artifacts::checked(
        "values-client",
        &build.folder,
        &mut build.client("client", false),
    )?;
    let reverse = super::artifacts::checked(
        "values-client-reverse",
        &build.folder,
        &mut build.client("client", true),
    )?;
    assert_eq!(original.stdout, reverse.stdout);
    let metadata = build.folder.join("repeat.rgm");
    let repeated = super::artifacts::checked(
        "values-repeat",
        &build.folder,
        std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-lowir",
                "--crate-type=lib",
                "--crate-name",
                "values",
                "--crate-disambiguator",
                "first",
                "--metadata-output",
            ])
            .arg(&metadata)
            .arg(build.fixture("first")),
    )?;
    assert_eq!(bytes, std::fs::read(metadata)?);
    assert_eq!(
        repeated.stdout,
        std::fs::read(build.folder.join("first.lowir"))?
    );
    Ok(())
}

#[test]
#[ignore = "requires configured pinned CPPGM object backend and Rust toolchain"]
fn scalar_values_execute_through_imported_patterns_bounds_and_calls()
-> Result<(), Box<dyn std::error::Error>> {
    let build = super::artifacts::build_values("values-native")?;
    build.qualify()?;
    build.execute("client")
}

#[test]
fn scalar_values_do_not_authorize_calls_or_expose_private_bodies()
-> Result<(), Box<dyn std::error::Error>> {
    let build = super::artifacts::build_values("values-permissions")?;
    build.qualify()?;
    for (name, text) in [
        (
            "nonconst-function",
            "const C:u8=one::runtime(0);fn main(){}",
        ),
        ("call-value", "fn main(){let _=one::U8();}"),
        ("wrong-type", "const C:bool=one::U8;fn main(){}"),
        ("private-body", "fn main(){let _=one::seed();}"),
    ] {
        let path = build.folder.join(format!("{name}.rs"));
        std::fs::write(&path, text)?;
        let independent = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--cap-lints=allow",
                "--crate-name=permission",
                "--emit=obj",
                "--extern",
            ])
            .arg(format!(
                "one={}",
                build.folder.join("libfirst.rlib").display()
            ))
            .arg("-o")
            .arg(build.folder.join(format!("{name}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            build.folder.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(independent.status.code(), Some(1), "independent {name}");
        let candidate = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-lowir", "--extern"])
            .arg(format!("one={}", build.first.display()))
            .arg(&path)
            .output()?;
        std::fs::write(
            build.folder.join(format!("{name}.candidate.stderr")),
            &candidate.stderr,
        )?;
        assert_eq!(candidate.status.code(), Some(1), "candidate {name}");
        if name == "nonconst-function" {
            assert!(String::from_utf8_lossy(&candidate.stderr).contains("non-const"));
        }
    }
    Ok(())
}

#[test]
fn imported_values_have_canonical_identities_and_no_initializer_or_global()
-> Result<(), Box<dyn std::error::Error>> {
    let owner = CrateIdentity::new("library", "values")?;
    let mut constants = Vec::new();
    for (name, ty, bits) in [
        ("UNIT", Scalar::Unit, 0),
        ("NEGATIVE", Scalar::I8, 255),
        ("NEGATIVE_ZERO", Scalar::F32, 0x8000_0000),
        ("NAN", Scalar::F64, 0x7ff8_0000_0000_1234),
    ] {
        constants.push(Constant::new(
            ExternalDef::new(
                owner.clone(),
                vec!["nested".into(), name.into()],
                ExternalKind::Constant,
            )?,
            ScalarValue::new(ty, bits)?,
        )?);
    }
    let artifact = Interface::with_constants(owner.clone(), vec![], vec![], constants)?;
    let artifact = crates::decode(&crates::encode(&artifact)?)?;
    let dependencies = Arc::new(Dependencies::new(
        vec![artifact],
        vec![Dependency::new("values".into(), owner)?],
    )?);
    let report = super::imports::resolve("fn main(){}", dependencies)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    assert!(analysis.facts.initializers.is_empty());
    assert_eq!(analysis.facts.functions.len(), 1);
    let linkage = analysis.facts.crates.as_ref().ok_or("crate facts")?;
    let imported = report
        .resolution
        .crates
        .as_ref()
        .ok_or("imported identities")?;
    assert_eq!(imported.constants().count(), 4);
    for (definition, constant) in imported.constants() {
        let declared = &report.resolution.definitions[definition.0 as usize];
        assert!(declared.node.is_none());
        assert_eq!(declared.kind, rustgm::resolve::model::DefKind::Const);
        assert_eq!(imported.definition(constant.definition()), Some(definition));
        assert!(linkage.imported_constant(definition));
        assert!(linkage.rust_symbol(definition).is_none());
        let value = Reader::new(&analysis.facts)
            .definition(definition)
            .ok_or("missing imported value")?;
        match constant.value().ty() {
            Scalar::Unit => assert!(matches!(
                Reader::new(&analysis.facts).view(value)?,
                View::Product {
                    length: 0,
                    variant: None
                }
            )),
            Scalar::I8 => assert_eq!(value.as_scalar()?, Value::Integer(255)),
            Scalar::F32 => assert_eq!(
                value.as_scalar()?,
                Value::Float(rustgm::literal::value::FloatBits::F32(0x8000_0000))
            ),
            Scalar::F64 => assert_eq!(
                value.as_scalar()?,
                Value::Float(rustgm::literal::value::FloatBits::F64(
                    0x7ff8_0000_0000_1234
                ))
            ),
            _ => return Err("unexpected fixture scalar".into()),
        }
    }
    Ok(())
}
