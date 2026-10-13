//! Demanded product values share flat storage; repeats retain one element.
use super::{Declarations, Error, Kind, NodeKey, Queries, Query, Type, Value};
use crate::{resolve::expansion::Report, syntax::ast::List};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::sema) struct Id(u32);
#[derive(Clone, Copy)]
enum Contents {
    Fields {
        start: u32,
        end: u32,
        variant: Option<super::VariantId>,
    },
    Repeat {
        value: Value,
        length: u64,
    },
}
#[derive(Default)]
pub(in crate::sema) struct Products {
    nodes: Vec<Contents>,
    fields: Vec<Value>,
}
impl Products {
    pub(super) fn repeated(&self, id: Id) -> Result<Option<(Value, u64)>, &'static str> {
        match self
            .nodes
            .get(id.0 as usize)
            .ok_or("missing constant product")?
        {
            Contents::Repeat { value, length } => Ok(Some((*value, *length))),
            Contents::Fields { .. } => Ok(None),
        }
    }
    fn insert(&mut self, contents: Contents) -> Result<Value, &'static str> {
        let id = Id(u32::try_from(self.nodes.len()).map_err(|_| "too many constant products")?);
        self.nodes.push(contents);
        Ok(Value::Product(id))
    }
    pub(super) fn fields(
        &mut self,
        fields: &[Value],
        variant: Option<super::VariantId>,
    ) -> Result<Value, &'static str> {
        self.sequence(fields.iter().copied(), variant)
    }
    pub(super) fn sequence(
        &mut self,
        fields: impl ExactSizeIterator<Item = Value>,
        variant: Option<super::VariantId>,
    ) -> Result<Value, &'static str> {
        let start = u32::try_from(self.fields.len()).map_err(|_| "too many constant fields")?;
        let end = u32::try_from(fields.len())
            .ok()
            .and_then(|n| start.checked_add(n))
            .ok_or("too many constant fields")?;
        self.fields.extend(fields);
        self.insert(Contents::Fields {
            start,
            end,
            variant,
        })
    }
    pub(super) fn repeat(&mut self, value: Value, length: u64) -> Result<Value, &'static str> {
        self.insert(Contents::Repeat { value, length })
    }
    pub(super) fn length(&self, id: Id) -> Result<u64, &'static str> {
        match self
            .nodes
            .get(id.0 as usize)
            .ok_or("missing constant product")?
        {
            Contents::Fields { start, end, .. } => Ok(u64::from(end - start)),
            Contents::Repeat { length, .. } => Ok(*length),
        }
    }
    pub(super) fn get(&self, id: Id, index: u64) -> Result<Value, &'static str> {
        match *self
            .nodes
            .get(id.0 as usize)
            .ok_or("missing constant product")?
        {
            Contents::Fields { start, end, .. } if index < u64::from(end - start) => {
                let index = usize::try_from(u64::from(start) + index)
                    .map_err(|_| "constant field index overflow")?;
                self.fields
                    .get(index)
                    .copied()
                    .ok_or("missing constant field")
            }
            Contents::Repeat { value, length } if index < length => Ok(value),
            _ => Err("constant index is out of bounds"),
        }
    }
    pub(super) fn variant(&self, id: Id) -> Result<Option<super::VariantId>, &'static str> {
        match self
            .nodes
            .get(id.0 as usize)
            .ok_or("missing constant product")?
        {
            Contents::Fields { variant, .. } => Ok(*variant),
            Contents::Repeat { .. } => Ok(None),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Expression {
    Fields(List),
    Repeat { value: NodeKey, length: u64 },
}
impl Expression {
    pub(super) fn dependencies(
        self,
        report: &Report,
        key: NodeKey,
        mut visit: impl FnMut(NodeKey),
    ) {
        match self {
            Self::Fields(fields) => {
                let unit = report.package.unit(key.file);
                for &node in unit.configured.ast.list(fields).iter().rev() {
                    if unit.configured.active[node.0 as usize] {
                        visit(NodeKey { node, ..key });
                    }
                }
            }
            Self::Repeat { value, .. } => visit(value),
        }
    }
}
impl Queries<'_> {
    pub(super) fn product_expression(&self, key: NodeKey) -> Result<Option<Expression>, Error> {
        let facts = self
            .typing
            .facts
            .node(key)
            .ok_or_else(|| self.error(Query::Node(key), "missing constant product facts"))?;
        let child = |node| NodeKey { node, ..key };
        Ok(
            match (Declarations::kind(self.report, key), facts.operation) {
                (Kind::Tuple(fields) | Kind::Array(fields), _) => Some(Expression::Fields(fields)),
                (
                    Kind::Block {
                        flavor: crate::syntax::ast::BlockFlavor::Plain,
                        statements,
                        tail: None,
                    }
                    | Kind::Expansion {
                        children: statements,
                        tail: None,
                    },
                    _,
                ) if self
                    .report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(statements)
                    .iter()
                    .all(|node| {
                        !self.report.package.unit(key.file).configured.active[node.0 as usize]
                    }) =>
                {
                    Some(Expression::Fields(statements))
                }
                (Kind::Repeat { value, .. }, _) => {
                    let Some(Type::Array { length, .. }) =
                        facts.ty.and_then(|ty| self.typing.facts.types.kind(ty))
                    else {
                        return Err(self.error(Query::Node(key), "missing constant array type"));
                    };
                    Some(Expression::Repeat {
                        value: child(value),
                        length,
                    })
                }
                _ => None,
            },
        )
    }
    pub(super) fn product_value(
        &mut self,
        key: NodeKey,
        expression: Expression,
    ) -> Result<Value, Error> {
        let query = Query::Node(key);
        match expression {
            Expression::Fields(fields) => {
                let unit = self.report.package.unit(key.file);
                let mut values = Vec::new();
                for &node in unit.configured.ast.list(fields) {
                    if unit.configured.active[node.0 as usize] {
                        self.budget()?;
                        values.push(self.value(Query::Node(NodeKey { node, ..key }))?);
                    }
                }
                self.typing
                    .facts
                    .constant_products
                    .fields(&values, None)
                    .map_err(|e| self.error(query, e))
            }
            Expression::Repeat { value, length } => {
                let value = self.value(Query::Node(value))?;
                self.typing
                    .facts
                    .constant_products
                    .repeat(value, length)
                    .map_err(|e| self.error(query, e))
            }
        }
    }
}
