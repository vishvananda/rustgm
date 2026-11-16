//! Isolation of failed bodies: an analysis-only census records each one, and
//! the bring-up profile sets each aside as a trapping stub. Ordinary checking
//! keeps its first error.
use super::{Checker, Declarations, Error, Function, Scratch, State};
use crate::{
    resolve::{expansion::Report, model::DefId, unit::NodeKey},
    sema::census::{self, Census, ItemKind, Phase},
    support::bringup::{self, Owner, Position},
};

/// What a function body that reached a failed body records as its own failure.
pub(super) const BLOCKED: &str =
    "body requires the hidden type of a function whose body did not type";

/// Where a failed body goes instead of ending the crate's checking.
pub(super) enum Isolation<'a> {
    /// An analysis-only census records it.
    Census(&'a mut Census),
    /// The bring-up profile replaces it by a trapping stub.
    Stubs(&'a mut SetAside),
}
impl Isolation<'_> {
    pub(super) fn progress_item(
        &mut self,
        report: &Report,
        definition: DefId,
        body: Option<NodeKey>,
    ) -> Result<(), Error> {
        match self {
            Self::Census(census) => {
                census.progress_item(report, ItemKind::Function, definition, body)
            }
            Self::Stubs(_) => Ok(()),
        }
    }
    pub(super) const fn checked(&mut self, count: usize) {
        if let Self::Census(census) = self {
            census.checked(Phase::Bodies, count);
        }
    }
}

/// The bring-up profile's function bodies that failed to type. Each function
/// keeps its checked signature, so its callers still type against it, and
/// lowering replaces its body by a trapping stub once the skip list names it.
/// Nothing typed inside a failed body is completed: its closures, inline
/// constants, compile-time definition and hidden types never become facts, so
/// whatever needs them fails in turn and is named for itself.
#[derive(Default)]
pub(super) struct SetAside {
    /// The functions set aside, in the order met; a blocked body's cause
    /// indexes it.
    functions: Vec<DefId>,
    /// The last failure of a function that the skip list does not name yet.
    unlisted: Option<Error>,
}
impl SetAside {
    /// Bodies are set aside only in the bring-up profile.
    pub(super) fn selected() -> Option<Self> {
        bringup::selected().map(|_| Self::default())
    }
    /// The failure that ends the typing stage once every body was checked:
    /// the last one of a function that the skip list does not name yet.
    pub(super) fn finish(self) -> Option<Error> {
        self.unlisted
    }
}

impl Checker<'_> {
    /// Isolate a failed body, keep it unusable and discard its partial state.
    /// Budget exhaustion stays sticky: the unchanged error ends checking.
    pub(super) fn isolate_body(
        &mut self,
        isolation: &mut Isolation<'_>,
        function: Function,
        error: Error,
    ) -> Result<usize, Error> {
        if census::exhausts_budget(&error.message) {
            return Err(error);
        }
        let cause = match isolation {
            Isolation::Census(census) => self.census_failure(census, function.definition, error),
            Isolation::Stubs(set_aside) => self.set_aside(set_aside, function, error),
        };
        self.facts
            .typed_function_bodies
            .remove(&function.definition);
        self.discard_body();
        Ok(cause)
    }
    fn census_failure(&mut self, census: &mut Census, owner: DefId, error: Error) -> usize {
        let error = self.report.diagnostic(error);
        if let Some(cause) = census.repeated(&error) {
            self.block_census(census, owner, cause);
            cause
        } else {
            census.failed(Phase::Bodies, census::describe(self.report, owner), &error)
        }
    }
    /// A body that reached a failed body is isolated in turn and never completed.
    pub(super) fn block_body(
        &mut self,
        isolation: &mut Isolation<'_>,
        function: Function,
        cause: usize,
    ) {
        match isolation {
            Isolation::Census(census) => self.block_census(census, function.definition, cause),
            Isolation::Stubs(set_aside) => {
                let error = Declarations::error(self.report, function.node, BLOCKED);
                self.set_aside(set_aside, function, error);
                self.facts
                    .typed_function_bodies
                    .remove(&function.definition);
            }
        }
    }
    fn block_census(&mut self, census: &mut Census, owner: DefId, cause: usize) {
        census.blocked(
            Phase::Bodies,
            census::describe(self.report, owner),
            census::site(self.report, owner),
            cause,
        );
        self.facts.typed_function_bodies.remove(&owner);
    }
    /// Name a failed function for the bring-up skip list unless the list
    /// already replaces it. The failure keeps where it was raised; one without
    /// a position is placed at the function.
    fn set_aside(&self, set_aside: &mut SetAside, function: Function, error: Error) -> usize {
        let error = if error.origin.is_some() {
            error
        } else {
            Declarations::error(self.report, function.node, error.message.clone())
        };
        let owner = Owner {
            file: function.node.file.0,
            node: function.node.node.get(),
            definition: function.definition.0,
        };
        if !self.report.package.bringup_listed(function.node)
            && let Some((file, at)) = error.origin
            && bringup::defer_owned(file.0, Position::Byte(at), &error.message, owner)
        {
            set_aside.unlisted = Some(error);
        }
        set_aside.functions.push(function.definition);
        set_aside.functions.len() - 1
    }
    /// Replace every per-body field; a failed body's slots are never reused.
    pub(super) fn discard_body(&mut self) {
        let mut scratch = Scratch::default();
        scratch.terms.files(self.facts.nodes.len());
        self.state = State::fresh(scratch);
    }
    pub(super) fn census_initializer(
        &mut self,
        index: usize,
        mut census: Option<&mut Census>,
    ) -> Result<(), Error> {
        if let Some(census) = census.as_deref_mut() {
            let initializer = self.facts.initializers[index];
            if !matches!(
                initializer.typing,
                crate::sema::model::InitializerTyping::Complete
            ) {
                census.progress_item(
                    self.report,
                    ItemKind::Initializer,
                    initializer.definition,
                    Some(initializer.value),
                )?;
            }
        }
        let result = self.initializer(index);
        let Some(census) = census else {
            return result;
        };
        match result {
            Ok(()) => {
                census.checked(Phase::Bodies, 1);
                Ok(())
            }
            Err(error) => {
                if census::exhausts_budget(&error.message) {
                    return Err(error);
                }
                let owner = self.facts.initializers[index].definition;
                self.census_failure(census, owner, error);
                self.facts.typed_function_bodies.remove(&owner);
                self.discard_body();
                Ok(())
            }
        }
    }
}
