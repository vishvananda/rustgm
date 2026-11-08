//! Function-pointer evidence preserves the real role and caller's current inference.
use super::{Action, Context, Decision, Declarations, Frame, Search, inference};
use crate::{
    support::{bindings::LangItem, constness::Constness},
    types::{
        Kind,
        infer::{self, Constructor},
    },
};
impl Search<'_> {
    pub(super) fn source_fn_pointer_goal(
        facts: &Declarations,
        goal: &super::Goal,
    ) -> Result<bool, String> {
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(false);
        };
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some(declaration) = catalog.language(LangItem::FnPtrTrait) else {
            return Ok(false);
        };
        if declaration.definition.0 != item.0 {
            return Ok(false);
        }
        catalog.validated_language(LangItem::FnPtrTrait)?;
        if !arguments.is_empty() {
            return Err("source FnPtr has unexpected trait arguments".into());
        }
        Ok(goal.constness() == Some(Constness::Ordinary))
    }
    /// rustc 1.98 decides builtin `FnPtr` evidence from the outer Self type
    /// alone. This is `TyKind::is_known_rigid` less function pointers: such a
    /// type has no `FnPtr` impl whatever its unresolved arguments. Variables,
    /// parameters, projections and opaque types are not rigid.
    pub(super) fn rigid_non_fn_pointer(
        facts: &Declarations,
        context: &mut Context,
        goal: &super::Goal,
    ) -> Result<bool, String> {
        match context.head(&facts.types, goal.subject()) {
            Ok(Some(
                Constructor::Primitive(_)
                | Constructor::Adt(..)
                | Constructor::AdtArgs(..)
                | Constructor::Never
                | Constructor::Tuple
                | Constructor::Array(_)
                | Constructor::ArrayTerm(_)
                | Constructor::ArrayInfer(_)
                | Constructor::Dynamic { .. }
                | Constructor::FunctionItem(..)
                | Constructor::FunctionItemArgs(..)
                | Constructor::Slice
                | Constructor::Reference { .. }
                | Constructor::Pointer { .. }
                | Constructor::Closure(..)
                | Constructor::Pattern(_)
                | Constructor::Foreign(_),
            )) => Ok(true),
            Ok(
                Some(
                    Constructor::Function { .. }
                    | Constructor::Parameter(_)
                    | Constructor::Projection(_)
                    | Constructor::Opaque(..)
                    | Constructor::Future(..),
                )
                | None,
            )
            | Err(infer::Error::Normalization) => Ok(false),
            Err(error) => Err(inference(error)),
        }
    }
    pub(super) fn source_fn_pointer_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if !Self::source_fn_pointer_goal(facts, &frame.goal)? {
            return Ok(None);
        }
        if frame.selected.is_some()
            || frame.deferred
            || frame.partial.is_some()
            || frame.goal.constness() != Some(Constness::Ordinary)
        {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                if !Self::rigid_non_fn_pointer(facts, &mut frame.inputs.context, &frame.goal)? {
                    return Ok(Some(Action::Complete(Decision::Deferred)));
                }
                // Coherence still asks whether a downstream crate could own
                // this Self type and so implement the trait for it.
                let decision = self.coherence_absence(facts, frame, Decision::Absent)?;
                return Ok(Some(Action::Complete(decision)));
            }
            Err(error) => return Err(inference(error)),
        };
        if !matches!(facts.types.kind(goal.ty), Some(Kind::Function { .. })) {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        frame.require_nominal_formed(facts, goal.ty)?;
        Ok(Some(self.complete_source_builtin(frame, goal, false)))
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/refuted_fn_pointers.rs"]
mod coherence_tests;
