//! A literal result is tied to its authenticated formatter declaration and call.
use super::{Body, Builtin, Engine, Error, NodeId, Report, Sources};
use crate::{
    expand::builtin,
    lex::token::Punctuation,
    literal::{Value, value::IntegerType},
    resolve::unit::{NodeKey, Unit},
    support::hash::HashMap,
    syntax::{ast::Kind, token_tree::error},
};
use std::borrow::Cow;
impl<S: Sources> Engine<'_, S> {
    pub(super) fn fold_literal_format(
        &self,
        index: usize,
        root: NodeId,
        newline: bool,
    ) -> Result<Option<String>, Error> {
        let call = &self.calls[index];
        let unit = self.loader.package.unit(call.file);
        let ast = &unit.configured.ast;
        let Kind::Expansion {
            children,
            tail: None,
        } = ast.node(root).kind
        else {
            return Ok(None);
        };
        let list = ast.list(children);
        let Some((&first, tail)) = list.split_first() else {
            return Ok(None);
        };
        if tail.is_empty() {
            return Ok(None);
        }
        let Kind::Literal(template_literal) = ast.node(first).kind else {
            return Ok(None);
        };
        let Value::String(template_bytes) = unit
            .literal(template_literal)
            .map_err(|e| Error::at(call.file, e.offset, e.message))?
        else {
            return Ok(None);
        };
        let Ok(template) = std::str::from_utf8(&template_bytes) else {
            return Ok(None);
        };
        for &entry in tail {
            let value_node = match ast.node(entry).kind {
                Kind::Binary {
                    op: Punctuation::Equal,
                    right,
                    ..
                } => right,
                _ => entry,
            };
            if !matches!(ast.node(value_node).kind, Kind::Literal(_)) {
                return Ok(None);
            }
        }
        let source = self
            .loader
            .package
            .require_format_contracts()
            .map_err(|message| call.diagnostic(message))?;
        source
            .charge(
                template.len().saturating_add(tail.len()).saturating_add(1),
                0,
            )
            .map_err(|message| call.diagnostic(message))?;
        let original = ast.node(call.visit.node).span;
        let payer = |steps, bytes| {
            source
                .charge(steps, bytes)
                .map_err(|message| error(original.start, message))
        };
        let actual =
            super::argument_bindings::collect(unit, call.file, root, call.capture, &payer)?;
        let Some(pieces) = collect_literal_pieces(unit, call.file, &actual.values)? else {
            return Ok(None);
        };
        let Some(named) = index_named_arguments(&actual.names) else {
            return Ok(None);
        };
        Ok(render_folded_template(template, &pieces, &named, newline))
    }
    pub(super) fn format_literal_value(
        &mut self,
        index: usize,
        definition: usize,
        root: NodeId,
    ) -> Result<builtin::Expansion, Error> {
        let role = self.source_eager_definition(definition)?;
        if !role.formatting() {
            return Err(Error::unlocated(
                "formatting changed its actual builtin role",
            ));
        }
        let call = &self.calls[index];
        let (file, grammar) = (
            call.file,
            call.grammar
                .bang()
                .map_err(|message| call.diagnostic(message))?,
        );
        let config = self.loader.package.crate_config(self.config, file).1;
        let newline = role == Builtin::FormatArgsNl;
        if let Some(folded) = self.fold_literal_format(index, root, newline)? {
            self.charge_literal_expansion(folded.len(), file)?;
            let constructor = self
                .loader
                .package
                .names
                .intern("from_str")
                .map_err(|message| Error::at(file, 0, message))?;
            return builtin::format_literal::expand_text(folded, constructor, grammar, config)
                .map_err(|e| Error::at(file, e.offset, e.message));
        }
        let unit = &self.loader.package.units[file.0 as usize];
        let Kind::Expansion {
            children,
            tail: None,
        } = unit.configured.ast.node(root).kind
        else {
            return Err(Error::at(file, 0, "missing actual formatting arguments"));
        };
        let [argument] = unit.configured.ast.list(children) else {
            return Err(Error::at(
                file,
                0,
                "formatting arguments require actual source argument construction",
            ));
        };
        let Kind::Literal(literal) = unit.configured.ast.node(*argument).kind else {
            return Err(Error::at(
                file,
                0,
                "formatting requires an actual string literal",
            ));
        };
        let value = unit
            .literal(literal)
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        let Value::String(bytes) = value else {
            return Err(Error::at(
                file,
                0,
                "formatting requires an actual string literal",
            ));
        };
        let byte_len = bytes.len();
        self.charge_literal_expansion(byte_len, file)?;
        let constructor = self
            .loader
            .package
            .names
            .intern("from_str")
            .map_err(|message| Error::at(file, 0, message))?;
        let unit = &self.loader.package.units[file.0 as usize];
        builtin::format_literal::expand(
            &unit.configured.ast,
            root,
            |literal| unit.literal(literal),
            newline,
            constructor,
            grammar,
            config,
        )
        .map_err(|e| Error::at(file, e.offset, e.message))
    }
    fn charge_literal_expansion(&self, len: usize, file: super::FileId) -> Result<(), Error> {
        self.loader
            .package
            .require_format_contracts()
            .map_err(|message| Error::at(file, 0, message))?
            .charge(
                len.saturating_mul(3).saturating_add(1),
                len.saturating_add(1)
                    .saturating_mul(32)
                    .saturating_add(8 * size_of::<crate::syntax::ast::Node>())
                    .saturating_add(size_of::<builtin::Expansion>() + 512),
            )
            .map_err(|message| Error::at(file, 0, message))
    }
}
fn collect_literal_pieces<'a>(
    unit: &'a Unit,
    file: super::FileId,
    values: &[NodeId],
) -> Result<Option<Vec<Cow<'a, str>>>, Error> {
    let mut pieces = Vec::with_capacity(values.len());
    for &node in values {
        let Kind::Literal(literal) = unit.configured.ast.node(node).kind else {
            return Ok(None);
        };
        match unit
            .literal(literal)
            .map_err(|e| Error::at(file, e.offset, e.message))?
        {
            Value::String(bytes) => {
                let Ok(text) = (match bytes {
                    Cow::Borrowed(slice) => std::str::from_utf8(slice).map(Cow::Borrowed),
                    Cow::Owned(owned) => String::from_utf8(owned)
                        .map(Cow::Owned)
                        .map_err(|e| e.utf8_error()),
                }) else {
                    return Ok(None);
                };
                pieces.push(text);
            }
            Value::Integer(integer) => {
                let target_type = integer.suffix.unwrap_or(IntegerType::I32);
                let Ok(materialized) = integer.materialize(target_type, false) else {
                    return Ok(None);
                };
                if materialized.overflow {
                    return Ok(None);
                }
                pieces.push(Cow::Owned(integer.magnitude.to_string()));
            }
            _ => return Ok(None),
        }
    }
    Ok(Some(pieces))
}
fn index_named_arguments<'a>(names: &'a [Option<Cow<'a, str>>]) -> Option<HashMap<&'a str, usize>> {
    let mut map = HashMap::default();
    let mut seen_named = false;
    for (ordinal, label) in names.iter().enumerate() {
        if let Some(label) = label {
            seen_named = true;
            if map.insert(label.as_ref(), ordinal).is_some() {
                return None;
            }
        } else if seen_named {
            return None;
        }
    }
    Some(map)
}
fn render_folded_template(
    template: &str,
    pieces: &[Cow<'_, str>],
    named: &HashMap<&str, usize>,
    newline: bool,
) -> Option<String> {
    let mut output = String::with_capacity(template.len().saturating_add(usize::from(newline)));
    let mut used = vec![false; pieces.len()];
    let mut implicit = 0usize;
    let mut chars = template.chars();
    while let Some(character) = chars.next() {
        match character {
            '{' => {
                if chars.clone().next() == Some('{') {
                    chars.next();
                    output.push('{');
                    continue;
                }
                let start = chars.as_str();
                let mut found_end = false;
                let mut spec_len = 0usize;
                for inner in chars.by_ref() {
                    if inner == '}' {
                        found_end = true;
                        break;
                    }
                    if inner == '{' {
                        return None;
                    }
                    spec_len += inner.len_utf8();
                }
                if !found_end {
                    return None;
                }
                let spec = &start[..spec_len];
                let selector = spec.strip_suffix(':').unwrap_or(spec);
                if selector.contains(':') {
                    return None;
                }
                let ordinal = if selector.is_empty() {
                    let current = implicit;
                    implicit += 1;
                    current
                } else if selector.bytes().all(|byte| byte.is_ascii_digit()) {
                    selector.parse::<usize>().ok()?
                } else if unicode_normalization::is_nfc(selector) {
                    *named.get(selector)?
                } else {
                    return None;
                };
                if ordinal >= pieces.len() {
                    return None;
                }
                used[ordinal] = true;
                output.push_str(&pieces[ordinal]);
            }
            '}' => {
                if chars.next() == Some('}') {
                    output.push('}');
                } else {
                    return None;
                }
            }
            other => output.push(other),
        }
    }
    if used.iter().any(|&flag| !flag) {
        return None;
    }
    if newline {
        output.push('\n');
    }
    Some(output)
}
impl Report {
    /// Charge the cold semantic continuation through the existing resolver-owned
    /// source budget after authenticating its actual produced value.
    pub(crate) fn prepare_formatting_value(&self, key: NodeKey, bytes: usize) -> Result<(), Error> {
        self.formatting_value(key)?;
        self.package
            .require_format_contracts()
            .map_err(|message| Error::at(key.file, 0, message))?
            .charge(1, bytes)
            .map_err(|message| Error::at(key.file, 0, message))
    }
    /// Classification and provenance only; ordinary constructor proof is separate.
    pub(crate) fn formatting_value(&self, key: NodeKey) -> Result<Builtin, Error> {
        self.formatting_source(key).map(|(role, _)| role)
    }
    pub(crate) fn formatting_root(&self, key: NodeKey) -> Result<NodeKey, Error> {
        self.formatting_source(key).map(|(_, root)| root)
    }
    fn formatting_source(&self, key: NodeKey) -> Result<(Builtin, NodeKey), Error> {
        let unit = self
            .package
            .units
            .get(key.file.0 as usize)
            .ok_or_else(|| Error::at(key.file, 0, "missing formatting source"))?;
        let actual = unit
            .configured
            .ast
            .nodes
            .get(key.node.get() as usize)
            .ok_or_else(|| Error::at(key.file, 0, "missing formatting result"))?;
        let failure = |message| Error::at(key.file, actual.span.start, message);
        if !matches!(
            actual.kind,
            Kind::LibraryValue {
                kind: crate::support::library_types::LibraryValue::FormatLiteral { .. }
                    | crate::support::library_types::LibraryValue::FormatOutput { .. }
                    | crate::support::library_types::LibraryValue::FormatSuperLet { .. },
                value: Some(_)
            } | Kind::LibraryValue {
                kind: crate::support::library_types::LibraryValue::FormatConstructor { .. },
                value: None
            }
        ) || unit.configured.active.get(key.node.get() as usize) != Some(&true)
        {
            return Err(failure("formatting changed its actual result tag"));
        }
        let source = self.package.require_format_contracts().map_err(failure)?;
        let raw = crate::resolve::unit::contexts::at_counted(
            &self.package.units,
            key.file,
            actual.span.start,
            |steps, _, _| source.charge(steps, 0),
        )?;
        let context = self
            .package
            .marks
            .outer(raw)
            .map_err(failure)?
            .ok_or_else(|| failure("formatting result has no actual expansion"))?;
        let view = super::permission_frames::FrameView {
            package: &self.package,
            definitions: &self.definitions,
            invocations: &self.invocations,
            facts: &self.resolution,
        };
        let frame = view.frame(context)?;
        let invocation = &self.invocations[context.0 as usize - 1];
        let Body::Builtin(role) = view.definition(frame.definition)?.body else {
            return Err(failure("formatting result has no actual builtin role"));
        };
        if !role.formatting()
            || invocation.node.file != key.file
            || actual.span.start < invocation.generated.start
            || actual.span.end > invocation.generated.end
        {
            return Err(failure("formatting result differs from its actual call"));
        }
        let root = unit.configured.ast.node(invocation.node.node);
        let selected = match root.kind {
            Kind::LibraryValue { .. } => invocation.node.node,
            Kind::Expansion {
                children,
                tail: Some(value),
            } if children.start == children.end => value,
            _ => return Err(failure("formatting call has no canonical result")),
        };
        if matches!(
            unit.configured
                .ast
                .nodes
                .get(selected.get() as usize)
                .map(|node| node.kind),
            Some(Kind::LibraryValue {
                kind: crate::support::library_types::LibraryValue::FormatOutput { .. },
                ..
            })
        ) {
            self.check_format_site(key, selected, invocation.generated)?;
        } else if selected != key.node
            || !matches!(
                actual.kind,
                Kind::LibraryValue {
                    kind: crate::support::library_types::LibraryValue::FormatLiteral { .. },
                    value: Some(_)
                }
            )
        {
            return Err(failure("formatting value is not its actual call result"));
        }
        Ok((
            role,
            NodeKey {
                node: selected,
                ..key
            },
        ))
    }
}
