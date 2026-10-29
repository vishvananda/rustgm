//! Impl formals preserve parsed constraints; nominal applications carry names only.
use super::{GenericFlavor, ItemKind, Kind, LexError, NodeId, Output, error};
use crate::{support::constness::Constness, syntax::ast::List};
impl Output<'_> {
    pub(super) fn generics(&mut self, implementation: Option<&str>) -> Result<(), LexError> {
        let parameters = self.site.ast.list(self.site.item.generics);
        if parameters.is_empty() {
            return Ok(());
        }
        self.text("<")?;
        for &node in parameters {
            (self.charge)(1, 0)?;
            let Kind::Generic(index) = self.site.ast.node(node).kind else {
                return Err(error(
                    self.site.original.start,
                    "derive lost an actual generic parameter",
                ));
            };
            let parameter = self.site.ast.generics[index as usize];
            if implementation.is_some() && parameter.flavor == GenericFlavor::Const {
                self.text("const")?;
            }
            self.spelling(parameter.name, parameter.flavor == GenericFlavor::Lifetime)?;
            if let Some(path) = implementation {
                if parameter.flavor == GenericFlavor::Const {
                    self.text(":")?;
                    self.field(parameter.ty.ok_or_else(|| {
                        error(parameter.name.start, "derived const formal lost its type")
                    })?)?;
                }
                let added = parameter.flavor == GenericFlavor::Type
                    && path != "crate::marker::StructuralPartialEq"
                    && !(path == "crate::default::Default"
                        && matches!(self.site.item.kind, ItemKind::Enum { .. }));
                let bounded = parameter.bounds.start != parameter.bounds.end;
                if bounded || added {
                    self.text(":")?;
                    self.bounds(parameter.bounds)?;
                    if added {
                        if bounded {
                            self.text("+")?;
                        }
                        if self.site.const_ {
                            self.text("[const]")?;
                        }
                        self.text(path)?;
                    }
                }
            }
            // Source defaults constrain the nominal declaration, not its impl formals.
            self.text(",")?;
        }
        self.text(">")
    }
    fn bounds(&mut self, bounds: List) -> Result<(), LexError> {
        for (index, &node) in self.site.ast.list(bounds).iter().enumerate() {
            (self.charge)(1, 0)?;
            let Kind::Bound {
                optional,
                constness,
                quantified,
                binders,
                value,
            } = self.site.ast.node(node).kind
            else {
                return Err(error(
                    self.site.original.start,
                    "source derive bound requires a wider parsed profile",
                ));
            };
            if quantified || binders.start != binders.end {
                return Err(error(
                    self.site.original.start,
                    "higher-ranked source derive bounds require a wider profile",
                ));
            }
            if index != 0 {
                self.text("+")?;
            }
            if optional {
                self.text("?")?;
            }
            match constness {
                Constness::Ordinary => {}
                Constness::Conditional => self.text("[const]")?,
                Constness::Const => self.text("const")?,
            }
            self.constraint_type(value)?;
        }
        Ok(())
    }
    fn constraint_type(&mut self, node: NodeId) -> Result<(), LexError> {
        if let Kind::Name(span, _) = self.site.ast.node(node).kind
            && self
                .site
                .source
                .get(span.start as usize..span.end as usize)
                .is_some_and(|name| name.starts_with('\''))
        {
            self.spelling(span, true)
        } else {
            self.field(node)
        }
    }
    pub(super) fn where_predicates(&mut self) -> Result<(), LexError> {
        let predicates = self.site.ast.list(self.site.item.where_clause);
        if !predicates.is_empty() {
            self.text("where")?;
        }
        for &node in predicates {
            (self.charge)(1, 0)?;
            let Kind::Where {
                quantified,
                binders,
                ty,
                bounds,
            } = self.site.ast.node(node).kind
            else {
                return Err(error(
                    self.site.original.start,
                    "derive lost its parsed where predicate",
                ));
            };
            if quantified || binders.start != binders.end {
                return Err(error(
                    self.site.original.start,
                    "higher-ranked source derive predicates require a wider profile",
                ));
            }
            self.constraint_type(ty)?;
            self.text(":")?;
            self.bounds(bounds)?;
            self.text(",")?;
        }
        Ok(())
    }
}
