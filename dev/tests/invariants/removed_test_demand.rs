//! Retirement evidence is borrowed from actual expansion and actual saved cfg rows.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        expansion::{Engine, attribute_effects::Effects},
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
    syntax::{ast::Kind, item::ItemKind, token_tree::Span},
};
use std::{fmt::Write, path::Path};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn input(source: &str) -> TestResult<InputFile> {
    Ok(InputFile {
        identity: "test-body.rs".into(),
        path: "test-body.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    })
}
fn expanded(source: &str) -> TestResult<Report> {
    Ok(crate::resolve::expansion::analyze(
        input(source)?,
        &mut NoFiles,
        &Config::default(),
    )?)
}
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/test-cfg-demand/{name}.rs"
    ))?)
}
fn selected(report: &Report) -> TestResult<(FileId, usize)> {
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit
            .configured
            .cfg_uses
            .as_ref()
            .is_some_and(|uses| uses.len() != 0)
        {
            return Ok((FileId(u32::try_from(file)?), 0));
        }
    }
    Err("missing actual saved cfg use".into())
}
fn classify(report: &Report) -> TestResult<bool> {
    let (file, ordinal) = selected(report)?;
    let mut session = report.configured_demand_session();
    let used = session.checked(file, ordinal)?;
    Ok(matches!(
        session.classify(&used)?,
        DemandDisposition::RemovedTestBody(_)
    ))
}
#[test]
fn strict_body_receipts_keep_own_root_header_and_live_sibling_demands() -> TestResult {
    for name in [
        "removed-descendant-cfg",
        "removed-descendant-cfg-attr",
        "alias-removed-descendant-cfg",
        "removed-child-module",
        "false-ancestor",
        "requested-feature",
        "generated-root-forwarded-predicate",
    ] {
        expanded(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "own-cfg-cfg-first",
        "own-cfg-test-first",
        "own-cfg-attr",
        "own-ancestor-cfg",
        "removed-plus-live-sibling",
        "body-root-inner-cfg",
    ] {
        let error = expanded(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("{name}: feature demand omitted"))?;
        assert!(
            error.to_string().contains("cfg_ub_checks"),
            "{name}: {error}"
        );
    }
    // Header forms remain demanded; the new body-root E0658 is covered above.
    let error = expanded("#[test]fn gone(#[cfg(ub_checks)]x:u32){}")
        .err()
        .ok_or("unobserved header demand omitted")?;
    assert!(error.to_string().contains("cfg_ub_checks"), "{error}");
    Ok(())
}
#[test]
fn ordinary_and_cfg_free_test_removal_allocate_no_demand_index() -> TestResult {
    let ordinary = expanded("fn keep(){}")?;
    assert!(ordinary.attribute_effects.is_none());
    let removed = expanded("#[test]fn gone(){}")?;
    let removals = removed.removed_functions().ok_or("actual removal owner")?;
    assert_eq!(removals.entries.len(), 1);
    assert!(removals.demand.is_none());
    println!(
        "Test receipts Removals={} Index={} Path={} Work={} Session={}",
        size_of::<Removals>(),
        size_of::<Index>(),
        size_of::<super::Path>(),
        size_of::<Work>(),
        size_of::<ConfiguredDemandSession<'_>>()
    );
    Ok(())
}
#[test]
fn saved_use_and_retirement_identity_changes_reject_before_skip() -> TestResult {
    let source = fixture("removed-descendant-cfg")?;
    let first = expanded(&source)?;
    let second = expanded(&source)?;
    assert!(classify(&first)?);
    let (file, ordinal) = selected(&first)?;
    let session = first.configured_demand_session();
    let used = session.checked(file, ordinal)?;
    assert!(second.configured_demand_session().classify(&used).is_err());
    for opponent in 0..11 {
        let mut report = expanded(&source)?;
        let (file, ordinal) = selected(&report)?;
        corrupt(&mut report, file, ordinal, opponent)?;
        assert!(classify(&report).is_err(), "opponent {opponent}");
    }
    let mut generated =
        expanded("macro_rules!m{()=>{#[test]fn gone(){#[cfg(ub_checks)]fn nested(){}}}}m!();")?;
    let (file, ordinal) = selected(&generated)?;
    let offset = generated
        .package
        .unit(file)
        .configured
        .cfg_uses
        .as_ref()
        .and_then(|uses| uses.get(ordinal))
        .ok_or("generated use")?
        .predicate
        .span
        .start;
    let origin = generated.package.units[file.0 as usize]
        .origins
        .iter_mut()
        .find(|origin| origin.generated.start <= offset && offset < origin.generated.end)
        .ok_or("generated actual origin")?;
    origin.joint = Some(!origin.joint.unwrap_or(false));
    assert!(classify(&generated).is_err());
    Ok(())
}
fn corrupt(report: &mut Report, file: FileId, ordinal: usize, opponent: usize) -> TestResult {
    let removals = report.removed_functions().ok_or("removals")?;
    let removal = &removals.entries[0];
    let declaration = removal.declaration.node;
    let definition = removal.definition;
    let scope = removal.scope;
    let body = removals.demand.as_ref().ok_or("body index")?.bodies[0].root;
    let edge = (opponent == 7)
        .then(|| retained_block_edge(report))
        .transpose()?;
    let used = *report
        .package
        .unit(file)
        .configured
        .cfg_uses
        .as_ref()
        .and_then(|uses| uses.get(ordinal))
        .ok_or("saved use")?;
    let effect = used.effect as usize;
    let unit = &mut report.package.units[file.0 as usize];
    match opponent {
        0 => unit.source = SourceFile::new(unit.source.text().as_bytes().to_vec())?,
        1 => unit.configured.effects[effect].capture = u32::MAX,
        2 => {
            let capture = unit.configured.effects[effect].capture as usize;
            let forest = &unit.configured.ast.attributes[capture].forest;
            unit.configured.ast.attributes[capture].forest = forest
                .map_spans(&mut |span: Span| {
                    Ok(Span {
                        start: span.start + 1,
                        end: span.end + 1,
                    })
                })
                .map_err(|error| error.message)?;
        }
        3 => {
            let saved = unit
                .configured
                .cfg_uses
                .as_mut()
                .and_then(|uses| uses.get_mut(ordinal))
                .ok_or("mutable saved row")?;
            saved.predicate.span.start += 1;
        }
        4 => {
            let Kind::Item(item) = unit.configured.ast.node(declaration).kind else {
                return Err("declaration".into());
            };
            let ItemKind::Function { body, .. } =
                &mut unit.configured.ast.items[item as usize].kind
            else {
                return Err("function".into());
            };
            *body = Some(declaration);
        }
        5 => {
            report.definitions[definition].local_inner_macros =
                !report.definitions[definition].local_inner_macros;
        }
        6 => {
            report.resolution.scopes[scope.0 as usize].barrier =
                !report.resolution.scopes[scope.0 as usize].barrier;
        }
        7 => unit.configured.ast.edges[edge.ok_or("missing actual child edge")?] = declaration,
        8 => unit.configured.ast.nodes[body.get() as usize].span.start += 1,
        9 => {
            let effect = &mut unit.configured.effects[effect].attribute.effect;
            match effect {
                crate::expand::configured::Effect::Cfg(value)
                | crate::expand::configured::Effect::CfgAttr(value) => *value = !*value,
                crate::expand::configured::Effect::Keep => {
                    return Err("expected actual cfg effect".into());
                }
            }
        }
        10 => {
            let capture = unit.configured.effects[effect].capture as usize;
            unit.configured.ast.attributes[capture].forest = unit.configured.ast.attributes
                [capture]
                .forest
                .map_spans(&mut |span: Span| {
                    if span.start == span.end {
                        Ok(Span {
                            start: span.start + 1,
                            end: span.end + 1,
                        })
                    } else {
                        Ok(span)
                    }
                })
                .map_err(|error| error.message)?;
        }
        _ => return Err("unknown opponent".into()),
    }
    Ok(())
}
fn retained_block_edge(report: &Report) -> TestResult<usize> {
    let removals = report.removed_functions().ok_or("removals")?;
    removals
        .demand
        .as_ref()
        .ok_or("body index")?
        .paths
        .iter()
        .find_map(|path| {
            if let Kind::Block {
                statements: children,
                ..
            } = path.snapshot.kind
            {
                path.children.first().and_then(|(ordinal, _)| {
                    (children.start as usize + *ordinal < children.end as usize)
                        .then_some(children.start as usize + *ordinal)
                })
            } else {
                None
            }
        })
        .ok_or_else(|| "actual retained block child edge".into())
}

fn wide(depth: usize, width: usize) -> TestResult<Report> {
    let mut source = "#[test]fn gone(){".to_owned();
    for _ in 0..depth {
        source.push('{');
    }
    for index in 0..width {
        write!(
            source,
            "#[cfg(all(ub_checks,ub_checks))]fn nested_{index}(){{}}"
        )?;
    }
    for _ in 0..depth {
        source.push('}');
    }
    source.push('}');
    expanded(&source)
}
#[test]
fn shared_parent_prefixes_and_warm_uses_have_bounded_linear_work() -> TestResult {
    for (depth, width) in [(4, 8), (32, 32)] {
        let report = wide(depth, width)?;
        let removals = report.removed_functions().ok_or("removals")?;
        let index = removals.demand.as_ref().ok_or("actual cold cfg index")?;
        assert!(index.paths.len() < 2 * depth + 4 * width + 16);
        assert!(index.retained_payload() <= removals.work.get().bytes);
        let (file, ordinal) = selected(&report)?;
        let mut session = report.configured_demand_session();
        let used = session.checked(file, ordinal)?;
        assert!(matches!(
            session.classify(&used)?,
            DemandDisposition::RemovedTestBody(_)
        ));
        let before = removals.work.get().steps;
        for _ in 0..32 {
            let used = session.checked(file, ordinal)?;
            assert!(matches!(
                session.classify(&used)?,
                DemandDisposition::RemovedTestBody(_)
            ));
        }
        let warm = removals.work.get().steps - before;
        assert!(warm <= 32 * 32, "depth{depth}/width{width}: {warm}");
        let before_failed = removals.work.get().steps;
        assert!(session.checked(file, usize::MAX).is_err());
        assert!(removals.work.get().steps > before_failed);
        println!(
            "Test receipt depth={depth} width={width} paths={} payload={} charged={} warm={warm}",
            index.paths.len(),
            index.retained_payload(),
            removals.work.get().bytes
        );
    }
    Ok(())
}
#[test]
fn failed_retirement_preflight_publishes_neither_flags_nor_receipts() -> TestResult {
    let source = "#[test]fn gone(){#[cfg(ub_checks)]fn nested(){}}";
    let config = Config::default();
    let mut files = NoFiles;
    let mut engine = Engine::create(
        input(source)?,
        &mut files,
        &config,
        &crate::resolve::bindings::Context::default(),
    )?;
    engine.scan()?;
    let active = engine.loader.package.units[0].configured.active.clone();
    let enabled: Vec<_> = engine.loader.package.units[0]
        .configured
        .owners
        .iter()
        .map(|owner| owner.enabled)
        .collect();
    let removals = Removals::default();
    removals.work.set(Work {
        steps: 9_999_990,
        bytes: 0,
    });
    engine.attribute_effects = Some(Box::new(Effects {
        removals,
        policy: None,
        visibilities: None,
    }));
    let call = engine
        .calls
        .iter()
        .position(|call| {
            matches!(
                call.grammar,
                crate::resolve::expansion::CallGrammar::Attribute
            )
        })
        .ok_or("actual Test call")?;
    assert!(engine.attempt(call).is_err());
    assert_eq!(engine.loader.package.units[0].configured.active, active);
    assert_eq!(
        engine.loader.package.units[0]
            .configured
            .owners
            .iter()
            .map(|owner| owner.enabled)
            .collect::<Vec<_>>(),
        enabled
    );
    let removals = &engine
        .attribute_effects
        .as_ref()
        .ok_or("cold owner")?
        .removals;
    assert!(removals.entries.is_empty());
    assert!(removals.demand.is_none());
    assert!(removals.work.get().steps > 10_000_000);
    Ok(())
}
#[test]
fn fallible_child_edges_stop_before_later_siblings() -> TestResult {
    let report = expanded("fn a(){}fn b(){}fn c(){}")?;
    let unit = &report.package.units[0];
    let root = report.package.modules[0].root;
    let mut expected = Vec::new();
    unit.configured.ast.push_children(root, &mut expected);
    assert!(expected.len() >= 3);
    let mut visited = Vec::new();
    let result = unit.configured.ast.try_for_each_child(root, &mut |child| {
        visited.push(child);
        if visited.len() == 2 {
            Err("stop at actual edge")
        } else {
            Ok(())
        }
    });
    assert_eq!(result, Err("stop at actual edge"));
    assert_eq!(visited, expected[..2]);
    Ok(())
}

struct SharedFiles {
    source: String,
}
impl Sources for SharedFiles {
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        if path.file_name() != Some(std::ffi::OsStr::new("shared.rs")) {
            return Ok(None);
        }
        Ok(Some(InputFile {
            identity: "one-physical-shared-file".into(),
            path: path.into(),
            source: SourceFile::new(self.source.as_bytes().to_vec())?,
        }))
    }
}
#[test]
fn later_module_instance_retires_its_own_original_test_and_cfg_use() -> TestResult {
    for shared in [
        fixture("removed-descendant-cfg")?,
        "macro_rules!make{()=>{#[test]fn gone(){#[cfg(ub_checks)]fn nested(){}}}}make!();"
            .to_owned(),
    ] {
        two_instances(shared)?;
    }
    Ok(())
}
fn two_instances(shared: String) -> TestResult {
    let mut files = SharedFiles { source: shared };
    // One forwarding step queues the second load after a generated Test too.
    let source = "#[path=\"shared.rs\"]mod first;macro_rules!load{()=>{later!();}}macro_rules!later{()=>{#[path=\"shared.rs\"]mod second;}}load!();";
    let report =
        crate::resolve::expansion::analyze(input(source)?, &mut files, &Config::default())?;
    let removals = report
        .removed_functions()
        .ok_or("actual separate removals")?;
    assert_eq!(removals.entries.len(), 2);
    let first = report.package.unit(removals.entries[0].owner.file);
    let second = report.package.unit(removals.entries[1].owner.file);
    assert!(!Arc::ptr_eq(
        first.source.original_image(),
        second.source.original_image()
    ));
    assert_eq!(
        first.source.original().as_ptr(),
        second.source.original().as_ptr()
    );
    let mut session = report.configured_demand_session();
    for removal in &removals.entries {
        let used = session.checked(removal.owner.file, 0)?;
        assert!(matches!(
            session.classify(&used)?,
            DemandDisposition::RemovedTestBody(_)
        ));
    }
    Ok(())
}
