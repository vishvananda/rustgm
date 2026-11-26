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

fn closed_type(facts: &Declarations, ty: TypeId) -> bool {
    facts.types.contains_parameters(ty) == Some(false)
        && facts.types.needs_region_context(ty) == Some(false)
        && facts.types.contains_projections(ty) == Some(false)
}

fn inherent_initializer(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Option<(DefId, TypeId)> {
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
            polarity: ImplPolarity::Positive,
            ..
        }
    ) {
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
    {
        return None;
    }
    Some((owner.self_definition, ty))
}

fn closed_inherent_initializer(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Option<TypeId> {
    let (owner, ty) = inherent_initializer(report, facts, definition)?;
    let source = facts.associated.get(facts.associated.owner(definition)?)?;
    if !matches!(
        source.kind,
        OwnerKind::Impl {
            constant: false,
            ..
        }
    ) {
        return None;
    }
    let self_type = closed_inherent_self(facts, owner)?;
    (local_nominal(report, facts, self_type) && closed_type(facts, self_type)
        || report.package.coherence_is_core()
            && matches!(facts.types.kind(self_type), Some(Kind::Primitive(tag)) if tag != Primitive::Str))
    .then_some(ty)
}

fn free_initializer(report: &Report, facts: &Declarations, definition: DefId) -> Option<TypeId> {
    let declaration = report.resolution.definitions.get(definition.0 as usize)?;
    if declaration.kind != DefKind::Const
        || report.package.names.get(declaration.name) == "_"
        || facts.associated.owner(definition).is_some()
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
    (declaration.node == Some(initializer.node)
        && matches!(initializer.kind, InitializerKind::Const)
        && initializer.type_checked()
        && initializer.ty == ty)
        .then_some(ty)
}

fn local_nominal(report: &Report, facts: &Declarations, ty: TypeId) -> bool {
    matches!(facts.types.kind(ty), Some(Kind::Adt(item, _))
    if report.resolution.definitions.get(item.0 as usize).is_some_and(|definition| {
        definition.node.is_some() && matches!(definition.kind, DefKind::Struct | DefKind::Enum)
    }))
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
        if inherent_initializer(report, facts, definition).is_some() {
            return self.inherent_constant(report, facts, definition).is_none();
        }
        if let Some(ty) = free_initializer(report, facts, definition) {
            return value_type(facts, ty, true).is_err();
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
        let captures = if let Some(schema) = facts.associated.schema(definition) {
            schema.captures
        } else if facts.associated.owner(definition).is_none() {
            0
        } else {
            return Err("source associated constant lost its completed owner environment");
        };
        self.frame(
            declaration,
            None,
            s::BinderKind::DeclarationHeader,
            captures,
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
    /// A closed original constant keeps its header's exact recipe identity.
    /// Callable recipes otherwise receive the caller's distinct binder frame.
    pub(crate) fn constant_value_type(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        ty: TypeId,
        caller: Context,
    ) -> Result<crate::support::crates::generics::TypeRef, &'static str> {
        let context = if self.declaration_policies()
            && closed_type(facts, ty)
            && facts
                .definitions
                .get(definition.0 as usize)
                .copied()
                .flatten()
                == Some(ty)
            && self.has_constant_original(definition)?
        {
            let body = self.functions.bodies[&definition];
            let original = *self
                .functions
                .contexts
                .get(body.0 as usize)
                .ok_or("source constant read lost its actual defining frame")?;
            Context {
                body: None,
                ..original
            }
        } else {
            caller
        };
        self.ty(facts, ty, context)
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
        facts: &mut Declarations,
        identity: &super::CrateIdentity,
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
            let body = self.completed_constant(facts, identity, (declaration, ty), codec, value)?;
            self.reserve(size_of::<s::ConstantHeader>())?;
            self.cold()?.constants.push(body);
        }
        Ok(())
    }

    fn completed_constant(
        &mut self,
        facts: &mut Declarations,
        identity: &super::CrateIdentity,
        (declaration, ty): (DeclarationRef, TypeId),
        codec: ValueType,
        value: crate::sema::values::Constant,
    ) -> Result<s::ConstantHeader, &'static str> {
        let raw_ty = self.ty(
            facts,
            ty,
            Context {
                owner: declaration,
                frame: None,
                body: None,
            },
        )?;
        let reader = Reader::new(facts);
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
            ValueType::InlineTuple => {
                let mut bytes = Vec::new();
                self.scalar_aggregate_bytes(facts, ty, value, &mut bytes)?;
                let mut payload = [0; size_of::<u128>()];
                payload
                    .get_mut(..bytes.len())
                    .ok_or("source tuple constant exceeds its inline payload")?
                    .copy_from_slice(&bytes);
                u128::from_le_bytes(payload)
            }
            ValueType::PromotedReference => {
                return Ok(s::ConstantHeader {
                    ty: raw_ty,
                    body: self.promoted_reference(facts, identity, declaration, ty, value)?,
                });
            }
        };
        Ok(s::ConstantHeader {
            ty: raw_ty,
            body: s::ConstantBody::Evaluated(s::ConstTerm::Value(s::ConstValue {
                ty: raw_ty,
                bits,
            })),
        })
    }
}
/// Wide named values use the source recipe and the existing payload width.
#[derive(Clone, Copy)]
enum ValueType {
    Scalar(Scalar),
    WideFloat,
    InlineTuple,
    PromotedReference,
}

fn value_type(facts: &Declarations, ty: TypeId, v14: bool) -> Result<ValueType, &'static str> {
    if ty == TypeId::UNIT {
        return Ok(ValueType::Scalar(Scalar::Unit));
    }
    match facts.types.kind(ty) {
        Some(Kind::Primitive(Primitive::F16 | Primitive::F128)) if v14 => Ok(ValueType::WideFloat),
        Some(Kind::Primitive(_)) => super::super::export_scalar(facts, ty)
            .map(ValueType::Scalar)
            .map_err(|_| "source wide primitive const value requires its wider profile"),
        Some(Kind::Tuple(fields)) if v14 => {
            let fields = facts
                .types
                .list(fields)
                .ok_or("source tuple constant lost its actual field types")?;
            let mut width = 0;
            for &field in fields {
                let ty = super::super::export_scalar(facts, field)
                    .map_err(|_| "source tuple constant requires actual scalar field types")?;
                width = s::inline_scalar_width(ty, width)?;
            }
            Ok(ValueType::InlineTuple)
        }
        Some(Kind::Reference { .. })
            if v14 && super::promoted_references::referent(facts, ty).is_some() =>
        {
            Ok(ValueType::PromotedReference)
        }
        Some(Kind::Reference { .. }) => {
            Err("source reference const value requires its wider profile")
        }
        Some(Kind::Pointer { .. } | Kind::Function { .. }) => {
            Err("source pointer const value requires its wider profile")
        }
        _ => Err("source aggregate const value requires its wider profile"),
    }
}
