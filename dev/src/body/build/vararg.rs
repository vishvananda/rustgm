//! Ellipsis storage consumes the checked parameter fact before ordinary bindings.
use super::{Builder, Error, Kind, LocalKind, NodeId, Operation};
impl Builder<'_> {
    pub(super) fn variadic_parameter(
        &mut self,
        parameter: NodeId,
        pattern: NodeId,
    ) -> Result<bool, Error> {
        let Kind::Parameter {
            ty: Some(annotation),
            ..
        } = self.kind(parameter)
        else {
            return Ok(false);
        };
        if !matches!(self.kind(annotation), Kind::Variadic) {
            return Ok(false);
        }
        let fact = self.fact(parameter)?;
        if !matches!(fact.operation, Some(Operation::Variadic))
            || !crate::body::vararg::variadic(&self.facts.types, &self.body)
            || fact.binding != self.fact(pattern)?.binding
        {
            return Err(self.error(
                parameter,
                "variadic local requires its actual checked parameter and binding",
            ));
        }
        let mutable = match self.kind(pattern) {
            Kind::Binding { mutable, .. } => mutable,
            Kind::Wildcard => false,
            _ => {
                return Err(self.error(
                    parameter,
                    "variadic binding pattern requires a wider source profile",
                ));
            }
        };
        let ty = self.ty(parameter)?;
        crate::sema::compiler_items::vararg::application(self.facts, ty)
            .map_err(|message| self.error(parameter, message))?;
        self.local(parameter, ty, LocalKind::Variadic, fact.binding, mutable)?;
        Ok(true)
    }
}
