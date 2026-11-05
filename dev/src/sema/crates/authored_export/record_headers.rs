//! Genuine source named structs retain canonical fields and declaration premises.
use super::{Declarations, DefId, DefKind, Report, Stage, g, s};
use crate::{
    resolve::model::Access,
    sema::members::Name,
    support::crates::RecordRepr,
    types::{ItemId, aggregate},
};
impl Stage {
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
            if definition.kind != DefKind::Struct {
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
        let repr = match shape.repr {
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
            } => RecordRepr::C,
            _ => return Err("source nominal representation requires its wider complete contract"),
        };
        if facts.aggregates.shared_mutation(item) != Some(aggregate::SharedMutation::Ordinary)
            || facts.aggregates.permits_dangling_pointers(item) != Some(false)
            || shape.kind != aggregate::Kind::Struct
            || facts.aggregates.destruction(item) != Some(aggregate::FieldDestruction::Fields)
        {
            return Err("source nominal destruction/shape requires its wider complete contract");
        }
        let [variant] = facts.members.variants(item) else {
            return Err("source struct lacks its actual field group");
        };
        let fields = facts
            .members
            .fields(*variant)
            .ok_or("source struct field group is unknown")?;
        let declaration = self.declaration(owner)?;
        let view = facts
            .publication_header_contracts(report, owner)
            .map_err(|_| "source nominal advisory contract is incomplete")?;
        let contract = self
            .header_contract(declaration, view)
            .map_err(|_| "source nominal advisory contract is unsupported")?;
        if contract.inline.is_some() {
            return Err("source nominal cannot carry a callable inline policy");
        }
        let context = self.frame(declaration, None, s::BinderKind::DeclarationHeader, 0)?;
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
            let Name::Named(name) = field.name else {
                return Err("source nominal tuple fields require their wider profile");
            };
            let name = facts.members.names.get(name);
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
