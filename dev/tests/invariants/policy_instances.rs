//! One physical module source retains distinct paid policy origins for each instance.
use super::*;
use crate::{
    expand::config::Config,
    resolve::load::{InputFile, Sources},
    sema::crates::authored_export::Stage,
    support::source::SourceFile,
};
use std::{fmt::Write, path::Path};
const SHARED: &str = "#![unstable(feature=\"shared_policy\",issue=\"none\")]pub mod child{}";
struct Shared;
impl Sources for Shared {
    fn read(&mut self, path: &Path) -> std::result::Result<Option<InputFile>, String> {
        if path.file_name().is_none_or(|name| name != "shared.rs") {
            return Ok(None);
        }
        Ok(Some(InputFile {
            identity: "shared.rs".into(),
            path: "shared.rs".into(),
            source: SourceFile::new(SHARED.as_bytes().to_vec())?,
        }))
    }
}
#[derive(Clone, Copy, Debug)]
struct Cost {
    configured: usize,
    steps: usize,
    warm: usize,
    bytes: usize,
    scratch: usize,
    failed: usize,
}
fn measure(width: usize, depth: bool) -> Result<Cost> {
    let mut source = fixture("provider")?;
    for index in 0..width {
        if depth {
            writeln!(
                &mut source,
                "#[stable(feature=\"depth\",since=\"1.0.0\")]pub mod d{index}{{"
            )?;
        }
        writeln!(&mut source, "#[path=\"shared.rs\"]pub mod s{index};")?;
    }
    if depth {
        source.push_str(&"}".repeat(width));
    }
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/availability-transport-v1/source-instances")
        .join(if depth { "depth" } else { "width" })
        .join(width.to_string());
    std::fs::create_dir_all(&input)?;
    std::fs::write(input.join("source-provider.rs"), &source)?;
    std::fs::write(input.join("shared.rs"), SHARED)?;
    let report = crate::test_source::source_provider_with_sources(
        &source,
        "policy_instances",
        &Config::default(),
        &mut Shared,
    )?;
    let configured = report.package.units.len();
    assert_eq!(
        configured,
        width + 1,
        "expanded modules need distinct configured instances"
    );
    let original = report
        .package
        .units
        .get(1)
        .ok_or("cached physical unit")?
        .source
        .original()
        .as_ptr();
    assert!(
        report.package.units[1..]
            .iter()
            .all(|unit| std::ptr::eq(unit.source.original().as_ptr(), original)),
        "configured instances did not reuse the physical parse-cache image"
    );
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let mut stage = Stage::begin_module_availability(&report, &mut facts, &identity)?;
    let modules = stage.module_contract_rows(&report, &facts)?;
    let before = stage.work();
    let policies = stage.availability_rows(&report, &facts, &modules)?;
    let cold = stage.work();
    assert_eq!(
        policies
            .policies
            .iter()
            .filter(|p| p.feature == "shared_policy")
            .count(),
        width
    );
    let repeated = stage.availability_rows(&report, &facts, &modules)?;
    let warm = stage.work();
    assert_eq!(
        policies, repeated,
        "warm lookup changed the actual source origins"
    );
    let cold_steps = cold.steps - before.steps;
    let warm_steps = warm.steps - cold.steps;
    assert!(warm_steps > 0 && warm_steps <= cold_steps);
    assert!(cold_steps <= 4096 + 128 * width * (width.ilog2() as usize + 2));
    stage.charge(crates::MAX_COMPONENTS - stage.work().steps)?;
    assert!(stage.availability_rows(&report, &facts, &modules).is_err());
    let failed = stage.work().steps - crates::MAX_COMPONENTS;
    assert!(
        failed > 0,
        "failed source collection lost attempted census work"
    );
    // These metrics cover unpaid staging; the independent client controls prove exports.
    Ok(Cost {
        configured,
        steps: cold_steps,
        warm: warm_steps,
        bytes: cold.reserved_bytes - before.reserved_bytes,
        scratch: cold.scratch_bytes,
        failed,
    })
}
#[test]
fn duplicate_physical_source_width_depth_and_failure_work_stay_bounded() -> Result {
    let mut observations =
        String::from("family,width,configured,cold_steps,warm_steps,bytes,scratch,failed_steps\n");
    for (family, widths, depth, factor) in [
        ("width", [8, 32, 128], false, 6),
        ("depth", [8, 16, 32], true, 3),
    ] {
        let mut steps = Vec::new();
        for width in widths {
            let cost = measure(width, depth)?;
            writeln!(
                &mut observations,
                "{family},{width},{},{},{},{},{},{}",
                cost.configured, cost.steps, cost.warm, cost.bytes, cost.scratch, cost.failed
            )?;
            steps.push(cost.steps);
        }
        let first = steps[1]
            .checked_sub(steps[0])
            .ok_or("nonmonotone first interval")?;
        let second = steps[2]
            .checked_sub(steps[1])
            .ok_or("nonmonotone second interval")?;
        assert!(first > 0 && second <= factor * first, "{family}: {steps:?}");
    }
    let folder =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../obj/parallel/availability-transport-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source-instance-work.csv"), observations)?;
    Ok(())
}
