//! Initializers retain declaration proof ownership and a separate executable root.
use super::{Body, Builder, Declarations, Error, Owner, Program, Report, Scratch, scopes};
use crate::support::hash::HashMap;
use crate::{body::ir::BodyKey, sema::model::Initializer, types::destruction::NeedsDrop};
pub(super) fn concrete(
    report: &Report,
    facts: &Declarations,
    scratch: &mut Scratch,
    program: &mut Program,
    prepared: &mut HashMap<BodyKey, Body>,
    mut query: Option<&mut NeedsDrop>,
) -> Result<(), Error> {
    for initializer in &facts.initializers {
        if facts.associated.owner(initializer.definition).is_some()
            || facts.generics.has_parameters(initializer.definition)
        {
            continue;
        }
        if let Some(body) = prepared.remove(&BodyKey::Declaration(initializer.definition)) {
            if body.source != initializer.semantic_source()
                || body.result != initializer.ty
                || !matches!(body.owner,Owner::Initializer{definition,..} if definition==initializer.definition)
            {
                return Err(Declarations::error(
                    report,
                    initializer.value,
                    "retained initializer identity or result changed",
                ));
            }
            program.bodies.push(body);
        } else {
            program.bodies.push(self::body(
                report,
                facts,
                scratch,
                initializer,
                query.as_deref_mut(),
            )?);
        }
    }
    Ok(())
}
pub(super) fn body(
    report: &Report,
    facts: &Declarations,
    scratch: &mut Scratch,
    initializer: &Initializer,
    query: Option<&mut crate::types::destruction::NeedsDrop>,
) -> Result<Body, Error> {
    let body = Body::new(
        Owner::Initializer {
            definition: initializer.definition,
            kind: initializer.kind,
        },
        initializer.semantic_source(),
        initializer.ty,
    );
    let scope_plan = scopes::initializer_with_scratch(
        report,
        facts,
        initializer,
        std::mem::take(&mut scratch.scopes),
    )?;
    let builder = Builder::new(report, facts, scratch, body, scope_plan, query);
    builder
        .run_root(initializer.value.node, None)
        .map_err(|e| report.diagnostic(e))
}
