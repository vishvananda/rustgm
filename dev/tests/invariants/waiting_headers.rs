//! Resuming real declaration requests visits the retained candidate batch once.
use super::*;
use crate::sema::bodies::{Requirement, Work};
use std::fmt::Write;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn suspended_candidate_headers_keep_linear_work_and_existing_hot_widths() -> TestResult {
    for width in [16, 64, 256] {
        let mut source = String::new();
        for index in 0..width {
            write!(source, "fn candidate_{index}()->u32{{7}}")?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let definitions: Vec<_> = facts
            .functions
            .iter()
            .map(|function| function.definition)
            .collect();
        assert_eq!(definitions.len(), width);
        let saved: Vec<_> = definitions
            .iter()
            .map(|id| facts.definitions[id.0 as usize].take())
            .collect();
        let mut checker = Checker::new(&report, &mut facts);
        let node = report
            .package
            .unit(crate::resolve::unit::FileId(0))
            .configured
            .ast
            .root;
        let mut members = Box::new(definitions.clone().into_iter());
        for (&expected, ty) in definitions.iter().zip(saved) {
            let dependency = checker
                .method_headers(node, members)?
                .ok_or("missing actual method declaration request")?;
            let Requirement::Definition(actual) = dependency.requirement else {
                return Err("method batch requested a different proof".into());
            };
            assert_eq!(actual, expected);
            checker.facts.definitions[actual.0 as usize] = Some(ty.ok_or("real declaration")?);
            let Some(Work::MethodArguments(resumed, Some(remaining))) = checker.state.work.pop()
            else {
                return Err("missing retained candidate frontier".into());
            };
            assert_eq!(resumed, node);
            members = remaining;
        }
        assert!(checker.method_headers(node, members)?.is_none());
        assert_eq!(checker.state.steps, width);
        assert!(matches!(
            checker.state.work.pop(),
            Some(Work::MethodArguments(resumed, None)) if resumed == node
        ));
        assert!(checker.state.work.is_empty());
    }
    assert_eq!(size_of::<Work>(), 40);
    assert_eq!(size_of::<super::super::calls::CallTypes>(), 720);
    Ok(())
}
