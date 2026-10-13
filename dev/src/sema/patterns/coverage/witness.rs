//! Cold diagnostics render one satisfying assignment without expanding sparse products.
use super::{Coverage, Id, PlaceId, Step, Values};
use crate::{
    sema::{
        members::{Name, Selection, VariantId},
        patterns::Projection,
    },
    syntax::ast::Kind,
};
use std::fmt::Write;

impl Coverage<'_> {
    pub(super) fn witness(&mut self, set: Id) -> Result<String, &'static str> {
        let assignments = self.graph.witness(set)?;
        if assignments.is_empty() {
            return Ok("_".into());
        }
        let mut result = String::from("{ ");
        for (index, (variable, rank)) in assignments.into_iter().enumerate() {
            if index != 0 {
                result.push_str(", ");
            }
            let input = &self.inputs[variable.0 as usize];
            self.path(input.place, &mut result)?;
            if matches!(input.values, Values::Length) {
                result.push_str(".len");
            }
            result.push_str(" = ");
            match &input.values {
                Values::Scalar(domain) => result.push_str(&domain.witness(rank)?),
                Values::Length => {
                    write!(result, "{rank}").map_err(|_| "cannot format slice length")?;
                }
                Values::Tags(tags) => {
                    let index = usize::try_from(rank).map_err(|_| "invalid constructor witness")?;
                    self.variant_name(
                        *tags.get(index).ok_or("invalid constructor witness")?,
                        &mut result,
                    )?;
                }
            }
        }
        result.push_str(" }");
        Ok(result)
    }
    fn variant_name(&self, id: VariantId, out: &mut String) -> Result<(), &'static str> {
        let variant = self.facts.members.variants[id.0 as usize];
        let definition = &self.report.resolution.definitions[variant.owner.0 as usize];
        out.push_str(self.report.package.names.get(definition.name));
        let unit = self.report.package.unit(variant.node.file);
        if let Kind::Variant(index) = unit.configured.ast.nodes[variant.node.node.0 as usize].kind {
            let span = unit.configured.ast.variants[index as usize].name;
            out.push_str("::");
            out.push_str(
                unit.source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    .ok_or("invalid constructor name span")?,
            );
        }
        Ok(())
    }
    fn path(&self, mut place: PlaceId, out: &mut String) -> Result<(), &'static str> {
        let mut steps = Vec::new();
        while let Some((parent, step)) = self.places[place.0 as usize].parent {
            steps.push(step);
            place = parent;
        }
        out.push('$');
        for step in steps.into_iter().rev() {
            match step {
                Step::Dereference => {
                    out.push_str(".*");
                    Ok(())
                }
                Step::Field(Projection::Field(Selection::Tuple(index))) => write!(out, ".{index}"),
                Step::Field(Projection::Field(Selection::Nominal(field))) => {
                    match self.facts.members.fields[field.0 as usize].name {
                        Name::Index(index) => write!(out, ".{index}"),
                        Name::Named(name) => write!(out, ".{}", self.facts.members.names.get(name)),
                    }
                }
                Step::Field(Projection::Index(index)) => write!(out, "[{index}]"),
                Step::Field(Projection::FromEnd(offset)) => write!(out, "[-{offset}]"),
                Step::Field(Projection::Subarray { start, end }) => write!(out, "[{start}..{end}]"),
                Step::Field(Projection::Subslice { prefix, suffix }) => {
                    write!(out, "[{prefix}..-{suffix}]")
                }
            }
            .map_err(|_| "cannot format coverage witness")?;
        }
        Ok(())
    }
}
