//! Complete version-two names and domains are structural facts, never paid proof.
#[path = "implementation_domains/lexical_modules.rs"]
mod module_index;
use super::super::{HeaderIndex, SourceRows};
use crate::support::{
    crates::{
        self, Namespace,
        generics::{Work, source as s},
        owner_catalog::DeclarationRef,
    },
    retained,
};
use module_index::ModuleIndex;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleVisibility {
    Public,
    Within(DeclarationRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingSource {
    Declaration,
    Import,
    Glob,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleRow {
    pub declaration: DeclarationRef,
    pub declaration_site: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleBindingRow {
    pub module: DeclarationRef,
    pub name: String,
    pub namespace: Namespace,
    pub target: s::DeclarationAddress,
    pub visibility: ModuleVisibility,
    pub source: BindingSource,
    pub source_site: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImplementationDomain {
    Unrestricted,
    RestrictedTo {
        module: DeclarationRef,
        clause_site: u32,
        path_site: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraitDomainRow {
    pub declaration: DeclarationRef,
    pub declaration_site: u32,
    pub defining_module: DeclarationRef,
    pub domain: ImplementationDomain,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleNamespace {
    pub root: DeclarationRef,
    pub modules: Vec<ModuleRow>,
    pub bindings: Vec<ModuleBindingRow>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleContractRows {
    pub namespace: ModuleNamespace,
    pub domains: Vec<TraitDomainRow>,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct ModuleContractColumns {
    rows: ModuleContractRows,
    index: ModuleIndex,
}
#[derive(Clone, Copy)]
pub struct ModuleContractView<'a> {
    columns: &'a ModuleContractColumns,
}
impl<'a> ModuleContractView<'a> {
    #[must_use]
    pub const fn rows(self) -> &'a ModuleContractRows {
        &self.columns.rows
    }
    /// # Errors
    /// Rejects an unknown actual declaration coordinate.
    pub fn defining_module(self, row: DeclarationRef) -> Result<DeclarationRef, &'static str> {
        self.columns.index.module(row)
    }
    /// # Errors
    /// Rejects a coordinate that is not a validated actual module.
    pub fn require_module(self, row: DeclarationRef) -> Result<(), &'static str> {
        if self.defining_module(row)? != row {
            return Err("source coordinate is not an actual module");
        }
        Ok(())
    }
    /// # Errors
    /// Rejects missing or mismatched completion, never infers Unrestricted.
    pub fn trait_domain(
        self,
        index: usize,
        declaration: DeclarationRef,
    ) -> Result<&'a TraitDomainRow, &'static str> {
        self.columns
            .rows
            .domains
            .get(index)
            .filter(|row| row.declaration == declaration)
            .ok_or("source implementation-domain completion is absent or foreign")
    }
}
impl ModuleContractRows {
    pub(crate) fn storage_bytes(
        &self,
        declarations: usize,
        work: &mut Work,
    ) -> Result<usize, &'static str> {
        work.charge(1)?;
        if self.namespace.modules.len() > crates::MAX_FUNCTIONS
            || self.domains.len() > crates::MAX_FUNCTIONS
            || self.namespace.bindings.len() > crates::MAX_COMPONENTS
            || declarations > crates::MAX_FUNCTIONS
        {
            return Err("module contract cardinality exceeds its bounded profile");
        }
        work.charge(self.namespace.bindings.len())?;
        Ok(size_of::<ModuleContractColumns>()
            .saturating_add(retained::vector(&self.namespace.modules))
            .saturating_add(retained::vector(&self.namespace.bindings))
            .saturating_add(retained::vector(&self.domains))
            .saturating_add(
                self.namespace
                    .bindings
                    .iter()
                    .map(|row| row.name.capacity())
                    .fold(0, usize::saturating_add),
            )
            .saturating_add(declarations.saturating_mul(3 * size_of::<u32>())))
    }
    /// Source producer only: decode requires an already canonical order.
    pub(crate) fn canonicalize(&mut self, work: &mut Work) -> Result<(), &'static str> {
        let base = self.storage_bytes(0, work)?;
        if base.saturating_add(
            self.namespace
                .bindings
                .len()
                .saturating_mul(2 * size_of::<usize>()),
        ) > crates::MAX_BYTES
        {
            return Err("module name ordering exceeds combined storage");
        }
        super::binding_order::sort(&mut self.namespace.bindings, work, &mut 0, compare)
    }
}
impl ModuleContractColumns {
    pub(super) fn validate(
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        raw: ModuleContractRows,
        base: usize,
        work: &mut Work,
    ) -> Result<(Self, usize, usize), &'static str> {
        let bytes = raw.storage_bytes(rows.declarations.len(), work)?;
        let scratch = module_index::scratch_bytes(rows.declarations.len())
            .saturating_add(rows.declarations.len().saturating_mul(size_of::<bool>()))
            .saturating_add(size_of::<Vec<bool>>());
        if base.saturating_add(bytes).saturating_add(scratch) > crates::MAX_BYTES {
            return Err("module contracts exceed combined staging storage");
        }
        let index = ModuleIndex::new(rows, headers, work)?;
        let result = Self { rows: raw, index };
        result.namespace(rows, work)?;
        result.domains(rows, headers, work)?;
        Ok((result, bytes, bytes.saturating_add(scratch)))
    }
    pub(super) const fn view(&self) -> ModuleContractView<'_> {
        ModuleContractView { columns: self }
    }
    fn namespace(&self, rows: &SourceRows, work: &mut Work) -> Result<(), &'static str> {
        let names = &self.rows.namespace;
        work.charge(rows.declarations.len().saturating_add(names.modules.len()))?;
        if rows
            .declarations
            .get(names.root.0 as usize)
            .is_none_or(|row| {
                row.definition.kind != s::DefinitionKind::Module || row.definition.parent.is_some()
            })
        {
            return Err("module namespace root is not its actual declaration");
        }
        let mut actual = rows
            .declarations
            .iter()
            .enumerate()
            .filter(|(_, row)| row.definition.kind == s::DefinitionKind::Module);
        for module in &names.modules {
            work.charge(1)?;
            let (index, _) = actual
                .next()
                .ok_or("module namespace has extra module rows")?;
            if module.declaration.0 as usize != index {
                return Err("module namespace has missing/reordered actual modules");
            }
            site(rows, module.declaration, module.declaration_site)?;
        }
        if actual.next().is_some() {
            return Err("module namespace omits an actual module");
        }
        let mut name_bytes = 0_usize;
        for entry in &names.bindings {
            work.charge(1 + entry.name.len())?;
            name_bytes = name_bytes.saturating_add(entry.name.len());
            if name_bytes > crates::MAX_NAME_BYTES {
                return Err("module binding names exceed their combined byte budget");
            }
            self.binding(rows, entry)?;
        }
        for pair in names.bindings.windows(2) {
            if compare(&pair[0], &pair[1], work)? != Ordering::Less {
                return Err("module namespace keys are duplicate or noncanonical");
            }
        }
        // Every supported direct module declaration has a binding, including private owners.
        work.charge(rows.declarations.len().saturating_add(names.bindings.len()))?;
        let mut direct = vec![false; rows.declarations.len()];
        for entry in &names.bindings {
            if entry.source == BindingSource::Declaration {
                direct[entry.target.declaration.0 as usize] = true;
            }
        }
        for (index, row) in rows.declarations.iter().enumerate() {
            work.charge(1)?;
            if matches!(
                row.definition.kind,
                s::DefinitionKind::Module | s::DefinitionKind::Trait | s::DefinitionKind::Function
            ) && row.definition.parent.is_some_and(|p| {
                rows.declarations[p.0 as usize].definition.kind == s::DefinitionKind::Module
            }) && !direct[index]
            {
                return Err("module namespace omits an actual direct declaration");
            }
        }
        Ok(())
    }
    fn binding(&self, rows: &SourceRows, entry: &ModuleBindingRow) -> Result<(), &'static str> {
        self.view().require_module(entry.module)?;
        site(rows, entry.module, entry.source_site)?;
        crates::identity::validate_component(&entry.name)?;
        if entry.target.artifact != s::ArtifactRef(0) {
            return Err("foreign module targets require the mixed-provider namespace profile");
        }
        let target = rows
            .declarations
            .get(entry.target.declaration.0 as usize)
            .ok_or("module target declaration is absent")?;
        let expected = match target.definition.kind {
            s::DefinitionKind::Module | s::DefinitionKind::Trait => Namespace::Type,
            s::DefinitionKind::Function => Namespace::Value,
            _ => return Err("module binding target requires a wider declaration profile"),
        };
        if expected != entry.namespace {
            return Err("module binding changed its target namespace");
        }
        match entry.visibility {
            ModuleVisibility::Public if !target.public => {
                return Err("public module binding exposes a private declaration");
            }
            ModuleVisibility::Within(ancestor) => {
                self.view().require_module(ancestor)?;
                if !self.index.within(entry.module, ancestor)? {
                    return Err("module binding visibility is not an actual ancestor");
                }
            }
            ModuleVisibility::Public => {}
        }
        if entry.source == BindingSource::Declaration
            && (target.definition.parent != Some(entry.module)
                || target.name.as_deref() != Some(entry.name.as_str())
                || target.public != matches!(entry.visibility, ModuleVisibility::Public))
        {
            return Err("direct module binding changed its actual declaration/owner/access");
        }
        Ok(())
    }
    fn domains(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(self.rows.domains.len().saturating_add(rows.traits.len()))?;
        if self.rows.domains.len() != rows.traits.len() {
            return Err("source implementation-domain rows are incomplete");
        }
        for (domain, header) in self.rows.domains.iter().zip(&rows.traits) {
            work.charge(1)?;
            if domain.declaration != header.declaration
                || domain.defining_module != self.index.module(header.declaration)?
            {
                return Err("implementation domain changed its actual trait/defining module");
            }
            site(rows, domain.declaration, domain.declaration_site)?;
            let restricted = header.flags.bits() & s::TraitFlags::RESTRICTED_IMPL != 0;
            match domain.domain {
                ImplementationDomain::Unrestricted if restricted => {
                    return Err("restricted trait lacks its complete domain");
                }
                ImplementationDomain::RestrictedTo {
                    module,
                    clause_site,
                    path_site,
                } => {
                    if !restricted {
                        return Err("domain restriction differs from its actual header claim");
                    }
                    self.view().require_module(module)?;
                    site(rows, domain.declaration, clause_site)?;
                    site(rows, domain.declaration, path_site)?;
                    if !self.index.within(domain.defining_module, module)? {
                        return Err("trait restriction is not an actual ancestor");
                    }
                }
                ImplementationDomain::Unrestricted => {}
            }
        }
        for implementation in &rows.implementations {
            work.charge(1)?;
            let application = implementation
                .trait_application
                .ok_or("inherent source impl requires its wider member profile")?;
            let application = &rows.trait_applications[application.0 as usize];
            if application.declaration.artifact != s::ArtifactRef(0) {
                return Err("foreign implementation domain requires a mixed-provider profile");
            }
            let Some(s::HeaderRef::Trait(index)) =
                headers.header(application.declaration.declaration, work)?
            else {
                return Err("implementation domain target has no actual trait header");
            };
            let domain = self
                .view()
                .trait_domain(index as usize, application.declaration.declaration)?;
            if let ImplementationDomain::RestrictedTo { module, .. } = domain.domain
                && !self
                    .index
                    .within(self.index.module(implementation.self_definition)?, module)?
            {
                return Err("provider implementation is outside its actual trait domain");
            }
        }
        Ok(())
    }
}
fn site(rows: &SourceRows, owner: DeclarationRef, site: u32) -> Result<(), &'static str> {
    if rows
        .declarations
        .get(owner.0 as usize)
        .is_none_or(|row| site >= row.sites)
    {
        return Err("module/domain site is outside its actual declaration");
    }
    Ok(())
}
fn compare(
    left: &ModuleBindingRow,
    right: &ModuleBindingRow,
    work: &mut Work,
) -> Result<Ordering, &'static str> {
    work.charge(1)?;
    let order = left.module.0.cmp(&right.module.0);
    if order != Ordering::Equal {
        return Ok(order);
    }
    for (left, right) in left.name.bytes().zip(right.name.bytes()) {
        work.charge(2)?;
        if left != right {
            return Ok(left.cmp(&right));
        }
    }
    work.charge(1)?;
    Ok(left
        .name
        .len()
        .cmp(&right.name.len())
        .then_with(|| left.namespace.cmp(&right.namespace)))
}

#[cfg(test)]
#[path = "../../../../tests/invariants/domain_budgets.rs"]
mod tests;
