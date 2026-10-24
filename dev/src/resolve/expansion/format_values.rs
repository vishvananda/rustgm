//! A literal result is tied to its actual local formatter declaration and call.
use super::{Body, Builtin, Engine, Error, FileId, Input, NodeId, Report, Sources};
use crate::{
    expand::builtin,
    resolve::{
        bindings::{Source, panic_contracts},
        unit::NodeKey,
    },
    syntax::{ast::Kind, interpolation::Inputs, token_tree::error},
};
impl<S: Sources> Engine<'_, S> {
    fn format_definition(&self, index: usize) -> Result<Builtin, Error> {
        let definition = self
            .definitions
            .get(index)
            .ok_or_else(|| Error::unlocated("missing actual formatting definition"))?;
        let Body::Builtin(role) = definition.body else {
            return Err(Error::unlocated("formatting changed its actual role"));
        };
        if !role.formatting() {
            return Err(Error::unlocated("formatting changed its actual role"));
        }
        let key = self
            .paths
            .checked_source_macro(index, definition)
            .map_err(Error::unlocated)?;
        let header =
            panic_contracts::published(&self.loader.package, key, definition.header.as_ref(), 1)?;
        if header.builtin != Some(role) || header.edition != definition.edition {
            return Err(Error::unlocated(
                "formatting differs from its defining header",
            ));
        }
        Ok(role)
    }
    pub(super) fn prepare_format_input(
        &self,
        file: FileId,
        capture: u32,
        definition: usize,
    ) -> Result<(), Error> {
        if !matches!(self.definitions[definition].body,Body::Builtin(role)if role.formatting()) {
            return Ok(());
        }
        self.format_definition(definition)?;
        let forest = &self.loader.package.unit(file).configured.ast.captures[capture as usize];
        let source =
            self.loader.package.source_core().ok_or_else(|| {
                Error::at(file, 0, "formatting requires its local source provider")
            })?;
        // Preflight before flattening/parsing the real argument forest. Actual
        // opaque payload copies are separately paid through their registry view.
        source
            .macros
            .charge(
                forest.nodes().len(),
                forest.payload_bytes().saturating_mul(20),
            )
            .map_err(|message| Error::at(file, 0, message))
    }
    pub(super) fn charge_format_opaque(
        &self,
        file: FileId,
        definition: usize,
        input: &Input,
    ) -> Result<(), Error> {
        if !matches!(self.definitions[definition].body,Body::Builtin(role)if role.formatting()) {
            return Ok(());
        }
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| Error::at(file, 0, "missing formatting provider"))?;
        let view = self.captures.view(|id| {
            self.loader
                .package
                .units
                .get(id.0 as usize)
                .map(|unit| unit.source.text())
        });
        let charge = |steps, bytes| {
            source
                .macros
                .charge(steps, bytes)
                .map_err(|message| error(0, message))
        };
        let counted = builtin::choice::Counted {
            inputs: &view,
            charge: &charge,
        };
        for &token in &input.tokens {
            counted
                .get(token)
                .map_err(|error| Error::at(file, error.offset, error.message))?;
        }
        Ok(())
    }
    pub(super) fn format_literal_value(
        &mut self,
        index: usize,
        definition: usize,
        root: NodeId,
    ) -> Result<builtin::Expansion, Error> {
        let role = self.format_definition(definition)?;
        let call = &self.calls[index];
        let (file, grammar) = (call.file, call.grammar);
        let Some(Source::Core(source)) = self.loader.package.source.as_deref() else {
            return Err(Error::at(
                file,
                0,
                "formatting requires its local source provider",
            ));
        };
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
            .map_err(|error| Error::at(file, error.offset, error.message))?;
        let crate::literal::Value::String(bytes) = value else {
            return Err(Error::at(
                file,
                0,
                "formatting requires an actual string literal",
            ));
        };
        source
            .macros
            .charge(
                bytes.len().saturating_mul(3).saturating_add(1),
                bytes
                    .len()
                    .saturating_add(1)
                    .saturating_mul(32)
                    .saturating_add(8 * size_of::<crate::syntax::ast::Node>())
                    .saturating_add(size_of::<builtin::Expansion>() + 512),
            )
            .map_err(|message| Error::at(file, 0, message))?;
        let constructor = self
            .loader
            .package
            .names
            .intern("from_str")
            .map_err(|message| Error::at(file, 0, message))?;
        builtin::format_literal::expand(
            &unit.configured.ast,
            root,
            |literal| unit.literal(literal),
            role == Builtin::FormatArgsNl,
            constructor,
            grammar,
            self.config,
        )
        .map_err(|error| Error::at(file, error.offset, error.message))
    }
}
impl Report {
    /// Charge the cold semantic continuation through the existing resolver-owned
    /// source budget after authenticating its actual produced value.
    pub(crate) fn prepare_formatting_value(&self, key: NodeKey, bytes: usize) -> Result<(), Error> {
        self.formatting_value(key)?;
        self.package
            .source_core()
            .ok_or_else(|| Error::at(key.file, 0, "missing actual formatting provider"))?
            .macros
            .charge(1, bytes)
            .map_err(|message| Error::at(key.file, 0, message))
    }
    /// Classification and provenance only; ordinary constructor proof is separate.
    pub(crate) fn formatting_value(&self, key: NodeKey) -> Result<Builtin, Error> {
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
                kind: crate::support::library_types::LibraryValue::FormatLiteral { .. },
                value: Some(_)
            }
        ) || unit.configured.active.get(key.node.get() as usize) != Some(&true)
        {
            return Err(failure("formatting changed its actual result tag"));
        }
        let source = self
            .package
            .source_core()
            .ok_or_else(|| failure("missing actual formatting provider"))?;
        let raw = crate::resolve::unit::contexts::at_counted(
            &self.package.units,
            key.file,
            actual.span.start,
            |steps, _, _| source.macros.charge(steps, 0),
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
        if selected != key.node {
            return Err(failure("formatting value is not its actual call result"));
        }
        Ok(role)
    }
}
