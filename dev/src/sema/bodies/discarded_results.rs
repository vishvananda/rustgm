//! Statement discard checks consume solved types and actual selected declarations.
use super::{Checker, Error, Kind, NodeId, Target, Type, TypeId};
use crate::{
    resolve::model::{DefId, DefKind},
    sema::{
        must_use::{
            WarningCode,
            discarded_types::{Requirement, TypeResult},
        },
        safety::Level,
    },
    syntax::ast::Unary,
};
impl Checker<'_> {
    pub(super) fn validate_discarded_result(
        &mut self,
        statement: NodeId,
        value: NodeId,
    ) -> Result<(), Error> {
        let key = self.key(statement);
        if self.facts.safety_policies.must_use(key) == Level::Allow {
            return Ok(());
        }
        let value = self.discarded_operand(value)?;
        if matches!(self.kind(value), Kind::Return(_)) {
            return Ok(());
        }
        let ty = self
            .facts
            .node(self.key(value))
            .and_then(|fact| fact.ty)
            .ok_or_else(|| self.error(value, "must_use discard requires its solved source type"))?;
        let mut issue = self.discarded_type(statement, value, ty)?;
        if let Some(definition) = self.discarded_callable(value)? {
            self.discarded_requirement(
                statement,
                value,
                Requirement::Declaration(definition),
                issue,
            )?;
            issue += 1;
        }
        if issue == 0 {
            let message = match self.kind(value) {
                Kind::Binary { .. } => Some("unused operation that must be used"),
                Kind::Unary {
                    op: Unary::Borrow | Unary::BorrowMut,
                    ..
                } => Some("unused borrow that must be used"),
                Kind::Unary {
                    op: Unary::Negate | Unary::Not | Unary::Dereference,
                    ..
                } => Some("unused unary operation that must be used"),
                _ => None,
            };
            if let Some(message) = message {
                self.discarded_warning(statement, value, 0, message.to_owned())?;
            }
        }
        if let Some(table) = self.facts.must_use_contracts.as_deref_mut() {
            table.work.statements += 1;
        }
        Ok(())
    }
    fn discarded_type(
        &mut self,
        statement: NodeId,
        value: NodeId,
        ty: TypeId,
    ) -> Result<u32, Error> {
        if matches!(
            self.facts.must_use_type(self.report, ty)?,
            TypeResult::Absent
        ) {
            return Ok(0);
        }
        let mut work = vec![ty];
        let mut issues = 0_u32;
        let mut steps = 0;
        while let Some(ty) = work.pop() {
            steps += 1;
            if steps > 1_000_000 {
                return Err(self.error(value, "must_use diagnostic output budget exceeded"));
            }
            match self.facts.must_use_type(self.report, ty)? {
                TypeResult::Absent => {}
                TypeResult::Direct(requirement) => {
                    self.discarded_requirement(statement, value, requirement, issues)?;
                    issues += 1;
                }
                TypeResult::Components => match self.facts.types.kind(ty) {
                    Some(Type::Tuple(list)) => work.extend(
                        self.facts
                            .types
                            .list(list)
                            .ok_or_else(|| self.error(value, "invalid discarded tuple type"))?
                            .iter()
                            .rev()
                            .copied(),
                    ),
                    Some(Type::Array { element, .. }) => work.push(element),
                    _ => return Err(self.error(value, "must_use component summary changed type")),
                },
            }
        }
        Ok(issues)
    }
    fn discarded_requirement(
        &mut self,
        statement: NodeId,
        value: NodeId,
        requirement: Requirement,
        issue: u32,
    ) -> Result<(), Error> {
        let message = match requirement {
            Requirement::Declaration(definition) => {
                let contract = self
                    .facts
                    .must_use_contract(self.report, definition)?
                    .ok_or_else(|| self.error(value, "must_use result lost its actual contract"))?;
                contract.reason.map_or_else(
                    || "unused must_use result".to_owned(),
                    |reason| format!("unused must_use result: {reason}"),
                )
            }
            Requirement::Closure => "unused closure that must be used".to_owned(),
        };
        self.discarded_warning(statement, value, issue, message)
    }
    fn discarded_warning(
        &mut self,
        statement: NodeId,
        value: NodeId,
        issue: u32,
        message: String,
    ) -> Result<(), Error> {
        let key = self.key(statement);
        let offset = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .node(value)
            .span
            .start;
        self.facts.must_use_diagnostic(
            self.report,
            key,
            offset,
            issue,
            WarningCode::UnusedMustUse,
            message,
        )
    }
    fn discarded_operand(&self, mut node: NodeId) -> Result<NodeId, Error> {
        for _ in 0..1_000_000 {
            match self.kind(node) {
                Kind::Group(inner)
                | Kind::Attributed { value: inner, .. }
                | Kind::Block {
                    tail: Some(inner), ..
                }
                | Kind::Expansion {
                    tail: Some(inner), ..
                } => node = inner,
                _ => return Ok(node),
            }
        }
        Err(self.error(node, "must_use operand ancestry budget exceeded"))
    }
    fn discarded_callable(&self, value: NodeId) -> Result<Option<DefId>, Error> {
        let definition = match self.kind(value) {
            Kind::Call { callee, .. } => self.discarded_function_path(callee)?,
            Kind::MethodCall { .. } | Kind::GenericMethodCall { .. } => self
                .facts
                .methods
                .calls
                .get(&self.key(value))
                .map(|call| call.method),
            _ => None,
        };
        let Some(definition) = definition else {
            return Ok(None);
        };
        if self
            .report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .is_none_or(|actual| actual.kind != DefKind::Function)
        {
            return Ok(None);
        }
        Ok(self
            .facts
            .must_use_contract(self.report, definition)?
            .map(|_| definition))
    }
    fn discarded_function_path(&self, mut callee: NodeId) -> Result<Option<DefId>, Error> {
        for _ in 0..1_000_000 {
            match self.kind(callee) {
                Kind::Group(inner) | Kind::Applied { base: inner, .. } => callee = inner,
                Kind::Name(..) | Kind::Path { .. } | Kind::Qualified { .. } => {
                    return Ok(self
                        .facts
                        .node(self.key(callee))
                        .and_then(|fact| fact.reference)
                        .and_then(|target| match target {
                            Target::Definition(definition) => Some(definition),
                            _ => None,
                        }));
                }
                _ => return Ok(None),
            }
        }
        Err(self.error(callee, "must_use callee ancestry budget exceeded"))
    }
}
