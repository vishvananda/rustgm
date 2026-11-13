//! rustc's derive also bounds each field type path that begins with one of the
//! item's type parameters, such as `I::Item`, unless the path is that
//! parameter alone (`find_type_parameters`).
use super::{Charge, GenericFlavor, Kind, LexError, NodeId, Site, Span, error};
use crate::syntax::ast::{List, MacroContext, TypeFlavor};

/// The bounded paths of the active fields' types, in field order, with
/// rustc's repetitions. Constant expressions and a qualified path's own
/// segments are not field type paths. rustc rejects a type macro in a generic
/// item's field.
pub(super) fn collect(
    site: &Site<'_>,
    fields: &[NodeId],
    charge: &Charge<'_>,
) -> Result<Vec<NodeId>, LexError> {
    let mut walk = Walk {
        site,
        charge,
        names: Vec::new(),
        found: Vec::new(),
    };
    for &node in site.ast.list(site.item.generics) {
        charge(1, 0)?;
        let Kind::Generic(index) = site.ast.node(node).kind else {
            return Err(error(
                site.original.start,
                "derive lost an actual generic parameter",
            ));
        };
        let parameter = site.ast.generics[index as usize];
        if parameter.flavor == GenericFlavor::Type {
            let name = walk.name(parameter.name)?;
            let growth = crate::support::retained::Growth::vector::<&str>(
                walk.names.len(),
                walk.names.capacity(),
                1,
            );
            charge(growth.work, growth.bytes)?;
            walk.names
                .try_reserve(1)
                .map_err(|_| error(site.original.start, "derived parameter allocation failed"))?;
            walk.names.push(name);
        }
    }
    if walk.names.is_empty() {
        return Ok(Vec::new());
    }
    for &field in fields {
        let mut node = field;
        while let Kind::Attributed { value, .. } = site.ast.node(node).kind {
            charge(1, 0)?;
            node = value;
        }
        let Kind::FieldDefinition { ty, .. } = site.ast.node(node).kind else {
            return Err(error(
                site.original.start,
                "derive bounds lost an actual field",
            ));
        };
        walk.ty(ty, false)?;
    }
    Ok(walk.found)
}

struct Walk<'a> {
    site: &'a Site<'a>,
    charge: &'a Charge<'a>,
    names: Vec<&'a str>,
    found: Vec<NodeId>,
}

impl<'a> Walk<'a> {
    /// A name's identifier, without a raw prefix, as rustc compares symbols.
    fn name(&self, span: Span) -> Result<&'a str, LexError> {
        let text = self
            .site
            .source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| error(span.start, "derived name is outside its source"))?;
        Ok(text.strip_prefix("r#").unwrap_or(text))
    }
    /// `quantified` is set beneath a `for<...>` binder, whose predicate would
    /// need that binder.
    fn ty(&mut self, node: NodeId, quantified: bool) -> Result<(), LexError> {
        (self.charge)(1, 0)?;
        let actual = self.site.ast.node(node);
        match actual.kind {
            Kind::Path {
                absolute: false,
                segments,
            } if self.parameter_path(segments)? => {
                if quantified {
                    return Err(error(
                        actual.span.start,
                        "higher-ranked derived field bounds require a wider profile",
                    ));
                }
                let growth = crate::support::retained::Growth::vector::<NodeId>(
                    self.found.len(),
                    self.found.capacity(),
                    1,
                );
                (self.charge)(growth.work, growth.bytes)?;
                self.found
                    .try_reserve(1)
                    .map_err(|_| error(actual.span.start, "derived bound allocation failed"))?;
                self.found.push(node);
                self.segments(segments, quantified)
            }
            Kind::Path { segments, .. } => self.segments(segments, quantified),
            Kind::Applied {
                arguments, output, ..
            } => self.arguments(arguments, output, quantified),
            Kind::Type { flavor, elements } => {
                let elements = self.site.ast.list(elements);
                // An array's length is a constant expression.
                let types = if flavor == TypeFlavor::Array {
                    elements.get(..1).unwrap_or(elements)
                } else {
                    elements
                };
                for &element in types {
                    self.ty(element, quantified)?;
                }
                Ok(())
            }
            Kind::Group(value) | Kind::FieldDefinition { ty: value, .. } => {
                self.ty(value, quantified)
            }
            Kind::Qualified {
                ty,
                trait_path,
                path,
            } => {
                self.ty(ty, quantified)?;
                if let Some(trait_path) = trait_path {
                    self.path_arguments(trait_path, quantified)?;
                }
                self.path_arguments(path, quantified)
            }
            Kind::FunctionType(index) => {
                let function = self.site.ast.function_types[index as usize];
                let quantified = quantified || function.binders.start != function.binders.end;
                for &parameter in self.site.ast.list(function.parameters) {
                    self.ty(parameter, quantified)?;
                }
                function
                    .result
                    .map_or(Ok(()), |result| self.ty(result, quantified))
            }
            Kind::TraitType { bounds, .. } => self.bounds(bounds, quantified),
            Kind::Macro {
                context: MacroContext::Type,
                ..
            } => Err(error(
                actual.span.start,
                "`derive` cannot be used on items with type macros",
            )),
            // Names, lifetimes, `!`, `_` and constant expressions.
            _ => Ok(()),
        }
    }
    /// A relative path of two or more segments whose first is a parameter.
    fn parameter_path(&self, segments: List) -> Result<bool, LexError> {
        let segments = self.site.ast.list(segments);
        let [first, _, ..] = segments else {
            return Ok(false);
        };
        let Kind::Name(span, _) = self.site.ast.node(*first).kind else {
            return Ok(false);
        };
        (self.charge)(self.names.len(), 0)?;
        let name = self.name(span)?;
        Ok(self.names.contains(&name))
    }
    fn segments(&mut self, segments: List, quantified: bool) -> Result<(), LexError> {
        for &segment in self.site.ast.list(segments) {
            (self.charge)(1, 0)?;
            if let Kind::Applied {
                arguments, output, ..
            } = self.site.ast.node(segment).kind
            {
                self.arguments(arguments, output, quantified)?;
            }
        }
        Ok(())
    }
    /// The generic arguments of a trait or associated path, which is not
    /// itself a field type path.
    fn path_arguments(&mut self, node: NodeId, quantified: bool) -> Result<(), LexError> {
        (self.charge)(1, 0)?;
        match self.site.ast.node(node).kind {
            Kind::Path { segments, .. } => self.segments(segments, quantified),
            Kind::Applied {
                arguments, output, ..
            } => self.arguments(arguments, output, quantified),
            _ => Ok(()),
        }
    }
    fn arguments(
        &mut self,
        arguments: List,
        output: Option<NodeId>,
        quantified: bool,
    ) -> Result<(), LexError> {
        for &argument in self.site.ast.list(arguments) {
            if let Kind::Constraint {
                target,
                value,
                bounds,
            } = self.site.ast.node(argument).kind
            {
                (self.charge)(1, 0)?;
                self.path_arguments(target, quantified)?;
                if let Some(value) = value {
                    self.ty(value, quantified)?;
                }
                self.bounds(bounds, quantified)?;
            } else {
                self.ty(argument, quantified)?;
            }
        }
        output.map_or(Ok(()), |output| self.ty(output, quantified))
    }
    fn bounds(&mut self, bounds: List, quantified: bool) -> Result<(), LexError> {
        for &bound in self.site.ast.list(bounds) {
            (self.charge)(1, 0)?;
            if let Kind::Bound {
                quantified: written,
                binders,
                value,
                ..
            } = self.site.ast.node(bound).kind
            {
                let quantified = quantified || written || binders.start != binders.end;
                self.path_arguments(value, quantified)?;
            }
        }
        Ok(())
    }
}
