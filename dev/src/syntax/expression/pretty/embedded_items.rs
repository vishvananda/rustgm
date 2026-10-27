//! Items nested in an assertion condition use their parsed declaration facts.
use super::{Action, Kind, LexError, Node, View, Writer, error};
use crate::syntax::{
    ast::List,
    item::{Abi, GenericFlavor, Item, ItemKind, Safety, Shape, Visibility},
};
impl<'a> Writer<'a> {
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive source declaration-node spelling dispatch"
    )]
    pub(super) fn declaration_syntax(
        &mut self,
        view: View<'a>,
        node: Node,
    ) -> Result<(), LexError> {
        use Action::Word as W;
        match node.kind {
            Kind::Generic(index) => {
                let generic = *view
                    .arena
                    .generics()
                    .get(index as usize)
                    .ok_or_else(|| error(node.span.start, "writer generic row is missing"))?;
                if let Some(default) = generic.default {
                    self.enqueue(&[W(" = "), view.n(default)])?;
                }
                if generic.bounds.start != generic.bounds.end {
                    self.enqueue(&[W(": "), view.ty().sequence(generic.bounds, " + ")])?;
                }
                if let Some(ty) = generic.ty {
                    self.enqueue(&[W(": "), view.ty().n(ty)])?;
                }
                self.enqueue(&[
                    W(if generic.flavor == GenericFlavor::Const {
                        "const "
                    } else {
                        ""
                    }),
                    view.span(generic.name),
                ])
            }
            Kind::FunctionType(index) => {
                let function =
                    *view.arena.functions().get(index as usize).ok_or_else(|| {
                        error(node.span.start, "writer function type row is missing")
                    })?;
                if let Some(result) = function.result {
                    self.enqueue(&[W(" -> "), view.ty().n(result)])?;
                }
                self.enqueue(&[
                    W("fn("),
                    view.ty().sequence(function.parameters, ", "),
                    W(")"),
                ])?;
                self.abi(view, function.abi)?;
                if function.unsafe_ {
                    self.enqueue(&[W("unsafe ")])?;
                }
                if function.binders.start != function.binders.end {
                    self.enqueue(&[
                        W("for<"),
                        view.ty().sequence(function.binders, ", "),
                        W("> "),
                    ])?;
                }
                Ok(())
            }
            Kind::FieldDefinition {
                name,
                visibility,
                ty,
            } => {
                self.enqueue(&[view.ty().n(ty)])?;
                if let Some(name) = name {
                    self.enqueue(&[view.span(name), W(": ")])?;
                }
                self.visibility(view, visibility)
            }
            Kind::Receiver {
                name,
                lifetime,
                mutable,
            } => self.enqueue(&[
                W("&"),
                view.span(lifetime),
                W(if lifetime.start == lifetime.end {
                    ""
                } else {
                    " "
                }),
                W(if mutable { "mut " } else { "" }),
                view.span(name),
            ]),
            Kind::Item(index) => {
                let item = *view
                    .arena
                    .items()
                    .get(index as usize)
                    .ok_or_else(|| error(node.span.start, "writer item row is missing"))?;
                let modifier = self.modifier(view, node, index)?;
                self.item(view, item, modifier)
            }
            Kind::Crate(items) => self.enqueue(&[view.sequence(items, " ")]),
            Kind::Expansion { children, tail } => {
                if let Some(tail) = tail {
                    self.enqueue(&[
                        W(if children.start == children.end {
                            ""
                        } else {
                            " "
                        }),
                        view.n(tail),
                    ])?;
                }
                self.enqueue(&[view.sequence(children, " ")])
            }
            Kind::Variant(index) => {
                let variant = *view
                    .arena
                    .variants()
                    .get(index as usize)
                    .ok_or_else(|| error(node.span.start, "writer variant row is missing"))?;
                if let Some(discriminant) = variant.discriminant {
                    self.enqueue(&[W(" = "), view.n(discriminant)])?;
                }
                self.fields(view, variant.shape, variant.fields)?;
                self.enqueue(&[view.span(variant.name)])
            }
            Kind::UseTree(index) => self.use_tree(view, index, node.span),
            Kind::Macro { path, capture, .. } => {
                self.capture(view, capture)?;
                self.enqueue(&[view.n(path), W("!")])
            }
            Kind::Attributed { attributes, value } => {
                self.enqueue(&[view.n(value)])?;
                self.attributes(view, attributes)
            }
            Kind::InlineAssembly(_)
            | Kind::AssemblyTemplate { .. }
            | Kind::AssemblyOperand { .. }
            | Kind::AssemblyRegister { .. }
            | Kind::AssemblyOrigin(_)
            | Kind::AssemblyOptions(_)
            | Kind::AssemblyPlaceholder { .. }
            | Kind::LibraryValue { .. } => Err(error(
                node.span.start,
                "source expression writer does not accept synthesized library values",
            )),
            _ => Err(error(
                node.span.start,
                "expression writer received an invalid declaration syntax node",
            )),
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive parsed item spelling dispatch"
    )]
    fn item(
        &mut self,
        view: View<'a>,
        item: Item,
        modifier: Option<&'static str>,
    ) -> Result<(), LexError> {
        use Action::Word as W;
        match item.kind {
            ItemKind::Function {
                flags,
                abi,
                parameters,
                result,
                body,
            } => {
                if let Some(body) = body {
                    self.enqueue(&[W(" "), view.n(body)])?;
                } else {
                    self.enqueue(&[W(";")])?;
                }
                self.where_clause(view, item.where_clause)?;
                if let Some(result) = result {
                    self.enqueue(&[W(" -> "), view.ty().n(result)])?;
                }
                self.enqueue(&[W("("), view.pattern().sequence(parameters, ", "), W(")")])?;
                self.named_header(view, item, "fn ")?;
                self.abi(view, abi)?;
                self.enqueue(&[
                    W(if flags.constant { "const " } else { "" }),
                    W(if flags.asynchronous { "async " } else { "" }),
                    W(safety(flags.safety)),
                ])?;
            }
            ItemKind::Struct { shape, fields } => {
                self.enqueue(&[W(if shape == Shape::Record { "" } else { ";" })])?;
                if shape == Shape::Tuple {
                    self.where_clause(view, item.where_clause)?;
                    self.fields(view, shape, fields)?;
                } else {
                    self.fields(view, shape, fields)?;
                    self.where_clause(view, item.where_clause)?;
                }
                self.named_header(view, item, "struct ")?;
            }
            ItemKind::Union { fields } => {
                self.fields(view, Shape::Record, fields)?;
                self.where_clause(view, item.where_clause)?;
                self.named_header(view, item, "union ")?;
            }
            ItemKind::Enum { variants } => {
                self.enqueue(&[W(" { "), view.sequence(variants, ", "), W(" }")])?;
                self.where_clause(view, item.where_clause)?;
                self.named_header(view, item, "enum ")?;
            }
            ItemKind::Const { ty, value } | ItemKind::Static { ty, value, .. } => {
                self.enqueue(&[W(";")])?;
                if let Some(value) = value {
                    self.enqueue(&[W(" = "), view.n(value)])?;
                }
                self.enqueue(&[W(": "), view.ty().n(ty)])?;
                let prefix = match item.kind {
                    ItemKind::Static { mutable: true, .. } => "static mut ",
                    ItemKind::Static { .. } => "static ",
                    _ => "const ",
                };
                self.named_header(view, item, prefix)?;
            }
            ItemKind::ForeignType => {
                self.enqueue(&[W(";")])?;
                self.named_header(view, item, "type ")?;
            }
            ItemKind::TypeAlias { bounds, ty } => {
                self.enqueue(&[W(";")])?;
                if let Some(ty) = ty {
                    self.enqueue(&[W(" = "), view.ty().n(ty)])?;
                }
                self.where_clause(view, item.where_clause)?;
                if bounds.start != bounds.end {
                    self.enqueue(&[W(": "), view.ty().sequence(bounds, " + ")])?;
                }
                self.named_header(view, item, "type ")?;
            }
            ItemKind::TraitAlias { constant, bounds } => {
                self.enqueue(&[W(" = "), view.ty().sequence(bounds, " + "), W(";")])?;
                self.named_header(view, item, if constant { "const trait " } else { "trait " })?;
            }
            ItemKind::Module { items } => {
                if let Some(items) = items {
                    self.enqueue(&[W(" { "), view.sequence(items, " "), W(" }")])?;
                } else {
                    self.enqueue(&[W(";")])?;
                }
                self.named_header(view, item, "mod ")?;
            }
            ItemKind::Use(tree) => self.enqueue(&[W("use "), view.n(tree), W(";")])?,
            ItemKind::MacroInvocation(value) => self.enqueue(&[view.n(value), W(";")])?,
            ItemKind::ExternCrate { rename } => {
                self.enqueue(&[W(";")])?;
                if let Some(rename) = rename {
                    self.enqueue(&[W(" as "), view.span(rename)])?;
                }
                self.named_header(view, item, "extern crate ")?;
            }
            ItemKind::ExternBlock {
                unsafe_,
                abi,
                items,
            } => {
                self.enqueue(&[W(" { "), view.sequence(items, " "), W(" }")])?;
                if let Some(abi) = abi {
                    self.enqueue(&[W(" "), view.n(abi)])?;
                }
                self.enqueue(&[W(if unsafe_ { "unsafe extern" } else { "extern" })])?;
            }
            ItemKind::MacroRules { capture } => {
                self.capture(view, capture)?;
                self.enqueue(&[W(" ")])?;
                self.named_header(view, item, "macro_rules! ")?;
            }
            ItemKind::DeclarativeMacro {
                parameters,
                capture,
            } => {
                self.capture(view, capture)?;
                if let Some(parameters) = parameters {
                    self.capture(view, parameters)?;
                }
                self.named_header(view, item, "macro ")?;
            }
            ItemKind::Trait {
                restriction,
                auto_,
                unsafe_,
                constant,
                bounds,
                items,
            } => {
                self.enqueue(&[W(" { "), view.sequence(items, " "), W(" }")])?;
                self.where_clause(view, item.where_clause)?;
                if bounds.start != bounds.end {
                    self.enqueue(&[W(": "), view.ty().sequence(bounds, " + ")])?;
                }
                self.named_header(view, item, "trait ")?;
                self.enqueue(&[
                    W(if constant { "const " } else { "" }),
                    W(if unsafe_ { "unsafe " } else { "" }),
                    W(if auto_ { "auto " } else { "" }),
                ])?;
                if let Some(restriction) = restriction {
                    self.enqueue(&[view.n(restriction), W(" ")])?;
                }
            }
            ItemKind::Impl {
                unsafe_,
                constant,
                negative,
                trait_path,
                ty,
                items,
            } => {
                self.enqueue(&[W(" { "), view.sequence(items, " "), W(" }")])?;
                self.where_clause(view, item.where_clause)?;
                self.enqueue(&[view.ty().n(ty)])?;
                if let Some(trait_path) = trait_path {
                    self.enqueue(&[
                        W(if negative { "!" } else { "" }),
                        W(if constant { "const " } else { "" }),
                        view.ty().n(trait_path),
                        W(" for "),
                    ])?;
                }
                self.generics(view, item.generics)?;
                self.enqueue(&[W(if unsafe_ { "unsafe impl " } else { "impl " })])?;
            }
        }
        if let Some(modifier) = modifier {
            self.enqueue(&[W(modifier), W(" ")])?;
        }
        self.visibility(view, item.visibility)
    }
    fn modifier(
        &mut self,
        view: View<'_>,
        node: Node,
        index: u32,
    ) -> Result<Option<&'static str>, LexError> {
        let mut width = view.arena.site_count();
        while width > 0 {
            // Pay this bounded-width step and the upcoming search comparison.
            self.pay(2, 0)?;
            width >>= 1;
        }
        self.pay(1, 0)?;
        let Some(actual) = view.arena.gate_site(index).copied() else {
            return Ok(None);
        };
        let keyword = if actual.kind.is_defaultness() {
            "default"
        } else if actual.kind == crate::syntax::gate_sites::Kind::FinalFunction {
            "final"
        } else {
            return Ok(None);
        };
        if actual.span.start < node.span.start
            || actual.span.end > node.span.end
            || view.arena.nodes().get(actual.node.get() as usize) != Some(&node)
            || view
                .source
                .get(actual.span.start as usize..actual.span.end as usize)
                != Some(keyword)
        {
            return Err(error(
                actual.span.start,
                "expression writer defaultness owner changed",
            ));
        }
        Ok(Some(keyword))
    }
    pub(super) fn visibility(
        &mut self,
        view: View<'a>,
        visibility: Visibility,
    ) -> Result<(), LexError> {
        use Action::Word as W;
        match visibility {
            Visibility::Private => Ok(()),
            Visibility::Public => self.enqueue(&[W("pub ")]),
            Visibility::Restricted(path) => self.enqueue(&[W("pub(in "), view.n(path), W(") ")]),
        }
    }
    fn abi(&mut self, view: View<'a>, abi: Abi) -> Result<(), LexError> {
        use Action::Word as W;
        match abi {
            Abi::Rust => Ok(()),
            Abi::Extern(None) => self.enqueue(&[W("extern ")]),
            Abi::Extern(Some(value)) => self.enqueue(&[W("extern "), view.n(value), W(" ")]),
        }
    }
    fn named_header(
        &mut self,
        view: View<'a>,
        item: Item,
        keyword: &'static str,
    ) -> Result<(), LexError> {
        self.generics(view, item.generics)?;
        let name = item
            .name
            .ok_or_else(|| error(0, "named item writer has no parsed name"))?;
        self.enqueue(&[Action::Word(keyword), view.span(name)])
    }
    fn generics(&mut self, view: View<'a>, list: List) -> Result<(), LexError> {
        if list.start == list.end {
            return Ok(());
        }
        self.enqueue(&[
            Action::Word("<"),
            view.ty().sequence(list, ", "),
            Action::Word(">"),
        ])
    }
    fn where_clause(&mut self, view: View<'a>, list: List) -> Result<(), LexError> {
        if list.start == list.end {
            return Ok(());
        }
        self.enqueue(&[Action::Word(" where "), view.ty().sequence(list, ", ")])
    }
    fn fields(&mut self, view: View<'a>, shape: Shape, fields: List) -> Result<(), LexError> {
        use Action::Word as W;
        match shape {
            Shape::Unit => Ok(()),
            Shape::Tuple => self.enqueue(&[W("("), view.ty().sequence(fields, ", "), W(")")]),
            Shape::Record => self.enqueue(&[W(" { "), view.ty().sequence(fields, ", "), W(" }")]),
        }
    }
    fn use_tree(
        &mut self,
        view: View<'a>,
        index: u32,
        span: crate::syntax::token_tree::Span,
    ) -> Result<(), LexError> {
        use Action::Word as W;
        let tree = *view
            .arena
            .uses()
            .get(index as usize)
            .ok_or_else(|| error(span.start, "writer use tree row is missing"))?;
        if let Some(rename) = tree.rename {
            self.enqueue(&[W(" as "), view.n(rename)])?;
        }
        if tree.glob.is_some() {
            self.enqueue(&[W("::*")])?;
        } else if tree.children.start != tree.children.end {
            self.enqueue(&[W("::{"), view.sequence(tree.children, ", "), W("}")])?;
        }
        if let Some(prefix) = tree.prefix {
            self.enqueue(&[view.n(prefix)])?;
        }
        Ok(())
    }
}
fn safety(safety: Safety) -> &'static str {
    match safety {
        Safety::Default => "",
        Safety::Safe => "safe ",
        Safety::Unsafe => "unsafe ",
    }
}
