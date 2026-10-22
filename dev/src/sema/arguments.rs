//! Selected generic uses share typed constant evaluation and written-argument validation.
use crate::{
    resolve::{
        expansion::Report,
        package::{Error, ModuleId},
        unit::NodeKey,
    },
    sema::{constants, generics::captures::Captures, model::Declarations},
    syntax::ast::Kind,
    types::{
        TypeId,
        arguments::{ConstId, Constant},
    },
};

pub(super) fn groups(
    report: &Report,
    key: NodeKey,
    enum_variant: bool,
) -> Result<
    (
        Option<crate::syntax::ast::List>,
        Option<crate::syntax::ast::List>,
    ),
    Error,
> {
    use crate::syntax::ast::List;
    let Some(list) = super::lifetimes::arguments(report, key, enum_variant)? else {
        return Ok((None, None));
    };
    let ast = &report.package.unit(key.file).configured.ast;
    let lifetimes = ast.list(list).iter().take_while(|&&node| {
        matches!(ast.node(node).kind, Kind::Name(span) if report.package.unit(key.file).spelling(span).is_ok_and(|s| s.starts_with('\'')))
    }).count();
    let middle = list
        .start
        .checked_add(
            u32::try_from(lifetimes)
                .map_err(|_| Declarations::error(report, key, "too many lifetime arguments"))?,
        )
        .ok_or_else(|| Declarations::error(report, key, "argument range overflow"))?;
    Ok((
        (middle != list.start).then_some(List {
            start: list.start,
            end: middle,
        }),
        (middle != list.end).then_some(List {
            start: middle,
            end: list.end,
        }),
    ))
}

pub(super) fn arity(
    parameters: &[super::generics::Parameter],
    count: usize,
    pending: impl Fn(crate::resolve::model::DefId) -> bool,
) -> Result<usize, &'static str> {
    let explicit = parameters
        .iter()
        .position(|p| p.anonymous)
        .unwrap_or(parameters.len());
    let parameters = &parameters[..explicit];
    let offset = parameters
        .iter()
        .take_while(|p| matches!(p.argument, crate::types::arguments::Argument::Lifetime(_)))
        .count();
    if count != 0
        && (count + offset > parameters.len()
            || parameters
                .iter()
                .skip(count + offset)
                .any(|p| p.default.is_none() && !pending(p.binding)))
    {
        return Err("incorrect number of type and const arguments");
    }
    Ok(offset)
}

pub(super) fn written(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
) -> super::generics::DeferredArgument {
    use super::generics::DeferredArgument;
    use crate::resolve::model::{Namespace, Target};
    let source = key;
    if matches!(Declarations::kind(report, source), Kind::InferType) {
        return DeferredArgument::Infer;
    }
    let value_path = facts.node(source).and_then(|f| f.reference).is_some_and(|target| {
        matches!(target, Target::Definition(id) if report.resolution.definitions[id.0 as usize].namespace == Namespace::Value)
    });
    if value_path
        || matches!(
            Declarations::kind(report, source),
            Kind::Literal(_) | Kind::Unary { .. } | Kind::Block { .. }
        )
    {
        DeferredArgument::Const(key)
    } else {
        DeferredArgument::Type(key)
    }
}

pub(super) struct Constants<'a> {
    pub report: &'a Report,
    pub facts: &'a mut Declarations,
    pub captures: &'a mut Captures,
    pub scratch: &'a mut constants::Scratch,
    pub discriminants: &'a mut constants::Discriminants,
}
#[derive(Default)]
pub(super) struct Cache {
    pub captures: Captures,
    pub scratch: constants::Scratch,
    pub discriminants: constants::Discriminants,
}
pub(super) enum Input {
    Parameter(ConstId),
    Expression(constants::ArgumentInput),
}
impl Constants<'_> {
    pub(super) fn argument(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        module: Option<ModuleId>,
    ) -> Result<ConstId, Error> {
        let input = match self.input(key, ty, module)? {
            Input::Parameter(parameter) => return Ok(parameter),
            Input::Expression(input) => input,
        };
        let value = crate::sema::constants::generic_argument(
            self.report,
            self.facts,
            self.scratch,
            self.discriminants,
            input.key,
            input.ty,
            input.module,
        )?;
        self.facts
            .types
            .intern_constant(Constant { ty, bits: value })
            .map_err(|e| Declarations::error(self.report, key, e))
    }

    /// Preserve standalone symbolic parameters; otherwise qualify the anonymous input once.
    pub(super) fn input(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        module: Option<ModuleId>,
    ) -> Result<Input, Error> {
        if let Some(id) = self.facts.const_parameter(self.report, key) {
            if self.facts.types.const_type(id) != Some(ty) {
                return Err(Declarations::error(
                    self.report,
                    key,
                    "const argument has the wrong type",
                ));
            }
            return Ok(Input::Parameter(id));
        }
        let module = module.ok_or_else(|| {
            Declarations::error(self.report, key, "missing const argument module")
        })?;
        self.captures.check(self.report, self.facts, key, module)?;
        Ok(Input::Expression(constants::ArgumentInput {
            key,
            ty,
            module,
        }))
    }
}
