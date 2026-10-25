//! A bounded observer sees real reached anonymous reference calls after ordinary proof.
use super::*;
use crate::{body::ir::BodyKey, resolve::unit::NodeKey};
use std::cell::RefCell;

struct Snapshot {
    source: NodeKey,
    key: BodyKey,
    code: *const Code,
    row: *const super::super::use_inputs::Row,
    contract: *const crate::body::regions::Contract,
    frame: usize,
    value: ValueId,
    selector: Instance,
    callback: Instance,
    opening: TypeId,
    root_result: TypeId,
    row_matches: bool,
    opponents: [(&'static str, bool); 4],
}
struct Capture {
    source: NodeKey,
    visits: usize,
    truncated: bool,
    rows: Vec<Snapshot>,
    foreign: Option<(
        crate::pipeline::execution::Context,
        crate::sema::values::execution::Target,
    )>,
}
thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}
struct Scope;
impl Scope {
    fn begin(
        source: NodeKey,
        foreign: Option<(
            crate::pipeline::execution::Context,
            crate::sema::values::execution::Target,
        )>,
    ) -> Result<Self, &'static str> {
        CAPTURE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err("anonymous selected observer cannot reenter");
            }
            *slot = Some(Capture {
                source,
                visits: 0,
                truncated: false,
                rows: Vec::new(),
                foreign,
            });
            Ok(Self)
        })
    }
    fn finish(self) -> Result<Capture, &'static str> {
        let result = CAPTURE
            .with(|slot| slot.borrow_mut().take())
            .ok_or("missing anonymous capture")?;
        drop(self);
        Ok(result)
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        CAPTURE.with(|slot| drop(slot.replace(None)));
    }
}

// Hook only after full validate_source has minted the actual existing Call and
// after callback/tuple/current-statement validation. No synthetic positive Call.
pub(super) fn record(
    facts: &Declarations,
    catalog: &Catalog,
    inputs: Option<&super::super::use_inputs::Inputs>,
    call: &Call<'_>,
    reference: Option<&ReferenceOpening<'_>>,
) {
    CAPTURE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(capture) = slot.as_mut() else {
            return;
        };
        if call.code.source.source() != Some(capture.source) {
            return;
        }
        capture.visits += 1;
        if capture.rows.len() == 16 {
            capture.truncated = true;
            return;
        }
        let Some(reference) = reference else {
            capture.truncated = true;
            return;
        };
        let Some(inputs) = inputs else {
            capture.truncated = true;
            return;
        };
        let Ok(row) = inputs.checked(catalog, call.frame.body, call.code) else {
            capture.truncated = true;
            return;
        };
        let foreign_inputs_rejected = capture.foreign.as_ref().is_some_and(|(context, target)| {
            context
                .inspect_const_call(*target, |foreign, foreign_inputs, index| {
                    foreign.code(index).is_some_and(|other| {
                        other.source == call.code.source
                            && other.result == call.code.result
                            && !std::ptr::eq(other, call.code)
                            && inputs.checked(foreign, index, other).is_err()
                            && foreign_inputs
                                .checked(catalog, call.frame.body, call.code)
                                .is_err()
                            && inputs.checked(catalog, call.frame.body, other).is_err()
                    })
                })
                .is_ok_and(std::convert::identity)
        });
        capture.rows.push(Snapshot {
            source: capture.source,
            key: call.code.owner.key(call.code.source),
            code: std::ptr::from_ref(call.code),
            row: std::rc::Rc::as_ptr(row),
            contract: std::sync::Arc::as_ptr(row.contract()),
            frame: call.frame.instance,
            value: call.value,
            selector: call.selector(),
            callback: call.callback(),
            opening: reference.signature,
            root_result: call.code.result,
            row_matches: reference.matches(row, call.frame, call.value),
            opponents: [
                (
                    "different Call",
                    !reference.matches(row, call.frame, ValueId(u32::MAX)),
                ),
                (
                    "missing Inputs",
                    super::super::use_inputs::Inputs::default()
                        .opening(catalog, call.frame.body, call.code, call.value)
                        .is_err(),
                ),
                ("foreign Inputs/Code/Catalog", foreign_inputs_rejected),
                (
                    "missing canonical Code",
                    Catalog::default()
                        .expression(facts, call.frame.body, capture.source, call.code.result)
                        .is_err(),
                ),
            ],
        });
    });
}

type TestResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn original() -> TestResult<(crate::resolve::expansion::Report, NodeKey)> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/const-eval-select-ctfe/reference-anonymous-selector.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "actual_anonymous_reference")?;
    let ast = &report
        .package
        .unit(crate::resolve::unit::FileId(0))
        .configured
        .ast;
    let index = ast
        .nodes
        .iter()
        .position(|node| {
            node.span.start == 2386 && matches!(node.kind, crate::syntax::ast::Kind::Cast { .. })
        })
        .ok_or("actual consuming Cast")?;
    let root = NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: crate::syntax::ast::NodeId::new(u32::try_from(index)?),
    };
    assert_eq!(root.node.get(), 303);
    Ok((report, root))
}
fn check_opening(
    facts: &Declarations,
    report: &crate::resolve::expansion::Report,
    row: &Snapshot,
) -> TestResult {
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        ..
    }) = facts.types.kind(row.opening)
    else {
        return Err("actual whole opened signature".into());
    };
    let fields = facts.types.list(parameters).ok_or("whole input tuple")?;
    assert_eq!(fields.len(), 1);
    for ty in [fields[0], result] {
        assert!(
            matches!(facts.types.kind(ty), Some(Kind::Reference { mutable: false, referent, .. }) if referent == TypeId::primitive(crate::types::Primitive::U8))
        );
    }
    let Some(Kind::FunctionItem {
        item,
        arguments: crate::types::GenericArgs::EMPTY,
        signature,
    }) = facts.types.kind(row.callback.ty())
    else {
        return Err("actual nongeneric F".into());
    };
    assert!(
        matches!(facts.types.kind(signature), Some(Kind::Function { bound_regions, .. }) if bound_regions > 0)
    );
    assert_eq!(
        report
            .package
            .names
            .get(report.resolution.definitions[item.0 as usize].name),
        "ct"
    );
    assert!(crate::sema::compiler_items::source_select(facts, row.selector)?.is_some());
    Ok(())
}

#[test]
fn actual_anonymous_reference_call_keeps_exact_row_frame_opening_and_foreign_opponents()
-> TestResult {
    let (foreign_report, foreign_root) = original()?;
    let foreign = crate::pipeline::analyze_source(&foreign_report)?;
    let foreign_context = foreign
        .prepared
        .constants
        .as_ref()
        .ok_or("foreign completed context")?
        .clone();
    let target = crate::sema::values::execution::Target::Expression {
        source: foreign_root,
        result: TypeId::primitive(crate::types::Primitive::Usize),
        context: None,
    };
    let (report, root) = original()?;
    let scope = Scope::begin(root, Some((foreign_context, target)))?;
    assert!(Scope::begin(root, None).is_err());
    let analysis = crate::pipeline::analyze_source(&report)?;
    assert!(
        analysis
            .facts
            .array_lengths
            .values()
            .any(|&length| length == 7)
    );
    let capture = scope.finish()?;
    assert!(CAPTURE.with(|slot| slot.borrow().is_none()));
    assert!(capture.visits > 0 && !capture.truncated && !capture.rows.is_empty());
    let mut identity = None;
    for row in capture.rows {
        assert_eq!(row.source, root);
        assert_eq!(row.key, BodyKey::Anonymous(root));
        assert!(!row.code.is_null() && !row.row.is_null() && !row.contract.is_null());
        assert!(row.value.0 < u32::MAX);
        assert!(row.row_matches);
        for (opponent, rejected) in row.opponents {
            assert!(rejected, "{opponent} acquired reference authority");
        }
        assert_eq!(
            row.root_result,
            TypeId::primitive(crate::types::Primitive::Usize)
        );
        check_opening(&analysis.facts, &report, &row)?;
        let current = (
            row.code,
            row.row,
            row.contract,
            row.frame,
            row.value,
            row.selector,
            row.callback,
            row.opening,
        );
        if let Some(before) = identity {
            assert_eq!(before, current);
        }
        identity = Some(current);
        println!(
            "anonymous selected: root={root:?} call={:?} frame={} selector={:?} F={:?} opening={:?} Code={:?} Row={:?} Contract={:?}",
            row.value,
            row.frame,
            row.selector,
            row.callback,
            row.opening,
            row.code,
            row.row,
            row.contract
        );
    }
    Ok(())
}
#[test]
fn anonymous_reference_observer_clears_on_interruption_and_rejects_reentry() -> TestResult {
    fn interrupted(root: NodeKey) -> TestResult {
        let _scope = Scope::begin(root, None)?;
        assert!(Scope::begin(root, None).is_err());
        Err("interrupted observer".into())
    }
    let (_, root) = original()?;
    assert!(interrupted(root).is_err());
    assert!(CAPTURE.with(|slot| slot.borrow().is_none()));
    let capture = Scope::begin(root, None)?.finish()?;
    assert_eq!(capture.visits, 0);
    assert!(!capture.truncated && capture.rows.is_empty());
    Ok(())
}
