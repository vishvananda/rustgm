//! The published declaration table uses the completed frontend declaration identities.
use super::{
    DeclarationLeaves, DeclarationRef, ExportMode, Stage, implementation_result, kind,
    non_type_sources, require_foreign_declaration, s, scope_owners, scoped_module_alias, statics,
};
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, DefKind, Namespace},
    },
    sema::model::Declarations,
    syntax::item::Shape,
};

pub(super) struct CollectedDeclarations {
    pub(super) leaves: DeclarationLeaves,
    pub(super) order: Vec<(DefId, s::DefinitionKind)>,
}

pub(super) const fn inventory_bytes(count: usize) -> usize {
    count.saturating_mul(size_of::<(DefId, s::DefinitionKind)>())
}

impl Stage {
    pub(super) fn exported_kind(
        &mut self,
        report: &Report,
        facts: &Declarations,
        id: DefId,
        mode: ExportMode,
    ) -> Result<Option<s::DefinitionKind>, &'static str> {
        let definition = &report.resolution.definitions[id.0 as usize];
        if self.declaration_policies() {
            match definition.kind {
                DefKind::TraitAlias => return Ok(Some(s::DefinitionKind::TraitAlias)),
                DefKind::ForeignType => {
                    require_foreign_declaration(report, facts, id)?;
                    return Ok(Some(s::DefinitionKind::ForeignType));
                }
                DefKind::Static => {
                    statics::require_definition(report, facts, id)?;
                    return Ok(Some(s::DefinitionKind::Static));
                }
                DefKind::ExternCrate => {
                    self.private_self_alias(report, id)?;
                    return Ok(None);
                }
                DefKind::Label => {
                    // The checked CFG owns each destination as a block identity.
                    if definition.namespace != Namespace::Label
                        || definition.node.is_none()
                        || definition.members.is_some()
                        || !matches!(definition.access, Access::Within(_))
                    {
                        return Err("source label changed its actual lexical declaration");
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }
        let associated = (mode == ExportMode::CompilerOperations || self.body_nodes())
            && facts.associated.type_contract(id).is_some();
        if associated || self.body_nodes() && implementation_result(report, facts, id) {
            return Ok(Some(s::DefinitionKind::AssociatedType));
        }
        if matches!(
            mode,
            ExportMode::Conversions | ExportMode::Nominals | ExportMode::CompilerOperations
        ) && definition.kind == DefKind::TypeAlias
        {
            if self.declaration_policies() && scoped_module_alias(report, facts, definition, id) {
                self.scoped_alias(report, facts, id)?;
                return Ok(Some(s::DefinitionKind::TypeAlias));
            }
            self.closed_alias(report, facts, id)?;
            return Ok(
                (definition.access == Access::Public).then_some(s::DefinitionKind::TypeAlias)
            );
        }
        if definition.kind == DefKind::Struct
            && matches!(mode, ExportMode::Nominals | ExportMode::CompilerOperations)
        {
            return Ok(Some(s::DefinitionKind::Struct));
        }
        if self.body_nodes() {
            match definition.kind {
                DefKind::Enum => return Ok(Some(s::DefinitionKind::Enum)),
                // One declaration names a variant: a unit or tuple variant's
                // value-namespace constructor, whose type-namespace alias has
                // no row, or a record variant's own type-namespace definition.
                DefKind::Variant if definition.namespace == Namespace::Value => {
                    return Ok(Some(s::DefinitionKind::Variant));
                }
                DefKind::Variant => {
                    let variant = facts
                        .members
                        .constructor(id)
                        .ok_or("source variant lacks its actual constructor")?;
                    return Ok(
                        (facts.members.variants[variant.0 as usize].shape == Shape::Record)
                            .then_some(s::DefinitionKind::Variant),
                    );
                }
                // A tuple or unit struct's constructor is the variant
                // declaration of its struct's single field group.
                DefKind::Constructor if definition.namespace == Namespace::Value => {
                    self.struct_constructor(report, facts, id)?;
                    return Ok(Some(s::DefinitionKind::Variant));
                }
                // Version thirteen: a const generic parameter, whose slot
                // records its primitive type.
                DefKind::Generic if definition.namespace == Namespace::Value => {
                    return Ok(Some(s::DefinitionKind::ConstParameter));
                }
                // Version thirteen: a named const item. An unnamed const has
                // no row: its provider already evaluated it and no consumer
                // can name it.
                DefKind::Const => {
                    return if report.package.names.get(definition.name) == "_" {
                        Ok(None)
                    } else if facts.associated.owner(id).is_some()
                        && !self.range_required_constant(facts, id)
                        && !self.authored_required_constant(report, facts, id)
                        && !self.provided_constant(report, facts, id)
                        && self.inherent_constant(report, facts, id).is_none()
                    {
                        Err("source associated constants require their wider profile")
                    } else {
                        Ok(Some(s::DefinitionKind::Constant))
                    };
                }
                _ => {}
            }
        }
        kind(definition.kind, definition.namespace)
    }
    pub(super) fn declarations(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        mode: ExportMode,
    ) -> Result<CollectedDeclarations, &'static str> {
        let admission = self.declaration_inventory(report, facts, mode)?;
        let mut leaves = DeclarationLeaves::default();
        let mut features =
            non_type_sources::Coverage::new(report, self, mode.feature_predicates())?;
        let definitions = report.resolution.definitions.len();
        let lexical_bytes = definitions.saturating_mul(size_of::<u32>()).saturating_add(
            report
                .resolution
                .scopes
                .len()
                .saturating_mul(size_of::<Option<DefId>>()),
        );
        self.reserve(lexical_bytes)?;
        let mut owners = scope_owners(report, facts, self)?;
        let root = report
            .resolution
            .crate_root
            .ok_or("source provider has no actual root")?;
        let provider = report
            .package
            .source_core()
            .ok_or("source provider policy disappeared")?;
        let mut ordinals = vec![0_u32; definitions];
        for &(id, kind) in &admission {
            let index = id.0 as usize;
            let definition = &report.resolution.definitions[index];
            features.observe(report, facts, id, self)?;
            if !mode.module_contracts()
                && definition.kind == DefKind::Trait
                && crate::resolve::impl_restrictions::clause(report, id)
                    .map_err(|_| "source trait implementation domain is invalid")?
                    .is_some()
            {
                return Err("restricted trait implementation policy transport is not implemented");
            }
            let parent = self.declaration_parent(report, facts, &mut owners, id, kind)?;
            let ordinal = if let Some(parent) = parent {
                let next = ordinals
                    .get_mut(parent.0 as usize)
                    .ok_or("source lexical owner is unknown")?;
                let ordinal = *next;
                *next = next.checked_add(1).ok_or("source child ordinal overflow")?;
                ordinal
            } else {
                0
            };
            // The resolver names its exact root `crate`; its external module
            // spelling belongs to the authenticated artifact identity.
            let name = if id == root {
                provider.provider.name()
            } else {
                report.package.names.get(definition.name)
            };
            self.reserve(name.len())?;
            let row = self.definitions[index].ok_or("source declaration lost its reserved row")?;
            let parent = parent.map(|id| self.declaration(id)).transpose()?;
            if kind == s::DefinitionKind::Variant {
                self.publish_variant(facts, id, row)?;
            }
            leaves.record(self, id, row, kind)?;
            self.rows.declarations.push(s::Declaration {
                definition: s::Definition { kind, parent },
                ordinal,
                name: Some(name.to_owned()),
                public: definition.access == Access::Public,
                sites: 1,
                local: None,
            });
        }
        drop(ordinals);
        drop(owners);
        self.release_temporary(lexical_bytes)?;
        features.finish()?;
        Ok(CollectedDeclarations {
            leaves,
            order: admission,
        })
    }

    fn declaration_inventory(
        &mut self,
        report: &Report,
        facts: &Declarations,
        mode: ExportMode,
    ) -> Result<Vec<(DefId, s::DefinitionKind)>, &'static str> {
        let count = report.resolution.definitions.len();
        self.reserve(inventory_bytes(count))?;
        let mut admitted = Vec::with_capacity(count);
        for index in 0..count {
            self.charge(1)?;
            let id =
                DefId(u32::try_from(index).map_err(|_| "source declaration identity overflow")?);
            if let Some(kind) = self.exported_kind(report, facts, id, mode)? {
                self.definitions[index] = Some(DeclarationRef(
                    u32::try_from(admitted.len())
                        .map_err(|_| "source declaration table overflow")?,
                ));
                admitted.push((id, kind));
            }
        }
        self.reserve(admitted.len().saturating_mul(size_of::<s::Declaration>()))?;
        self.rows.declarations.reserve_exact(admitted.len());
        Ok(admitted)
    }
}
