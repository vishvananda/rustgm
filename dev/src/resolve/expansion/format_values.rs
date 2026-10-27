//! A literal result is tied to its actual local formatter declaration and call.
use super::{Body, Builtin, Engine, Error, NodeId, Report, Sources};
use crate::{
    expand::builtin,
    resolve::{bindings::Source, unit::NodeKey},
    syntax::ast::Kind,
};
impl<S: Sources> Engine<'_, S> {
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
