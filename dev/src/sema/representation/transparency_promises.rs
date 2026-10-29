//! Public transparency is an authenticated representation contract, not visibility.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    syntax::attribute::Input,
    types::aggregate::{Repr, Representation},
};

pub(super) fn validate(
    report: &Report,
    key: NodeKey,
    owner: usize,
    representation: Option<Repr>,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    let unit = report.package.unit(key.file);
    let mut seen = false;
    for (attribute, forest) in
        unit.retained_attributes(unit.configured.owners[owner].attributes.clone())
    {
        if attribute.name(forest, unit.source.text()) != Some("rustc_pub_transparent") {
            continue;
        }
        let error = |message| Error::at(key.file, attribute.span.start, message);
        if seen {
            return Err(error("multiple rustc_pub_transparent attributes"));
        }
        seen = true;
        if attribute.unsafe_wrapper || attribute.input != Input::Empty {
            return Err(error("rustc_pub_transparent requires a word attribute"));
        }
        if !report.package.requested_internal_attributes()
            && !allowances.allows_at(
                key.file,
                attribute.span,
                &FeatureName::new("rustc_attrs").map_err(error)?,
            )?
        {
            return Err(error("rustc_pub_transparent requires rustc_attrs"));
        }
        if !representation.is_some_and(|repr| repr.representation == Representation::Transparent) {
            return Err(error(
                "rustc_pub_transparent requires a repr(transparent) nominal declaration",
            ));
        }
        // The actual configured attribute remains on its original owner. Local
        // layout follows the checked repr; this contract grants no field access.
        // Current metadata rejects transparent records, so it cannot lose the
        // external private-field eligibility contract during an export.
    }
    Ok(())
}
