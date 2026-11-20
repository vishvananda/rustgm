//! Version thirteen carries inherent impls: an implementation header with no
//! trait application whose member links name only its own methods. Its Self is
//! a nominal that this provider declares. Older profiles reject it by name,
//! and every consumer re-proves its Self, inputs and methods.
use super::{ArtifactRef, DefinitionKind, ImplHeader, Profile, SourceRows, Type};
use crate::support::crates::generics::Work;

impl ImplHeader {
    pub(crate) fn closed_inherent_constant_owner(&self, types: &[Type]) -> bool {
        self.trait_application.is_none()
            && !self.constant
            && !self.unsafe_
            && self.polarity.is_positive()
            && self.scope == self.self_definition
            && self.region_captures == 0
            && self.region_bounds.start == self.region_bounds.end
            && self.inputs.start == self.inputs.end
            && self.declared_inputs == 0
            && (self.core_primitive_inherent
                || matches!(types.get(self.self_type.0 as usize), Some(Type::Nominal { declaration, arguments })
                    if declaration.artifact == ArtifactRef(0) && arguments.start == arguments.end))
    }
}

pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.implementations.len().saturating_add(1))?;
    for implementation in &rows.implementations {
        work.charge(1)?;
        if implementation.core_primitive_inherent {
            let primitive = match rows.types.get(implementation.self_type.0 as usize) {
                Some(Type::F16 | Type::F128) => true,
                Some(Type::Scalar(tag)) => *tag != crate::support::crates::Scalar::Unit,
                _ => false,
            };
            if !profile.declaration_policies()
                || !implementation.closed_inherent_constant_owner(&rows.types)
                || !primitive
            {
                return Err(
                    "source Core primitive authority requires an actual closed inherent owner",
                );
            }
            continue;
        }
        if implementation.trait_application.is_some() {
            continue;
        }
        if !profile.body_nodes() {
            return Err("inherent source impl requires its wider member profile");
        }
        own_nominal(rows, implementation)?;
    }
    Ok(())
}

/// An inherent impl may extend only a nominal declared by this provider.
fn own_nominal(rows: &SourceRows, implementation: &ImplHeader) -> Result<(), &'static str> {
    let error = "imported inherent impl Self is not a nominal of its own provider";
    let Some(Type::Nominal { declaration, .. }) =
        rows.types.get(implementation.self_type.0 as usize)
    else {
        return Err(error);
    };
    if declaration.artifact != ArtifactRef(0)
        || implementation.unsafe_
        || implementation.constant
        || !implementation.polarity.is_positive()
        || !rows
            .declarations
            .get(declaration.declaration.0 as usize)
            .is_some_and(|row| {
                matches!(
                    row.definition.kind,
                    DefinitionKind::Struct | DefinitionKind::Enum
                )
            })
    {
        return Err(error);
    }
    Ok(())
}
#[cfg(test)]
#[path = "../../../../tests/invariants/traitless_impl_profiles.rs"]
mod tests;
