//! A CFI type encoding is retained separately from representation and layout.
use crate::{
    literal::{self, Value},
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        model::{DefId, DefKind, Namespace},
        package::{Error, ModuleId},
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{diagnostics::WarningCode, hash::HashMap, retained::Growth},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::{Forest, Span},
    },
};

struct Encoding {
    span: Span,
    text: Box<str>,
}

#[derive(Default)]
pub(super) struct Draft {
    requests: HashMap<ModuleId, bool>,
    pending: HashMap<NodeKey, Encoding>,
}

impl Draft {
    pub(super) fn consume(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        (owner, declaration): (NodeKey, NodeKey),
        (attribute, retained): (Attribute, usize),
        forest: &Forest,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        facts.reserve_must_use_source(owner, attribute.span)?;
        let error = |message| Error::at(declaration.file, attribute.span.start, message);
        if attribute.unsafe_wrapper || attribute.style != Style::Outer {
            return Err(error("`cfi_encoding` requires a normal outer attribute"));
        }
        if !self.requested(report, facts, declaration)?
            && !allowances.allows_attribute(
                owner,
                retained,
                attribute,
                &FeatureName::new("cfi_encoding").map_err(error)?,
            )?
        {
            return Err(error(
                "cfi_encoding requires this crate's cfi_encoding feature",
            ));
        }
        let unit = report.package.unit(declaration.file);
        let text = encoding(unit.source.text(), attribute, forest).map_err(error)?;
        let ast = &unit.configured.ast;
        if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
            if matches!(ast.items[index as usize].kind, ItemKind::Struct { .. }
                | ItemKind::Enum { .. } | ItemKind::Union { .. } | ItemKind::ForeignType))
        {
            return facts.must_use_diagnostic(
                report,
                owner,
                attribute.span.start,
                0,
                WarningCode::UnusedAttributes,
                "cfi_encoding applies only to struct, enum, union and foreign type declarations"
                    .into(),
            );
        }
        if self.pending.contains_key(&declaration) {
            return Err(error("duplicate cfi_encoding attribute"));
        }
        let growth =
            Growth::map::<NodeKey, Encoding>(self.pending.len(), self.pending.capacity(), 1);
        crate::sema::header_attributes::reserve_source_contracts(
            facts,
            growth.work,
            growth.bytes.saturating_add(text.len()),
        )?;
        self.pending
            .try_reserve(1)
            .map_err(|_| error("cfi_encoding draft allocation failed"))?;
        self.pending.insert(
            declaration,
            Encoding {
                span: attribute.span,
                text,
            },
        );
        Ok(())
    }

    fn requested(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        declaration: NodeKey,
    ) -> Result<bool, Error> {
        let root = report.package.file_crate_root(declaration.file);
        if let Some(&requested) = self.requests.get(&root) {
            return Ok(requested);
        }
        let requested = report.package.requested_cfi_encoding(declaration.file)?;
        let growth =
            Growth::map::<ModuleId, bool>(self.requests.len(), self.requests.capacity(), 1);
        crate::sema::header_attributes::reserve_source_contracts(facts, growth.work, growth.bytes)?;
        self.requests
            .try_reserve(1)
            .map_err(|_| Error::unlocated("cfi_encoding request cache allocation failed"))?;
        self.requests.insert(root, requested);
        Ok(requested)
    }

    pub(super) fn bind(mut self, report: &Report, facts: &mut Declarations) -> Result<(), Error> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let count = self.pending.len();
        let growth = Growth::map::<DefId, Box<str>>(0, 0, count);
        let definitions = &report.resolution.definitions;
        crate::sema::header_attributes::reserve_source_contracts(
            facts,
            definitions.len().saturating_add(growth.work),
            growth
                .bytes
                .saturating_add(size_of::<HashMap<DefId, Box<str>>>()),
        )?;
        let mut encodings = HashMap::default();
        encodings
            .try_reserve(count)
            .map_err(|_| Error::unlocated("cfi_encoding definition allocation failed"))?;
        for (index, actual) in definitions.iter().enumerate() {
            let Some(key) = actual.node else {
                continue;
            };
            if actual.namespace != Namespace::Type {
                continue;
            }
            let Some(encoding) = self.pending.remove(&key) else {
                continue;
            };
            let error = |message| Error::at(key.file, encoding.span.start, message);
            let id =
                DefId(u32::try_from(index).map_err(|_| error("cfi_encoding identity overflow"))?);
            if !matches!(
                actual.kind,
                DefKind::Struct | DefKind::Enum | DefKind::ForeignType
            ) || (actual.kind == DefKind::ForeignType)
                != report.resolution.foreign_owners.contains_key(&id)
            {
                return Err(error(
                    "cfi_encoding requires its actual nominal or foreign type",
                ));
            }
            encodings.insert(id, encoding.text);
        }
        if let Some((key, encoding)) = self
            .pending
            .into_iter()
            .min_by_key(|(key, value)| (key.file.0, value.span.start))
        {
            return Err(Error::at(
                key.file,
                encoding.span.start,
                "cfi_encoding lost its actual type declaration",
            ));
        }
        facts
            .must_use_contracts
            .get_or_insert_default()
            .cfi_encodings = Some(Box::new(encodings));
        Ok(())
    }
}

fn encoding(source: &str, attribute: Attribute, forest: &Forest) -> Result<Box<str>, &'static str> {
    const ERROR: &str = "cfi_encoding requires a string literal";
    let Input::Equals(range) = attribute.input else {
        return Err(ERROR);
    };
    if range.end != range.start + 1 {
        return Err(ERROR);
    }
    let token = forest.token(range.start).ok_or(ERROR)?;
    let Some(Value::String(bytes)) = literal::interpret(source, token).map_err(|e| e.message)?
    else {
        return Err(ERROR);
    };
    let text = String::from_utf8(bytes.into_owned()).map_err(|_| ERROR)?;
    if text.trim().is_empty() {
        return Err("cfi_encoding requires a nonempty string literal");
    }
    Ok(text.into_boxed_str())
}

impl Declarations {
    /// The original decoded encoding of the actual local type, if present.
    #[must_use]
    pub fn cfi_encoding(&self, definition: DefId) -> Option<&str> {
        self.must_use_contracts
            .as_deref()?
            .cfi_encodings
            .as_deref()?
            .get(&definition)
            .map(Box::as_ref)
    }
}
