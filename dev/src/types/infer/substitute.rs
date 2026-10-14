//! Instantiate typed templates into one body's graph without rechecking syntax.
use super::{
    Constructor, Context, Error, Store, TermId, TypeId, Value,
    arguments::{Argument, Slot},
};
use crate::types::{
    arguments::ConstKind,
    parameters::{ParameterId, ParameterKind},
};
use std::collections::HashMap;

impl Context {
    /// Replace rigid parameters simultaneously with type/const inference terms.
    /// Arguments must already live in the destination body context; references
    /// to the caller's universals are represented by free region identities.
    ///
    /// # Errors
    /// Rejects duplicate/mismatched bindings, invalid identities and work limits.
    pub fn instantiate(
        &mut self,
        store: &Store,
        template: TypeId,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<TermId, Error> {
        self.account_work(bindings.len())?;
        let mut parameters = HashMap::with_capacity(bindings.len());
        for &(id, argument) in bindings {
            let declaration = store.parameter(id).ok_or(Error::InvalidIdentity)?;
            match (declaration.kind, argument) {
                (ParameterKind::Type { .. }, Argument::Type(term)) => {
                    self.root(term)?;
                }
                (ParameterKind::Const { ty }, Argument::Const(term))
                    if self.constants().ty(term)? == ty => {}
                _ => return Err(Error::Mismatch),
            }
            if parameters.insert(id, argument).is_some() {
                return Err(Error::InvalidIdentity);
            }
        }
        let root = self.known(store, template)?;
        let mut completed = HashMap::new();
        let mut work = vec![(root, false)];
        let mut children = Vec::new();
        while let Some((term, finish)) = work.pop() {
            self.step()?;
            if completed.contains_key(&term) {
                continue;
            }
            if let Value::Known(ty) = self.terms[term.0 as usize].value
                && store.contains_parameters(ty) == Some(false)
            {
                completed.insert(term, term);
                continue;
            }
            let kind = self.materialize(store, term)?;
            if let Constructor::Parameter(id) = kind
                && let Some(Argument::Type(value)) = parameters.get(&id)
            {
                completed.insert(term, *value);
                continue;
            }
            let entry = self.terms[term.0 as usize];
            if !finish {
                work.push((term, true));
                self.account_work((entry.end - entry.start) as usize)?;
                work.extend(
                    self.edges[entry.start as usize..entry.end as usize]
                        .iter()
                        .rev()
                        .map(|id| (*id, false)),
                );
                continue;
            }
            children.clear();
            for child in &self.edges[entry.start as usize..entry.end as usize] {
                children.push(*completed.get(child).ok_or(Error::InvalidIdentity)?);
            }
            let result = self.replace_parameters(store, kind, &children, &parameters)?;
            completed.insert(term, result);
        }
        completed.get(&root).copied().ok_or(Error::InvalidIdentity)
    }
    fn replace_parameters(
        &mut self,
        store: &Store,
        kind: Constructor,
        children: &[TermId],
        bindings: &HashMap<ParameterId, Argument>,
    ) -> Result<TermId, Error> {
        if let Constructor::ArrayTerm(id) = kind
            && let Some(ConstKind::Parameter(parameter)) = store.const_kind(id)
            && let Some(Argument::Const(id)) = bindings.get(&parameter)
        {
            return self.construct(Constructor::ArrayInfer(*id), children);
        }
        let (item, args, function) = match kind {
            Constructor::AdtArgs(item, args) => (item, args, false),
            Constructor::FunctionItemArgs(item, args) => (item, args, true),
            _ => return self.construct(kind, children),
        };
        let mut arguments = Vec::new();
        let mut types = children[usize::from(function)..].iter();
        let count = self.slots(args)?.len();
        self.account_work(count)?;
        for index in 0..count {
            arguments.push(match self.slots(args)?[index] {
                Slot::Lifetime(region) => Argument::Lifetime(region),
                Slot::Type => Argument::Type(*types.next().ok_or(Error::InvalidIdentity)?),
                Slot::Const(id) => {
                    if let Some(ConstKind::Parameter(parameter)) = self.constants().value(id)?
                        && let Some(Argument::Const(value)) = bindings.get(&parameter)
                    {
                        Argument::Const(*value)
                    } else {
                        Argument::Const(id)
                    }
                }
            });
        }
        if function {
            self.function_item(
                item,
                *children.first().ok_or(Error::InvalidIdentity)?,
                &arguments,
            )
        } else {
            self.nominal(item, &arguments)
        }
    }
}
