//! Source derivation accounting scales by actual image/token slots and receipts.
use crate::test_source;
use std::fmt::Write as _;
type Usage = (usize, usize, usize);
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn provider(source: &str) -> TestResult<crate::resolve::expansion::Report> {
    use crate::{
        resolve,
        support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    };
    let identity = CrateIdentity::new("minicore", "derive_storage")?;
    let context = resolve::bindings::Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    Ok(resolve::expansion::analyze_with_context(
        resolve::load::InputFile {
            identity: "image.rs".into(),
            path: "image.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut super::super::role_declarations::NoFiles,
        &crate::expand::config::Config::default(),
        &context,
    )?)
}
fn measure(source: &str) -> TestResult<(Usage, Usage)> {
    let report = provider(source)?;
    let payer = &report.package.source_core().ok_or("source payer")?.macros;
    let cold = payer.usage();
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|e| e.derives.as_deref())
        .ok_or("actual derives")?
        .entries;
    for entry in entries {
        report.expanded_derive(entry.owner, entry.effect)?;
    }
    let warm = payer.usage();
    assert!(warm.0 > cold.0);
    assert_eq!(warm.1, cold.1);
    assert_eq!(warm.2, cold.2);
    Ok((cold, warm))
}
fn scoped(prefix: &str, declarations: &str) -> String {
    format!("{prefix}mod image_cases {{use crate::derive;use crate::clone::Clone;{declarations}}}")
}
#[test]
fn repeated_small_derives_and_long_names_pay_linear_images_without_setup_repayment() -> TestResult {
    let prefix = test_source::fixture("pa31/tests/spec/source-location/location-copy-clone.rs")?;
    let setup = measure(&scoped(&prefix, ""))?.0;
    let original =
        test_source::fixture("pa31/tests/spec/derive-image-storage/upstream-from-coroutine.rs")?;
    let original_paid = measure(&scoped(&prefix, &original))?.0;
    assert!(original_paid.0 > setup.0 && original_paid.1 > setup.1);
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut source = String::new();
        for index in 0..count {
            write!(&mut source, "#[derive(Clone)]struct Tiny{index}<G>(G);")?;
        }
        let (cold, warm) = measure(&scoped(&prefix, &source))?;
        let delta = (cold.0 - setup.0, cold.1 - setup.1);
        if let Some((work, bytes)) = previous {
            assert!(delta.0 <= work * 5);
            assert!(delta.1 <= bytes * 5);
        }
        previous = Some(delta);
        eprintln!(
            "small derives={count} setup={setup:?} delta={delta:?} cold={cold:?} warm={warm:?}"
        );
    }
    let mut previous = None;
    for width in [64, 256, 1024] {
        let name = "n".repeat(width);
        let source = format!("#[derive(Clone)]struct {name}<G>(G);");
        let (cold, warm) = measure(&scoped(&prefix, &source))?;
        let delta = (cold.0 - setup.0, cold.1 - setup.1);
        if let Some((work, bytes)) = previous {
            assert!(delta.0 <= work * 5);
            assert!(delta.1 <= bytes * 5);
        }
        previous = Some(delta);
        assert!(
            delta.1 < 256_000 + width * 64,
            "identifier width={width}: {delta:?}"
        );
        eprintln!("long name={width} delta={delta:?} cold={cold:?} warm={warm:?}");
    }
    Ok(())
}
