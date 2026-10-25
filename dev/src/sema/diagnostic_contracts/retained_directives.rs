//! Consume retained attribute forests once and bind formals to real declarations.
use super::*;
use crate::{literal::{self, Value}, syntax::{ast::Kind, attribute::Input, item::ItemKind, token_tree::{Delimiter, TreeKind, TreeRange}}};

pub(super) struct Parser<'a, 'b> {
    pub report: &'a Report,
    pub facts: &'b mut Declarations,
    pub key: NodeKey,
    pub offset: u32,
    pub forest: &'a Forest,
    pub source: &'a str,
    pub parameters: Vec<(DefId, usize)>,
    issue: u32,
}
impl Parser<'_, '_> {
    pub(super) fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error> { super::charge(self.facts, self.key, steps, bytes) }
    pub(super) fn warn(&mut self, code: WarningCode, message: &'static str) -> Result<(), Error> {
        let issue = self.issue;
        self.issue = self.issue.saturating_add(1);
        super::warning(self.facts, self.report, self.key, self.offset, issue, code, message)
    }
    fn malformed(&mut self) -> Result<(), Error> {
        self.warn(WarningCode::MalformedDiagnosticAttributes, "malformed on_unimplemented attribute")
    }
    pub(super) fn slot(&self, name: &str) -> Option<Slot> {
        match name {
            "Self" => Some(Slot::Subject),
            "This:resolved" => Some(Slot::Trait),
            _ => self.parameters.iter().find_map(|&(binding, ordinal)| {
                let definition = self.report.resolution.definitions.get(binding.0 as usize)?;
                (self.report.package.names.get(definition.name) == name).then_some(Slot::Parameter { binding, ordinal })
            }),
        }
    }
    pub(super) fn literal(&mut self, index: u32) -> Result<Option<String>, Error> {
        let Some(token) = self.forest.token(index) else { return Ok(None) };
        self.charge(1, 2 * token.end.saturating_sub(token.start) as usize)?;
        let Some(Value::String(value)) = literal::interpret(self.source, token).map_err(|error| Error::at(self.key.file, error.offset, error.message))? else { return Ok(None) };
        Ok(Some(String::from_utf8(value.into_owned()).map_err(|_| Error::at(self.key.file, self.offset, "diagnostic literal is not UTF-8"))?))
    }
    fn parts(&mut self, range: TreeRange) -> Result<Vec<TreeRange>, Error> {
        let mut cursor = range.start;
        let mut values = Vec::new();
        while cursor < range.end {
            let start = cursor;
            while cursor < range.end && !self.forest.punctuation(cursor, crate::lex::token::Punctuation::Comma) {
                self.charge(1, 0)?;
                cursor = self.forest.after(cursor);
            }
            self.charge(1, 4 * size_of::<TreeRange>())?;
            values.push(TreeRange { start, end: cursor });
            cursor = cursor.saturating_add(1);
        }
        Ok(values)
    }
    fn options(&mut self, parts: &[TreeRange], directive: &mut Directive) -> Result<(), Error> {
        for range in parts {
            let Some(name) = self.forest.name(self.source, range.start) else { self.malformed()?; continue };
            if !matches!(name, "message" | "label" | "note") || range.end != range.start + 3
                || !self.forest.punctuation(range.start + 1, crate::lex::token::Punctuation::Equal) {
                self.malformed()?; continue;
            }
            let Some(value) = self.literal(range.start + 2)? else { self.malformed()?; continue };
            let Some(template) = super::template_text::parse(self, &value)? else { continue };
            if name == "note" {
                self.charge(1, 4 * size_of::<Template>())?;
                directive.notes.push(template);
            } else {
                let exists = if name == "message" { directive.message.is_some() } else { directive.label.is_some() };
                if exists { self.warn(WarningCode::MalformedDiagnosticAttributes, "duplicate on_unimplemented option")?; continue }
                if name == "message" { directive.message = Some(template) } else { directive.label = Some(template) }
            }
        }
        Ok(())
    }
    fn condition(&mut self, range: TreeRange) -> Result<Condition, Error> {
        let Some(name) = self.forest.name(self.source, range.start) else { return Ok(Condition::Unsupported) };
        if range.end != range.start + 3 || !self.forest.punctuation(range.start + 1, crate::lex::token::Punctuation::Equal) { return Ok(Condition::Unsupported) }
        self.charge(self.parameters.len(), 0)?;
        let Some(slot) = self.slot(name) else { return Ok(Condition::Unsupported) };
        let Some(value) = self.literal(range.start + 2)? else { return Ok(Condition::Unsupported) };
        Ok(match value.as_str() {
            "()" => Condition::Unit(slot),
            "unsafe fn" => Condition::UnsafeFunction(slot),
            _ => Condition::Unsupported,
        })
    }
    fn conditional(&mut self, part: TreeRange, mode: Mode) -> Result<Option<Directive>, Error> {
        if mode != Mode::Internal || self.forest.after(part.start + 1) != part.end
            || !matches!(self.forest.tree(part.start + 1).map(|tree| tree.kind), Some(TreeKind::Group { delimiter: Delimiter::Parenthesis, .. })) {
            self.malformed()?; return Ok(None);
        }
        let Some(range) = self.forest.children(part.start + 1) else { self.malformed()?; return Ok(None) };
        let nested = self.parts(range)?;
        let Some((&condition, options)) = nested.split_first() else { self.malformed()?; return Ok(None) };
        let mut directive = Directive { condition: Some(self.condition(condition)?), ..Default::default() };
        self.options(options, &mut directive)?;
        Ok(Some(directive))
    }
    fn parse(&mut self, attribute: Attribute, mode: Mode) -> Result<Vec<Directive>, Error> {
        let Input::Delimited(group) = attribute.input else { self.malformed()?; return Ok(Vec::new()) };
        let Some(TreeKind::Group { delimiter: Delimiter::Parenthesis, .. }) = self.forest.tree(group).map(|tree| tree.kind) else { self.malformed()?; return Ok(Vec::new()) };
        let Some(range) = self.forest.children(group) else { self.malformed()?; return Ok(Vec::new()) };
        let parts = self.parts(range)?;
        if parts.is_empty() || attribute.unsafe_wrapper { self.malformed()?; return Ok(Vec::new()) }
        let mut directives = Vec::new();
        let mut defaults = Vec::new();
        for part in parts {
            if self.forest.name(self.source, part.start) != Some("on") {
                self.charge(1, 4 * size_of::<TreeRange>())?; defaults.push(part); continue;
            }
            if let Some(directive) = self.conditional(part, mode)? {
                self.charge(1, 4 * size_of::<Directive>())?; directives.push(directive);
            }
        }
        let mut root = Directive::default();
        self.options(&defaults, &mut root)?;
        self.charge(1, 4 * size_of::<Directive>())?; directives.push(root);
        Ok(directives)
    }
}

pub(super) fn collect(report: &Report, facts: &mut Declarations, site: SourceSite<'_>,
    definitions: &HashMap<NodeKey, DefId>) -> Result<(), Error> {
    let SourceSite { owner: key, effect, attribute, forest, mode } = site;
    let Some((declaration, definition, item)) = checked_declaration(report, facts, site, definitions)? else { return Ok(()) };
    let unit = report.package.unit(key.file);
    let parameters = formal_slots(report, facts, key, item)?;
    let mut parser = Parser { report, facts, key, offset: attribute.span.start, forest,
        source: unit.source.text(), parameters, issue: 0 };
    let directives = parser.parse(attribute, mode)?;
    let previous = parser.facts.must_use_contracts.as_deref().and_then(|table| table.diagnostics.as_deref())
        .and_then(|table| table.declarations.get(&definition)).map(|contract| (
            contract.mode,
            contract.directives.iter().filter(|directive| directive.condition.is_none()).any(|directive| directive.message.is_some()),
            contract.directives.iter().filter(|directive| directive.condition.is_none()).any(|directive| directive.label.is_some()),
        ));
    if let Some((_, message, label)) = previous {
        for directive in directives.iter().filter(|directive| directive.condition.is_none()) {
            if message && directive.message.is_some() { parser.warn(WarningCode::MalformedDiagnosticAttributes, "duplicate on_unimplemented option")?; }
            if label && directive.label.is_some() { parser.warn(WarningCode::MalformedDiagnosticAttributes, "duplicate on_unimplemented option")?; }
        }
    }
    if previous.is_some_and(|(old, _, _)| old == Mode::Internal || mode == Mode::Internal) {
        return Err(Error::at(key.file, attribute.span.start, "multiple rustc_on_unimplemented contracts are not supported"));
    }
    if previous.is_some() {
        return Err(Error::at(key.file, attribute.span.start, "duplicate stable diagnostic contract merging is not implemented"));
    }
    charge(parser.facts, key, 1, 4 * size_of::<(DefId, Contract)>())?;
    let table = parser.facts.must_use_contracts.get_or_insert_default().diagnostics.get_or_insert_default();
    table.declarations.insert(definition, Contract { owner: key, declaration, definition, effect,
        span: attribute.span, mode, directives });
    Ok(())
}

fn checked_declaration(report: &Report, facts: &mut Declarations, site: SourceSite<'_>, definitions: &HashMap<NodeKey, DefId>)
    -> Result<Option<(NodeKey, DefId, crate::syntax::item::Item)>, Error> {
    let SourceSite { owner: key, attribute, mode, .. } = site;
    let unit = report.package.unit(key.file);
    let mut declaration = key;
    let mut remaining = unit.configured.ast.nodes.len();
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(declaration.node).kind {
        remaining = remaining.checked_sub(1).ok_or_else(|| Error::at(key.file, attribute.span.start, "diagnostic owner ancestry exceeded"))?;
        declaration.node = value;
    }
    let Some(&definition) = definitions.get(&declaration) else {
        warning(facts, report, key, attribute.span.start, 0,
            if mode == Mode::Internal { WarningCode::UnusedAttributes } else { WarningCode::MisplacedDiagnosticAttributes },
            if mode == Mode::Internal { "rustc_on_unimplemented requires a trait owner" } else { "diagnostic::on_unimplemented requires a trait owner" })?;
        return Ok(None);
    };
    let Kind::Item(index) = unit.configured.ast.node(declaration.node).kind else { return Err(Error::at(key.file, attribute.span.start, "diagnostic trait identity has no item")) };
    let item = unit.configured.ast.items[index as usize];
    if !matches!(item.kind, ItemKind::Trait {..}) { return Err(Error::at(key.file, attribute.span.start, "diagnostic identity is not its actual trait")) }
    Ok(Some((declaration, definition, item)))
}
fn formal_slots(report: &Report, facts: &mut Declarations, key: NodeKey, item: crate::syntax::item::Item) -> Result<Vec<(DefId, usize)>, Error> {
    let unit = report.package.unit(key.file);
    let mut parameters = Vec::new();
    for &node in unit.configured.ast.list(item.generics) {
        if !unit.configured.active[node.get() as usize] { continue }
        let binding = facts.node(NodeKey { node, ..key }).and_then(|fact| fact.binding)
            .ok_or_else(|| Error::at(key.file, unit.configured.ast.node(key.node).span.start, "diagnostic parameter has no actual binding"))?;
        charge(facts, key, 1, 4 * size_of::<(DefId, usize)>())?;
        parameters.push((binding, parameters.len()));
    }
    Ok(parameters)
}
