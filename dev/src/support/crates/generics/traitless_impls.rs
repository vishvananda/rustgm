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
    pub(crate) fn inherent_constant_owner(&self, types: &[Type]) -> bool {
        self.trait_application.is_none()
            && !self.constant
            && !self.unsafe_
            && self.polarity.is_positive()
            && self.scope == self.self_definition
            && self.region_captures == 0
            && self.region_bounds.start == self.region_bounds.end
            && (self.core_primitive_inherent
                || matches!(types.get(self.self_type.0 as usize), Some(Type::Nominal { declaration, .. })
                    if declaration.artifact == ArtifactRef(0)))
    }

    /// The constant header validator also checks the completed owner scope;
    /// the type-domain validator checks every argument of a nominal Self.
    pub(crate) fn closed_inherent_constant_owner(&self, types: &[Type]) -> bool {
        self.inherent_constant_owner(types)
            && self.inputs.start == self.inputs.end
            && self.declared_inputs == 0
    }
}

pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.implementations.len().saturating_add(1))?;
    let slice_owner = slice_owner(rows, profile, work)?;
    for implementation in &rows.implementations {
        work.charge(1)?;
        if implementation.core_primitive_inherent {
            let primitive = match rows.types.get(implementation.self_type.0 as usize) {
                Some(Type::F16 | Type::F128) => true,
                Some(Type::Scalar(tag)) => *tag != crate::support::crates::Scalar::Unit,
                _ => false,
            };
            let slice = slice_owner == Some(implementation.self_definition)
                && slice_generic_element(rows, implementation);
            if !(profile.declaration_policies()
                && (primitive && implementation.closed_inherent_constant_owner(&rows.types)
                    || slice && implementation.inherent_constant_owner(&rows.types)))
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

fn slice_generic_element(rows: &SourceRows, implementation: &ImplHeader) -> bool {
    let Some(Type::Slice(element)) = rows.types.get(implementation.self_type.0 as usize) else {
        return false;
    };
    let Some(Type::Parameter(slot)) = rows.types.get(element.0 as usize) else {
        return false;
    };
    rows.slots.get(slot.0 as usize).is_some_and(|slot| {
        slot.owner == implementation.self_definition
            && slot.ordinal == 0
            && slot.index == 0
            && matches!(slot.kind, super::ParameterKind::Type { sized: Some(_) })
    })
}

/// An inherent impl may extend a nominal or (in v14) a trait object whose
/// principal trait is declared by this provider.
fn own_nominal(
    rows: &SourceRows,
    profile: Profile,
    implementation: &ImplHeader,
) -> Result<(), &'static str> {
    let error = "imported inherent impl Self is not a nominal of its own provider";
    if implementation.unsafe_ || implementation.constant || !implementation.polarity.is_positive() {
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
