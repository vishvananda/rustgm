//! One typed argument environment serves body fields and source trait children.
use super::Fields;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        ItemId, Kind,
        arguments::{Argument as Canonical, ConstKind},
        infer::{Context, Error, Instantiation, arguments::Argument},
        substitute::Bound,
    },
};
#[derive(Debug)]
pub(in crate::sema) enum FieldError {
    Inference(Error),
    Declaration(&'static str),
}
impl From<Error> for FieldError {
    fn from(error: Error) -> Self {
        Self::Inference(error)
    }
}
impl From<&'static str> for FieldError {
    fn from(error: &'static str) -> Self {
        Self::Declaration(error)
    }
}
impl std::fmt::Display for FieldError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inference(error) => error.fmt(out),
            Self::Declaration(error) => out.write_str(error),
        }
    }
}
impl std::error::Error for FieldError {}

impl Fields {
    pub(in crate::sema) fn for_arguments(
        facts: &Declarations,
        context: &mut Context,
        owner: ItemId,
        arguments: Vec<Argument>,
    ) -> Result<Self, FieldError> {
        let Some(parameters) = facts.generics.parameters.get(&DefId(owner.0)) else {
            let count = facts
                .lifetimes
                .parameters
                .get(&DefId(owner.0))
                .map_or(0, Vec::len);
            if arguments.len() != count {
                return Err("invalid nominal lifetime argument count".into());
            }
            let regions = arguments
                .into_iter()
                .map(|argument| match argument {
                    Argument::Lifetime(region) => Ok(region),
                    _ => Err(FieldError::Declaration(
                        "nominal lifetime argument kind mismatch",
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Self {
                regions: Bound::new(regions),
                parameters: Instantiation::new(context, &facts.types, &[])?,
            });
        };
        if parameters.len() != arguments.len() {
            return Err("invalid nominal argument count".into());
        }
        let mut regions = Vec::new();
        let mut bindings = Vec::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            let id = match (parameter.argument, argument) {
                (Canonical::Lifetime(_), Argument::Lifetime(region)) => {
                    regions.push(region);
                    continue;
                }
                (Canonical::Type(ty), Argument::Type(_)) => {
                    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                        return Err("missing nominal type parameter".into());
                    };
                    id
                }
                (Canonical::Const(id), Argument::Const(_)) => {
                    let Some(ConstKind::Parameter(id)) = facts.types.const_kind(id) else {
                        return Err("missing nominal const parameter".into());
                    };
                    id
                }
                _ => return Err("nominal argument kind mismatch".into()),
            };
            bindings.push((id, argument));
        }
        Ok(Self {
            regions: Bound::new(regions),
            parameters: Instantiation::new(context, &facts.types, &bindings)?,
        })
    }
}
