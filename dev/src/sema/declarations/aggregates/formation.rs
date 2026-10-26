//! Prepare immutable nominal requirements through type edges, never field templates.
use super::{Checker, Error, FileId, State, Type, TypeId, Work};
use crate::types::{GenericArgs, arguments::Argument};

impl Checker<'_> {
    pub(in super::super) fn constant_type_contract(
        &mut self,
        ty: TypeId,
        finish: bool,
    ) -> Result<(), Error> {
        let error = |message| Error::at(FileId(0), 0, message);
        let state = self
            .nominals
            .contracts
            .get(&ty)
            .copied()
            .unwrap_or(State::Unseen);
        if state == State::Complete {
            return Ok(());
        }
        if finish {
            self.nominals.contracts.insert(ty, State::Complete);
            return Ok(());
        }
        if state == State::Active {
            return Err(error("cycle in constant type requirements"));
        }
        self.nominals.contracts.insert(ty, State::Active);
        self.work.push(Work::ValueContract(ty, true));
        match self
            .facts
            .types
            .kind(ty)
            .ok_or_else(|| error("missing constant type"))?
        {
            Type::Adt(item, arguments) => {
                self.work.push(Work::NominalContract(item, false));
                self.constant_argument_contracts(arguments)?;
            }
            Type::Projection(application) => {
                for argument in self
                    .facts
                    .types
                    .projection_inputs(application)
                    .map_err(error)?
                {
                    if let Argument::Type(ty) = argument {
                        self.work.push(Work::ValueContract(ty, false));
                    }
                }
            }
            Type::Closure {
                arguments,
                signature,
                environment,
                ..
            } => {
                self.work.extend([
                    Work::ValueContract(signature, false),
                    Work::ValueContract(environment, false),
                ]);
                self.constant_argument_contracts(arguments)?;
            }
            Type::Tuple(list) => {
                let fields = self
                    .facts
                    .types
                    .list(list)
                    .ok_or_else(|| error("missing constant tuple types"))?;
                self.work.extend(
                    fields
                        .iter()
                        .rev()
                        .map(|&ty| Work::ValueContract(ty, false)),
                );
            }
            Type::Array { element, .. }
            | Type::ArrayTerm { element, .. }
            | Type::Slice(element) => {
                self.work.push(Work::ValueContract(element, false));
            }
            Type::Reference {
                referent: pointee, ..
            }
            | Type::Pattern { base: pointee, .. }
            | Type::Pointer { pointee, .. } => {
                self.work.push(Work::ValueContract(pointee, false));
            }
            Type::Function {
                parameters, result, ..
            } => {
                self.work.push(Work::ValueContract(result, false));
                let parameters = self
                    .facts
                    .types
                    .list(parameters)
                    .ok_or_else(|| error("missing constant callable types"))?;
                self.work.extend(
                    parameters
                        .iter()
                        .rev()
                        .map(|&ty| Work::ValueContract(ty, false)),
                );
            }
            Type::FunctionItem { arguments, .. }
            | Type::Opaque(_, arguments)
            | Type::Dynamic { arguments, .. } => {
                self.constant_argument_contracts(arguments)?;
            }
            Type::Never | Type::Primitive(_) | Type::Parameter(_) => {}
        }
        Ok(())
    }

    fn constant_argument_contracts(&mut self, arguments: GenericArgs) -> Result<(), Error> {
        for argument in self
            .facts
            .types
            .arguments(arguments)
            .ok_or_else(|| Error::at(FileId(0), 0, "missing constant type arguments"))?
        {
            if let Argument::Type(ty) = argument {
                self.work.push(Work::ValueContract(ty, false));
            }
        }
        Ok(())
    }
}
