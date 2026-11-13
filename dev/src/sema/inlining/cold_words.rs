//! Stable body advisories neither authorize execution nor create backend promises.
use super::{Attribute, Declarations, Error, Input, ItemKind, Kind, NodeKey, Report, WarningCode};
use crate::{
    resolve::{
        bindings::{AttributePositions, FeatureName},
        expansion::MacroAllowanceSession,
    },
    syntax::ast::{BlockFlavor, NodeId, Syntax},
};

#[derive(Default)]
pub(in crate::sema) struct ColdState<'unit> {
    seen: bool,
    has_prototypes: bool,
    positions: Option<Box<AttributePositions<'unit>>>,
}
impl ColdState<'_> {
    pub(in crate::sema) fn has_prototypes(&self) -> bool {
        self.has_prototypes
    }
    pub(in crate::sema) fn begin_owner(&mut self) {
        self.seen = false;
    }
}

fn body_target<L>(ast: &Syntax<L>, node: NodeId) -> Result<bool, &'static str> {
    let node = match ast.node(node).kind {
        Kind::Statement { value, .. } => value,
        _ => node,
    };
    Ok(match ast.node(node).kind {
        Kind::Item(index) => matches!(ast.items[index as usize].kind, ItemKind::Function { .. }),
        Kind::Closure { .. }
        | Kind::Block {
            flavor: BlockFlavor::Async | BlockFlavor::AsyncMove,
            ..
        } => true,
        Kind::ConstClosure { closure, gate } => {
            let site = ast
                .expression_gate(gate)
                .ok_or("cold lost its actual const closure gate")?;
            if site.node != node || site.validate(ast, gate).map_err(|e| e.message)? != closure {
                return Err("cold const closure differs from its actual gated child");
            }
            true
        }
        _ => false,
    })
}

pub(in crate::sema) fn consume_cold<'unit>(
    report: &'unit Report,
    facts: &mut Declarations,
    (owner, declaration): (NodeKey, NodeKey),
    attribute: Attribute,
    allowances: &mut MacroAllowanceSession<'_>,
    state: &mut ColdState<'unit>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("cold requires a plain word attribute"));
    }
    facts.require_lint_context(report)?;
    let unit = report.package.unit(declaration.file);
    let ast = &unit.configured.ast;
    if !matches!(ast.node(declaration.node).kind, Kind::Item(_))
        && !report
            .package
            .requested_expression_attributes()
            .map_err(error)?
        && !allowances.allows_at(
            declaration.file,
            attribute.span,
            &FeatureName::new("stmt_expr_attributes").map_err(error)?,
        )?
    {
        if state
            .positions
            .as_ref()
            .is_none_or(|positions| !positions.is_for(unit))
        {
            // The borrowed Unit remains alive throughout this transient pass.
            state.positions = None;
            crate::sema::header_attributes::reserve_source_contracts(
                facts,
                1,
                size_of::<AttributePositions<'_>>(),
            )?;
            let positions = AttributePositions::prepare(unit, declaration.file, |work, bytes| {
                crate::sema::header_attributes::reserve_source_contracts(facts, work, bytes)
            })?;
            state.positions = Some(Box::new(positions));
        }
        let positions = state
            .positions
            .as_ref()
            .ok_or_else(|| error("cold lost its configured attribute positions"))?;
        if positions
            .requires_expression_feature(unit, owner.node)
            .map_err(error)?
        {
            return Err(error(
                "source expression attributes require stmt_expr_attributes",
            ));
        }
    }
    if !state.seen
        && matches!(ast.node(declaration.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Function { body: None, .. }))
    {
        crate::sema::header_attributes::capture_cold_prototype(report, facts, owner, declaration)?;
        state.has_prototypes = true;
    }
    let target = body_target(ast, declaration.node).map_err(error)?;
    let message = if state.seen {
        Some("duplicate cold attribute")
    } else if !target {
        Some("cold attribute has no effect outside a function-like body")
    } else {
        None
    };
    state.seen = true;
    if let Some(message) = message {
        facts.must_use_diagnostic(
            report,
            owner,
            attribute.span.start,
            0,
            WarningCode::UnusedAttributes,
            message.into(),
        )?;
    }
    Ok(())
}
