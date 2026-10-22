//! Argument-local callable expectations use only the already checked prefix.
#[cfg(test)]
#[path = "../../../tests/invariants/prefix_arguments.rs"]
mod tests;
use super::{Checker, Constructor, Dependency, Error, NodeId, TermId, Work};
use crate::support::hash::{HashMap, HashSet};
use crate::syntax::ast::{Kind, Unary};
use crate::types::Store;
use crate::types::infer::{Context, Error as InferError};

#[derive(Default)]
pub(super) struct Inputs {
    data: Option<Box<Data>>,
}
#[derive(Default)]
struct Data {
    generics: usize,
    obligations: usize,
    methods: usize,
    origins: HashMap<TermId, NodeId>,
    clauses: HashMap<NodeId, Vec<usize>>,
    method_roots: HashMap<NodeId, TermId>,
    seeded: HashMap<NodeId, usize>,
    adjustments: usize,
    freshenings: HashSet<usize>,
}
impl Inputs {
    pub(super) fn freshening(&mut self, index: usize) {
        self.data
            .get_or_insert_with(Default::default)
            .freshenings
            .insert(index);
    }
    pub(super) fn owner(
        &mut self,
        context: &mut Context,
        store: &Store,
        calls: &super::calls::CallTypes,
        adjustments: &[super::PendingAdjustment],
        callable: Option<TermId>,
        site: NodeId,
    ) -> Result<Option<(NodeId, Vec<usize>)>, InferError> {
        if calls.obligations.is_empty() {
            return Ok(None);
        }
        let data = self.data.get_or_insert_with(Default::default);
        data.populate(context, store, calls)?;
        for (index, adjustment) in adjustments.iter().enumerate().skip(data.adjustments) {
            context.account_work(1)?;
            if data.freshenings.remove(&index) {
                data.forward(context, store, adjustment)?;
            }
        }
        data.adjustments = adjustments.len();
        let mut root = if let Some(callable) = callable {
            context.root(callable)?
        } else if let Some(&root) = data.method_roots.get(&site) {
            root
        } else {
            return Ok(None);
        };
        loop {
            context.account_work(1)?;
            if matches!(
                context.head(store, root)?,
                Some(Constructor::Reference { .. })
            ) {
                let child = context.child(store, root, 0)?;
                root = context.root(child)?;
            } else {
                break;
            }
        }
        let Some(&source) = data.origins.get(&root) else {
            return Ok(None);
        };
        let Some(clauses) = data.clauses.get(&source) else {
            return Ok(None);
        };
        let cursor = data.seeded.entry(source).or_default();
        let new = clauses
            .get(*cursor..)
            .ok_or(InferError::InvalidIdentity)?
            .to_vec();
        *cursor = clauses.len();
        context.account_work(new.len())?;
        Ok(Some((source, new)))
    }
}

impl Data {
    fn populate(
        &mut self,
        context: &mut Context,
        store: &Store,
        calls: &super::calls::CallTypes,
    ) -> Result<(), InferError> {
        for &(source, definition, term) in &calls.generic[self.generics..] {
            context.account_work(1)?;
            let root = context.root(term)?;
            if !matches!(context.head(store, root)?, Some(Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _)) if item.0 == definition.0)
            {
                return Err(InferError::InvalidIdentity);
            }
            self.origins.insert(root, source);
        }
        self.generics = calls.generic.len();
        for (index, (source, _)) in calls.obligations.iter().enumerate().skip(self.obligations) {
            context.account_work(1)?;
            self.clauses.entry(*source).or_default().push(index);
        }
        self.obligations = calls.obligations.len();
        for (source, pending) in &calls.methods[self.methods..] {
            context.account_work(1)?;
            self.method_roots
                .insert(*source, context.root(pending.callable)?);
        }
        self.methods = calls.methods.len();
        Ok(())
    }
    fn forward(
        &mut self,
        context: &mut Context,
        store: &Store,
        adjustment: &super::PendingAdjustment,
    ) -> Result<(), InferError> {
        use crate::types::infer::arguments::Argument;
        if adjustment.kind != super::AdjustmentKind::Equality || adjustment.reset {
            return Ok(());
        }
        let from = context.root(adjustment.actual)?;
        let Some(&source) = self.origins.get(&from) else {
            return Ok(());
        };
        let to = context.root(adjustment.expected)?;
        let item = |head| match head {
            Some(Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _)) => {
                Some(item)
            }
            _ => None,
        };
        if item(context.head(store, from)?).is_none()
            || item(context.head(store, from)?) != item(context.head(store, to)?)
        {
            return Ok(());
        }
        let left = context.item_arguments(store, from)?;
        let right = context.item_arguments(store, to)?;
        if left.len() != right.len() {
            return Ok(());
        }
        context.account_work(left.len())?;
        for (a, b) in left.into_iter().zip(right) {
            let same = match (a, b) {
                (Argument::Type(a), Argument::Type(b)) => context.root(a)? == context.root(b)?,
                (Argument::Const(a), Argument::Const(b)) => a == b,
                (Argument::Lifetime(_), Argument::Lifetime(_)) => true,
                _ => false,
            };
            if !same {
                return Ok(());
            }
        }
        self.origins.insert(to, source);
        Ok(())
    }
}

// Only reference prefixes may supply this early contextual expectation.
// Projections keep their owning normalizer and the final coercion remains required.
fn reference_frontier(
    context: &mut Context,
    store: &Store,
    mut term: TermId,
) -> Result<Option<TermId>, InferError> {
    let mut borrowed = false;
    loop {
        context.account_work(1)?;
        match context.head(store, term) {
            Ok(Some(Constructor::Reference { .. })) => {
                borrowed = true;
                term = context.child(store, term, 0)?;
            }
            Ok(None) if borrowed => return Ok(Some(term)),
            Ok(_) | Err(InferError::Normalization | InferError::Unresolved) => return Ok(None),
            Err(error) => return Err(error),
        }
    }
}

impl Checker<'_> {
    fn call_parameter(&mut self, call: NodeId, parameter: u32) -> Result<TermId, Error> {
        let signature = *self
            .state
            .call_types
            .active
            .get(&call)
            .ok_or_else(|| self.error(call, "missing argument call signature"))?;
        let Some(Constructor::Function { parameters, .. }) = self.head(call, signature)? else {
            return Err(self.error(call, "call argument requires a function signature"));
        };
        if parameter >= parameters {
            return Err(self.error(call, "invalid call argument position"));
        }
        self.child(call, signature, parameter as usize)
    }
    pub(super) fn prepare_call_argument(
        &mut self,
        call: NodeId,
        argument: NodeId,
        parameter: u32,
    ) -> Result<Option<Dependency>, Error> {
        let expected = self.call_parameter(call, parameter)?;
        if let Some(frontier) =
            reference_frontier(&mut self.state.infer, &self.facts.types, expected)
                .map_err(|e| self.error(call, e))?
            && let Some(dependency) = self.refine_call_inputs(call, frontier)?
        {
            return Ok(Some(dependency));
        }
        self.state.work.push(Work::Enter(argument, Some(expected)));
        Ok(None)
    }
    pub(super) fn coerce_call_argument(
        &mut self,
        call: NodeId,
        argument: NodeId,
        parameter: u32,
    ) -> Result<Option<Dependency>, Error> {
        let expected = self.call_parameter(call, parameter)?;
        let actual = self.term(argument)?;
        if matches!(
            self.contextual_head(argument, actual)?,
            Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
        ) && self.state.infer.head(&self.facts.types, expected) == Ok(None)
            && let Some(dependency) = self.refine_call_inputs(call, expected)?
        {
            return Ok(Some(dependency));
        }
        let exclusive = matches!(
            self.contextual_head(call, expected)?,
            Some(Constructor::Reference { mutable: true, .. })
        );
        let mark = self.state.adjustments.len();
        self.coerce(argument, expected)?;
        let mut site = argument;
        while let Kind::Group(value) | Kind::Attributed { value, .. } = self.kind(site) {
            site = value;
        }
        for index in mark..self.state.adjustments.len() {
            if self.state.adjustments[index].node == site
                && !matches!(
                    self.kind(site),
                    Kind::Unary {
                        op: Unary::BorrowMut,
                        ..
                    }
                )
                && let Some((adjustment, _)) = self.state.reference_adjustments.get_mut(&index)
            {
                adjustment.two_phase = exclusive && adjustment.dereferences == 1;
            }
        }
        Ok(None)
    }
}
