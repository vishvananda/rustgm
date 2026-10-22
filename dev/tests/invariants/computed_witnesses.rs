//! An observed source witness cannot select another initializer, owner or result.
use super::*;
use crate::sema::values::execution::{Executor, Progress, Request, Target};
use std::{cell::Cell, rc::Rc};
struct Capture(Rc<Cell<usize>>);
impl Executor for Capture {
    fn check(
        &mut self,
        _report: &crate::resolve::expansion::Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<(), Error> {
        let Target::SourceAssociated { source, result } = target else {
            return Ok(());
        };
        assert!(source.validate(facts, result).is_ok());
        assert_eq!(
            source.selected.result_type(facts),
            Err("selected constant arguments are not concrete")
        );
        let other = facts
            .functions
            .iter()
            .find_map(|f| f.body.filter(|body| *body != source.caller))
            .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing foreign body"))?;
        assert!(
            SourceComputation {
                caller: other,
                ..source
            }
            .validate(facts, result)
            .is_err()
        );
        assert!(
            SourceComputation {
                original: other,
                ..source
            }
            .validate(facts, result)
            .is_err()
        );
        assert!(
            SourceComputation {
                selected: Instance {
                    arguments: crate::types::GenericArgs::EMPTY,
                    ..source.selected
                },
                ..source
            }
            .validate(facts, result)
            .is_err()
        );
        assert!(
            source
                .validate(facts, TypeId::primitive(crate::types::Primitive::U16))
                .is_err()
        );
        let region = facts
            .types
            .arguments(source.selected.arguments)
            .and_then(|mut a| {
                a.find_map(|a| match a {
                    Argument::Lifetime(r) if r != RegionId::STATIC => Some(r),
                    _ => None,
                })
            })
            .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing source lifetime"))?;
        let record = &mut facts.body_regions[region.0 as usize - 1];
        let previous = *record;
        record.body = other;
        assert!(source.validate(facts, result).is_err());
        facts.body_regions[region.0 as usize - 1] = previous;
        assert!(source.validate(facts, result).is_ok());
        self.0.set(self.0.get() + 1);
        Err(Error::at(
            super::super::FileId(0),
            0,
            "captured checked source witness",
        ))
    }
    fn advance(
        &mut self,
        _: &crate::resolve::expansion::Report,
        _: &mut Declarations,
        _: &Request,
    ) -> Result<Progress, Error> {
        Ok(Progress::Complete(crate::sema::values::Constant::scalar(
            crate::sema::model::Scalar::Integer(1),
        )))
    }
}
#[test]
fn foreign_and_stale_source_witnesses_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/lifetime-initializer-execution/fresh-dispatcher.rs"),
    )?;
    let report = crate::test_source::resolve(&text, true)?;
    let count = Rc::new(Cell::new(0));
    let facts = crate::sema::declarations::analyze_with_executor(
        &report,
        Box::new(Capture(Rc::clone(&count))),
    )?;
    let result = crate::pipeline::analyze(&report, facts);
    let error = result.err().ok_or("source capture did not stop analysis")?;
    assert!(
        error
            .to_string()
            .contains("captured checked source witness"),
        "{error}"
    );
    assert_eq!(count.get(), 1);
    Ok(())
}
#[test]
fn common_request_layout_and_closed_argument_rules_are_unchanged() {
    assert_eq!(size_of::<SourceComputation>(), 24);
    assert_eq!(size_of::<Target>(), 32);
    assert_eq!(size_of::<Request>(), 56);
    assert_eq!(size_of::<(Target, usize)>(), 40);
    assert_eq!(size_of::<crate::const_eval::code::Catalog>(), 192);
    assert_eq!(size_of::<crate::const_eval::code::Code>(), 328);
}
