//! Genuine source named structs retain canonical fields and declaration premises.
use super::{Declarations, DefId, DefKind, Report, Stage, g, s};
use crate::{
    resolve::model::Access,
    sema::members::Name,
    support::crates::RecordRepr,
    syntax::item::Shape,
    types::{ItemId, aggregate},
};
impl Stage {
    /// Version thirteen: a tuple or unit struct's constructor travels as its
    /// struct's single variant declaration. Every consumer re-derives its
    /// shape, payload and visibility from the struct, so the forms whose
    /// shape or visibility the struct's rows cannot carry stay rejected.
    pub(super) fn struct_constructor(
        &mut self,
        report: &Report,
        facts: &Declarations,
        constructor: DefId,
    ) -> Result<(), &'static str> {
        self.charge(2)?;
        let variant = facts
            .members
            .constructor(constructor)
            .ok_or("source struct constructor lacks its actual field group")?;
        let declaration = facts.members.variants[variant.0 as usize];
        let owner = report
            .resolution
            .definitions
            .get(declaration.owner.0 as usize)
            .ok_or("source struct constructor lost its actual struct")?;
        if owner.kind != DefKind::Struct || facts.members.variants(declaration.owner) != [variant] {
            return Err("source struct constructor changed its actual struct owner");
        }
        // `struct T();` has the unit form's empty field rows.
        if declaration.shape == Shape::Tuple && declaration.start == declaration.end {
            return Err("source empty tuple struct constructors require their wider profile");
        }
        let node = owner
            .node
            .ok_or("source struct constructor lost its actual declaration")?;
        // No row carries the crate-wide narrowing `#[non_exhaustive]` applies.
        if facts
            .non_exhaustive_contract(report, node)
            .map_err(|_| "source struct non_exhaustive contract is incomplete")?
            .is_some()
        {
            return Err("source non_exhaustive struct constructors require their wider profile");
        }
        Ok(())
    }
    pub(super) fn nominals(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        self.reserve(
            report
                .package
                .modules
                .len()
                .saturating_mul(size_of::<Option<super::DeclarationRef>>()),
        )?;
        let mut modules = vec![None; report.package.modules.len()];
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.charge(1)?;
            if definition.kind == DefKind::Module {
                let owner =
                    DefId(u32::try_from(index).map_err(|_| "source nominal module overflow")?);
                let module = crate::resolve::impl_restrictions::module(
                    &report.package,
                    &report.resolution,
                    owner,
                )?;
                modules[module.0 as usize] = Some(self.declaration(owner)?);
            }
        }
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.charge(1)?;
            if definition.kind != DefKind::Struct
                && !(definition.kind == DefKind::Enum && self.body_nodes())
            {
                continue;
            }
            let owner = DefId(u32::try_from(index).map_err(|_| "source nominal owner overflow")?);
            self.nominal(report, facts, owner, &modules)?;
        }
        Ok(())
    }
    fn nominal(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
        modules: &[Option<super::DeclarationRef>],
    ) -> Result<(), &'static str> {
        let item = ItemId(owner.0);
        let shape = facts
            .aggregates
            .get(item)
            .ok_or("source nominal lacks its completed shape")?;
        let enumeration = shape.kind == aggregate::Kind::Enum;
        let repr = record_repr(shape.repr, enumeration, self.body_nodes())?;
        if facts.aggregates.shared_mutation(item) != Some(aggregate::SharedMutation::Ordinary)
            || facts.aggregates.permits_dangling_pointers(item) != Some(false)
            || !(shape.kind == aggregate::Kind::Struct || enumeration)
            || facts.aggregates.destruction(item) != Some(aggregate::FieldDestruction::Fields)
        {
            return Err("source nominal destruction/shape requires its wider complete contract");
        }
        let fields = if enumeration {
            &[]
        } else {
            let [variant] = facts.members.variants(item) else {
                return Err("source struct lacks its actual field group");
            };
            facts
                .members
                .fields(*variant)
                .ok_or("source struct field group is unknown")?
        };
        let declaration = self.declaration(owner)?;
        let contract = if enumeration {
            self.enum_contract(report, facts, owner, declaration)?
        } else {
            let view = facts
                .publication_header_contracts(report, owner)
                .map_err(|_| "source nominal advisory contract is incomplete")?;
            let contract = self
                .header_contract(declaration, view)
                .map_err(|_| "source nominal advisory contract is unsupported")?;
            if contract.inline.is_some() {
                return Err("source nominal cannot carry a callable inline policy");
            }
            contract
        };
        let context = self.nominal_frame(declaration)?;
        let self_ty = facts
            .definitions
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .ok_or("source nominal lacks its canonical type cell")?;
        let self_type = self.ty(facts, self_ty, context)?;
        let start = u32::try_from(self.rows.fields.len())
            .map_err(|_| "source nominal field count overflow")?;
        self.nominal_fields(report, facts, fields, declaration, context, modules)?;
        let end = u32::try_from(self.rows.fields.len())
            .map_err(|_| "source nominal field count overflow")?;
        let input_start =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source nominal clause overflow")?;
        for input in
            crate::sema::declaration_inputs::own(&facts.generics, facts.crates.as_deref(), owner)?
        {
            self.clause(facts, input, context, None)?;
        }
        let input_end =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source nominal clause overflow")?;
        let declared_inputs = u32::try_from(
            crate::sema::declaration_inputs::declared(
                &facts.generics,
                facts.crates.as_deref(),
                owner,
            )?
            .count(),
        )
        .map_err(|_| "source nominal declared input count overflow")?;
        self.reserve(4 * size_of::<s::NominalHeader>())?;
        self.rows.nominals.push(s::NominalHeader {
            contract,
            declaration,
            scope: declaration,
            self_type,
            fields: g::Range { start, end },
            inputs: g::Range {
                start: input_start,
                end: input_end,
            },
            declared_inputs,
            repr,
        });
        if enumeration {
            self.enum_header(facts, owner, declaration, context)?;
        }
        Ok(())
    }
    fn nominal_fields(
        &mut self,
        report: &Report,
        facts: &Declarations,
        fields: &[crate::sema::members::Field],
        declaration: super::DeclarationRef,
        context: super::Context,
        modules: &[Option<super::DeclarationRef>],
    ) -> Result<(), &'static str> {
        for (ordinal, field) in fields.iter().enumerate() {
            self.charge(1)?;
            // Version thirteen: a tuple struct's fields are named by their
            // ordinals, which no record field can spell.
            let positional;
            let name = match field.name {
                Name::Named(name) => facts.members.names.get(name),
                Name::Index(index) if self.body_nodes() && index as usize == ordinal => {
                    positional = index.to_string();
                    &positional
                }
                Name::Index(_) => {
                    return Err("source nominal tuple fields require their wider profile");
                }
            };
            let source = field
                .origin
                .source()
                .ok_or("source nominal field has no original source identity")?;
            let unit = report.package.unit(source.file);
            if let Some(index) = unit
                .owner_indices
                .get(source.node.get() as usize)
                .copied()
                .flatten()
                && !unit.configured.owners[index.get() as usize - 1]
                    .attributes
                    .is_empty()
            {
                return Err(
                    "source field annotations require their wider complete policy contract",
                );
            }
            let visibility = match field.access {
                Access::Public => s::ModuleVisibility::Public,
                Access::Within(module) => s::ModuleVisibility::Within(
                    modules
                        .get(module.0 as usize)
                        .copied()
                        .flatten()
                        .ok_or("source field visibility lacks its actual module")?,
                ),
                Access::ExternalPrivate => {
                    return Err("source nominal field visibility requires its wider contract");
                }
            };
            let ty = self.ty(facts, field.ty, context)?;
            let site = self.site(declaration, source)?;
            self.reserve(4 * size_of::<s::NominalField>() + name.len())?;
            self.rows.fields.push(s::NominalField {
                owner: declaration,
                ordinal: u32::try_from(ordinal).map_err(|_| "source nominal ordinal overflow")?,
                name: name.to_owned(),
                ty,
                visibility,
                site,
            });
        }
        Ok(())
    }
}
/// The record representation a nominal header carries; an enum's integer
/// tag travels in its variant table. Version thirteen also carries a
/// transparent struct.
fn record_repr(
    repr: aggregate::Repr,
    enumeration: bool,
    body_nodes: bool,
) -> Result<RecordRepr, &'static str> {
    Ok(match repr {
        aggregate::Repr {
            representation: aggregate::Representation::Rust,
            packed: None,
            align: None,
            tag: None,
            arguments: aggregate::ArgumentPassing::Ordinary,
        } => RecordRepr::Rust,
        aggregate::Repr {
            representation: aggregate::Representation::C,
            packed: None,
            align: None,
            tag: None,
            arguments: aggregate::ArgumentPassing::Ordinary,
        } if !enumeration => RecordRepr::C,
        aggregate::Repr {
            representation: aggregate::Representation::Transparent,
            packed: None,
            align: None,
            tag: None,
            arguments: aggregate::ArgumentPassing::Ordinary,
        } if !enumeration && body_nodes => RecordRepr::Transparent,
        // An enum's integer tag travels in its variant table.
        aggregate::Repr {
            representation: aggregate::Representation::Rust,
            packed: None,
            align: None,
            tag: Some(_),
            arguments: aggregate::ArgumentPassing::Ordinary,
        } if enumeration => RecordRepr::Rust,
        _ => return Err("source nominal representation requires its wider complete contract"),
    })
}
