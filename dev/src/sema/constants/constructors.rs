//! Constant construction consumes typed constructor and field selections.
use super::{
    Declarations, DefId, Error, Kind, NodeKey, Operation, Queries, Query, Report, Type, Value,
    VariantId, products,
};
use crate::syntax::ast::List;

#[derive(Clone, Copy)]
pub(super) enum Expression {
    Positional {
        variant: VariantId,
        callee: NodeKey,
        arguments: List,
    },
    Record {
        variant: VariantId,
        fields: List,
        base: Option<NodeKey>,
    },
}
impl Expression {
    pub(super) fn dependencies(
        self,
        report: &Report,
        key: NodeKey,
        mut visit: impl FnMut(NodeKey),
    ) {
        let fields = match self {
            Self::Positional { arguments, .. } => arguments,
            Self::Record { fields, base, .. } => {
                if let Some(base) = base {
                    visit(base);
                }
                fields
            }
        };
        products::Expression::Fields(fields).dependencies(report, key, &mut visit);
        if let Self::Positional { callee, .. } = self {
            visit(callee);
        }
    }
}
impl Queries<'_> {
    pub(super) fn constructor_expression(&self, key: NodeKey) -> Result<Option<Expression>, Error> {
        let child = |node| NodeKey { node, ..key };
        Ok(match Declarations::kind(self.report, key) {
            Kind::Struct { fields, base, .. } => {
                let Some(Operation::Construct { variant }) =
                    self.typing.facts.node(key).and_then(|f| f.operation)
                else {
                    return Err(
                        self.error(Query::Node(key), "missing constant constructor selection")
                    );
                };
                Some(Expression::Record {
                    variant,
                    fields,
                    base: base.map(child),
                })
            }
            Kind::Call { callee, arguments } => {
                let Some(Type::FunctionItem { item, .. }) = self
                    .typing
                    .facts
                    .node(child(callee))
                    .and_then(|f| f.ty)
                    .and_then(|ty| self.typing.facts.types.kind(ty))
                else {
                    return Ok(None);
                };
                self.typing
                    .facts
                    .members
                    .constructors
                    .get(&DefId(item.0))
                    .copied()
                    .map(|variant| Expression::Positional {
                        variant,
                        callee: child(callee),
                        arguments,
                    })
            }
            _ => None,
        })
    }
    pub(super) fn constructor_value(
        &mut self,
        key: NodeKey,
        expression: Expression,
    ) -> Result<Value, Error> {
        let (variant, fields, base, positional) = match expression {
            Expression::Positional {
                variant,
                callee,
                arguments,
            } => {
                if !matches!(self.value(Query::Node(callee))?, Value::Constructor(selected) if selected == variant)
                {
                    return Err(self.error(
                        Query::Node(key),
                        "constant callee differs from its selected constructor",
                    ));
                }
                (variant, arguments, None, true)
            }
            Expression::Record {
                variant,
                fields,
                base,
            } => (variant, fields, base, false),
        };
        let member = self.typing.facts.members.variants[variant.0 as usize];
        let mut values = vec![None; (member.end - member.start) as usize];
        let unit = self.report.package.unit(key.file);
        let mut ordinal = 0;
        for &node in unit.configured.ast.list(fields) {
            if !unit.configured.active[node.0 as usize] {
                continue;
            }
            self.budget()?;
            let value_key = NodeKey { node, ..key };
            let index = if positional {
                ordinal
            } else {
                self.constructor_field(value_key, variant)?
            };
            let value = self.value(Query::Node(value_key))?;
            let slot = values.get_mut(index).ok_or_else(|| {
                self.error(Query::Node(key), "constant constructor field overflow")
            })?;
            if slot.replace(value).is_some() {
                return Err(self.error(Query::Node(key), "duplicate constant constructor field"));
            }
            ordinal += 1;
        }
        let base = base.map(|base| self.value(Query::Node(base))).transpose()?;
        let mut complete = Vec::with_capacity(values.len());
        for (index, value) in values.into_iter().enumerate() {
            self.budget()?;
            complete.push(if let Some(value) = value {
                value
            } else {
                let Some(Value::Product(base)) = base else {
                    return Err(self.error(Query::Node(key), "missing constant update field"));
                };
                if self
                    .typing
                    .facts
                    .constant_products
                    .variant(base)
                    .map_err(|e| self.error(Query::Node(key), e))?
                    != Some(variant)
                {
                    return Err(self.error(
                        Query::Node(key),
                        "constant update base has a different constructor",
                    ));
                }
                self.typing
                    .facts
                    .constant_products
                    .get(base, index as u64)
                    .map_err(|e| self.error(Query::Node(key), e))?
            });
        }
        if complete.is_empty() {
            return Ok(Value::Variant(variant));
        }
        self.typing
            .facts
            .constant_products
            .fields(&complete, Some(variant))
            .map_err(|e| self.error(Query::Node(key), e))
    }
    fn constructor_field(&self, mut key: NodeKey, variant: VariantId) -> Result<usize, Error> {
        while let Kind::Attributed { value, .. } = Declarations::kind(self.report, key) {
            key.node = value;
        }
        let Some(Operation::FieldInit { field }) =
            self.typing.facts.node(key).and_then(|f| f.operation)
        else {
            return Err(self.error(Query::Node(key), "missing constant field selection"));
        };
        if self.typing.facts.members.variant(field) != Some(variant) {
            return Err(self.error(
                Query::Node(key),
                "constant field belongs to a different constructor",
            ));
        }
        self.typing
            .facts
            .members
            .ordinal(field)
            .map(|index| index as usize)
            .ok_or_else(|| self.error(Query::Node(key), "missing constant field ordinal"))
    }
}
