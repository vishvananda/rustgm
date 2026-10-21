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
            continue;
        }
        program
            .bodies
            .push(body(report, facts, scratch, closure, query.as_deref_mut())?);
    }
    Ok(())
}

/// Source closure bodies share one constructor across symbolic and concrete proof.
pub(super) fn body(
    report: &Report,
    facts: &Declarations,
    scratch: &mut Scratch,
    closure: &crate::sema::closures::Body,
    query: Option<&mut NeedsDrop>,
) -> Result<Body, Error> {
    let Some(Type::Function { result, .. }) = facts.types.kind(closure.signature) else {
        return Err(Declarations::error(
            report,
            closure.source,
            "missing closure signature",
        ));
    };
    let scope_plan =
        scopes::closure_with_scratch(report, facts, closure, std::mem::take(&mut scratch.scopes))?;
    let body = Body::new(
        Owner::Closure {
            identity: closure.identity,
            signature: closure.signature,
            execution: None,
        },
        closure.semantic_source(),
        result,
    );
    Builder::new(report, facts, scratch, body, scope_plan, query)
        .run_root(closure.body.node, Some(closure.parameters))
        .map_err(|e| report.diagnostic(e))
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
