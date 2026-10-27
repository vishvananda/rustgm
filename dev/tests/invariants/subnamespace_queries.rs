//! Kind-filtered lookup reuses unfiltered scope caches and pays every resumed candidate.
use super::super::{Body, MacroTarget, role_declarations::with_engine};
use super::*;
use crate::expand::builtin::Builtin;
use std::{cell::Cell, fmt::Write, mem::size_of};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn kind_filtering_keeps_warm_misses_pending_and_failed_attempts_separate() -> TestResult {
    for width in [8, 32, 128] {
        let mut source = String::from("#![feature(no_core,rustc_attrs)]\n#![no_core]\n");
        writeln!(
            source,
            "#[rustc_builtin_macro(test)]macro_rules! chosen{{($i:item)=>{{}}}}"
        )?;
        for _ in 0..width {
            writeln!(
                source,
                "#[rustc_builtin_macro(line)]macro_rules! chosen{{()=>{{}}}}"
            )?;
        }
        with_engine(&source, true, |engine| {
            engine.run()?;
            assert_eq!(engine.definitions.len(), width + 1);
            let last = &engine.definitions[width];
            let name = (last.name, ItemContext::ROOT);
            let head = engine
                .scopes
                .define(last.preceding_scope, last.name, width)?;
            let bindings = Bindings {
                definitions: &engine.definitions,
            };
            let ancestry = Ancestry::default();
            let steps = Cell::new(0);
            let pay = |amount| {
                steps.set(steps.get() + amount);
                Ok(())
            };
            for _ in 0..2 {
                assert_eq!(
                    bindings.textual_kind(
                        &mut engine.scopes,
                        &ancestry,
                        head,
                        name,
                        ExpansionId(0),
                        (BuiltinKind::Attribute, &pay)
                    )?,
                    Lookup::Found(0)
                );
            }
            assert_eq!(steps.get(), 2 * (width + 1));
            assert_eq!(
                bindings.textual(&mut engine.scopes, &ancestry, head, name, ExpansionId(0))?,
                Lookup::Found(width)
            );
            let fail = |amount| {
                steps.set(steps.get() + amount);
                Err(error(0, "test candidate budget exhausted"))
            };
            for _ in 0..2 {
                assert!(
                    bindings
                        .textual_kind(
                            &mut engine.scopes,
                            &ancestry,
                            head,
                            name,
                            ExpansionId(0),
                            (BuiltinKind::Attribute, &fail)
                        )
                        .is_err()
                );
            }
            assert_eq!(steps.get(), 2 * (width + 1) + 2);
            let pending = engine.scopes.output()?;
            assert_eq!(
                bindings.textual_kind(
                    &mut engine.scopes,
                    &ancestry,
                    pending,
                    name,
                    ExpansionId(0),
                    (BuiltinKind::Attribute, &pay)
                )?,
                Lookup::Pending(pending)
            );
            engine.scopes.complete(pending, head)?;
            assert_eq!(
                bindings.textual_kind(
                    &mut engine.scopes,
                    &ancestry,
                    pending,
                    name,
                    ExpansionId(0),
                    (BuiltinKind::Attribute, &pay)
                )?,
                Lookup::Found(0)
            );
            assert!(matches!(
                engine.definitions[0].body,
                Body::Builtin(Builtin::Test)
            ));
            eprintln!(
                "kind-filter width={width} attempts={} unfiltered_steps={}",
                steps.get(),
                engine.scopes.lookup_steps
            );
            Ok(())
        })?;
    }
    assert_eq!(size_of::<MacroTarget>(), 16);
    assert_eq!(size_of::<Definition>(), 136);
    assert_eq!(size_of::<Body>(), 96);
    eprintln!(
        "kind-layout target={} definition={} body={} scopes={}",
        size_of::<MacroTarget>(),
        size_of::<Definition>(),
        size_of::<Body>(),
        size_of::<Scopes>()
    );
    Ok(())
}
