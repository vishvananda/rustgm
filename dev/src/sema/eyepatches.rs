//! `#[may_dangle]` records which arguments an unsafe destructor allows to expire.
//! Owned fields retain their independent drop requirements.
use crate::support::hash::HashSet;
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        model::DefId,
        package::Error,
        unit::NodeKey,
    },
    sema::{destructors::Coverage, model::Declarations},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::{GenericFlavor, ItemKind},
        token_tree::Forest,
    },
    types::{ItemId, RegionId, arguments::Argument},
};

/// rustc's placement rule for the attribute.
pub(super) const MISPLACED: &str =
    "`#[may_dangle]` must be applied to a lifetime or type generic parameter in `Drop` impl";

/// Validate one `#[may_dangle]`: its word form, its feature, and that its
/// owner is a lifetime or type parameter of a trait impl. The `Drop` impl
/// headers check the trait and the impl's safety.
/// # Errors
/// Rejects any other form, an ungated use and any other owner.
pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    (owner, declaration): (NodeKey, NodeKey),
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.style != Style::Outer
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Err(error("may_dangle requires an outer word attribute"));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("may_dangle attribute has no actual name token"))?
        .span;
    if !report.package.requested_dropck_eyepatch().map_err(error)?
        && !allowances.allows_at(
            declaration.file,
            name,
            &FeatureName::new("dropck_eyepatch").map_err(error)?,
        )?
    {
        return Err(error(
            "`#[may_dangle]` requires dropck_eyepatch or exact token allowance",
        ));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    let parameter = matches!(ast.node(declaration.node).kind, Kind::Generic(index)
        if ast.generics.get(index as usize).is_some_and(|parameter| parameter.flavor != GenericFlavor::Const));
    if !parameter {
        return Err(error(MISPLACED));
    }
    // The parser keeps a parameter's attributed node in its owner's list.
    crate::sema::header_attributes::reserve_source_contracts(facts, ast.items.len(), 0)?;
    let in_trait_impl = ast.items.iter().any(|item| {
        matches!(
            item.kind,
            ItemKind::Impl {
                trait_path: Some(_),
                ..
            }
        ) && ast.list(item.generics).contains(&owner.node)
    });
    if !in_trait_impl {
        return Err(error(MISPLACED));
    }
    Ok(())
}

/// Whether the trait impl at `source` marks any configured parameter.
/// # Errors
/// Rejects an attribute with arguments.
pub(super) fn marked(report: &Report, source: NodeKey) -> Result<bool, Error> {
    Ok(marked_parameters(report, source)?
        .next()
        .transpose()?
        .is_some())
}

fn marked_parameters(
    report: &Report,
    source: NodeKey,
) -> Result<impl Iterator<Item = Result<NodeKey, Error>> + '_, Error> {
    let unit = report.package.unit(source.file);
    let ast = &unit.configured.ast;
    let Kind::Item(index) = ast.node(source.node).kind else {
        return Err(Declarations::error(
            report,
            source,
            "missing impl item syntax",
        ));
    };
    Ok(ast
        .list(ast.items[index as usize].generics)
        .iter()
        .filter_map(move |&node| {
            if unit.configured.active.get(node.get() as usize) != Some(&true) {
                return None;
            }
            unit.word_attribute(Some(node), "may_dangle")
                .map_err(|error| Error::at(source.file, error.offset, error.message))
                .map(|marked| {
                    marked.then(|| {
                        let mut node = node;
                        while let Kind::Attributed { value, .. } = ast.node(node).kind {
                            node = value;
                        }
                        NodeKey { node, ..source }
                    })
                })
                .transpose()
        }))
}

/// Map the marked, checked impl parameters to their positions in its nominal self type.
pub(super) fn arguments(
    report: &Report,
    facts: &Declarations,
    (owner, item): (DefId, ItemId),
    coverage: &Coverage,
    lifetimes: &[(DefId, RegionId)],
    source: NodeKey,
) -> Result<Box<[u32]>, Error> {
    let error = |message| Declarations::error(report, source, message);
    let mut bindings = marked_parameters(report, source)?
        .map(|key| {
            facts
                .node(key?)
                .and_then(|node| node.binding)
                .ok_or_else(|| error("missing resolved Drop parameter"))
        })
        .collect::<Result<HashSet<_>, _>>()?;
    let inverse = facts
        .types
        .arguments(coverage.arguments)
        .ok_or_else(|| error("missing checked Drop argument coverage"))?;
    let mut marked = facts
        .generics
        .parameters
        .get(&owner)
        .into_iter()
        .flatten()
        .zip(inverse)
        .filter_map(|(parameter, nominal)| bindings.remove(&parameter.binding).then_some(nominal))
        .collect::<HashSet<_>>();
    for &(binding, region) in lifetimes {
        if !bindings.remove(&binding) {
            continue;
        }
        let Some((0, slot)) = region.bound_parts() else {
            return Err(error(
                "marked Drop lifetime is outside its implementation binder",
            ));
        };
        let nominal = *coverage
            .regions
            .get(usize::from(slot))
            .ok_or_else(|| error("missing checked Drop lifetime coverage"))?;
        marked.insert(Argument::Lifetime(nominal));
    }
    if !bindings.is_empty() {
        return Err(error(
            "marked Drop parameters differ from their checked nominal arguments",
        ));
    }
    let Some(crate::types::Kind::Adt(_, nominal)) = facts
        .definitions
        .get(item.0 as usize)
        .and_then(|&ty| ty)
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err(error("missing nominal Drop schema"));
    };
    facts
        .types
        .arguments(nominal)
        .ok_or_else(|| error("missing nominal Drop arguments"))?
        .enumerate()
        .filter(|(_, argument)| marked.contains(argument))
        .map(|(index, _)| {
            u32::try_from(index).map_err(|_| error("too many nominal Drop arguments"))
        })
        .collect()
}
