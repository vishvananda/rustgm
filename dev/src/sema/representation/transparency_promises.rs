//! Retained representation contracts require actual owner and feature authority.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    syntax::{ast::Kind, attribute::Input, item::ItemKind},
    types::aggregate::{Repr, Representation},
};

#[derive(Default)]
pub(super) struct Policies {
    pub indirect: bool,
    pub lane_limit: Option<u64>,
}
pub(super) fn validate(
    report: &Report,
    facts: &mut crate::sema::model::Declarations,
    key: NodeKey,
    owner: usize,
    representation: Option<Repr>,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<Policies, Error> {
    let unit = report.package.unit(key.file);
    let mut seen = false;
    let mut policies = Policies::default();
    for (attribute, forest) in
        unit.retained_attributes(unit.configured.owners[owner].attributes.clone())
    {
        let name = attribute.name(forest, unit.source.text());
        if name == Some("rustc_simd_monomorphize_lane_limit") {
            if policies.lane_limit.is_some() {
                return Err(Error::at(
                    key.file,
                    attribute.span.start,
                    "duplicate SIMD lane limit attribute",
                ));
            }
            policies.lane_limit = Some(super::lane_limits::parse(
                report, facts, key, attribute, forest, allowances,
            )?);
            continue;
        }
        if !matches!(
            name,
            Some("rustc_pub_transparent" | "rustc_pass_indirectly_in_non_rustic_abis")
        ) {
            continue;
        }
        let error = |message| Error::at(key.file, attribute.span.start, message);
        let is_indirect = name == Some("rustc_pass_indirectly_in_non_rustic_abis");
        if std::mem::replace(
            if is_indirect {
                &mut policies.indirect
            } else {
                &mut seen
            },
            true,
        ) {
            return Err(error("duplicate nominal representation contract"));
        }
        if attribute.unsafe_wrapper || attribute.input != Input::Empty {
            return Err(error(
                "nominal representation contract requires a word attribute",
            ));
        }
        if !report.package.requested_internal_attributes()
            && !allowances.allows_at(
                key.file,
                attribute.span,
                &FeatureName::new("rustc_attrs").map_err(error)?,
            )?
        {
            return Err(error(
                "nominal representation contract requires rustc_attrs",
            ));
        }
        if is_indirect {
            if !matches!(unit.configured.ast.node(key.node).kind,
                Kind::Item(item) if matches!(unit.configured.ast.items[item as usize].kind, ItemKind::Struct { .. }))
            {
                return Err(error(
                    "indirect argument contract requires a struct declaration",
                ));
            }
            if representation.is_some_and(|repr| repr.representation == Representation::Transparent)
            {
                return Err(error(
                    "indirect argument contract conflicts with repr(transparent)",
                ));
            }
        } else if !representation
            .is_some_and(|repr| repr.representation == Representation::Transparent)
        {
            return Err(error(
                "rustc_pub_transparent requires a repr(transparent) nominal declaration",
            ));
        }
        // The actual configured attribute remains on its original owner. Local
        // layout follows the checked repr; this contract grants no field access.
        // Current metadata rejects transparent records, so it cannot lose the
        // external private-field eligibility contract during an export.
    }
    Ok(policies)
}
