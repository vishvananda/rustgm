//! Bring-up attribution names the function item that owns a located failure.
use super::{
    ir::{BodyOrigin, ProofSite},
    regions::Contract,
};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::{DiagnosticFile, Error, Package},
        unit::{FileId, NodeKey},
    },
    sema::model::{Declarations, Function},
    support::bringup::{Position, Stage, Stubs, selected},
    syntax::{
        ast::{Kind, NodeId},
        item::ItemKind,
    },
};
use std::collections::HashSet;

/// A source file's skip-list key; see `Package::skip_list_path`.
fn path(package: &Package, file: FileId) -> String {
    package.skip_list_path(file)
}

/// Whether the bring-up profile replaces this function item by a trapping stub.
pub(crate) fn stub(report: &Report, item: NodeKey) -> bool {
    report.package.bringup_listed(item)
}

/// The body nodes of the functions that the bring-up profile replaced by
/// trapping stubs. A body records its block, not its function item.
pub(crate) fn stub_bodies(report: &Report, facts: &Declarations) -> HashSet<NodeKey> {
    if selected().is_none_or(Stubs::is_empty) {
        return HashSet::new();
    }
    facts
        .functions
        .iter()
        .filter(|function| stub(report, function.node))
        .filter_map(|function| function.body)
        .collect()
}

/// Listed functions by source span. A stub replaces its function's body and
/// with it the closures and inline constants inside, which no later stage
/// builds or evaluates; nothing typed inside a body that did not type even
/// completed. A function item nested in a stub is a function of its own.
pub(crate) struct Stubbed(Vec<(FileId, u32, u32, DefId)>);
impl Stubbed {
    /// Every listed function with a body.
    pub(crate) fn listed(report: &Report, facts: &Declarations) -> Self {
        Self::select(report, facts, |_| true)
    }
    /// The listed functions whose bodies did not type.
    pub(crate) fn untyped(report: &Report, facts: &Declarations) -> Self {
        Self::select(report, facts, |function| {
            !facts.typed_function_bodies.contains(&function.definition)
        })
    }
    fn select(report: &Report, facts: &Declarations, keep: impl Fn(&Function) -> bool) -> Self {
        if selected().is_none_or(Stubs::is_empty) {
            return Self(Vec::new());
        }
        let spans = facts
            .functions
            .iter()
            .filter(|function| {
                function.body.is_some() && keep(function) && stub(report, function.node)
            })
            .map(|function| {
                let unit = report.package.unit(function.node.file);
                let span = unit.configured.ast.node(function.node.node).span;
                (
                    function.node.file,
                    span.start,
                    span.end,
                    function.definition,
                )
            });
        Self(spans.collect())
    }
    /// Whether a syntax node belongs to one of these functions: was written
    /// inside one and inside no function item nested in it.
    pub(crate) fn contains(&self, report: &Report, facts: &Declarations, key: NodeKey) -> bool {
        if self.0.is_empty() {
            return false;
        }
        // Only the few positions inside a listed item pay for the exact owner.
        written(report, key).into_iter().any(|(file, at)| {
            self.0
                .iter()
                .any(|&(listed, start, end, _)| listed == file && start <= at && at <= end)
                && owner(report, facts, file, at).is_some_and(|(function, _)| {
                    self.0
                        .iter()
                        .any(|&(.., definition)| definition == function.definition)
                })
        })
    }
}
/// Where a syntax node was written: its own position and, for a node that a
/// macro generated, where the expansion projects it and each macro call
/// that led to it, innermost first.
fn written(report: &Report, key: NodeKey) -> Vec<(FileId, u32)> {
    let at = report
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .span
        .start;
    let mut positions = vec![(key.file, at)];
    // Only a node that a macro generated pays for its projection.
    if report.package.origin(key.file, at).is_none() {
        return positions;
    }
    let projected = report.diagnostic(Error::at(key.file, at, String::new()));
    if let DiagnosticFile::Source(file) = projected.file {
        positions.push((file, projected.span.start));
    }
    positions.extend(
        projected
            .expansions
            .iter()
            .map(|trace| (trace.call.file, trace.call.span.start)),
    );
    positions
}

/// The innermost function item containing a failure's unprojected position.
#[derive(Debug)]
pub struct Blocker {
    /// Crate-relative source path, as the skip list spells it.
    pub path: String,
    /// The function item's byte offset in that file.
    pub offset: u32,
    pub name: String,
    /// The first failure met inside this function.
    pub message: String,
    /// The stage that recorded the failure; none for a failure that stopped
    /// the compilation without being recorded.
    pub stage: Option<Stage>,
}
/// Every failure of one compilation: those owned by a function that a stub can
/// replace, and the messages of those that no skip-list entry can remove.
#[derive(Debug, Default)]
pub struct Attribution {
    pub blockers: Vec<Blocker>,
    pub unattributed: Vec<String>,
}

/// Lets a stage continue past one body's located failure in the bring-up
/// profile, so that a single compilation reports every body the stage rejects.
pub(crate) struct Deferral(Stage, Option<Error>);
impl Deferral {
    pub(crate) const fn new(stage: Stage) -> Self {
        Self(stage, None)
    }
    /// The body's result, or nothing once its failure has been set aside.
    /// Outside the bring-up profile, and for unlocated failures, the failure
    /// is returned at once.
    pub(crate) fn admit<T>(&mut self, result: Result<T, Error>) -> Result<Option<T>, Error> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                let located = error.origin.is_some_and(|(file, at)| {
                    crate::support::bringup::defer(
                        self.0,
                        file.0,
                        Position::Byte(at),
                        &error.message,
                    )
                });
                if located {
                    self.1 = Some(error);
                    Ok(None)
                } else {
                    Err(error)
                }
            }
        }
    }
    /// Like `admit`, for a stage whose failure belongs to the body it worked
    /// on wherever it was raised: a constant that this body demands may fail
    /// inside a callee that is sound at run time.
    pub(crate) fn admit_for<T>(
        &mut self,
        report: &Report,
        body: BodyOrigin,
        result: Result<T, Error>,
    ) -> Result<Option<T>, Error> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) if defer_site(self.0, Some(body.root_site()), &error.message) => {
                self.1 = Some(report.body_error(body.root_site(), error.message.clone()));
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
    /// Fail the stage with the last failure it set aside, if any.
    pub(crate) fn finish(self) -> Result<(), Error> {
        self.1.map_or(Ok(()), Err)
    }
}
/// Set aside a failure of a stage that names syntax nodes rather than source
/// positions. The stage must still fail with such a failure when it ends.
pub(crate) fn defer_site(stage: Stage, site: Option<ProofSite>, message: &str) -> bool {
    let Some(ProofSite::Source(key)) = site else {
        return false;
    };
    crate::support::bringup::defer(stage, key.file.0, Position::Node(key.node.get()), message)
}

/// Pass over a failed soundness proof of this body, keeping its code. The
/// proof must have no result that a later stage reads.
pub(crate) fn unproven(stage: Stage, body: BodyOrigin, message: &str) -> bool {
    unproven_site(stage, Some(body.root_site()), message)
}
/// Like `unproven`, for a stage that names the proof's own syntax node.
pub(crate) fn unproven_site(stage: Stage, site: Option<ProofSite>, message: &str) -> bool {
    let Some(ProofSite::Source(key)) = site else {
        return false;
    };
    crate::support::bringup::unproven(stage, key.file.0, Position::Node(key.node.get()), message)
}

/// The innermost function item with a body that contains a source position.
fn owner<'a>(
    report: &Report,
    facts: &'a Declarations,
    file: FileId,
    at: u32,
) -> Option<(&'a Function, u32)> {
    let ast = &report.package.unit(file).configured.ast;
    facts
        .functions
        .iter()
        .filter(|function| function.node.file == file && function.body.is_some())
        .filter_map(|function| {
            let span = ast.node(function.node.node).span;
            (span.start <= at && at <= span.end).then_some((
                span.end - span.start,
                span.start,
                function,
            ))
        })
        .min_by_key(|&(length, start, _)| (length, start))
        .map(|(_, start, function)| (function, start))
}

/// Like `owner`, among the function items with a body that the resolver
/// declared: for a failure raised where no body facts were returned.
fn declared_owner(report: &Report, file: FileId, at: u32) -> Option<(DefId, u32)> {
    let ast = &report.package.unit(file).configured.ast;
    report
        .resolution
        .definitions
        .iter()
        .zip(0..)
        .filter(|(definition, _)| definition.kind == DefKind::Function)
        .filter_map(|(definition, index)| {
            let key = definition.node.filter(|key| key.file == file)?;
            let node = ast.node(key.node);
            let Kind::Item(item) = node.kind else {
                return None;
            };
            let body = matches!(
                ast.items.get(item as usize)?.kind,
                ItemKind::Function { body: Some(_), .. }
            );
            (body && node.span.start <= at && at <= node.span.end).then_some((
                node.span.end - node.span.start,
                node.span.start,
                DefId(index),
            ))
        })
        .min_by_key(|&(length, start, _)| (length, start))
        .map(|(_, start, definition)| (definition, start))
}

/// Name the functions that own this compilation's failures, for the bring-up
/// skip list: the failures its stages set aside, then the one that stopped it.
/// A failure whose stage named its owning function belongs to that function;
/// any other belongs to the innermost function item with a body that contains
/// its position, among the body facts when given and otherwise among the
/// resolver's declarations. Unlocated and imported failures are unattributed.
/// Empty outside the bring-up profile.
#[must_use]
pub fn blockers(report: &Report, facts: Option<&Declarations>, error: &Error) -> Attribution {
    let mut attribution = Attribution::default();
    if selected().is_none() {
        return attribution;
    }
    let mut failures = Vec::new();
    for failure in crate::support::bringup::deferred() {
        match offset(report, &failure) {
            Some(at) => failures.push((
                FileId(failure.file),
                at,
                failure.message,
                failure.owner,
                Some(failure.stage),
            )),
            None => attribution.unattributed.push(failure.message),
        }
    }
    // A stage that recorded failures ends with one of them, located or not.
    match error.origin {
        Some((file, at)) => {
            if !failures.iter().any(|seen| seen.0 == file && seen.1 == at) {
                failures.push((file, at, error.message.clone(), None, None));
            }
        }
        None if failures.iter().any(|seen| seen.2 == error.message) => {}
        None => attribution.unattributed.push(error.message.clone()),
    }
    for (file, at, message, owner, stage) in failures {
        let mut blocker = match owner {
            Some(owner) => owned(report, owner, message),
            None => blocker(report, facts, file, at, message),
        };
        blocker.stage = stage;
        if !attribution
            .blockers
            .iter()
            .any(|seen| seen.offset == blocker.offset && seen.path == blocker.path)
        {
            attribution.blockers.push(blocker);
        }
    }
    attribution
}
/// The function item that a stage named as a failure's owner.
fn owned(report: &Report, owner: crate::support::bringup::Owner, message: String) -> Blocker {
    let file = FileId(owner.file);
    let offset = NodeId::checked(owner.node).map_or(0, |node| {
        report
            .package
            .unit(file)
            .configured
            .ast
            .node(node)
            .span
            .start
    });
    let name = report
        .resolution
        .definitions
        .get(owner.definition as usize)
        .map_or("?", |definition| report.package.names.get(definition.name));
    Blocker {
        path: path(&report.package, file),
        offset,
        name: name.to_owned(),
        message,
        stage: None,
    }
}
/// The byte offset of a failure set aside, if its position still exists.
fn offset(report: &Report, failure: &crate::support::bringup::Deferred) -> Option<u32> {
    match failure.position {
        Position::Byte(at) => Some(at),
        Position::Node(node) => NodeId::checked(node).map(|node| {
            let ast = &report.package.unit(FileId(failure.file)).configured.ast;
            ast.node(node).span.start
        }),
    }
}
/// Name the function item that owns a position. Outside every function, the
/// position itself is the entry: it only takes effect where it is the anchor
/// of a synthesized body.
fn blocker(
    report: &Report,
    facts: Option<&Declarations>,
    file: FileId,
    at: u32,
    message: String,
) -> Blocker {
    let found = match facts {
        Some(facts) => {
            owner(report, facts, file, at).map(|(function, start)| (function.definition, start))
        }
        None => declared_owner(report, file, at),
    };
    let (offset, name) = found.map_or((at, "<item>"), |(definition, start)| {
        let name = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .map_or("?", |definition| report.package.names.get(definition.name));
        (start, name)
    });
    Blocker {
        path: path(&report.package, file),
        offset,
        name: name.to_owned(),
        message,
        stage: None,
    }
}
/// The functions that keep their bodies although a soundness proof of them
/// failed, once per function and message. Empty outside the bring-up profile.
#[must_use]
pub fn unproven_bodies(report: &Report, facts: &Declarations) -> Vec<Blocker> {
    let mut bodies: Vec<Blocker> = Vec::new();
    for failure in crate::support::bringup::unproven_proofs() {
        let Some(at) = offset(report, &failure) else {
            continue;
        };
        let mut body = blocker(
            report,
            Some(facts),
            FileId(failure.file),
            at,
            failure.message,
        );
        body.stage = Some(failure.stage);
        if !bodies.iter().any(|seen| {
            seen.offset == body.offset && seen.path == body.path && seen.message == body.message
        }) {
            bodies.push(body);
        }
    }
    bodies
}

/// A stubbed function proves nothing about the body it replaces: drop the
/// lifetime facts that typing collected for it and for every body nested in it.
pub(crate) fn relax(report: &Report, facts: &Declarations, contracts: &mut [Contract]) {
    if selected().is_none_or(Stubs::is_empty) {
        return;
    }
    let package = &report.package;
    let position = |key: NodeKey| package.unit(key.file).configured.ast.node(key.node).span;
    let stubbed = facts
        .functions
        .iter()
        .filter(|function| function.body.is_some() && stub(report, function.node))
        .map(|function| (function.node.file, position(function.node)))
        .collect::<Vec<_>>();
    for contract in contracts {
        let Some(source) = contract.source.source() else {
            continue;
        };
        let at = position(source).start;
        // Only the few positions inside a stubbed item pay for the exact owner.
        if stubbed
            .iter()
            .any(|&(file, span)| file == source.file && span.start <= at && at <= span.end)
            && owner(report, facts, source.file, at)
                .is_some_and(|(function, _)| stub(report, function.node))
        {
            contract.obligations.clear();
            contract.proofs.clear();
        }
    }
}

/// Whether the bring-up profile stubs the function that owns a nested body.
pub(crate) fn stubbed(report: &Report, facts: &Declarations, owner: DefId) -> bool {
    selected().is_some()
        && facts
            .function_indices
            .get(&owner)
            .and_then(|&index| facts.functions.get(index))
            .is_some_and(|function| stub(report, function.node))
}
