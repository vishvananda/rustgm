//! Ordinary typed calls and bindings; draft inputs are adopted at installation.
use super::{Charge, Config, Draft, Expansion, MacroContext, NameId, template::Template};
use crate::{
    expand::crate_syntax,
    lex::token::LexError,
    literal::Value,
    support::{
        library_types::{FormatConstructor, LibraryValue},
        retained::Growth,
    },
    syntax::{
        ast::{Ast, BlockFlavor, Kind, List, Node, NodeId, Unary},
        token_tree::{Span, error},
    },
};
pub(super) struct Output<'a> {
    ast: Ast<'static>,
    source: String,
    sites: Vec<NodeId>,
    inputs: Vec<(NodeId, usize)>,
    names: [NameId; 4],
    captured: Vec<NodeId>,
    charge: &'a Charge<'a>,
}
impl<'a> Output<'a> {
    pub(super) fn build(
        template: Template<'_>,
        count: usize,
        names: [NameId; 4],
        symbols: &mut crate::support::names::Names,
        grammar: MacroContext,
        config: &Config,
        charge: &'a Charge<'a>,
    ) -> Result<Draft, LexError> {
        let explicit = count;
        let count = count.saturating_add(template.captured.len());
        let constructor_count = template.arguments.len();
        let site_count = count.saturating_add(constructor_count).saturating_add(3);
        charge(
            site_count.saturating_add(count),
            site_count
                .saturating_mul(size_of::<NodeId>())
                .saturating_add(count.saturating_mul(size_of::<(NodeId, usize)>()))
                .saturating_add(
                    count
                        .saturating_add(constructor_count)
                        .saturating_mul(size_of::<NodeId>()),
                ),
        )?;
        let mut output = Self {
            ast: Ast::default(),
            source: String::new(),
            sites: vec![NodeId::new(0); site_count],
            inputs: Vec::with_capacity(count),
            names,
            captured: Vec::new(),
            charge,
        };
        let first = output.input_tuple(explicit, &template.captured, constructor_count, symbols)?;
        let second = output.argument_array(&template.arguments)?;
        charge(1, size_of::<Value<'_>>())?;
        output
            .ast
            .literals
            .push(Value::ByteString(template.bytes.into()));
        let template = output.node("template", Kind::Literal(0))?;
        let items = output.name("__format_items")?;
        let items = output.node(
            "&",
            Kind::Unary {
                op: Unary::Borrow,
                operand: items,
            },
        )?;
        let callee = output.constructor(2, FormatConstructor::Arguments)?;
        let arguments = output.list(&[template, items])?;
        let call = output.node("call", Kind::Call { callee, arguments })?;
        // Input evaluation and Argument construction are outside this unsafe scope.
        let unsafe_call = output.node(
            "{}",
            Kind::Block {
                flavor: BlockFlavor::Unsafe,
                statements: List { start: 0, end: 0 },
                tail: Some(call),
            },
        )?;
        let statements = output.list(&[first, second])?;
        let block = output.node(
            "{}",
            Kind::Expansion {
                children: statements,
                tail: Some(unsafe_call),
            },
        )?;
        let site_nodes = std::mem::take(&mut output.sites);
        let sites = output.list(&site_nodes)?;
        output.ast.root = output.node(
            "format_output",
            Kind::LibraryValue {
                kind: LibraryValue::FormatOutput {
                    sites: crate::support::library_types::FormatSites {
                        start: sites.start,
                        end: sites.end,
                    },
                },
                value: Some(block),
            },
        )?;
        if grammar == MacroContext::Statements {
            let tail = output.ast.root;
            output.ast.root = output.node(
                "format_statement",
                Kind::Expansion {
                    children: List { start: 0, end: 0 },
                    tail: Some(tail),
                },
            )?;
        }
        // All literal payloads are owned and this skeleton has no attributes or
        // deferred token forests; names are interpreted after source installation.
        let syntax = crate_syntax::configure("", output.ast, config)?;
        Ok(Draft {
            expansion: Expansion {
                source: output.source,
                syntax,
            },
            inputs: output.inputs,
            captured: output.captured,
        })
    }
    fn input_tuple(
        &mut self,
        explicit: usize,
        captures: &[&str],
        constructors: usize,
        symbols: &mut crate::support::names::Names,
    ) -> Result<NodeId, LexError> {
        let count = explicit.saturating_add(captures.len());
        (self.charge)(
            captures.len(),
            captures.len().saturating_mul(size_of::<NodeId>()),
        )?;
        self.captured = Vec::with_capacity(captures.len());
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            let slot = Self::index(3 + constructors + index)?;
            let input = if index < explicit {
                self.node(
                    "input",
                    Kind::LibraryValue {
                        kind: LibraryValue::FormatInput { slot },
                        value: None,
                    },
                )?
            } else {
                let spelling = captures[index - explicit];
                let symbol = symbols
                    .intern_exact_paid(spelling, &mut |growth| {
                        (self.charge)(growth.work, growth.bytes).map_err(|error| error.message)
                    })
                    .map_err(|message| error(0, message))?;
                let span = self.text(spelling)?;
                let identifier = self.at(span, Kind::Name(span, Some(symbol)))?;
                self.captured.push(identifier);
                self.node("input", Kind::Group(identifier))?
            };
            self.sites[slot as usize] = input;
            if index < explicit {
                self.inputs.push((input, index));
            }
            values.push(self.node(
                "&",
                Kind::Unary {
                    op: Unary::Borrow,
                    operand: input,
                },
            )?);
        }
        let list = self.list(&values)?;
        let tuple = self.node("tuple", Kind::Tuple(list))?;
        self.binding("__format_values", tuple, 0)
    }
    fn argument_array(&mut self, order: &[super::template::Argument]) -> Result<NodeId, LexError> {
        let mut arguments = Vec::with_capacity(order.len());
        for (ordinal, input) in order.iter().enumerate() {
            let receiver = self.name("__format_values")?;
            let name = self.text(&input.input.to_string())?;
            let field = self.node(
                "field",
                Kind::Field {
                    value: receiver,
                    name,
                },
            )?;
            let callee = self.constructor(3 + ordinal, input.constructor)?;
            let arguments_list = self.list(&[field])?;
            arguments.push(self.node(
                "call",
                Kind::Call {
                    callee,
                    arguments: arguments_list,
                },
            )?);
        }
        let list = self.list(&arguments)?;
        let array = self.node("array", Kind::Array(list))?;
        self.binding("__format_items", array, 1)
    }
    fn index(index: usize) -> Result<u32, LexError> {
        u32::try_from(index).map_err(|_| error(0, "formatting syntax index overflow"))
    }
    fn text(&mut self, text: &str) -> Result<Span, LexError> {
        let growth = Growth::vector::<u8>(
            self.source.len(),
            self.source.capacity(),
            text.len().saturating_add(1),
        );
        (self.charge)(text.len().saturating_add(growth.work), growth.bytes)?;
        self.source
            .try_reserve(text.len().saturating_add(1))
            .map_err(|_| error(0, "formatting image allocation failed"))?;
        let start = Self::index(self.source.len())?;
        self.source.push_str(text);
        let end = Self::index(self.source.len())?;
        self.source.push(' ');
        Ok(Span { start, end })
    }
    fn node(&mut self, spelling: &str, kind: Kind) -> Result<NodeId, LexError> {
        let span = self.text(spelling)?;
        self.at(span, kind)
    }
    fn at(&mut self, span: Span, kind: Kind) -> Result<NodeId, LexError> {
        let growth = Growth::vector::<Node>(self.ast.nodes.len(), self.ast.nodes.capacity(), 1);
        (self.charge)(growth.work.saturating_add(1), growth.bytes)?;
        self.ast
            .nodes
            .try_reserve(1)
            .map_err(|_| error(0, "formatting node allocation failed"))?;
        let node = NodeId::checked(Self::index(self.ast.nodes.len())?)
            .ok_or_else(|| error(0, "formatting node overflow"))?;
        self.ast.nodes.push(Node { span, kind });
        Ok(node)
    }
    fn list(&mut self, nodes: &[NodeId]) -> Result<List, LexError> {
        let growth =
            Growth::vector::<NodeId>(self.ast.edges.len(), self.ast.edges.capacity(), nodes.len());
        (self.charge)(nodes.len().saturating_add(growth.work), growth.bytes)?;
        self.ast
            .edges
            .try_reserve(nodes.len())
            .map_err(|_| error(0, "formatting edge allocation failed"))?;
        let start = Self::index(self.ast.edges.len())?;
        self.ast.edges.extend_from_slice(nodes);
        Ok(List {
            start,
            end: Self::index(self.ast.edges.len())?,
        })
    }
    fn name(&mut self, name: &str) -> Result<NodeId, LexError> {
        let span = self.text(name)?;
        self.at(span, Kind::Name(span, None))
    }
    fn binding(&mut self, name: &str, value: NodeId, slot: usize) -> Result<NodeId, LexError> {
        let span = self.text(name)?;
        let pattern = self.at(
            span,
            Kind::Binding {
                name: span,
                symbol: None,
                mutable: false,
                by_ref: false,
                subpattern: None,
            },
        )?;
        let value = self.node(
            "let",
            Kind::Let {
                pattern,
                ty: None,
                value: Some(value),
                otherwise: None,
            },
        )?;
        let node = self.node(
            "super_let",
            Kind::LibraryValue {
                kind: LibraryValue::FormatSuperLet {
                    slot: Self::index(slot)?,
                },
                value: Some(value),
            },
        )?;
        self.sites[slot] = node;
        Ok(node)
    }
    fn constructor(
        &mut self,
        slot: usize,
        constructor: FormatConstructor,
    ) -> Result<NodeId, LexError> {
        let index = match constructor {
            FormatConstructor::Arguments => 0,
            FormatConstructor::Debug => 1,
            FormatConstructor::Display => 2,
            FormatConstructor::LowerHex => 3,
        };
        let node = self.node(
            constructor.name(),
            Kind::LibraryValue {
                kind: LibraryValue::FormatConstructor {
                    slot: Self::index(slot)?,
                    constructor,
                    name: self.names[index],
                },
                value: None,
            },
        )?;
        self.sites[slot] = node;
        Ok(node)
    }
}
