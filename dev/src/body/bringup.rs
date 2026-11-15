//! Bring-up attribution names the function item that owns a located failure.
use super::{
    ir::{BodyOrigin, ProofSite},
    regions::Contract,
};
use crate::{
    resolve::{
        expansion::Report,
        model::DefId,
        package::{Error, Package},
        unit::{FileId, NodeKey},
    },
    sema::model::{Declarations, Function},
    support::bringup::{Position, Stubs, selected},
    syntax::ast::NodeId,
};
use std::collections::HashSet;

/// A source file's skip-list key: its path below the crate root's directory,
/// or its whole path when it lies elsewhere.
fn path(package: &Package, file: FileId) -> String {
    let path = &package.file_paths[file.0 as usize];
    package
        .file_paths
        .first()
        .and_then(|root| root.parent())
        .and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// Whether the bring-up profile replaces this function item by a trapping stub.
pub(crate) fn stub(report: &Report, item: NodeKey) -> bool {
    selected().is_some_and(|stubs| {
        let package = &report.package;
        let start = package
            .unit(item.file)
            .configured
            .ast
            .node(item.node)
            .span
            .start;
        stubs.contains(&path(package, item.file), start)
    })
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
#[derive(Default)]
pub(crate) struct Deferral(Option<Error>);
impl Deferral {
    /// The body's result, or nothing once its failure has been set aside.
    /// Outside the bring-up profile, and for unlocated failures, the failure
    /// is returned at once.
    pub(crate) fn admit<T>(&mut self, result: Result<T, Error>) -> Result<Option<T>, Error> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                let located = error.origin.is_some_and(|(file, at)| {
                    crate::support::bringup::defer(file.0, Position::Byte(at), &error.message)
                });
                if located {
                    self.0 = Some(error);
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
            Err(error) if defer_site(Some(body.root_site()), &error.message) => {
                self.0 = Some(report.body_error(body.root_site(), error.message.clone()));
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
    /// Fail the stage with the last failure it set aside, if any.
    pub(crate) fn finish(self) -> Result<(), Error> {
        self.0.map_or(Ok(()), Err)
    }
}
/// Set aside a failure of a stage that names syntax nodes rather than source
/// positions. The stage must still fail with such a failure when it ends.
pub(crate) fn defer_site(site: Option<ProofSite>, message: &str) -> bool {
    let Some(ProofSite::Source(key)) = site else {
        return false;
    };
    crate::support::bringup::defer(key.file.0, Position::Node(key.node.get()), message)
}

/// Pass over a failed soundness proof of this body, keeping its code. The
/// proof must have no result that a later stage reads.
pub(crate) fn unproven(body: BodyOrigin, message: &str) -> bool {
    unproven_site(Some(body.root_site()), message)
}
/// Like `unproven`, for a stage that names the proof's own syntax node.
pub(crate) fn unproven_site(site: Option<ProofSite>, message: &str) -> bool {
    let Some(ProofSite::Source(key)) = site else {
        return false;
    };
    crate::support::bringup::unproven(key.file.0, Position::Node(key.node.get()), message)
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

/// Name the functions that own this compilation's failures, for the bring-up
/// skip list: the failures its stages set aside, then the one that stopped it.
/// Unlocated and imported failures, and positions outside every function item
/// that has a body, are unattributed. Empty outside the bring-up profile.
#[must_use]
pub fn blockers(report: &Report, facts: &Declarations, error: &Error) -> Attribution {
    let mut attribution = Attribution::default();
    if selected().is_none() {
        return attribution;
    }
    let mut failures = Vec::new();
    for failure in crate::support::bringup::deferred() {
        match offset(report, &failure) {
            Some(at) => failures.push((FileId(failure.file), at, failure.message)),
            None => attribution.unattributed.push(failure.message),
        }
    }
    match error.origin {
        Some((file, at)) => {
            if !failures.iter().any(|seen| seen.0 == file && seen.1 == at) {
                failures.push((file, at, error.message.clone()));
            }
        }
        None => attribution.unattributed.push(error.message.clone()),
    }
    for (file, at, message) in failures {
        let blocker = blocker(report, facts, file, at, message);
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
    facts: &Declarations,
    file: FileId,
    at: u32,
    message: String,
) -> Blocker {
    let (offset, name) = owner(report, facts, file, at).map_or((at, "<item>"), |found| {
        let name = report
            .resolution
            .definitions
            .get(found.0.definition.0 as usize)
            .map_or("?", |definition| report.package.names.get(definition.name));
        (found.1, name)
    });
    Blocker {
        path: path(&report.package, file),
        offset,
        name: name.to_owned(),
        message,
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
        let body = blocker(report, facts, FileId(failure.file), at, failure.message);
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
