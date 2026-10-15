//! Apply elaborated aliases/nominals with shared variables and ordered defaults.
use super::{Host, error, term};
use crate::{
    resolve::{model::DefKind, package::Error, unit::NodeKey},
    sema::generics::{AnnotationArgument as Written, parameter_id},
    types::{
        ItemId,
        arguments::Argument as Canonical,
        infer::{Instantiation, TermId, arguments::Argument},
        substitute::Bound,
    },
};

pub(in crate::sema) struct Fields {
    pub regions: Bound,
    pub parameters: Instantiation,
}

pub(super) fn infer(host: &mut impl Host, key: NodeKey) -> Result<TermId, Error> {
    let use_ = host
        .facts()
        .generics
        .annotation_uses
        .get(&key)
        .ok_or_else(|| error(host, key, "missing partial generic annotation"))?;
    let (owner, supplied) = (use_.owner, use_.arguments.clone());
    let parameters = host.facts().generics.parameters[&owner].clone();
    if host.facts().alias_requirements.contains(owner)
        || parameters.iter().zip(&supplied).any(|(parameter, source)| {
            matches!(source, Written::Default(_))
                && host.facts().alias_requirements.contains(parameter.binding)
        })
    {
        return Err(error(
            host,
            key,
            "inferred alias proofs require delayed projection normalization",
        ));
    }
    let mut regions = Vec::new();
    let mut arguments = Vec::with_capacity(parameters.len());
    let mut bindings = Vec::new();
    for (parameter, source) in parameters.iter().zip(&supplied) {
        if let Written::Lifetime(region) = *source {
            regions.push(region);
            arguments.push(Argument::Lifetime(region));
            continue;
        }
        let id = parameter_id(&host.facts().types, parameter.argument)
            .map_err(|e| error(host, key, e))?;
        let (facts, infer) = host.inference();
        let argument = infer
            .parameter_variable(&facts.types, id)
            .map_err(|e| error(host, key, e))?;
        match (*source, argument) {
            (Written::Type(source), Argument::Type(value)) => {
                let actual = term(host, source)?;
                let (facts, infer) = host.inference();
                infer
                    .equate(&facts.types, value, actual)
                    .map_err(|e| error(host, key, e))?;
            }
            (Written::Const(id), Argument::Const(value)) => {
                let (facts, infer) = host.inference();
                let known = infer
                    .known_const(&facts.types, id)
                    .map_err(|e| error(host, key, e))?;
                host.inference()
                    .1
                    .equate_consts(value, known)
                    .map_err(|e| error(host, key, e))?;
            }
            (Written::Infer | Written::Default(_), _) => {}
            _ => {
                return Err(error(
                    host,
                    key,
                    "partial annotation argument kind mismatch",
                ));
            }
        }
        arguments.push(argument);
        bindings.push((id, argument));
    }
    let (facts, infer) = host.inference();
    let mut fields = Fields {
        regions: Bound::new(regions),
        parameters: Instantiation::new(infer, &facts.types, &bindings)
            .map_err(|e| error(host, key, e))?,
    };
    for (index, source) in supplied.iter().enumerate() {
        if let Written::Default(default) = *source {
            fields.default(host, key, default, arguments[index])?;
        }
    }
    if host.report().resolution.definitions[owner.0 as usize].kind == DefKind::TypeAlias {
        let template = host.facts().definitions[owner.0 as usize]
            .ok_or_else(|| error(host, key, "missing alias template"))?;
        let template = fields
            .regions
            .apply(&mut host.inference().0.types, template)
            .map_err(|e| error(host, key, e))?;
        let (facts, infer) = host.inference();
        infer
            .instantiate_with(&mut facts.types, template, &mut fields.parameters)
            .map_err(|e| error(host, key, e))
    } else {
        let term = host
            .inference()
            .1
            .nominal(ItemId(owner.0), &arguments)
            .map_err(|e| error(host, key, e))?;
        host.nominal(term, fields);
        Ok(term)
    }
}

impl Fields {
    pub(in crate::sema) fn default(
        &mut self,
        host: &mut impl Host,
        key: NodeKey,
        default: Canonical,
        argument: Argument,
    ) -> Result<(), Error> {
        match (default, argument) {
            (Canonical::Type(ty), Argument::Type(term)) => {
                let ty = self
                    .regions
                    .apply(&mut host.inference().0.types, ty)
                    .map_err(|e| error(host, key, e))?;
                let (facts, infer) = host.inference();
                let expected = infer
                    .instantiate_with(&mut facts.types, ty, &mut self.parameters)
                    .map_err(|e| error(host, key, e))?;
                let (facts, infer) = host.inference();
                infer
                    .equate(&facts.types, term, expected)
                    .map_err(|e| error(host, key, e))
            }
            (Canonical::Const(id), Argument::Const(term)) => {
                let (facts, infer) = host.inference();
                let value = infer
                    .instantiate_const(&facts.types, id, &self.parameters)
                    .map_err(|e| error(host, key, e))?;
                host.inference()
                    .1
                    .equate_consts(term, value)
                    .map_err(|e| error(host, key, e))
            }
            _ => Err(error(host, key, "generic default argument kind mismatch")),
        }
    }
}
