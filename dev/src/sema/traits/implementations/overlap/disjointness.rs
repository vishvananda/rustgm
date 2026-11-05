//! Only a proven unsized storage tail excludes an existential Sized requirement.
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::{
        model::Declarations,
        nominals::sizes::Conditions,
        projections::{Cursor, Progress},
        traits::implementations::Availability,
    },
    types::{
        Primitive,
        infer::{self, Constructor, Context, TermId, arguments::Argument},
        lifetime_proof::{Obligations, ScopeId},
        sizedness::{Condition, Rule},
    },
};

#[derive(Default)]
struct Storage {
    conditions: Conditions,
    known: HashMap<TermId, bool>,
    active: HashSet<TermId>,
    path: Vec<TermId>,
    cursor: Option<Cursor>,
}

pub(super) fn has_unsized(
    facts: &mut Declarations,
    context: &mut Context,
    inputs: &super::inputs::Inputs,
) -> Result<bool, String> {
    let mut storage = Storage::default();
    let mut proof = Obligations::new(1);
    for &term in &inputs.sized {
        if !storage.demand(facts, context, term, &mut proof)? {
            continue;
        }
        // Header lifetime premises are admissible here; the selected
        // normal form is not authority to invent additional premises.
        inputs.prove(facts, context, &proof)?;
        return Ok(true);
    }
    Ok(false)
}

/// Only local/fundamental traits with orphan-closed inputs permit implicit
/// absence. Apply this at every ordinary trait child, not just the outer goal:
/// a matching conditional impl may depend on a downstream-local trait argument.
pub(in crate::sema) fn absence_closed(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &infer::predicates::Predicate,
) -> Result<bool, String> {
    let Some((item, _)) = goal.trait_arguments() else {
        return Ok(false);
    };
    let definition = crate::resolve::model::DefId(item.0);
    let local = facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
        .is_some_and(|owner| {
            owner.source.is_some()
                && matches!(owner.kind,
                    crate::sema::associated::OwnerKind::Trait { definition: id, .. }
                    if id == definition)
        });
    if !(local || crate::sema::header_attributes::fundamental_policy(facts, definition)?) {
        return Ok(false);
    }
    if closed_inputs(facts, context, goal)? {
        return Ok(true);
    }
    let predicate = match context.resolved_predicate(&mut facts.types, goal) {
        Ok(value) => value,
        Err(infer::Error::Unresolved | infer::Error::Normalization) => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    Ok(predicate.is_concrete(&facts.types)? && !predicate.contains_opaques(&facts.types)?)
}

/// A downstream crate cannot make a nonfundamental upstream nominal local by
/// choosing one of its arguments. Check only orphan-relevant roots; walking
/// every nested generic argument both loses this fact and repeats shared work.
fn closed_inputs(
    facts: &Declarations,
    context: &mut Context,
    goal: &infer::predicates::Predicate,
) -> Result<bool, String> {
    for mut term in goal.input_roots() {
        loop {
            context.account_work(1).map_err(|e| e.to_string())?;
            match context.head(&facts.types, term) {
                Ok(Some(Constructor::Reference { .. })) => {
                    term = context
                        .child(&facts.types, term, 0)
                        .map_err(|e| e.to_string())?;
                }
                Ok(Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _))) => {
                    if crate::sema::header_attributes::fundamental_policy(
                        facts,
                        crate::resolve::model::DefId(item.0),
                    )? {
                        return Ok(false);
                    }
                    break;
                }
                Ok(Some(
                    Constructor::Primitive(_)
                    | Constructor::Pointer { .. }
                    | Constructor::Tuple
                    | Constructor::Array(_)
                    | Constructor::ArrayTerm(_)
                    | Constructor::ArrayInfer(_)
                    | Constructor::Slice
                    | Constructor::Function { .. }
                    | Constructor::Never,
                )) => break,
                Ok(_) | Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                    return Ok(false);
                }
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    Ok(true)
}

impl Storage {
    fn demand(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        mut term: TermId,
        proof: &mut Obligations,
    ) -> Result<bool, String> {
        let answer = loop {
            context.account_work(1).map_err(|e| e.to_string())?;
            term = context.root(term).map_err(|e| e.to_string())?;
            if let Some(&answer) = self.known.get(&term) {
                break answer;
            }
            if !self.active.insert(term) {
                return Err("cyclic coherence storage condition".into());
            }
            self.path.push(term);
            let head = match context.head(&facts.types, term) {
                Ok(head) => head,
                Err(infer::Error::Normalization) => break false,
                Err(error) => return Err(error.to_string()),
            };
            match head {
                Some(Constructor::Slice | Constructor::Primitive(Primitive::Str)) => break true,
                Some(Constructor::Tuple) => {
                    let count = context
                        .arity(&facts.types, term)
                        .map_err(|e| e.to_string())?
                        .ok_or("missing coherence tuple arity")?;
                    let Some(last) = count.checked_sub(1) else {
                        break false;
                    };
                    term = context
                        .child(&facts.types, term, last)
                        .map_err(|e| e.to_string())?;
                }
                Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) => {
                    match facts.types.nominal_size_rule(item) {
                        Some(Rule::Unsized) => break true,
                        Some(Rule::Argument(slot)) => {
                            let arguments = context
                                .item_arguments(&facts.types, term)
                                .map_err(|e| e.to_string())?;
                            let Some(&Argument::Type(value)) = arguments.get(slot as usize) else {
                                return Err("invalid coherence storage argument".into());
                            };
                            term = value;
                        }
                        Some(Rule::Projection(_)) => {
                            let Some(next) = self.projected(facts, context, term, proof)? else {
                                break false;
                            };
                            term = next;
                        }
                        None | Some(Rule::Sized) => break false,
                    }
                }
                _ => break false,
            }
        };
        for term in self.path.drain(..) {
            self.active.remove(&term);
            self.known.insert(term, answer);
        }
        Ok(answer)
    }

    fn projected(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        term: TermId,
        proof: &mut Obligations,
    ) -> Result<Option<TermId>, String> {
        let ty = match context.resolved(&mut facts.types, term) {
            Ok(ty) => ty,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        let before = self.conditions.steps();
        let Condition::Type(tail) = self.conditions.apply(&mut facts.types, ty)? else {
            return Err("projected storage condition lost its tail".into());
        };
        context
            .account_work(self.conditions.steps() - before)
            .map_err(|e| e.to_string())?;
        if facts.types.contains_parameters(tail) != Some(false) {
            return Ok(None);
        }
        if self.cursor.is_none() {
            self.cursor = Some(Cursor::new(
                None,
                proof,
                ScopeId::ROOT,
                Availability::Declarations,
            )?);
        }
        let cursor = self.cursor.as_mut().ok_or("missing coherence normalizer")?;
        cursor.start(facts, tail)?;
        match cursor.advance(facts, proof, &mut |_, _| Ok(None))? {
            Progress::Complete { ty, steps } => {
                context.account_work(steps).map_err(|e| e.to_string())?;
                if ty == tail {
                    return Ok(None);
                }
                context
                    .known(&facts.types, ty)
                    .map(Some)
                    .map_err(|e| e.to_string())
            }
            Progress::Definition(_) | Progress::Dependency(_) => {
                Err("coherence normalization requires completed declarations".into())
            }
        }
    }
}
