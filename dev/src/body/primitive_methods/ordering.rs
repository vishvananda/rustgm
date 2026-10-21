//! Primitive ordering returns ordinary canonical enum aggregates through checked CFGs.
use super::{Validated, value};
use crate::{
    body::ir::{Block, BlockId, Body, Rvalue, Terminator, ValueId, ValueList},
    lex::token::Punctuation,
    sema::{members::VariantId, model::Declarations},
    support::comparisons::Method,
    types::{Kind, Primitive, TypeId},
};
struct Results {
    ordering: TypeId,
    variants: [VariantId; 3],
    option: Option<[VariantId; 2]>,
}
impl Results {
    fn new(facts: &Declarations, source: &Validated) -> Result<Self, &'static str> {
        let (ordering, option) = if source.comparison == Some(Method::PartialCmp) {
            let Some(Kind::Adt(owner, arguments)) = facts.types.kind(source.result) else {
                return Err("partial_cmp factory has no option result");
            };
            let [none, some] = facts.members.variants(owner) else {
                return Err("partial_cmp factory has invalid option schema");
            };
            let Some(crate::types::arguments::Argument::Type(ty)) = facts
                .types
                .arguments(arguments)
                .and_then(|mut args| args.next())
            else {
                return Err("partial_cmp factory has no ordering argument");
            };
            (ty, Some([*none, *some]))
        } else {
            (source.result, None)
        };
        let Some(Kind::Adt(owner, _)) = facts.types.kind(ordering) else {
            return Err("ordering factory has no nominal result");
        };
        let [less, equal, greater] = facts.members.variants(owner) else {
            return Err("ordering factory has invalid result schema");
        };
        Ok(Self {
            ordering,
            variants: [*less, *equal, *greater],
            option,
        })
    }
    fn result(&self, body: &mut Body, ordinal: Option<usize>) -> Result<ValueId, &'static str> {
        let start =
            u32::try_from(body.arguments.len()).map_err(|_| "too many ordering arguments")?;
        let selected = ordinal
            .map(|ordinal| {
                self.variants
                    .get(ordinal)
                    .copied()
                    .ok_or("invalid ordering result")
            })
            .transpose()?;
        let ordering = selected
            .map(|variant| {
                value(
                    body,
                    self.ordering,
                    Rvalue::Aggregate {
                        variant: Some(variant),
                        fields: ValueList { start, end: start },
                    },
                )
            })
            .transpose()?;
        if let Some([none, some]) = self.option {
            if let Some(ordering) = ordering {
                body.arguments.push(ordering);
            }
            let end =
                u32::try_from(body.arguments.len()).map_err(|_| "too many ordering arguments")?;
            value(
                body,
                body.result,
                Rvalue::Aggregate {
                    variant: Some(if ordering.is_some() { some } else { none }),
                    fields: ValueList { start, end },
                },
            )
        } else {
            ordering.ok_or("total ordering has unordered result")
        }
    }
}
fn branch(
    body: &mut Body,
    start: u32,
    condition: ValueId,
    yes: u32,
    no: u32,
) -> Result<u32, &'static str> {
    let end = u32::try_from(body.statements.len()).map_err(|_| "too many ordering statements")?;
    body.blocks.push(Block {
        start,
        end,
        source: body.source.node,
        terminator: Terminator::Branch {
            condition,
            yes: BlockId(yes),
            no: BlockId(no),
        },
    });
    Ok(end)
}
fn finish(
    body: &mut Body,
    results: &Results,
    start: u32,
    ordinal: Option<usize>,
) -> Result<u32, &'static str> {
    let result = results.result(body, ordinal)?;
    crate::body::build::finish_factory_block(body, start, result)?;
    u32::try_from(body.statements.len()).map_err(|_| "too many ordering statements")
}
pub(super) fn build(
    body: &mut Body,
    facts: &Declarations,
    source: &Validated,
    left: ValueId,
    right: ValueId,
) -> Result<(), &'static str> {
    let results = Results::new(facts, source)?;
    let boolean = TypeId::primitive(Primitive::Bool);
    let less = value(
        body,
        boolean,
        Rvalue::Binary {
            operator: Punctuation::Less,
            left,
            right,
        },
    )?;
    let start = branch(body, 0, less, 1, 2)?;
    let start = finish(body, &results, start, Some(0))?;
    let equal = value(
        body,
        boolean,
        Rvalue::Binary {
            operator: Punctuation::EqualEqual,
            left,
            right,
        },
    )?;
    let start = branch(body, start, equal, 3, 4)?;
    let start = finish(body, &results, start, Some(1))?;
    if source.comparison == Some(Method::PartialCmp) {
        let greater = value(
            body,
            boolean,
            Rvalue::Binary {
                operator: Punctuation::Greater,
                left,
                right,
            },
        )?;
        let start = branch(body, start, greater, 5, 6)?;
        let start = finish(body, &results, start, Some(2))?;
        finish(body, &results, start, None)?;
    } else {
        finish(body, &results, start, Some(2))?;
    }
    Ok(())
}
