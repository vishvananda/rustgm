//! General typed attribute visits do not consume source permission work.
use super::*;
use std::fmt::Write as _;

fn payer<'a>(
    engine: &'a Engine<'_, NoFiles>,
) -> TestResult<&'a crate::resolve::lint_policy::producer::Producer> {
    engine
        .attribute_effects
        .as_deref()
        .and_then(|owner| owner.policy.as_ref())
        .ok_or_else(|| "actual shared attribute payer".into())
}
fn initial(engine: &Engine<'_, NoFiles>) -> TestResult<(Visit, FileId, NodeKey)> {
    let visit = *engine
        .frames
        .first()
        .ok_or("actual initial traversal frame")?;
    let file = engine.loader.package.modules[visit.module.0 as usize].file;
    Ok((
        visit,
        file,
        NodeKey {
            file,
            node: visit.node,
        },
    ))
}

#[test]
fn generated_attributes_charge_the_general_payer_linearly() -> TestResult {
    for prefix in [1_usize, 8, 32] {
        let mut previous = None;
        for width in [8_usize, 32, 128] {
            let mut source = String::from(
                "#![feature(no_core)]#![no_core]macro_rules! make{($($name:ident),*)=>{$(",
            );
            for _ in 0..prefix {
                source.push_str("#[doc=\"retained\"]");
            }
            source.push_str("#[inline]fn $name()->u8{7u8})*}}make!(");
            for index in 0..width {
                if index != 0 {
                    source.push(',');
                }
                write!(source, "function_{index}")?;
            }
            source.push_str(");");
            with_engine(&source, true, |engine| {
                engine.run()?;
                assert_eq!(engine.invocations.len(), 1);
                assert_eq!(engine.invocations[0].parser_calls, 0);
                let functions = engine
                    .paths
                    .state
                    .facts
                    .definitions
                    .iter()
                    .filter(|definition| {
                        definition.kind == crate::resolve::model::DefKind::Function
                            && engine
                                .loader
                                .package
                                .names
                                .get(definition.name)
                                .starts_with("function_")
                    })
                    .count();
                assert_eq!(functions, width);
                let work = payer(engine)?.work();
                let bound = 4096 + 128 * width * (prefix + 1);
                assert!(
                    work.steps <= bound,
                    "{width}/{prefix}: {} > {bound}",
                    work.steps
                );
                let (permissions, _, _) = engine
                    .loader
                    .package
                    .source_core()
                    .ok_or("source")?
                    .macros
                    .usage();
                assert!(
                    permissions < 1024,
                    "ordinary generated attributes spent permission work"
                );
                if let Some((old_width, old_work, old_permissions)) = previous {
                    assert!(work.steps > old_work);
                    assert!(work.steps <= old_work * (width / old_width) + 4096);
                    assert_eq!(permissions, old_permissions);
                }
                previous = Some((width, work.steps, permissions));
                eprintln!(
                    "generated attribute width={width} prefix={prefix} work={} bytes={} permission-work={permissions}",
                    work.steps, work.bytes
                );
                Ok(())
            })?;
        }
    }
    Ok(())
}

#[test]
fn ordinary_attributes_do_not_spend_or_revive_panic_permissions() -> TestResult {
    with_engine(
        "#![feature(no_core)]#![no_core]#[doc=\"kept\"]#[inline]fn target(){}",
        true,
        |engine| {
            let (visit, file, _) = initial(engine)?;
            let contracts = &engine.loader.package.source_core().ok_or("source")?.macros;
            let before = contracts.usage().0;
            contracts.charge(
                999_999_usize
                    .checked_sub(before)
                    .ok_or("unexpected initial permission work")?,
                0,
            )?;
            engine.run()?;
            assert_eq!(
                engine
                    .loader
                    .package
                    .source_core()
                    .ok_or("source")?
                    .macros
                    .usage()
                    .0,
                999_999
            );
            let before = payer(engine)?.work().steps;
            assert!(
                engine
                    .loader
                    .package
                    .source_core()
                    .ok_or("source")?
                    .macros
                    .charge(2, 0)
                    .is_err()
            );
            let error = engine
                .macro_attribute(visit, file)
                .err()
                .ok_or("failed permission owner revived")?;
            assert!(
                error.message.contains("source macro contracts exceed"),
                "{error}"
            );
            assert_eq!(payer(engine)?.work().steps, before);
            assert_eq!(counts(engine), (0, 0, 1));
            Ok(())
        },
    )
}

#[test]
fn exhausted_attribute_payer_blocks_children_and_failed_retries() -> TestResult {
    with_engine(
        "#![feature(no_core)]#![no_core]#[doc=\"kept\"]fn target(){missing!();}",
        true,
        |engine| {
            let (visit, file, key) = initial(engine)?;
            engine.ensure_attribute_policy(key)?;
            let policy = payer(engine)?;
            let work = policy.work();
            assert!(
                policy
                    .attribute_read(
                        key,
                        10_000_001_usize
                            .checked_sub(work.steps)
                            .ok_or("unexpected initial attribute work")?
                    )
                    .is_err()
            );
            let exhausted = payer(engine)?.work().steps;
            assert_eq!(exhausted, 10_000_001);
            let before = counts(engine);
            let error = engine
                .run()
                .err()
                .ok_or("exhausted attribute payer expanded children")?;
            assert!(
                error
                    .message
                    .contains("source lint policy work or storage budget exceeded"),
                "{error}"
            );
            assert!(engine.calls.is_empty());
            assert_eq!(counts(engine), before);
            let first = payer(engine)?.work().steps;
            assert!(first > exhausted);
            assert!(engine.macro_attribute(visit, file).is_err());
            assert!(payer(engine)?.work().steps > first);
            assert_eq!(counts(engine), before);
            assert!(
                engine
                    .loader
                    .package
                    .source_core()
                    .ok_or("source")?
                    .macros
                    .charge(0, 0)
                    .is_ok()
            );
            Ok(())
        },
    )
}
