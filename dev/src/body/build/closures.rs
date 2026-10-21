//! Closure CFGs use source body identities and the same ownership/loan pipeline as functions.
use super::{
    Body, Builder, Declarations, Error, Owner, Program, Report, Rvalue, Scratch, Type, TypeId,
    ValueId, ValueList, scopes,
};
use crate::{syntax::ast::NodeId, types::destruction::NeedsDrop};

pub(super) fn concrete(
    report: &Report,
    facts: &Declarations,
    scratch: &mut Scratch,
    program: &mut Program,
    mut query: Option<&mut NeedsDrop>,
) -> Result<(), Error> {
    for closure in facts.closures.all() {
        if closure
            .owner
            .is_some_and(|owner| facts.generics.has_parameters(owner))
        {
            return Err(Declarations::error(
                report,
                closure.source,
                "generic closure body construction requires a checked template",
            ));
        }
        let Some(Type::Function { result, .. }) = facts.types.kind(closure.signature) else {
            return Err(Declarations::error(
                report,
                closure.source,
                "missing closure signature",
            ));
        };
        let scope_plan = scopes::closure_with_scratch(
            report,
            facts,
            closure,
            std::mem::take(&mut scratch.scopes),
        )?;
        let body = Body::new(
            Owner::Closure {
                identity: closure.identity,
                signature: closure.signature,
            },
            closure.body,
            result,
        );
        let builder = Builder::new(
            report,
            facts,
            scratch,
            body,
            scope_plan,
            query.as_deref_mut(),
        );
        program.bodies.push(
            builder
                .run(Some(closure.parameters))
                .map_err(|e| report.diagnostic(e))?,
        );
    }
    Ok(())
}
impl Builder<'_> {
    pub(super) fn closure_value(&mut self, node: NodeId, ty: TypeId) -> Result<ValueId, Error> {
        let Some(Type::Closure {
            environment: TypeId::UNIT,
            ..
        }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(node, "closure environment construction is not implemented"));
        };
        let start = self.index(node, self.body.arguments.len())?;
        self.evaluate(
            node,
            ty,
            Rvalue::Aggregate {
                variant: None,
                fields: ValueList { start, end: start },
            },
        )
    }
}
