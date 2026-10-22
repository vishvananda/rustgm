//! Variadic arguments retain their actual types after ordinary inference fallback.
use super::{Checker, Error, Kind, NodeId, Primitive, Type};

impl Checker<'_> {
    pub(super) fn validate_variadic_arguments(
        &self,
        call: NodeId,
        fixed: usize,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        let Kind::Call { arguments, .. } = self.kind(call) else {
            return Err(self.error(call, "variadic call requires ordinary call syntax"));
        };
        let unit = self.report.package.unit(self.state.file);
        for &argument in unit
            .configured
            .ast
            .list(arguments)
            .iter()
            .filter(|node| self.active(**node))
            .skip(fixed)
        {
            let term = self.term(argument)?;
            let ty = solved.types[term.0 as usize];
            let message = match self.facts.types.kind(ty) {
                Some(Type::Primitive(
                    Primitive::Bool
                    | Primitive::I8
                    | Primitive::U8
                    | Primitive::I16
                    | Primitive::U16
                    | Primitive::F32,
                )) => "variadic argument requires an explicit C promotion cast",
                Some(Type::FunctionItem { .. }) => {
                    "variadic function item requires an explicit function pointer cast"
                }
                _ => continue,
            };
            return Err(self.error(argument, message));
        }
        Ok(())
    }
}
