//! Observe actual source Copy completion separately from the anonymous selector frontier.
use super::*;
use crate::{
    body::ir::{Action, Place, ProjectionKind, ValueId},
    resolve::unit::{FileId, NodeKey},
    syntax::ast::{Kind as Syntax, NodeId, Unary},
    types::traits::Application,
};
use std::cell::RefCell;

#[derive(Debug)]
struct Read {
    statement: usize,
    id: ValueId,
    source: Option<NodeKey>,
    ty: TypeId,
    kind: Option<Kind>,
    place: Place,
    projection: Option<(Place, ProjectionKind, TypeId)>,
    goal: Option<Predicate>,
    application: Option<Application>,
    copied: Option<bool>,
    validated_owner: Option<crate::resolve::model::DefId>,
}
struct Capture {
    source: NodeKey,
    visits: usize,
    truncated: bool,
    declaration: Option<crate::resolve::model::DefId>,
    reads: Vec<Read>,
}
thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}
struct Scope;
impl Scope {
    fn begin(source: NodeKey) -> Result<Self> {
        CAPTURE.with(|slot| -> Result {
            let mut slot = slot.try_borrow_mut()?;
            if slot.is_some() {
                return Err("read frontier observer cannot reenter".into());
            }
            *slot = Some(Capture {
                source,
                visits: 0,
                truncated: false,
                declaration: None,
                reads: Vec::new(),
            });
            Ok(())
        })?;
        Ok(Self)
    }
    fn finish(self) -> Result<Capture> {
        let captured = CAPTURE
            .with(|slot| slot.borrow_mut().take())
            .ok_or("capture")?;
        drop(self);
        Ok(captured)
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        CAPTURE.with(|slot| drop(slot.replace(None)));
    }
}
pub(super) fn record(facts: &Declarations, body: &Body, prepared: &Prepared) {
    CAPTURE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(capture) = slot.as_mut() else {
            return;
        };
        if body.source.source() != Some(capture.source) {
            return;
        }
        capture.visits += 1;
        capture.declaration = prepared.requirements.declaration;
        for (index, statement) in body.statements.iter().enumerate() {
            let Action::Evaluate(id) = statement.action else {
                continue;
            };
            let Some(value) = body.values.get(id.0 as usize) else {
                capture.truncated = true;
                break;
            };
            let Rvalue::Read(place) = value.kind else {
                continue;
            };
            if capture.reads.len() == 32 {
                capture.truncated = true;
                break;
            }
            let goal = prepared
                .requirements
                .goals
                .iter()
                .find(|goal| goal.ty == value.ty)
                .copied();
            let application = goal.and_then(|goal| match goal.bound {
                Bound::Trait(id) => facts.types.trait_application(id),
                _ => None,
            });
            let projection = match place {
                Place::Projection(id) => body
                    .projections
                    .get(id.0 as usize)
                    .map(|p| (p.base, p.kind, p.ty)),
                _ => None,
            };
            capture.reads.push(Read {
                statement: index,
                id,
                source: body.syntax_site(statement.source).ok(),
                ty: value.ty,
                kind: facts.types.kind(value.ty),
                place,
                projection,
                goal,
                application,
                copied: prepared.decisions.get(&value.ty).copied(),
                validated_owner: goal.and_then(|goal| {
                    facts
                        .generics
                        .implementations()
                        .find(|header| header.predicate == goal)
                        .map(|header| header.owner)
                }),
            });
        }
    });
}

fn source_keys(report: &crate::resolve::expansion::Report) -> Result<(NodeKey, NodeKey)> {
    let ast = &report.package.unit(FileId(0)).configured.ast;
    let index = ast
        .nodes
        .iter()
        .position(|node| {
            node.span.start == 2386
                && matches!(
                    node.kind,
                    Syntax::Unary {
                        op: Unary::Dereference,
                        ..
                    }
                )
        })
        .ok_or("actual dereference at byte2386")?;
    let dereference = NodeKey {
        file: FileId(0),
        node: NodeId::new(u32::try_from(index)?),
    };
    // Constant preparation checks the complete root. Its Cast consumes the
    // actual dereference node; their shared start offset supplies no identity.
    let mut parents = ast.nodes.iter().enumerate().filter(
        |(_, node)| matches!(node.kind, Syntax::Cast { value, .. } if value == dereference.node),
    );
    let (index, _) = parents.next().ok_or("actual consuming cast root")?;
    assert!(parents.next().is_none());
    let root = NodeKey {
        file: dereference.file,
        node: NodeId::new(u32::try_from(index)?),
    };
    assert_ne!(root, dereference);
    Ok((root, dereference))
}

#[test]
fn anonymous_reference_read_frontier_keeps_actual_copy_selection_observable() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/const-eval-select-ctfe/reference-anonymous-selector.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "anonymous_read_frontier")?;
    let (root, dereference) = source_keys(&report)?;
    let copy = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "Copy")
        .ok_or("actual Copy declaration")?;
    let scope = Scope::begin(root)?;
    assert!(Scope::begin(root).is_err());
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
    println!(
        "anonymous-root={root:?} dereference={dereference:?} visits={} Copy={:?} truncated={} reads={}",
        capture.visits,
        capture.declaration,
        capture.truncated,
        capture.reads.len()
    );
    assert!(capture.visits > 0 && !capture.truncated && !capture.reads.is_empty());
    assert_eq!(
        capture.declaration,
        Some(crate::resolve::model::DefId(u32::try_from(copy)?))
    );
    for read in &capture.reads {
        println!(
            "statement={} Read={:?} source={:?} canonical-type={:?} kind={:?} place={:?} projection={:?} goal={:?} application={:?} selected-copy={:?} validated-owner={:?}",
            read.statement,
            read.id,
            read.source,
            read.ty,
            read.kind,
            read.place,
            read.projection,
            read.goal,
            read.application,
            read.copied,
            read.validated_owner
        );
    }
    assert!(
        capture
            .reads
            .iter()
            .any(|read| read.source == Some(dereference))
    );
    for read in capture.reads {
        if read.source == Some(dereference) {
            assert_eq!(read.copied, Some(true));
            assert!(
                read.validated_owner.is_some(),
                "actual source read lost semantic impl completion"
            );
        }
        assert!(
            read.source.is_some()
                && read.kind.is_some()
                && read.goal.is_some()
                && read.copied.is_some()
        );
        assert_eq!(
            matches!(read.place, Place::Projection(_)),
            read.projection.is_some()
        );
        let application = read.application.ok_or("actual Copy application")?;
        assert_eq!(application.item.0, u32::try_from(copy)?);
        assert_eq!(application.arguments, crate::types::GenericArgs::EMPTY);
    }
    Ok(())
}
