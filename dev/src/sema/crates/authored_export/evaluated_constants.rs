//! Version thirteen: each named constant's header row carries its declared
//! primitive type and its completed value. No initializer travels; every
//! consumer re-proves the value's canonical bits for that type.
use super::{Context, Declarations, DefId, DefKind, Report, Stage, s};
use crate::{
    sema::values::Reader,
    support::crates::Scalar,
    types::{Kind, TypeId},
};
impl Stage {
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
        if !self.body_nodes()
            || facts.associated.owner(definition).is_some()
            || Reader::new(facts).definition(definition).is_none()
        {
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
            let scalar = value_type(facts, ty)?;
            let reader = Reader::new(facts);
            let value = reader
                .definition(id)
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
