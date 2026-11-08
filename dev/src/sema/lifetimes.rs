//! Lifetime parameter identities and reusable substitutions over declaration templates.
use super::model::Declarations;
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    syntax::ast::{Kind, List},
    types::{GenericArgs, Store, TypeId, substitute::Bound},
};
use std::collections::hash_map::Entry;
#[cfg(test)]
#[path = "../../tests/invariants/explicit_bounds.rs"]
mod tests;

#[derive(Clone)]
pub struct FunctionRegions {
    /// All source/body parameter slots; the callable type may omit unused slots.
    pub total: u32,
    /// Owner lifetime slots precede the method's own early and late parameters.
    pub captures: u32,
    /// Early arguments are supplied in declaration order when naming the item.
    pub early: Vec<(DefId, u16)>,
    /// Used late slots, in canonical first-use order, instantiated at each call.
    pub late: Vec<u16>,
    pub has_declared_late: bool,
    pub bounds: Vec<(crate::types::RegionId, crate::types::RegionId)>,
}

impl FunctionRegions {
    /// Exact declaration-to-callable relocation; source slots stay distinct from
    /// early callable arguments and the public late-bound function frame.
    pub(crate) fn callable_mapping(
        &self,
        work: &mut usize,
    ) -> Result<Vec<crate::types::RegionId>, &'static str> {
        use crate::types::RegionId;
        if self.total > 65_536 {
            return Err("too many callable lifetime slots");
        }
        let mut mapping = vec![RegionId::STATIC; self.total as usize];
        *work = work.saturating_add(mapping.len());
        for (ordinal, &index) in self.late.iter().enumerate() {
            *work = work.saturating_add(1);
            *mapping
                .get_mut(usize::from(index))
                .ok_or("invalid callable late lifetime slot")? = RegionId::bound(
                0,
                u16::try_from(ordinal).map_err(|_| "too many callable parameters")?,
            )?;
        }
        for ordinal in 0..self.captures as usize + self.early.len() {
            *work = work.saturating_add(1);
            let index = if ordinal < self.captures as usize {
                ordinal
            } else {
                usize::from(self.early[ordinal - self.captures as usize].1)
            };
            *mapping
                .get_mut(index)
                .ok_or("invalid callable early lifetime slot")? = RegionId::bound(
                1,
                u16::try_from(ordinal).map_err(|_| "too many callable parameters")?,
            )?;
        }
        Ok(mapping)
    }
}

#[derive(Default)]
pub struct Lifetimes {
    pub(crate) rigid_inputs: Option<Box<crate::types::variance::RigidInputs>>,
    pub callables: crate::types::callable::Templates,
    pub nominal_bounds: Vec<(crate::types::ItemId, crate::types::outlives::Pair)>,
    /// `nominal_bounds` by owner, once a nominal's outlives contract is demanded.
    explicit_bounds: Option<Box<ExplicitBounds>>,
    /// Completed field and declared requirements; all consumers share this result.
    pub nominal_requirements: crate::types::outlives::Bounds,
    pub functions: HashMap<DefId, FunctionRegions>,
    pub variances: crate::types::variance::Variances,
    /// Declaration order, independent of a particular use's argument lifetimes.
    pub parameters: HashMap<DefId, Vec<DefId>>,
    pub named: HashMap<DefId, crate::types::RegionId>,
    substitutions: HashMap<GenericArgs, Bound>,
}

/// Declarations only append explicit nominal bounds, so demands extend this
/// index with the bounds published since the previous one, each indexed and
/// charged once, instead of grouping every bound of the crate again.
#[derive(Default)]
struct ExplicitBounds {
    indexed: usize,
    owners: HashMap<crate::types::ItemId, Vec<crate::types::outlives::Pair>>,
}

impl Lifetimes {
    /// Substitute one nominal or alias template in this declaration store.
    ///
    /// # Errors
    /// Rejects invalid arguments/types and exhausted substitution resources.
    pub fn substitute(
        &mut self,
        types: &mut Store,
        args: GenericArgs,
        ty: TypeId,
    ) -> Result<TypeId, &'static str> {
        types.kind(ty).ok_or("invalid lifetime template type")?;
        if args == GenericArgs::EMPTY {
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

    /// An early consumer demands only one nominal's reached outlives contract;
    /// the later complete inference republishes the same declared result.
    ///
    /// # Errors
    /// Propagates exhausted payment and reached-inference refusals.
    pub(crate) fn demand_reached_outlives<C>(
        &mut self,
        types: &mut Store,
        definitions: &crate::types::aggregate::Definitions,
        item: crate::types::ItemId,
        clauses: impl Fn(crate::types::ItemId) -> C,
        pay: crate::support::retained::Payment<'_>,
    ) -> Result<(), &'static str>
    where
        C: IntoIterator<Item = crate::types::parameters::clauses::Clause>,
    {
        if self.nominal_requirements.get(item).is_some() {
            return Ok(());
        }
        if self
            .explicit_bounds
            .as_ref()
            .is_some_and(|explicit| explicit.indexed > self.nominal_bounds.len())
        {
            // The bounds were replaced, not appended: index them again.
            self.explicit_bounds = None;
        }
        if self.explicit_bounds.is_none() {
            pay(1, size_of::<ExplicitBounds>())?;
        }
        let explicit = self.explicit_bounds.get_or_insert_with(Box::default);
        let fresh = self
            .nominal_bounds
            .get(explicit.indexed..)
            .ok_or("invalid explicit nominal bound index")?;
        // Each new bound may start an owner row; a row's slots are at most
        // four times its bounds.
        let growth = crate::support::retained::Growth::map::<
            crate::types::ItemId,
            Vec<crate::types::outlives::Pair>,
        >(
            explicit.owners.len(),
            explicit.owners.capacity(),
            fresh.len(),
        );
        pay(
            fresh.len().saturating_add(growth.work).saturating_add(1),
            size_of_val(fresh)
                .saturating_mul(4)
                .saturating_add(growth.bytes),
        )?;
        for &(owner, pair) in fresh {
            explicit.owners.entry(owner).or_default().push(pair);
        }
        explicit.indexed = self.nominal_bounds.len();
        self.nominal_requirements.append_reached(
            types,
            definitions,
            item,
            &explicit.owners,
            clauses,
            pay,
        )
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
            parenthesized: false,
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

/// Parenthesized call-trait inputs are a retained syntax distinction, including `Fn()`.
pub(super) fn call_arguments(
    report: &Report,
    key: NodeKey,
) -> Result<Option<(List, Option<crate::syntax::ast::NodeId>)>, Error> {
    let ast = &report.package.unit(key.file).configured.ast;
    let mut node = key.node;
    if let Kind::Path { segments, .. } = ast.node(node).kind {
        let Some((last, prefix)) = ast.list(segments).split_last() else {
            return Err(Declarations::error(report, key, "empty call-trait path"));
        };
        if prefix
            .iter()
            .any(|node| matches!(ast.node(*node).kind, Kind::Applied { .. }))
        {
            return Err(Declarations::error(
                report,
                key,
                "call-trait arguments require the final path segment",
            ));
        }
        node = *last;
    }
    Ok(match ast.node(node).kind {
        Kind::Applied {
            arguments,
            output,
            parenthesized: true,
            ..
        } => Some((arguments, output)),
        _ => None,
    })
}
