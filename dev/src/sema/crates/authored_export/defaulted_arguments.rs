//! Emit each explicit default from the canonical declaration parameter and binder.
use super::{Declarations, DefId, Stage, canonical_nodes::Context, g, s};
use crate::types::{
    TypeId,
    arguments::{Argument, ConstId, ConstKind, trees::Aggregate},
};

impl Stage {
    pub(super) fn nominal_parameter_defaults(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        context: Context,
    ) -> Result<(), &'static str> {
        self.parameter_defaults(facts, owner, None, context)
    }

    pub(super) fn trait_defaults(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        self_type: (TypeId, g::TypeRef),
        context: Context,
    ) -> Result<(), &'static str> {
        self.parameter_defaults(facts, owner, Some(self_type), context)
    }

    fn parameter_defaults(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        self_type: Option<(TypeId, g::TypeRef)>,
        context: Context,
    ) -> Result<(), &'static str> {
        for parameter in facts.generics.parameters.get(&owner).into_iter().flatten() {
            self.charge(1)?;
            let Some(default) = parameter.default else {
                continue;
            };
            let id = crate::sema::generics::parameter_id(&facts.types, parameter.argument)?;
            let slot = *self
                .graph
                .parameters
                .get(&id)
                .ok_or("source defaulted parameter has no reserved slot")?;
            match (parameter.argument, default) {
                (Argument::Type(_), Argument::Type(default)) => {
                    let value = if let Some((_, reference)) =
                        self_type.filter(|(ty, _)| *ty == default)
                    {
                        reference
                    } else if self.declaration_policies() {
                        self.ty(facts, default, context)?
                    } else {
                        return Err(
                            "source parameter default other than its trait's Self requires a wider consumer",
                        );
                    };
                    let row = self
                        .rows
                        .slots
                        .get_mut(slot.0 as usize)
                        .ok_or("source defaulted slot is outside its table")?;
                    if row.default.replace(value).is_some() {
                        return Err("source parameter default was recorded twice");
                    }
                }
                (Argument::Const(formal), Argument::Const(default))
                    if self.declaration_policies() =>
                {
                    if facts.types.const_type(formal) != facts.types.const_type(default) {
                        return Err("source const default changed its actual formal type");
                    }
                    let value = self.const_default(facts, default)?;
                    self.reserve(4 * size_of::<s::ConstDefault>())?;
                    self.cold()?
                        .const_defaults
                        .push(s::ConstDefault { slot, value });
                }
                _ => {
                    return Err(
                        "source generic default requires its actual parameter kind and profile",
                    );
                }
            }
        }
        Ok(())
    }

    fn const_default(
        &mut self,
        facts: &Declarations,
        constant: ConstId,
    ) -> Result<s::ConstDefaultValue, &'static str> {
        match facts.types.const_kind(constant) {
            Some(ConstKind::Value(value)) => Ok(s::ConstDefaultValue::Scalar(value.bits)),
            Some(ConstKind::Aggregate(aggregate)) => {
                let Some(Aggregate::Fields {
                    variant: None,
                    values,
                    ..
                }) = facts.types.aggregate_constant(aggregate)
                else {
                    return Err("source const default requires a closed record of scalar fields");
                };
                self.charge(values.len())?;
                self.reserve(values.len().saturating_mul(size_of::<u128>()))?;
                let mut fields = Vec::with_capacity(values.len());
                for &id in values {
                    let Some(ConstKind::Value(value)) = facts.types.const_kind(id) else {
                        return Err("source const default field requires its actual scalar value");
                    };
                    fields.push(value.bits);
                }
                Ok(s::ConstDefaultValue::Record(fields))
            }
            _ => Err("source const default requires its evaluated closed value"),
        }
    }

    pub(super) fn sort_const_defaults(&mut self) -> Result<(), &'static str> {
        let count = self.rows.cold().const_defaults.len();
        if count == 0 {
            return Ok(());
        }
        self.charge(count.saturating_mul(count.ilog2() as usize + 1))?;
        self.cold()?
            .const_defaults
            .sort_unstable_by_key(|row| row.slot.0);
        Ok(())
    }
}
