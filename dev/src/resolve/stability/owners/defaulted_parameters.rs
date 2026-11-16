//! Regular availability on a generic parameter belongs to a type or const
//! parameter with a default, as in alloc's
//! `#[unstable(feature = "allocator_api", issue = "32838")] A: Allocator = Global`.
//! rustc reads it only where another crate writes an explicit argument for
//! that parameter, so a use in its own crate never consults it; stability
//! metadata export is not implemented, so no other crate can either. The
//! policy stays in the inventory and its parameter's row stays empty, as for
//! any generic. rustc 1.98's attribute targets admit no other parameter: not
//! one without a default, a const parameter or a lifetime.
use super::{Draft, Error, Package};
use crate::resolve::stability::PolicyKind;
use crate::{
    resolve::{
        model::{DefKind, Definition},
        unit::NodeKey,
    },
    support::hash::HashSet,
    syntax::{ast::Kind, item::GenericFlavor},
};

/// Consume `definition`'s availability when it is a generic parameter.
/// Returns false for every other definition.
pub(super) fn consume(
    package: &Package,
    definition: &Definition,
    draft: &mut Draft<'_>,
    used: &mut HashSet<(NodeKey, bool)>,
) -> Result<bool, Error> {
    if definition.kind != DefKind::Generic {
        return Ok(false);
    }
    let Some(key) = definition.node else {
        return Ok(true);
    };
    let error = |message| super::error(package, key, message);
    if draft.attribute(key, PolicyKind::Const).is_some() {
        return Err(error(
            "const library stability is not supported on generic parameters",
        ));
    }
    if draft.attribute(key, PolicyKind::Regular).is_none() {
        return Ok(true);
    }
    draft.charge(4, 0)?;
    let ast = &package.unit(key.file).configured.ast;
    let Kind::Generic(index) = ast.node(key.node).kind else {
        return Err(error(
            "generic parameter availability requires its actual parsed parameter",
        ));
    };
    let parameter = ast
        .generics
        .get(index as usize)
        .ok_or_else(|| error("generic parameter availability has no parsed parameter"))?;
    if parameter.flavor != GenericFlavor::Type || parameter.default.is_none() {
        return Err(error(
            "library stability on a generic parameter requires a type parameter with a default",
        ));
    }
    used.insert((key, false));
    Ok(true)
}
