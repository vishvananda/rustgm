//! Pattern and type nodes share the same borrowed, checked source arena.
use super::{Action, Kind, LexError, Node, View, Writer, error};
use crate::{
    support::constness::Constness,
    syntax::ast::{PatternFlavor, TypeFlavor},
};
impl<'a> Writer<'a> {
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive pattern and type spelling dispatch"
    )]
    pub(super) fn pattern_type(&mut self, view: View<'a>, node: Node) -> Result<(), LexError> {
        use Action::Word as W;
        match node.kind {
            Kind::Wildcard | Kind::InferType => self.text("_"),
            Kind::NeverType => self.text("!"),
            Kind::RestPattern => self.text(".."),
            Kind::Variadic => self.text("..."),
            Kind::Binding {
                name,
                mutable,
                by_ref,
                subpattern,
                ..
            } => {
                if let Some(value) = subpattern {
                    self.enqueue(&[W(" @ "), view.n(value)])?;
                }
                self.enqueue(&[
                    W(if by_ref { "ref " } else { "" }),
                    W(if mutable { "mut " } else { "" }),
                    view.span(name),
                ])
            }
            Kind::ReferencePattern { mutable, pattern } => {
                self.enqueue(&[W(if mutable { "&mut " } else { "&" }), view.n(pattern)])
            }
            Kind::Pattern {
                flavor,
                path,
                elements,
            } => {
                let (open, close, separator) = match flavor {
                    PatternFlavor::Tuple | PatternFlavor::Variant => ("(", ")", ", "),
                    PatternFlavor::Slice => ("[", "]", ", "),
                    PatternFlavor::Or => ("", "", " | "),
                };
                let singleton = flavor == PatternFlavor::Tuple && view.list(elements)?.len() == 1;
                self.enqueue(&[
                    W(open),
                    view.sequence(elements, separator),
                    W(if singleton { ",)" } else { close }),
                ])?;
                if let Some(path) = path {
                    self.enqueue(&[view.n(path)])?;
                }
                Ok(())
            }
            Kind::PatternRange {
                start,
                end,
                inclusive,
            }
            | Kind::Range {
                start,
                end,
                inclusive,
            } => {
                if let Some(end) = end {
                    self.enqueue(&[view.child(end, 21)])?;
                }
                self.enqueue(&[W(if inclusive { "..=" } else { ".." })])?;
                if let Some(start) = start {
                    self.enqueue(&[view.child(start, 21)])?;
                }
                Ok(())
            }
            Kind::StructPattern { path, fields, rest } => self.enqueue(&[
                view.n(path),
                W(" { "),
                view.sequence(fields, ", "),
                W(if rest && fields.start != fields.end {
                    ", .. }"
                } else if rest {
                    ".. }"
                } else {
                    " }"
                }),
            ]),
            Kind::PatternField { name, pattern } => {
                if matches!(view.node(pattern)?.kind, Kind::Binding { name: actual, .. } if actual == name)
                {
                    self.enqueue(&[view.pattern().n(pattern)])
                } else {
                    self.enqueue(&[view.span(name), W(": "), view.pattern().n(pattern)])
                }
            }
            Kind::Struct { path, fields, base } => {
                if fields.start == fields.end && base.is_none() {
                    return self.enqueue(&[view.n(path), W(" {}")]);
                }
                self.enqueue(&[Action::EndBox])?;
                self.enqueue(&[
                    if base.is_some() {
                        Action::Space { offset: 0 }
                    } else {
                        Action::TrailingComma { offset: -4 }
                    },
                    W("}"),
                ])?;
                if let Some(base) = base {
                    self.enqueue(&[
                        W(if fields.start == fields.end {
                            ".."
                        } else {
                            ", .."
                        }),
                        view.n(base),
                    ])?;
                }
                self.enqueue(&[
                    view.expr().n(path),
                    W(" "),
                    Action::BeginBox {
                        indent: 0,
                        consistent: true,
                    },
                    W("{"),
                    Action::Space { offset: 0 },
                    view.expr().sequence(fields, ", "),
                ])
            }
            Kind::FieldInit { name, value } => {
                if matches!(view.node(value)?.kind, Kind::Name(actual, _) if actual == name) {
                    self.enqueue(&[view.n(value)])
                } else {
                    self.enqueue(&[view.span(name), W(": "), view.n(value)])
                }
            }
            Kind::TypePattern { .. } => Err(error(
                node.span.start,
                "pattern type source expression printing requires its explicit macro syntax",
            )),
            Kind::Type { flavor, elements } => self.type_shape(view, flavor, elements),
            Kind::Applied {
                base,
                arguments,
                output,
                parenthesized,
            } => {
                if let Some(output) = output {
                    self.enqueue(&[W(" -> "), view.n(output)])?;
                }
                self.enqueue(&[
                    view.n(base),
                    W(if parenthesized {
                        "("
                    } else if view.mode == super::Mode::Type {
                        "<"
                    } else {
                        "::<"
                    }),
                    view.sequence(arguments, ", "),
                    W(if parenthesized { ")" } else { ">" }),
                ])
            }
            Kind::Qualified {
                ty,
                trait_path,
                path,
            } => {
                self.enqueue(&[W(">::"), view.n(path)])?;
                if let Some(trait_path) = trait_path {
                    self.enqueue(&[W(" as "), view.ty().n(trait_path)])?;
                }
                self.enqueue(&[W("<"), view.ty().n(ty)])
            }
            Kind::TraitType { opaque, bounds } => self.enqueue(&[
                W(if opaque { "impl " } else { "dyn " }),
                view.ty().sequence(bounds, " + "),
            ]),
            Kind::PreciseCapture(parameters) => {
                self.enqueue(&[W("use<"), view.sequence(parameters, ", "), W(">")])
            }
            Kind::Constraint {
                target,
                value,
                bounds,
            } => {
                if let Some(value) = value {
                    self.enqueue(&[W(" = "), view.n(value)])?;
                } else {
                    self.enqueue(&[W(": "), view.ty().sequence(bounds, " + ")])?;
                }
                self.enqueue(&[view.n(target)])
            }
            Kind::Bound {
                optional,
                constness,
                quantified,
                binders,
                value,
            } => {
                self.enqueue(&[
                    W(match constness {
                        Constness::Ordinary => "",
                        Constness::Const => "const ",
                        Constness::Conditional => "[const] ",
                    }),
                    W(if optional { "?" } else { "" }),
                    view.n(value),
                ])?;
                if quantified {
                    self.enqueue(&[W("for<"), view.sequence(binders, ", "), W("> ")])?;
                }
                Ok(())
            }
            Kind::Where {
                quantified,
                binders,
                ty,
                bounds,
            } => {
                self.enqueue(&[view.ty().n(ty), W(": "), view.ty().sequence(bounds, " + ")])?;
                if quantified {
                    self.enqueue(&[W("for<"), view.sequence(binders, ", "), W("> ")])?;
                }
                Ok(())
            }
            Kind::ImplRestriction { path } => self.enqueue(&[W("impl("), view.n(path), W(")")]),
            _ => self.declaration_syntax(view, node),
        }
    }
    fn type_shape(
        &mut self,
        view: View<'a>,
        flavor: TypeFlavor,
        elements: crate::syntax::ast::List,
    ) -> Result<(), LexError> {
        use Action::Word as W;
        let children = view.list(elements)?;
        match flavor {
            TypeFlavor::Tuple => self.enqueue(&[
                W("("),
                view.sequence(elements, ", "),
                W(if children.len() == 1 { ",)" } else { ")" }),
            ]),
            TypeFlavor::Slice => self.enqueue(&[W("["), view.sequence(elements, ""), W("]")]),
            TypeFlavor::Array => self.enqueue(&[W("["), view.sequence(elements, "; "), W("]")]),
            TypeFlavor::Ref | TypeFlavor::RefMut => {
                let (ty, lifetime) = match children {
                    [ty] => (*ty, None),
                    [lifetime, ty] => (*ty, Some(*lifetime)),
                    _ => {
                        return Err(error(
                            0,
                            "expression writer reference type has invalid children",
                        ));
                    }
                };
                self.enqueue(&[
                    W(if flavor == TypeFlavor::RefMut {
                        "mut "
                    } else {
                        ""
                    }),
                    view.ty().n(ty),
                ])?;
                if let Some(lifetime) = lifetime {
                    self.enqueue(&[view.n(lifetime), W(" ")])?;
                }
                self.enqueue(&[W("&")])
            }
            TypeFlavor::PointerConst | TypeFlavor::PointerMut => self.enqueue(&[
                W(if flavor == TypeFlavor::PointerConst {
                    "*const "
                } else {
                    "*mut "
                }),
                view.sequence(elements, ""),
            ]),
        }
    }
}
