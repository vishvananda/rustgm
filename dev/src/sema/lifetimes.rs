//! Lifetime parameter identities and reusable substitutions over declaration templates.
use super::model::Declarations;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    syntax::ast::{Kind, List},
    types::{RegionArgs, Store, TypeId, substitute::Bound},
};
use std::collections::{HashMap, hash_map::Entry};

pub struct FunctionRegions {
    /// All source/body parameter slots; the callable type may omit unused slots.
    pub total: u32,
    /// Early arguments are supplied in declaration order when naming the item.
    pub early: Vec<(DefId, u16)>,
    /// Used late slots, in canonical first-use order, instantiated at each call.
    pub late: Vec<u16>,
    pub has_declared_late: bool,
    pub bounds: Vec<(crate::types::RegionId, crate::types::RegionId)>,
}

#[derive(Default)]
pub struct Lifetimes {
    pub callables: crate::types::callable::Templates,
    pub nominal_bounds: Vec<(crate::types::ItemId, crate::types::outlives::Pair)>,
    pub functions: HashMap<DefId, FunctionRegions>,
    pub variances: crate::types::variance::Variances,
    /// Declaration order, independent of a particular use's argument lifetimes.
    pub parameters: HashMap<DefId, Vec<DefId>>,
    pub named: HashMap<DefId, crate::types::RegionId>,
    substitutions: HashMap<RegionArgs, Bound>,
}

impl Lifetimes {
    /// Substitute one nominal or alias template in this declaration store.
    ///
    /// # Errors
    /// Rejects invalid arguments/types and exhausted substitution resources.
    pub fn substitute(
        &mut self,
        types: &mut Store,
        args: RegionArgs,
        ty: TypeId,
    ) -> Result<TypeId, &'static str> {
        types.kind(ty).ok_or("invalid lifetime template type")?;
        if args == RegionArgs::EMPTY {
            return Ok(ty);
        }
        let substitution = match self.substitutions.entry(args) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Bound::new(
                types
                    .region_args(args)
                    .ok_or("invalid lifetime arguments")?
                    .to_vec(),
            )),
        };
        substitution.apply(types, ty)
    }
}

/// Resolution selects whether the penultimate segment owns an enum variant.
/// Empty argument lists request ordinary inference, including on module paths.
pub(super) fn arguments(
    report: &Report,
    key: NodeKey,
    enum_variant: bool,
) -> Result<Option<List>, Error> {
    let mut node = key.node;
    let ast = &report.package.unit(key.file).configured.ast;
    if let Kind::Path { segments, .. } = ast.node(node).kind {
        let segments = ast.list(segments);
        let Some((last, prefix)) = segments.split_last() else {
            return Err(Declarations::error(report, key, "empty type path"));
        };
        let final_arguments = argument_list(report, NodeKey { node: *last, ..key })?;
        for (index, segment) in prefix.iter().enumerate() {
            if matches!(ast.node(*segment).kind, Kind::Applied { .. }) {
                let arguments = argument_list(
                    report,
                    NodeKey {
                        node: *segment,
                        ..key
                    },
                )?;
                if enum_variant && index + 1 == prefix.len() {
                    if final_arguments.is_some() {
                        return Err(Declarations::error(
                            report,
                            key,
                            "lifetime arguments cannot occur on both enum and variant",
                        ));
                    }
                    node = *segment;
                } else if arguments.is_some() {
                    return Err(Declarations::error(
                        report,
                        key,
                        "lifetime arguments require the selected type or enum path segment",
                    ));
                }
            }
        }
        if node == key.node {
            node = *last;
        }
    }
    argument_list(report, NodeKey { node, ..key })
}
fn argument_list(report: &Report, key: NodeKey) -> Result<Option<List>, Error> {
    let ast = &report.package.unit(key.file).configured.ast;
    let node = key.node;
    match ast.node(node).kind {
        Kind::Applied {
            arguments,
            output: None,
            ..
        } => Ok((arguments.start != arguments.end).then_some(arguments)),
        Kind::Applied { .. } => Err(Declarations::error(
            report,
            key,
            "parenthesized trait arguments require trait semantics",
        )),
        _ => Ok(None),
    }
}
