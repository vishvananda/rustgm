//! Reification keeps callable identity distinct from a zero-sized closure value.
use super::{Constant, Evaluator, Failure, Reader, Type, TypeId, View};
use crate::types::instance::Instance;

impl Evaluator<'_> {
    pub(crate) fn reify(&self, source: TypeId, value: Constant) -> Result<Constant, Failure> {
        match self.facts().types.kind(source) {
            Some(Type::FunctionItem { .. } | Type::Function { .. }) => {
                value
                    .as_function()
                    .ok_or("constant function conversion lost its selected callable")?;
                Ok(value)
            }
            Some(Type::Closure {
                environment: TypeId::UNIT,
                ..
            }) => {
                if !matches!(
                    Reader::new(self.facts()).view(value)?,
                    View::Product {
                        length: 0,
                        variant: None,
                    }
                ) {
                    return Err("constant closure conversion requires an empty environment".into());
                }
                Ok(Constant::function(Instance::closure(
                    &self.facts().types,
                    source,
                )?))
            }
            _ => Err("constant function conversion requires a noncapturing callable".into()),
        }
    }
}
