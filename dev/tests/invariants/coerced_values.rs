//! Explicit nominal casts forward the real adjusted value and retain its owner.
use super::forward_nominal_cast;
use crate::{
    body::{
        self,
        callable_inputs::NominalUse,
        ir::{Body, Rvalue},
    },
    resolve::unit::{FileId, NodeKey},
    sema::model::{Declarations, Operation},
    syntax::ast::{Kind, NodeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    facts: Declarations,
    body: Body,
    row: NominalUse,
}
fn fixture() -> Result<Fixture> {
    let text = crate::test_source::fixture("pa31/tests/spec/coercion-casts/slice-once.rs")?;
    let report = crate::test_source::source_provider(&text, "coercion_cast_values")?;
    let crate::pipeline::Analysis { facts, prepared } = crate::pipeline::analyze_source(&report)?;
    let mut casts = Vec::new();
    for (file, nodes) in facts.nodes.iter().enumerate() {
        for (node, fact) in nodes.iter().enumerate() {
            let Some(Operation::CoercionCast { source, target }) = fact.operation else {
                continue;
            };
            let file = FileId(u32::try_from(file)?);
            let node = NodeId::new(u32::try_from(node)?);
            let Kind::Cast { value, .. } = report.package.unit(file).configured.ast.node(node).kind
            else {
                return Err("coercion selection belongs to another syntax kind".into());
            };
            casts.push((NodeKey { file, node: value }, source, target));
        }
    }
    assert_eq!(casts.len(), 1, "actual explicit nominal selection");
    let (source, from, target) = casts[0];
    let program = body::concrete_selected(
        &report,
        &facts,
        |owner| body::templates::owns(&facts, owner),
        prepared.concrete,
        body::Selection::Complete,
    )?;
    let mut selected = None;
    for body in program.bodies {
        let row = body
            .nominal_uses()
            .iter()
            .find(|row| row.source == source && (row.from, row.to) == (from, target))
            .copied();
        if let Some(row) = row {
            assert!(selected.is_none(), "nominal use duplicated across bodies");
            selected = Some((body, row));
        }
    }
    let (body, row) = selected.ok_or("actual explicit cast adjustment")?;
    Ok(Fixture { facts, body, row })
}

#[test]
fn nominal_cast_forwards_one_real_unsizing_and_its_retained_use() -> Result {
    let Fixture { facts, body, row } = fixture()?;
    assert_eq!(std::mem::size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(std::mem::size_of::<crate::types::TypeId>(), 4);
    assert_eq!(std::mem::size_of::<crate::types::Kind>(), 24);
    assert_eq!(std::mem::size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(
        forward_nominal_cast(&facts, &body, row.operation, row.source, row.from, row.to)?,
        row.operation
    );
    assert!(matches!(
        body.values[row.operation.0 as usize].kind,
        Rvalue::Unsize { .. }
    ));
    assert!(
        !body
            .values
            .iter()
            .any(|value| matches!(value.kind, Rvalue::Cast { .. }))
    );
    assert_eq!(body.nominal_uses().len(), 1);
    body.validate_nominal_inputs(&facts)?;
    Ok(())
}

#[test]
fn nominal_cast_rejects_missing_changed_scalar_and_foreign_claims() -> Result {
    let Fixture {
        mut facts,
        mut body,
        row,
    } = fixture()?;
    let foreign = fixture()?;
    assert_eq!(
        facts.types.kind(row.from),
        foreign.facts.types.kind(row.from)
    );
    let forward = |facts: &Declarations, body: &Body, source, target| {
        forward_nominal_cast(facts, body, row.operation, source, row.from, target)
    };
    assert!(forward(&foreign.facts, &body, row.source, row.to).is_err());
    assert!(forward(&facts, &body, row.source, row.from).is_err());
    assert!(
        forward(
            &facts,
            &body,
            body.source.source().ok_or("body source")?,
            row.to
        )
        .is_err()
    );
    let terminal = facts
        .unsizing_adjustments
        .remove(&row.source)
        .ok_or("selected terminal")?;
    assert!(forward(&facts, &body, row.source, row.to).is_err());
    facts.unsizing_adjustments.insert(row.source, terminal);
    let inputs = body.resolved_callables.take();
    assert!(forward(&facts, &body, row.source, row.to).is_err());
    body.resolved_callables = inputs;
    let original = body.values[row.operation.0 as usize].kind;
    body.values[row.operation.0 as usize].kind = Rvalue::Cast {
        kind: crate::sema::model::Cast::Pointer,
        operand: row.operand,
        direct_variant: false,
    };
    assert!(forward(&facts, &body, row.source, row.to).is_err());
    body.values[row.operation.0 as usize].kind = original;
    assert_eq!(forward(&facts, &body, row.source, row.to)?, row.operation);
    Ok(())
}
