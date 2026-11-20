//! Version thirteen: each named constant's header row carries its declared
//! primitive type and its completed value. No initializer travels; every
//! consumer re-proves the value's canonical bits for that type.
use super::{Context, Declarations, DefId, DefKind, Report, Stage, s};
use crate::{
    resolve::package::Error,
    sema::{
        associated::{ImplPolarity, OwnerKind},
        model::InitializerKind,
        values::{Evaluator, Instance, Reader},
    },
    support::crates::Scalar,
    types::{GenericArgs, Kind, Primitive, TypeId},
};

pub(super) fn core_primitive_owner(report: &Report, facts: &Declarations, owner: DefId) -> bool {
    report.package.coherence_is_core()
        && facts.associated.schema(owner).is_some_and(|schema| {
            matches!(facts.types.kind(schema.self_type), Some(Kind::Primitive(tag)) if tag != Primitive::Str)
                && schema.captures == 0
                && schema.bounds.is_empty()
                && !facts.generics.has_parameters(owner)
        })
        && crate::sema::declaration_inputs::own_view(&facts.generics, facts.crates.as_deref(), owner)
            .is_ok_and(|inputs| inputs.len() == 0)
}

fn primitive_constant(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Option<Instance> {
    let declaration = report.resolution.definitions.get(definition.0 as usize)?;
    if declaration.kind != DefKind::Const {
        return None;
    }
    let owner = facts.associated.get(facts.associated.owner(definition)?)?;
    if !matches!(
        owner.kind,
        OwnerKind::Impl {
            trait_type: None,
            unsafe_: false,
            constant: false,
            polarity: ImplPolarity::Positive,
            ..
        }
    ) || !core_primitive_owner(report, facts, owner.self_definition)
    {
        return None;
    }
    let initializer = facts
        .initializers
        .get(facts.initializer_index(definition)?)?;
    let ty = facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten()?;
    if declaration.node != Some(initializer.node)
        || !matches!(initializer.kind, InitializerKind::Const)
        || !initializer.type_checked()
        || initializer.ty != ty
        || value_type(facts, ty).is_err()
    {
        return None;
    }
    Some(Instance {
        definition,
        arguments: GenericArgs::EMPTY,
    })
}

/// Evaluate every eligible primitive member, including ones no original reads.
pub(crate) fn prepare_primitive_constants(
    report: &Report,
    facts: &mut Declarations,
    identity: &crate::support::crates::CrateIdentity,
) -> Result<(), Error> {
    if !report.package.coherence_is_core() {
        return Ok(());
    }
    super::require_provider_identity(report, identity).map_err(Error::unlocated)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source export has no actual role catalog"))?
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let count = facts.initializers.len();
    crate::sema::header_attributes::reserve_source_contracts(facts, count, 0)?;
    let mut values = Evaluator::new(report, facts);
    for index in 0..count {
        let definition = values.facts().initializers[index].definition;
        if let Some(instance) = primitive_constant(report, values.facts(), definition) {
            values.associated(instance)?;
        }
    }
    Ok(())
}

impl Stage {
    pub(super) fn primitive_constant(
        &self,
        report: &Report,
        facts: &Declarations,
        definition: DefId,
    ) -> Option<Instance> {
        if !self.declaration_policies() {
            return None;
        }
        primitive_constant(report, facts, definition)
    }
    pub(super) fn range_required_constant(&self, facts: &Declarations, definition: DefId) -> bool {
        self.declaration_policies()
            && crate::sema::compiler_items::source_range_headers(facts).is_some_and(
                |(_, [min, max, _])| definition == min.definition || definition == max.definition,
            )
    }

    /// Version thirteen: value code 17 names the exported constant
    /// declaration a body reads. Its value travels once, in that
    /// declaration's header row.
    pub(crate) fn constant_read(
        &mut self,
        definition: DefId,
    ) -> Result<s::ValueKind, &'static str> {
        self.charge(1)?;
        let Some(row) = self
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
        else {
            return Err(
                "source constant read of another crate's declaration requires its wider profile",
            );
        };
        if self
            .rows
            .declarations
            .get(row.0 as usize)
            .is_none_or(|row| row.definition.kind != s::DefinitionKind::Constant)
        {
            return Err("source constant read names no validated constant declaration");
        }
        Ok(s::ValueKind::Constant(row))
    }
    /// Version thirteen: a free const's initializer is not an original. Its
    /// value travels evaluated in its header row; an unnamed const's travels
    /// nowhere, since no consumer can name it.
    pub(crate) fn evaluated_initializer(
        &mut self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<bool, &'static str> {
        self.charge(1)?;
        if !self.body_nodes() {
            return Ok(false);
        }
        if facts.associated.owner(definition).is_some() {
            return Ok(self.declaration_policies()
                && self
                    .definitions
                    .get(definition.0 as usize)
                    .is_some_and(Option::is_some)
                && Reader::new(facts)
                    .associated(Instance {
                        definition,
                        arguments: GenericArgs::EMPTY,
                    })
                    .is_some());
        }
        if Reader::new(facts).definition(definition).is_none() {
            return Ok(false);
        }
        Ok(self
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .is_none_or(|row| {
                self.rows
                    .declarations
                    .get(row.0 as usize)
                    .is_some_and(|row| row.definition.kind == s::DefinitionKind::Constant)
            }))
    }
    /// One row per exported constant declaration, in declaration order.
    pub(super) fn constant_headers(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        // Every declaration row was paid when it was pushed; a provider
        // without constants pays nothing more here.
        if !self
            .rows
            .declarations
            .iter()
            .any(|row| row.definition.kind == s::DefinitionKind::Constant)
        {
            return Ok(());
        }
        self.charge(report.resolution.definitions.len())?;
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            if definition.kind != DefKind::Const {
                continue;
            }
            let Some(declaration) = self.definitions.get(index).copied().flatten() else {
                continue;
            };
            let id = DefId(u32::try_from(index).map_err(|_| "source constant identity overflow")?);
            let ty = facts
                .definitions
                .get(index)
                .copied()
                .flatten()
                .ok_or("source constant export requires its completed value")?;
            if self.range_required_constant(facts, id) {
                let ty = self.ty(
                    facts,
                    ty,
                    Context {
                        owner: declaration,
                        frame: None,
                        body: None,
                    },
                )?;
                self.reserve(size_of::<s::ConstantHeader>())?;
                self.cold()?
                    .constants
                    .push(s::ConstantHeader { ty, value: None });
                continue;
            }
            let scalar = value_type(facts, ty)?;
            let reader = Reader::new(facts);
            let value = if let Some(instance) = self.primitive_constant(report, facts, id) {
                reader.associated(instance)
            } else {
                reader.definition(id)
            }
            .ok_or("source constant export requires its completed value")?;
            let value = super::super::export::value_bits(scalar, reader.view(value)?)
                .map_err(|_| "source constant value disagrees with its declared type")?;
            let ty = self.ty(
                facts,
                ty,
                Context {
                    owner: declaration,
                    frame: None,
                    body: None,
                },
            )?;
            self.reserve(size_of::<s::ConstantHeader>())?;
            self.cold()?.constants.push(s::ConstantHeader {
                ty,
                value: Some(s::ConstTerm::Value(s::ConstValue {
                    ty,
                    bits: value.bits(),
                })),
            });
        }
        Ok(())
    }
}
/// A constant travels only as one primitive value or unit; every wider value
/// keeps its fence by name.
fn value_type(facts: &Declarations, ty: TypeId) -> Result<Scalar, &'static str> {
    if ty == TypeId::UNIT {
        return Ok(Scalar::Unit);
    }
    match facts.types.kind(ty) {
        Some(Kind::Primitive(_)) => super::super::export_scalar(facts, ty)
            .map_err(|_| "source wide primitive const value requires its wider profile"),
        Some(Kind::Reference { .. }) => {
            Err("source reference const value requires its wider profile")
        }
        Some(Kind::Pointer { .. } | Kind::Function { .. }) => {
            Err("source pointer const value requires its wider profile")
        }
        _ => Err("source aggregate const value requires its wider profile"),
    }
}
