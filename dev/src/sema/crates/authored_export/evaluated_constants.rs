//! Keep completed values and actual associated declaration originals in the
//! same declaration-ordered header column.
use super::{Context, DeclarationRef, Declarations, DefId, DefKind, Report, Stage, s};
use crate::{
    resolve::package::Error,
    sema::{
        associated::{ImplPolarity, OwnerKind},
        model::InitializerKind,
        values::{Evaluator, Instance, Reader},
    },
    support::crates::{Scalar, ScalarValue},
    types::{GenericArgs, Kind, Primitive, TypeId},
};

fn closed_inherent_self(facts: &Declarations, owner: DefId) -> Option<TypeId> {
    let schema = facts.associated.schema(owner)?;
    (schema.captures == 0
        && schema.bounds.is_empty()
        && !facts.generics.has_parameters(owner)
        && crate::sema::declaration_inputs::own_view(
            &facts.generics,
            facts.crates.as_deref(),
            owner,
        )
        .is_ok_and(|inputs| inputs.len() == 0))
    .then_some(schema.self_type)
}

pub(super) fn core_primitive_owner(report: &Report, facts: &Declarations, owner: DefId) -> bool {
    report.package.coherence_is_core()
        && closed_inherent_self(facts, owner).is_some_and(|ty| {
            matches!(facts.types.kind(ty), Some(Kind::Primitive(tag)) if tag != Primitive::Str)
        })
}

fn closed_inherent_initializer(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Option<TypeId> {
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
    ) {
        return None;
    }
    let self_type = closed_inherent_self(facts, owner.self_definition)?;
    match facts.types.kind(self_type) {
        Some(Kind::Primitive(tag))
            if tag != Primitive::Str && report.package.coherence_is_core() => {}
        Some(Kind::Adt(item, GenericArgs::EMPTY))
            if report
                .resolution
                .definitions
                .get(item.0 as usize)
                .is_some_and(|definition| {
                    definition.node.is_some()
                        && matches!(definition.kind, DefKind::Struct | DefKind::Enum)
                }) => {}
        _ => return None,
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
    {
        return None;
    }
    Some(ty)
}

fn inherent_constant(report: &Report, facts: &Declarations, definition: DefId) -> Option<Instance> {
    let ty = closed_inherent_initializer(report, facts, definition)?;
    value_type(facts, ty, true).ok()?;
    Some(Instance {
        definition,
        arguments: GenericArgs::EMPTY,
    })
}

/// Evaluate each eligible closed inherent constant, including unused members.
pub(crate) fn prepare_inherent_constants(
    report: &Report,
    facts: &mut Declarations,
    identity: &crate::support::crates::CrateIdentity,
) -> Result<(), Error> {
    let count = facts.initializers.len();
    if count == 0 {
        return Ok(());
    }
    super::require_provider_identity(report, identity).map_err(Error::unlocated)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source export has no actual role catalog"))?
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    crate::sema::header_attributes::reserve_source_contracts(facts, count, 0)?;
    let mut values = Evaluator::new(report, facts);
    for index in 0..count {
        let definition = values.facts().initializers[index].definition;
        if let Some(instance) = inherent_constant(report, values.facts(), definition) {
            values.associated(instance)?;
        }
    }
    Ok(())
}

impl Stage {
    pub(super) fn inherent_constant(
        &self,
        report: &Report,
        facts: &Declarations,
        definition: DefId,
    ) -> Option<Instance> {
        if !self.declaration_policies() {
            return None;
        }
        inherent_constant(report, facts, definition)
    }
    pub(super) fn range_required_constant(&self, facts: &Declarations, definition: DefId) -> bool {
        self.declaration_policies()
            && crate::sema::compiler_items::source_range_headers(facts).is_some_and(
                |(_, [min, max, _])| definition == min.definition || definition == max.definition,
            )
    }
    pub(super) fn authored_required_constant(
        &self,
        report: &Report,
        facts: &Declarations,
        definition: DefId,
    ) -> bool {
        if !self.declaration_policies()
            || facts.associated.trait_owner(definition).is_none()
            || facts.initializer_index(definition).is_some()
        {
            return false;
        }
        let Some(key) = report.resolution.definitions[definition.0 as usize].node else {
            return false;
        };
        let ast = &report.package.unit(key.file).configured.ast;
        let crate::syntax::ast::Kind::Item(index) = ast.node(key.node).kind else {
            return false;
        };
        ast.items.get(index as usize).is_some_and(|item| {
            matches!(
                item.kind,
                crate::syntax::item::ItemKind::Const { value: None, .. }
            )
        })
    }

    pub(super) fn provided_constant(
        &self,
        report: &Report,
        facts: &Declarations,
        definition: DefId,
    ) -> bool {
        if !self.declaration_policies() {
            return false;
        }
        if matches!(
            closed_inherent_initializer(report, facts, definition)
                .and_then(|ty| facts.types.kind(ty)),
            Some(Kind::Adt(_, GenericArgs::EMPTY))
        ) {
            return true;
        }
        facts
            .associated
            .owner(definition)
            .and_then(|owner| facts.associated.get(owner))
            .is_some_and(|owner| {
                owner.source.is_some()
                    && matches!(
                        owner.kind,
                        OwnerKind::Trait { .. }
                            | OwnerKind::Impl {
                                trait_type: Some(_),
                                ..
                            }
                    )
            })
            && facts
                .initializer_index(definition)
                .and_then(|index| facts.initializers.get(index))
                .is_some_and(|initializer| {
                    initializer.definition == definition
                        && initializer.type_checked()
                        && matches!(initializer.kind, InitializerKind::Const)
                })
    }

    pub(super) fn provided_constants(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<Vec<DefId>, &'static str> {
        let mut declarations = Vec::new();
        if self.declaration_policies() {
            for initializer in &facts.initializers {
                self.charge(1)?;
                if self.provided_constant(report, facts, initializer.definition) {
                    self.reserve(4 * size_of::<DefId>())?;
                    declarations.push(initializer.definition);
                }
            }
            self.charge(
                declarations
                    .len()
                    .saturating_mul(declarations.len().checked_ilog2().unwrap_or(0) as usize + 1),
            )?;
            declarations.sort_unstable_by_key(|definition| definition.0);
        }
        Ok(declarations)
    }

    pub(super) fn constant_lexical_owners(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owners: &mut [Option<DefId>],
    ) -> Result<(), &'static str> {
        for initializer in &facts.initializers {
            self.charge(1)?;
            if !self.provided_constant(report, facts, initializer.definition) {
                continue;
            }
            let definition = &report.resolution.definitions[initializer.definition.0 as usize];
            let module = report.resolution.scopes[definition.scope.0 as usize].module;
            let scope = *report
                .resolution
                .node_scopes
                .get(&(module, initializer.node.node))
                .ok_or("source constant initializer has no actual lexical scope")?;
            if owners[scope.0 as usize]
                .replace(initializer.definition)
                .is_some()
            {
                return Err("source constant initializer has a duplicate lexical owner");
            }
        }
        Ok(())
    }

    fn constant_frame(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        declaration: DeclarationRef,
    ) -> Result<Context, &'static str> {
        let schema = facts
            .associated
            .schema(definition)
            .ok_or("source associated constant lost its completed owner environment")?;
        self.frame(
            declaration,
            None,
            s::BinderKind::DeclarationHeader,
            schema.captures,
        )
    }

    pub(super) fn reserve_constant_original(
        &mut self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<(), &'static str> {
        let declaration = self.declaration(definition)?;
        let context = self.constant_frame(facts, definition, declaration)?;
        self.reserve_original(
            definition,
            context,
            crate::support::crates::owner_catalog::BodyKind::Constant,
        )?;
        Ok(())
    }

    pub(crate) fn has_constant_original(
        &mut self,
        definition: DefId,
    ) -> Result<bool, &'static str> {
        if !self.declaration_policies() {
            return Ok(false);
        }
        self.charge(1)?;
        Ok(self.functions.bodies.get(&definition).is_some_and(|body| {
            self.rows.body_owners[body.0 as usize].kind
                == crate::support::crates::owner_catalog::BodyKind::Constant
        }))
    }

    fn associated_constant_header(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        declaration: DeclarationRef,
        range: bool,
        ty: TypeId,
    ) -> Result<s::ConstantHeader, &'static str> {
        let (context, body) = if range {
            (
                Context {
                    owner: declaration,
                    frame: None,
                    body: None,
                },
                s::ConstantBody::Required,
            )
        } else if self.has_constant_original(definition)? {
            let context = self.original_context(definition)?;
            let body = context
                .body()
                .ok_or("source constant lost its original body identity")?;
            (
                Context {
                    body: None,
                    ..context
                },
                s::ConstantBody::Original(body),
            )
        } else {
            (
                self.constant_frame(facts, definition, declaration)?,
                s::ConstantBody::Required,
            )
        };
        Ok(s::ConstantHeader {
            ty: self.ty(facts, ty, context)?,
            body,
        })
    }

    /// Value code 17 reads a constant with a completed value or a closed
    /// inherent initializer identified by the declaration header.
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
            let range = self.range_required_constant(facts, id);
            if range
                || self.authored_required_constant(report, facts, id)
                || self.has_constant_original(id)?
            {
                let header = self.associated_constant_header(facts, id, declaration, range, ty)?;
                self.reserve(size_of::<s::ConstantHeader>())?;
                self.cold()?.constants.push(header);
                continue;
            }
            let codec = value_type(facts, ty, self.declaration_policies())?;
            let reader = Reader::new(facts);
            let value = if let Some(instance) = self.inherent_constant(report, facts, id) {
                reader.associated(instance)
            } else {
                reader.definition(id)
            }
            .ok_or("source constant export requires its completed value")?;
            let bits = match codec {
                ValueType::Scalar(scalar) => {
                    super::super::export::value_bits(scalar, reader.view(value)?)
                        .map(ScalarValue::bits)
                        .map_err(|_| "source constant value disagrees with its declared type")?
                }
                ValueType::WideFloat => reader
                    .encode_primitive(ty, value)
                    .map(|(bits, _)| bits)
                    .map_err(|_| "source constant value disagrees with its declared type")?,
            };
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
                body: s::ConstantBody::Evaluated(s::ConstTerm::Value(s::ConstValue { ty, bits })),
            });
        }
        Ok(())
    }
}
/// Wide named values use the source recipe and the existing payload width.
enum ValueType {
    Scalar(Scalar),
    WideFloat,
}

fn value_type(facts: &Declarations, ty: TypeId, wide: bool) -> Result<ValueType, &'static str> {
    if ty == TypeId::UNIT {
        return Ok(ValueType::Scalar(Scalar::Unit));
    }
    match facts.types.kind(ty) {
        Some(Kind::Primitive(Primitive::F16 | Primitive::F128)) if wide => Ok(ValueType::WideFloat),
        Some(Kind::Primitive(_)) => super::super::export_scalar(facts, ty)
            .map(ValueType::Scalar)
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
