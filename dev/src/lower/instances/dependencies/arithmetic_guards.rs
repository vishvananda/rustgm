//! Demand source arithmetic panic bodies and their actual caller locations.
use super::{Body, Declarations, Dependencies, Instance, Kind, Rvalue};
use crate::{
    body::ir::{Action, Owner},
    lower::arithmetic::{ArithmeticCheck, negation},
    resolve::expansion::Report,
    sema::compiler_items::arithmetic_guards,
    support::bindings::LangItem,
    syntax::ast::Unary,
};

impl Dependencies {
    pub(super) fn arithmetic_guards(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        body: &Body,
        calls: &mut Vec<Instance>,
    ) -> Result<(), String> {
        if report.package.source_core().is_none() || body.source.source().is_none() {
            return Ok(());
        }
        self.charge(body.statements.len())?;
        let mut forwards = None;
        let mut targets = Vec::new();
        for statement in &body.statements {
            let Action::Evaluate(id) = statement.action else {
                continue;
            };
            let value = body
                .values
                .get(id.0 as usize)
                .ok_or("arithmetic guard lost its typed value")?;
            for role in roles(facts, body, value.kind).into_iter().flatten() {
                let Some(route) = arithmetic_guards::prepare(report, facts, role)? else {
                    continue;
                };
                if !targets.contains(&route.instance) {
                    targets.push(route.instance);
                }
                if !route.tracked {
                    continue;
                }
                let forwards_caller = if let Some(value) = forwards {
                    value
                } else {
                    let value = matches!(body.owner, Owner::Function { definition, .. }
                        if facts.recorded_caller_tracking(definition)?);
                    forwards = Some(value);
                    value
                };
                if !forwards_caller {
                    let site = body.syntax_site(statement.source)?;
                    arithmetic_guards::prepare_location(report, facts, site)?;
                }
            }
        }
        calls.extend(targets);
        Ok(())
    }
}

fn roles(facts: &Declarations, body: &Body, operation: Rvalue) -> [Option<LangItem>; 2] {
    let primitive = |id: crate::body::ir::ValueId| match body
        .values
        .get(id.0 as usize)
        .and_then(|value| facts.types.kind(value.ty))
    {
        Some(Kind::Primitive(primitive)) => Some(primitive),
        _ => None,
    };
    match operation {
        Rvalue::Unary {
            operator: Unary::Negate,
            operand,
            literal,
        } => [
            primitive(operand)
                .and_then(|kind| negation(kind, literal))
                .map(|_| LangItem::PanicNegOverflow),
            None,
        ],
        Rvalue::Binary { operator, left, .. } => primitive(left)
            .and_then(|kind| ArithmeticCheck::binary(operator, kind))
            .map_or([None; 2], ArithmeticCheck::roles),
        _ => [None; 2],
    }
}
