//! Version thirteen carries inherent impls: an implementation header with no
//! trait application whose member links name only its own methods. Its Self is
//! a nominal that this provider declares. Older profiles reject it by name,
//! and every consumer re-proves its Self, inputs and methods.
use super::{ArtifactRef, DefinitionKind, ImplHeader, Profile, SourceRows, Type};
use crate::support::{
    bindings::LangItem,
    crates::{generics::Work, owner_catalog::DeclarationRef},
};

impl ImplHeader {
    pub(crate) fn inherent_constant_owner(&self) -> bool {
        self.trait_application.is_none()
            && !self.unsafe_
            && self.polarity.is_positive()
            && self.scope == self.self_definition
    }

    /// The constant header validator also checks the completed owner scope;
    /// the type-domain validator checks every argument of a nominal Self.
    pub(crate) fn closed_inherent_constant_owner(&self, types: &[Type]) -> bool {
        self.inherent_constant_owner()
            && !self.constant
            && self.region_captures == 0
            && self.region_bounds.start == self.region_bounds.end
            && self.inputs.start == self.inputs.end
            && self.declared_inputs == 0
            && (self.core_primitive_inherent
                || matches!(types.get(self.self_type.0 as usize), Some(Type::Nominal { declaration, .. })
                    if declaration.artifact == ArtifactRef(0)))
    }
}

pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.implementations.len().saturating_add(1))?;
    let _ = slice_owner(rows, profile, work)?;
    for implementation in &rows.implementations {
        work.charge(1)?;
        if implementation.core_primitive_inherent {
            let builtin = matches!(
                rows.types.get(implementation.self_type.0 as usize),
                Some(
                    Type::Scalar(_)
                        | Type::F16
                        | Type::F128
                        | Type::Str
                        | Type::Never
                        | Type::Slice(_)
                        | Type::Array { .. }
                        | Type::RawPointer { .. }
                        | Type::Reference { .. }
                        | Type::Tuple(_)
                        | Type::Function(_)
                )
            );
            if !(profile.declaration_policies()
                && builtin
                && implementation.trait_application.is_none()
                && !implementation.unsafe_
                && implementation.polarity.is_positive()
                && implementation.scope == implementation.self_definition)
            {
                return Err(
                    "source Core primitive authority requires an actual supported inherent owner",
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
        own_nominal(rows, profile, implementation)?;
    }
    Ok(())
}

fn slice_owner(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<Option<DeclarationRef>, &'static str> {
    if !profile.declaration_policies() {
        return Ok(None);
    }
    let roles = rows.policy.roles.slice(&rows.roles)?;
    work.charge(roles.len())?;
    let Some(method) = roles.iter().find(|role| role.role == LangItem::SliceLen) else {
        return Ok(None);
    };
    if method.declaration.artifact != rows.policy.provider {
        return Err("source slice length role belongs to another provider");
    }
    rows.declarations
        .get(method.declaration.declaration.0 as usize)
        .map(|row| row.definition.parent)
        .ok_or("source slice length role lost its actual inherent owner")
}

/// An inherent impl may extend a nominal or (in v14) a trait object whose
/// principal trait is declared by this provider.
fn own_nominal(
    rows: &SourceRows,
    profile: Profile,
    implementation: &ImplHeader,
) -> Result<(), &'static str> {
    let error = "imported inherent impl Self is not a nominal of its own provider";
    if implementation.unsafe_
        || (implementation.constant && !profile.declaration_policies())
        || !implementation.polarity.is_positive()
    {
        return Err(error);
    }
    let valid = match rows.types.get(implementation.self_type.0 as usize) {
        Some(Type::Nominal { declaration, .. }) => {
            declaration.artifact == ArtifactRef(0)
                && rows
                    .declarations
                    .get(declaration.declaration.0 as usize)
                    .is_some_and(|row| {
                        matches!(
                            row.definition.kind,
                            DefinitionKind::Struct | DefinitionKind::Enum
                        )
                    })
        }
        Some(Type::Dynamic(reference)) if profile.declaration_policies() => rows
            .cold()
            .dynamics
            .objects
            .get(reference.0 as usize)
            .and_then(|row| row.principal)
            .is_some_and(|principal| {
                principal.artifact == ArtifactRef(0)
                    && rows
                        .declarations
                        .get(principal.declaration.0 as usize)
                        .is_some_and(|row| row.definition.kind == DefinitionKind::Trait)
            }),
        _ => false,
    };
    if !valid {
        return Err(error);
    }
    Ok(())
}
#[cfg(test)]
#[path = "../../../../tests/invariants/traitless_impl_profiles.rs"]
mod tests;
