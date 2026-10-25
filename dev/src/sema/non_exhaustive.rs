//! Local openness contracts validate source owners; foreign transport stays closed.
use crate::{
    resolve::{expansion::Report, package::Error, unit::NodeKey},
    sema::{model::Declarations, must_use::WarningCode},
    support::{hash::HashMap, source::OriginalImage},
    syntax::{ast::Kind, attribute::Input, item::ItemKind, token_tree::Span},
};
use std::{cell::Cell, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Struct,
    Enum,
    Variant,
}
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub owner: NodeKey,
    pub declaration: NodeKey,
    pub span: Span,
    pub target: Target,
}
struct Contract {
    view: View,
    effect: usize,
    image: Arc<OriginalImage>,
}
#[derive(Default)]
pub(in crate::sema) struct Contracts {
    entries: HashMap<NodeKey, Contract>,
    steps: Cell<usize>,
    bytes: Cell<usize>,
}
impl Contracts {
    fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.steps.set(self.steps.get().saturating_add(steps));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.steps.get() > 10_000_000 || self.bytes.get() > 64 * 1024 * 1024 {
            return Err("non_exhaustive contract work or storage budget exceeded");
        }
        Ok(())
    }
}
impl Declarations {
    /// Read a checked local contract. Imported declarations have no source key.
    /// # Errors
    /// Rejects changed source images, owners, effects or declaration roles.
    pub fn non_exhaustive_contract(
        &self,
        report: &Report,
        declaration: NodeKey,
    ) -> Result<Option<View>, Error> {
        let unit = report
            .package
            .units
            .get(declaration.file.0 as usize)
            .ok_or_else(|| {
                Error::at(
                    declaration.file,
                    0,
                    "non_exhaustive declaration source is missing",
                )
            })?;
        if unit.include_context.is_some() {
            return Err(Error::at(
                declaration.file,
                0,
                "non_exhaustive contract has no defining source owner",
            ));
        }
        target(unit, declaration.node)
            .map_err(|message| Error::at(declaration.file, 0, message))?;
        let Some(contracts) = self
            .must_use_contracts
            .as_deref()
            .and_then(|owner| owner.non_exhaustive.as_deref())
        else {
            return Ok(None);
        };
        contracts
            .charge(1, 0)
            .map_err(|message| Error::at(declaration.file, 0, message))?;
        let Some(contract) = contracts.entries.get(&declaration) else {
            return Ok(None);
        };
        let view = contract.view;
        let error = || {
            Error::at(
                view.owner.file,
                view.span.start,
                "non_exhaustive contract actual source owner changed",
            )
        };
        if !Arc::ptr_eq(&contract.image, unit.source.original_image()) {
            return Err(error());
        }
        let owner = unit
            .owner_indices
            .get(view.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .ok_or_else(error)?;
        if !owner.enabled
            || owner.node != view.owner.node
            || !owner.attributes.contains(&contract.effect)
        {
            return Err(error());
        }
        let mut node = owner.node;
        while let Kind::Attributed { value, .. } = unit
            .configured
            .ast
            .nodes
            .get(node.get() as usize)
            .ok_or_else(error)?
            .kind
        {
            contracts
                .charge(1, 0)
                .map_err(|message| Error::at(view.owner.file, view.span.start, message))?;
            node = value;
        }
        if node != declaration.node
            || target(unit, node).ok() != Some(view.target)
            || !capture_matches(unit, contract)
        {
            return Err(error());
        }
        Ok(Some(view))
    }
    pub(in crate::sema) fn non_exhaustive_declarations(
        &self,
    ) -> impl Iterator<Item = NodeKey> + '_ {
        self.must_use_contracts
            .iter()
            .filter_map(|owner| owner.non_exhaustive.as_deref())
            .flat_map(|contracts| contracts.entries.keys().copied())
    }
}

pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    declaration: NodeKey,
    attribute: crate::syntax::attribute::Attribute,
    effect: usize,
) -> Result<(), Error> {
    let unit = report.package.unit(owner.file);
    facts.reserve_must_use_source(owner, attribute.span)?;
    let error = |message| Error::at(owner.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error(
            "non_exhaustive attribute requires word form without arguments or an unsafe wrapper",
        ));
    }
    if matches!(
        unit.configured.ast.node(declaration.node).kind,
        Kind::FieldDefinition { .. }
    ) {
        return facts.must_use_diagnostic(
            report,
            owner,
            attribute.span.start,
            0,
            WarningCode::UnusedAttributes,
            "non_exhaustive attribute cannot be used on struct fields".into(),
        );
    }
    let role = target(unit, declaration.node).map_err(error)?;
    retain(
        report,
        facts,
        unit,
        View {
            owner,
            declaration,
            span: attribute.span,
            target: role,
        },
        effect,
    )
}

fn retain(
    report: &Report,
    facts: &mut Declarations,
    unit: &crate::resolve::unit::Unit,
    view: View,
    effect: usize,
) -> Result<(), Error> {
    let error = |message| Error::at(view.owner.file, view.span.start, message);
    let common = facts.must_use_contracts.get_or_insert_default();
    let first = common.non_exhaustive.is_none();
    let storage = crate::support::retained::new_map::<NodeKey, Contract>(1)
        .saturating_add(usize::from(first) * size_of::<Contracts>());
    common.work.contract_input_bytes = common.work.contract_input_bytes.saturating_add(storage);
    if common.work.contract_input_bytes > 64 * 1024 * 1024 {
        return Err(error("non_exhaustive contract storage budget exceeded"));
    }
    let contracts = common.non_exhaustive.get_or_insert_default();
    contracts.charge(1, storage).map_err(error)?;
    if contracts.entries.contains_key(&view.declaration) {
        return facts.must_use_diagnostic(
            report,
            view.owner,
            view.span.start,
            0,
            WarningCode::UnusedAttributes,
            "duplicate non_exhaustive attribute".into(),
        );
    }
    if contracts.entries.len() >= 1_000_000 {
        return Err(error("non_exhaustive contract count budget exceeded"));
    }
    contracts.entries.insert(
        view.declaration,
        Contract {
            view,
            effect,
            image: Arc::clone(unit.source.original_image()),
        },
    );
    Ok(())
}

fn capture_matches(unit: &crate::resolve::unit::Unit, contract: &Contract) -> bool {
    let Some(effect) = unit.configured.effects.get(contract.effect) else {
        return false;
    };
    if unit
        .configured
        .ast
        .attributes
        .get(effect.capture as usize)
        .is_none()
    {
        return false;
    }
    unit.retained_attributes(contract.effect..contract.effect + 1)
        .next()
        .is_some_and(|(attribute, forest)| {
            attribute.span == contract.view.span
                && attribute.input == Input::Empty
                && !attribute.unsafe_wrapper
                && attribute.name(forest, unit.source.text()) == Some("non_exhaustive")
        })
}

fn target(
    unit: &crate::resolve::unit::Unit,
    node: crate::syntax::ast::NodeId,
) -> Result<Target, &'static str> {
    let Some(node) = unit.configured.ast.nodes.get(node.get() as usize) else {
        return Err("non_exhaustive declaration node is missing");
    };
    match node.kind {
        Kind::Item(index) => match unit
            .configured
            .ast
            .items
            .get(index as usize)
            .map(|item| item.kind)
        {
            Some(ItemKind::Struct { .. }) => Ok(Target::Struct),
            Some(ItemKind::Enum { .. }) => Ok(Target::Enum),
            _ => Err("non_exhaustive attribute requires a struct, enum or enum variant"),
        },
        Kind::Variant(index) if unit.configured.ast.variants.get(index as usize).is_some() => {
            Ok(Target::Variant)
        }
        _ => Err("non_exhaustive attribute requires a struct, enum or enum variant"),
    }
}
