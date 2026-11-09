//! One caller retains its size proof frontier across missing field declarations.
use super::{Declarations, Environment, ItemId, Kind, TypeId, assumed, prepare};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    Primitive,
    parameters::environment::BuiltinQuery,
    sizedness::{Condition, Rule},
};
use std::sync::Arc;

pub(in crate::sema) enum Progress {
    Ready { sized: bool, provisional: bool },
    Fields,
}
pub(in crate::sema) enum Step {
    Ready { sized: bool, provisional: bool },
    Fields(ItemId),
    Projection(TypeId),
}
#[derive(Clone, Copy)]
struct Answer {
    sized: bool,
    provisional: bool,
}
pub(in crate::sema) struct Query {
    environment: Option<Arc<Environment>>,
    permissions: Option<Box<BuiltinQuery>>,
    answers: HashMap<TypeId, Answer>,
    temporary: Vec<TypeId>,
    tails: HashMap<TypeId, TypeId>,
    conditions: super::Conditions,
    active: HashSet<TypeId>,
    root: Option<TypeId>,
    projection: Option<TypeId>,
    work: Vec<(TypeId, bool)>,
    placeholders: Option<crate::types::parameters::shape::Placeholders>,
    pub steps: usize,
}
impl Query {
    /// A negative coherence query's size proof under the other header's
    /// Sized premises, if it has them; see [`Self::placeholder_begin`].
    pub fn with_placeholders(
        environment: Option<Arc<Environment>>,
        placeholders: Option<crate::types::parameters::shape::Placeholders>,
    ) -> Self {
        let mut query = Self::new(environment);
        query.placeholders = placeholders;
        query
    }
    pub fn new(environment: Option<Arc<Environment>>) -> Self {
        Self {
            environment,
            permissions: None,
            answers: HashMap::default(),
            temporary: Vec::new(),
            tails: HashMap::default(),
            conditions: super::Conditions::default(),
            active: HashSet::default(),
            root: None,
            projection: None,
            work: Vec::new(),
            placeholders: None,
            steps: 0,
        }
    }
    pub fn demand(
        &mut self,
        facts: &mut Declarations,
        root: TypeId,
        mut normalize: impl FnMut(&mut Declarations, TypeId) -> Result<TypeId, &'static str>,
    ) -> Result<Progress, &'static str> {
        loop {
            match self.advance(facts, root)? {
                Step::Ready { sized, provisional } => {
                    return Ok(Progress::Ready { sized, provisional });
                }
                Step::Fields(_) => return Ok(Progress::Fields),
                Step::Projection(ty) => {
                    let normalized = normalize(facts, ty)?;
                    self.projection = None;
                    if normalized == ty {
                        self.known(ty, facts.types.is_sized(ty) == Some(true));
                    } else {
                        self.tail(ty, normalized)?;
                    }
                }
            }
        }
    }
    /// A trait selector can prove a hidden projection on its existing stack,
    /// retaining that child's evidence instead of launching a separate proof.
    pub fn advance(
        &mut self,
        facts: &mut Declarations,
        root: TypeId,
    ) -> Result<Step, &'static str> {
        if let Some(previous) = self.root {
            if previous != root {
                return Err("unfinished nominal size query changed its subject");
            }
        } else {
            self.root = Some(root);
            self.work.push((root, false));
        }
        if let Some(ty) = self.projection {
            return Ok(Step::Projection(ty));
        }
        while let Some((ty, finish)) = self.work.pop() {
            self.steps += 1;
            if self.steps.saturating_add(self.work.len()) > 1_000_000 {
                return Err("nominal size query work budget exceeded");
            }
            if self.answers.contains_key(&ty) {
                continue;
            }
            if finish {
                let answer = self.finish(facts, ty)?;
                self.save(ty, answer);
            } else if let Some(item) = self.begin(facts, ty)? {
                self.work.push((ty, false));
                return Ok(Step::Fields(item));
            }
            if let Some(ty) = self.projection {
                return Ok(Step::Projection(ty));
            }
        }
        let answer = *self
            .answers
            .get(&root)
            .ok_or("missing nominal size result")?;
        self.root = None;
        for ty in self.temporary.drain(..) {
            self.answers.remove(&ty);
        }
        Ok(Step::Ready {
            sized: answer.sized,
            provisional: answer.provisional,
        })
    }
    pub fn answer_projection(
        &mut self,
        ty: TypeId,
        sized: bool,
        provisional: bool,
    ) -> Result<(), &'static str> {
        if self.projection != Some(ty) {
            return Err("nominal size answer does not match its pending projection");
        }
        self.projection = None;
        self.save(ty, Answer { sized, provisional });
        Ok(())
    }
    fn save(&mut self, ty: TypeId, answer: Answer) {
        self.active.remove(&ty);
        if answer.provisional {
            self.temporary.push(ty);
        }
        self.answers.insert(ty, answer);
    }
    fn known(&mut self, ty: TypeId, sized: bool) {
        self.save(
            ty,
            Answer {
                sized,
                provisional: false,
            },
        );
    }
    fn tail(&mut self, ty: TypeId, tail: TypeId) -> Result<(), &'static str> {
        if !self.active.insert(ty) {
            return Err("cycle in nominal size normalization");
        }
        self.tails.insert(ty, tail);
        self.work.push((ty, true));
        self.work.push((tail, false));
        Ok(())
    }
    fn begin(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<Option<ItemId>, &'static str> {
        if let Some(environment) = &self.environment
            && assumed(&facts.types, ty, environment, &mut self.permissions)?
        {
            self.known(ty, true);
            return Ok(None);
        }
        if self.placeholder_begin(facts, ty)? {
            return Ok(None);
        }
        match facts.types.kind(ty).ok_or("invalid nominal size subject")? {
            Kind::Adt(item, arguments) => {
                if facts.types.nominal_sizes_sealed() {
                    self.fence(facts.types.nominal_size_rule(item))?;
                    let before = self.conditions.steps();
                    let condition = self.conditions.apply(&mut facts.types, ty)?;
                    self.steps += self.conditions.steps() - before;
                    self.condition(ty, condition)?;
                    return Ok(None);
                }
                if let Some(missing) = prepare(facts, item)? {
                    let pending = facts
                        .generics
                        .size_rules
                        .as_ref()
                        .ok_or("missing size rules")?;
                    if pending.active.contains(&missing) {
                        // Positive recursive field formation is checked again
                        // after declarations finish. Never retain it as a fact.
                        self.save(
                            ty,
                            Answer {
                                sized: true,
                                provisional: true,
                            },
                        );
                        return Ok(None);
                    }
                    return Ok(Some(missing));
                }
                let pending = facts
                    .generics
                    .size_rules
                    .as_mut()
                    .ok_or("missing size rules")?;
                self.fence(pending.rules.get(item))?;
                let before = pending.rules.steps;
                let condition = pending.rules.apply(&mut facts.types, item, arguments)?;
                self.steps += pending.rules.steps - before;
                self.condition(ty, condition)?;
            }
            Kind::Tuple(list) => {
                if !self.active.insert(ty) {
                    return Err("cycle in nominal size tuple");
                }
                self.work.push((ty, true));
                for &child in facts
                    .types
                    .list(list)
                    .ok_or("invalid size tuple")?
                    .iter()
                    .rev()
                {
                    if self.steps.saturating_add(self.work.len()) >= 1_000_000 {
                        return Err("nominal size pending work budget exceeded");
                    }
                    self.work.push((child, false));
                }
            }
            Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } => {
                self.tail(ty, element)?;
            }
            Kind::Projection(_) | Kind::Opaque(..) => {
                self.projection = Some(ty);
            }
            Kind::Parameter(_) => self.known(ty, facts.types.is_sized(ty) == Some(true)),
            Kind::Slice(_) | Kind::Primitive(Primitive::Str) => self.known(ty, false),
            _ => self.known(ty, true),
        }
        Ok(None)
    }
    /// rustc 1.98's `sized_conditions` under the other header's premises: a
    /// premise is Sized, a rigid parameter, trait object or foreign type only
    /// as a premise, an array always, a tuple as its last element and a
    /// pattern as its base. A nominal, slice, `str` or projection keeps its
    /// rule below, a projection descending into the same negative query. No
    /// parameter declaration or interned property is read.
    fn placeholder_begin(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
    ) -> Result<bool, &'static str> {
        let Some(placeholders) = &self.placeholders else {
            return Ok(false);
        };
        let (member, work) = placeholders.sized(ty);
        self.steps += work;
        if member {
            self.known(ty, true);
            return Ok(true);
        }
        match facts.types.kind(ty).ok_or("invalid nominal size subject")? {
            Kind::Parameter(_) | Kind::Dynamic { .. } | Kind::Foreign(_) => self.known(ty, false),
            Kind::Array { .. } | Kind::ArrayTerm { .. } => self.known(ty, true),
            Kind::Pattern { base, .. } => self.tail(ty, base)?,
            Kind::Tuple(list) => match facts.types.list(list).ok_or("invalid size tuple")?.last() {
                Some(&element) => self.tail(ty, element)?,
                None => self.known(ty, true),
            },
            // Every other kind keeps its premise-less rule: a nominal by its
            // sizedness rule, a projection or opaque in the same negative
            // query, a slice or `str` never and the builtin Sized types always.
            // The match lists every kind, so a new one must choose.
            Kind::Adt(..)
            | Kind::Projection(_)
            | Kind::Opaque(..)
            | Kind::Slice(_)
            | Kind::Never
            | Kind::Primitive(_)
            | Kind::Future { .. }
            | Kind::Reference { .. }
            | Kind::Pointer { .. }
            | Kind::Function { .. }
            | Kind::FunctionItem { .. }
            | Kind::Closure { .. } => return Ok(false),
        }
        Ok(true)
    }
    /// Under placeholders a projected-size nominal's size needs a nested
    /// proof with header premises, so it is the named fence, as in the shape
    /// walk of the negative builtin proof.
    fn fence(&self, rule: Option<Rule>) -> Result<(), &'static str> {
        if self.placeholders.is_some() && matches!(rule, Some(Rule::Projection(_))) {
            return Err(crate::sema::nominals::PLACEHOLDER_SIZE_FENCE);
        }
        Ok(())
    }
    fn condition(&mut self, ty: TypeId, condition: Condition) -> Result<(), &'static str> {
        match condition {
            Condition::Sized => self.known(ty, true),
            Condition::Unsized => self.known(ty, false),
            Condition::Type(tail) => self.tail(ty, tail)?,
        }
        Ok(())
    }
    fn finish(&self, facts: &Declarations, ty: TypeId) -> Result<Answer, &'static str> {
        if let Some(tail) = self.tails.get(&ty) {
            return self
                .answers
                .get(tail)
                .copied()
                .ok_or("missing size tail result");
        }
        let Some(Kind::Tuple(list)) = facts.types.kind(ty) else {
            return Err("missing nominal size dependency");
        };
        let mut answer = Answer {
            sized: true,
            provisional: false,
        };
        for child in facts.types.list(list).ok_or("invalid size tuple")? {
            let child = self
                .answers
                .get(child)
                .ok_or("missing size tuple element")?;
            answer.sized &= child.sized;
            answer.provisional |= child.provisional;
        }
        Ok(answer)
    }
}
