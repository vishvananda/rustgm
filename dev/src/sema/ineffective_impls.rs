//! Direct unstable impl annotations consume resolved header paths, never bodies.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::Error,
        stability::{Inventory, Level},
        unit::NodeKey,
    },
    sema::{associated::OwnerKind, model::Declarations},
    support::{diagnostics::WarningCode, retained::Growth},
    syntax::ast::{Kind, TypeFlavor},
};
#[cfg(test)]
#[path = "../../tests/invariants/ineffective_impls.rs"]
mod tests;

pub(super) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(producer) = report
        .lint_producer()
        .filter(|producer| producer.has_impl_annotations())
    else {
        return Ok(());
    };
    producer.attribute_read(
        NodeKey {
            file: report.package.modules[0].file,
            node: report.package.modules[0].root,
        },
        facts.associated.owners().len(),
    )?;
    for index in 0..facts.associated.owners().len() {
        let owner = facts.associated.owners()[index];
        let (
            Some(key),
            OwnerKind::Impl {
                target,
                trait_type: Some(trait_type),
                ..
            },
        ) = (owner.source, owner.kind)
        else {
            continue;
        };
        let Some(annotation) = producer.impl_annotation(&report.package, key)? else {
            continue;
        };
        if annotation.unstable.is_none() && annotation.bound.is_none() {
            continue;
        }
        let inventory = report
            .package
            .library_policy()
            .and_then(crate::resolve::stability::Library::inventory)
            .ok_or_else(|| {
                Error::unlocated("unstable impl lint requires a completed staged library inventory")
            })?;
        check(
            report,
            facts,
            inventory,
            owner.self_definition,
            (key, target, trait_type),
            annotation,
        )?;
    }
    Ok(())
}
fn check(
    report: &Report,
    facts: &mut Declarations,
    inventory: &Inventory,
    self_definition: DefId,
    header: (NodeKey, NodeKey, NodeKey),
    annotation: crate::resolve::lint_policy::ImplAnnotation<'_>,
) -> Result<(), Error> {
    let (key, target, trait_type) = header;
    let producer = report
        .lint_producer()
        .ok_or_else(|| Declarations::error(report, key, "impl lint producer is missing"))?;
    let policy = inventory
        .local_declaration(self_definition)
        .map_err(|e| Declarations::error(report, key, e))?;
    if annotation.bound.is_some()
        && !policy.is_some_and(|policy| matches!(policy.level, Level::Unstable { .. }))
    {
        return Err(Declarations::error(
            report,
            key,
            "unstable_feature_bound requires an actual unstable trait impl",
        ));
    }
    let Some(attribute) = annotation.unstable else {
        return Ok(());
    };
    let policy = policy
        .filter(|policy| policy.origin == key && matches!(policy.level, Level::Unstable { .. }))
        .ok_or_else(|| {
            Declarations::error(
                report,
                key,
                "direct unstable impl annotation lost its actual stability owner",
            )
        })?;
    // Required attribute/owner validation above is independent of this lint walk.
    if report.package.lint_cap().suppresses_lints() {
        return Ok(());
    }
    let mut visitor = Visitor {
        report,
        facts,
        inventory,
        work: Vec::new(),
        stable: true,
    };
    visitor.require_trait(trait_type)?;
    visitor.push(target)?;
    visitor.push(trait_type)?;
    visitor.walk()?;
    producer.attribute_read(
        key,
        annotation.bounds.iter().fold(1_usize, |work, bound| {
            work.saturating_add(bound.len())
                .saturating_add(policy.feature.len())
        }),
    )?;
    if !visitor.stable
        || annotation
            .bounds
            .iter()
            .any(|bound| bound.as_ref() == policy.feature.as_ref())
    {
        return Ok(());
    }
    producer.diagnostic_read(key, 88)?;
    facts.must_use_diagnostic(
        report,
        annotation.owner,
        attribute.span.start,
        self_definition.0,
        WarningCode::IneffectiveUnstableTraitImpl,
        "unstable annotation has no effect on an impl of stable types and a stable trait".into(),
    )?;
    Ok(())
}

struct Visitor<'a> {
    report: &'a Report,
    facts: &'a Declarations,
    inventory: &'a Inventory,
    work: Vec<NodeKey>,
    stable: bool,
}
impl Visitor<'_> {
    fn charge(&self, key: NodeKey, steps: usize, bytes: usize) -> Result<(), Error> {
        self.report
            .lint_producer()
            .ok_or_else(|| Declarations::error(self.report, key, "impl lint producer is missing"))?
            .diagnostic_payload(key, steps, bytes)
            .map_err(Into::into)
    }
    fn push(&mut self, key: NodeKey) -> Result<(), Error> {
        let growth = Growth::vector::<NodeKey>(self.work.len(), self.work.capacity(), 1);
        self.charge(key, 1 + growth.work, growth.bytes)?;
        self.work.push(key);
        Ok(())
    }
    fn definition(&self, mut key: NodeKey) -> Result<DefId, Error> {
        loop {
            self.charge(key, 1, 0)?;
            match self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .node(key.node)
                .kind
            {
                Kind::Applied { base, .. } => key.node = base,
                Kind::Group(value) => key.node = value,
                _ => break,
            }
        }
        let Some(Target::Definition(id)) = self.facts.node(key).and_then(|node| node.reference)
        else {
            return Err(Declarations::error(
                self.report,
                key,
                "impl lint requires the actual resolved path; associated or unresolved paths remain required",
            ));
        };
        Ok(id)
    }
    fn require_trait(&self, key: NodeKey) -> Result<(), Error> {
        let id = self.definition(key)?;
        if self
            .report
            .resolution
            .definitions
            .get(id.0 as usize)
            .is_none_or(|definition| definition.kind != DefKind::Trait)
        {
            return Err(Declarations::error(
                self.report,
                key,
                "impl lint requires an actual resolved trait declaration",
            ));
        }
        Ok(())
    }
    fn path(&mut self, key: NodeKey) -> Result<(), Error> {
        let id = self.definition(key)?;
        let definition = self
            .report
            .resolution
            .definitions
            .get(id.0 as usize)
            .ok_or_else(|| {
                Declarations::error(self.report, key, "impl lint path declaration is missing")
            })?;
        if definition.kind == DefKind::Primitive {
            return Ok(());
        }
        if definition.node.is_none()
            || self
                .facts
                .crates
                .as_deref()
                .and_then(super::crates::Linkage::origin_catalog)
                .map(|catalog| catalog.declaration_if_imported(id))
                .transpose()
                .map_err(|e| Declarations::error(self.report, key, e))?
                .flatten()
                .is_some()
        {
            return Err(Declarations::error(
                self.report,
                key,
                "imported stability observation for ineffective_unstable_trait_impl is not implemented",
            ));
        }
        if self
            .inventory
            .local_declaration(id)
            .map_err(|e| Declarations::error(self.report, key, e))?
            .is_some_and(|policy| matches!(policy.level, Level::Unstable { .. }))
        {
            self.stable = false;
        }
        Ok(())
    }
    fn walk(&mut self) -> Result<(), Error> {
        while let Some(key) = self.work.pop() {
            self.charge(key, 1, 0)?;
            let unit = self.report.package.unit(key.file);
            match unit.configured.ast.node(key.node).kind {
                Kind::Name(..) => self.path(key)?,
                Kind::Path { segments, .. } => {
                    self.path(key)?;
                    for &node in unit.configured.ast.list(segments) {
                        self.arguments(NodeKey { node, ..key })?;
                    }
                }
                Kind::Applied {
                    arguments,
                    parenthesized: false,
                    output: None,
                    ..
                } => {
                    self.path(key)?;
                    for &node in unit.configured.ast.list(arguments) {
                        self.push(NodeKey { node, ..key })?;
                    }
                }
                Kind::Group(value) | Kind::Attributed { value, .. } => {
                    self.push(NodeKey { node: value, ..key })?;
                }
                Kind::NeverType => self.stable = false,
                Kind::Type { flavor, elements } => {
                    let elements = unit.configured.ast.list(elements);
                    if matches!(flavor, TypeFlavor::Ref | TypeFlavor::RefMut) {
                        self.push(NodeKey {
                            node: *elements.last().ok_or_else(|| {
                                Declarations::error(
                                    self.report,
                                    key,
                                    "impl lint reference type is incomplete",
                                )
                            })?,
                            ..key
                        })?;
                    } else {
                        let count = elements
                            .len()
                            .saturating_sub(usize::from(flavor == TypeFlavor::Array));
                        for &node in &elements[..count] {
                            self.push(NodeKey { node, ..key })?;
                        }
                    }
                }
                Kind::FunctionType(index) => self.function(key, index)?,
                Kind::Parameter { ty: Some(ty), .. } | Kind::FieldDefinition { ty, .. } => {
                    self.push(NodeKey { node: ty, ..key })?;
                }
                _ => {
                    return Err(Declarations::error(
                        self.report,
                        key,
                        "ineffective_unstable_trait_impl type or path shape remains required",
                    ));
                }
            }
        }
        Ok(())
    }
    fn arguments(&mut self, key: NodeKey) -> Result<(), Error> {
        self.charge(key, 1, 0)?;
        let unit = self.report.package.unit(key.file);
        match unit.configured.ast.node(key.node).kind {
            Kind::Name(..) | Kind::CrateRoot(_) => Ok(()),
            Kind::Applied {
                arguments,
                parenthesized: false,
                output: None,
                ..
            } => {
                for &node in unit.configured.ast.list(arguments) {
                    self.push(NodeKey { node, ..key })?;
                }
                Ok(())
            }
            _ => Err(Declarations::error(
                self.report,
                key,
                "impl lint generic path segment shape remains required",
            )),
        }
    }
    fn function(&mut self, key: NodeKey, index: u32) -> Result<(), Error> {
        let unit = self.report.package.unit(key.file);
        let function = unit
            .configured
            .ast
            .function_types
            .get(index as usize)
            .ok_or_else(|| {
                Declarations::error(self.report, key, "impl lint function pointer is missing")
            })?;
        if super::declarations::callable_convention(self.report, key, function.abi)?.abi
            == crate::types::callable::Abi::RustCall
        {
            self.stable = false;
        }
        for &node in unit.configured.ast.list(function.parameters) {
            self.push(NodeKey { node, ..key })?;
        }
        if let Some(result) = function
            .result
            .filter(|&node| unit.configured.ast.node(node).kind != Kind::NeverType)
        {
            self.push(NodeKey {
                node: result,
                ..key
            })?;
        }
        Ok(())
    }
}
