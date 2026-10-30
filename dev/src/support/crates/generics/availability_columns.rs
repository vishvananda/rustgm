//! Versioned library availability is complete owner data, never body permission.
use super::{Columns, DefinitionKind, ModuleContractRows};
use crate::support::{
    crates::{MAX_BYTES, MAX_COMPONENTS, generics::Work, owner_catalog::DeclarationRef},
    retained,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AvailabilityKind {
    Regular,
    Const,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvailabilitySince {
    Current,
    Version { major: u16, minor: u16, patch: u16 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvailabilityLevel {
    Stable(AvailabilitySince),
    StableWithCompatibility {
        since: AvailabilitySince,
        message: String,
    },
    Unstable {
        issue: Option<std::num::NonZeroU32>,
        reason: Option<String>,
    },
}
impl AvailabilityLevel {
    pub(crate) const fn stable(&self) -> bool {
        matches!(self, Self::Stable(_) | Self::StableWithCompatibility { .. })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvailabilityRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AvailabilityOrigin {
    Declaration { owner: DeclarationRef, site: u32 },
    Binding { binding: u32, site: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvailabilityPolicy {
    pub kind: AvailabilityKind,
    pub feature: String,
    pub level: AvailabilityLevel,
    pub origin: AvailabilityOrigin,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeclarationAvailability {
    pub declaration: DeclarationRef,
    pub regular: Option<AvailabilityRef>,
    pub constant: Option<AvailabilityRef>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingAvailability {
    pub binding: u32,
    pub regular: Option<AvailabilityRef>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AvailabilityRows {
    pub policies: Vec<AvailabilityPolicy>,
    pub declarations: Vec<DeclarationAvailability>,
    pub bindings: Vec<BindingAvailability>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AvailabilityColumns {
    rows: AvailabilityRows,
}
impl AvailabilityColumns {
    pub(crate) const fn rows(&self) -> &AvailabilityRows {
        &self.rows
    }
}
impl AvailabilityRows {
    pub(crate) fn storage_bytes(&self, work: &mut Work) -> Result<usize, &'static str> {
        work.charge(
            self.policies
                .len()
                .saturating_add(self.declarations.len())
                .saturating_add(self.bindings.len())
                .saturating_add(1),
        )?;
        if self
            .policies
            .len()
            .max(self.declarations.len())
            .max(self.bindings.len())
            > MAX_COMPONENTS
        {
            return Err("availability inventory exceeds component budget");
        }
        let mut bytes = size_of::<Self>()
            .saturating_add(retained::vector(&self.policies))
            .saturating_add(retained::vector(&self.declarations))
            .saturating_add(retained::vector(&self.bindings));
        for policy in &self.policies {
            work.charge(policy.feature.len().saturating_add(1))?;
            let mut letters = policy.feature.chars();
            if !letters
                .next()
                .is_some_and(|first| first == '_' || unicode_ident::is_xid_start(first))
                || !letters.all(unicode_ident::is_xid_continue)
            {
                return Err("invalid availability feature identifier");
            }
            bytes = bytes.saturating_add(policy.feature.capacity());
            if let AvailabilityLevel::StableWithCompatibility { message, .. } = &policy.level {
                work.charge(message.len().saturating_add(1))?;
                if policy.kind != AvailabilityKind::Regular {
                    return Err("module compatibility requires regular stable API policy");
                }
                if message.len() > super::MAX_REASON_BYTES {
                    return Err("module compatibility message exceeds transport limit");
                }
                bytes = bytes.saturating_add(message.capacity());
            }
            if let AvailabilityLevel::Unstable {
                reason: Some(reason),
                ..
            } = &policy.level
            {
                work.charge(reason.len())?;
                if reason.len() > super::MAX_REASON_BYTES {
                    return Err("availability reason exceeds transport limit");
                }
                bytes = bytes.saturating_add(reason.capacity());
            }
            if bytes > MAX_BYTES {
                return Err("availability columns exceed retained storage budget");
            }
        }
        Ok(bytes)
    }
    pub(crate) fn validate(
        self,
        columns: &Columns,
        headers: &[Option<super::HeaderRef>],
        modules: &ModuleContractRows,
        base: usize,
        work: &mut Work,
        profile: super::Profile,
    ) -> Result<AvailabilityColumns, &'static str> {
        let bytes = self.storage_bytes(work)?;
        if self.declarations.len() != columns.declarations.len()
            || self.bindings.len() != modules.namespace.bindings.len()
        {
            return Err("availability columns omit their complete declaration or binding census");
        }
        if profile.conversion_roles()
            && self
                .declarations
                .get(modules.namespace.root.0 as usize)
                .is_some_and(|row| row.regular.is_none())
        {
            return self.ordinary_absence(work);
        }
        self.staged(columns, headers, modules, base.saturating_add(bytes), work)
    }
    fn staged(
        self,
        columns: &Columns,
        headers: &[Option<super::HeaderRef>],
        modules: &ModuleContractRows,
        base: usize,
        work: &mut Work,
    ) -> Result<AvailabilityColumns, &'static str> {
        if self
            .declarations
            .get(modules.namespace.root.0 as usize)
            .is_none_or(|row| row.regular.is_none())
        {
            return Err("complete staged availability requires its actual crate-root policy");
        }
        let origins =
            retained::new_map::<(AvailabilityOrigin, AvailabilityKind), ()>(self.policies.len());
        let ancestry = AvailabilityAncestry::new(
            &columns.declarations,
            modules.namespace.root,
            base.saturating_add(self.policies.len())
                .saturating_add(origins),
            work,
        )?;
        let view = Validation {
            columns,
            headers,
            modules,
            ancestry,
        };
        let mut origins = crate::support::hash::HashSet::with_capacity_and_hasher(
            self.policies.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        for (index, policy) in self.policies.iter().enumerate() {
            work.charge(1)?;
            if !origins.insert((policy.origin, policy.kind)) {
                return Err("availability source origin has duplicate policy kinds");
            }
            self.origin_policy(index, policy)?;
        }
        let mut used = vec![false; self.policies.len()];
        for (index, row) in self.declarations.iter().enumerate() {
            work.charge(1)?;
            if row.declaration.0 as usize != index {
                return Err("availability declaration order or identity changed");
            }
            let declaration = &columns.declarations[index];
            if matches!(
                declaration.definition.kind,
                DefinitionKind::TypeParameter
                    | DefinitionKind::RegionParameter
                    | DefinitionKind::ValueParameter
                    | DefinitionKind::Local
            ) {
                if row.regular.is_some() || row.constant.is_some() {
                    return Err("local or parameter declaration cannot carry API availability");
                }
                continue;
            }
            if row.regular.is_none()
                && (declaration.public || view.trait_implementation(row.declaration))
            {
                return Err(
                    "API or trait implementation availability lacks its actual regular policy",
                );
            }
            self.declaration_policy(&view, *row, false, work)?;
            if let Some(reference) = row.regular {
                used[reference.0 as usize] = true;
            }
            self.declaration_policy(&view, *row, true, work)?;
            if let Some(reference) = row.constant {
                used[reference.0 as usize] = true;
            }
        }
        for (index, row) in self.bindings.iter().enumerate() {
            work.charge(1)?;
            if row.binding as usize != index {
                return Err("availability binding order or identity changed");
            }
            let binding = &modules.namespace.bindings[index];
            if binding.visibility == super::ModuleVisibility::Public && row.regular.is_none() {
                return Err("public availability binding lacks its own regular policy");
            }
            self.binding_policy(&view, index, row.regular, work)?;
            if let Some(reference) = row.regular {
                used[reference.0 as usize] = true;
            }
        }
        if used.iter().any(|used| !used) {
            return Err("availability policy has no actual declaration or binding owner");
        }
        Ok(AvailabilityColumns { rows: self })
    }
    /// Explicit v9 ordinary absence is complete data, never a synthesized policy.
    fn ordinary_absence(self, work: &mut Work) -> Result<AvailabilityColumns, &'static str> {
        work.charge(1)?;
        if !self.policies.is_empty() {
            return Err("ordinary availability absence cannot erase actual staged policies");
        }
        for (index, row) in self.declarations.iter().enumerate() {
            work.charge(1)?;
            if row.declaration.0 as usize != index
                || row.regular.is_some()
                || row.constant.is_some()
            {
                return Err(
                    "ordinary availability absence has partial or foreign declaration rows",
                );
            }
        }
        for (index, row) in self.bindings.iter().enumerate() {
            work.charge(1)?;
            if row.binding as usize != index || row.regular.is_some() {
                return Err("ordinary availability absence has partial or foreign binding rows");
            }
        }
        Ok(AvailabilityColumns { rows: self })
    }
    fn policy(&self, reference: AvailabilityRef) -> Result<&AvailabilityPolicy, &'static str> {
        self.policies
            .get(reference.0 as usize)
            .ok_or("availability policy reference is outside its actual inventory")
    }
    fn origin_policy(&self, index: usize, policy: &AvailabilityPolicy) -> Result<(), &'static str> {
        let expected = Some(AvailabilityRef(
            u32::try_from(index).map_err(|_| "availability policy index overflow")?,
        ));
        let actual = match policy.origin {
            AvailabilityOrigin::Declaration { owner, .. } => {
                let row = self
                    .declarations
                    .get(owner.0 as usize)
                    .ok_or("availability policy origin is foreign")?;
                if policy.kind == AvailabilityKind::Const {
                    row.constant
                } else {
                    row.regular
                }
            }
            AvailabilityOrigin::Binding { binding, .. } => {
                self.bindings
                    .get(binding as usize)
                    .ok_or("availability policy binding origin is foreign")?
                    .regular
            }
        };
        if actual != expected {
            return Err("availability policy differs from its actual origin inventory row");
        }
        Ok(())
    }
    fn declaration_policy(
        &self,
        view: &Validation<'_>,
        row: DeclarationAvailability,
        constant: bool,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let owner = row.declaration;
        let Some(reference) = (if constant { row.constant } else { row.regular }) else {
            return Ok(());
        };
        work.charge(1)?;
        let policy = self.policy(reference)?;
        if constant {
            if !view.constant(owner) {
                return Err("const availability requires its actual const declaration kind");
            }
            if policy.kind == AvailabilityKind::Regular
                && !matches!(policy.level, AvailabilityLevel::Unstable { .. })
            {
                return Err(
                    "implicit const availability requires its actual unstable regular origin",
                );
            }
        } else if policy.kind != AvailabilityKind::Regular {
            return Err("const policy cannot supply regular API availability");
        }
        let AvailabilityOrigin::Declaration {
            owner: origin,
            site,
        } = policy.origin
        else {
            return Err("declaration availability cannot inherit an import policy");
        };
        let source = view
            .columns
            .declarations
            .get(origin.0 as usize)
            .ok_or("availability origin declaration is foreign")?;
        if site >= source.sites || !view.ancestry.contains(origin, owner) {
            return Err("availability declaration origin has a foreign site or ancestry");
        }
        if origin != owner
            && !matches!(
                source.definition.kind,
                DefinitionKind::Module | DefinitionKind::Trait | DefinitionKind::SelfType
            )
        {
            return Err("availability inherited from a non-scope declaration");
        }
        if origin != owner && policy.level.stable() {
            return Err("stable declaration policy cannot be inherited");
        }
        if policy.kind == AvailabilityKind::Const && !view.constant(origin) {
            return Err("const policy origin is not its actual const header");
        }
        if policy.kind == AvailabilityKind::Regular
            && !matches!(
                source.definition.kind,
                DefinitionKind::Module
                    | DefinitionKind::Trait
                    | DefinitionKind::SelfType
                    | DefinitionKind::Function
            )
        {
            return Err("regular policy origin is not an actual API declaration");
        }
        Ok(())
    }
    fn binding_policy(
        &self,
        view: &Validation<'_>,
        index: usize,
        reference: Option<AvailabilityRef>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let columns = view.columns;
        let modules = view.modules;
        let ancestry = &view.ancestry;
        let binding = &modules.namespace.bindings[index];
        if binding.source == super::BindingSource::Declaration
            && (binding.target.artifact != columns.policy.provider
                || self
                    .declarations
                    .get(binding.target.declaration.0 as usize)
                    .is_none_or(|row| row.regular != reference))
        {
            return Err("direct binding differs from its actual declaration policy");
        }
        let Some(reference) = reference else {
            return Ok(());
        };
        let policy = self.policy(reference)?;
        if policy.kind != AvailabilityKind::Regular {
            return Err("import binding cannot carry const API availability");
        }
        match policy.origin {
            AvailabilityOrigin::Declaration { owner, site } => {
                let origin = columns
                    .declarations
                    .get(owner.0 as usize)
                    .ok_or("binding availability origin is foreign")?;
                let direct = binding.source == super::BindingSource::Declaration
                    && owner == binding.target.declaration;
                if !direct && policy.level.stable() {
                    return Err("stable module policy cannot supply an import policy");
                }
                if site >= origin.sites
                    || (!direct
                        && (!ancestry.contains(owner, binding.module)
                            || origin.definition.kind != DefinitionKind::Module))
                {
                    return Err("binding availability origin has a foreign site or ancestry");
                }
            }
            AvailabilityOrigin::Binding {
                binding: source,
                site,
            } => {
                let origin = modules
                    .namespace
                    .bindings
                    .get(source as usize)
                    .ok_or("import policy origin is foreign")?;
                if origin.source == super::BindingSource::Declaration
                    || origin.module != binding.module
                    || origin.source_site != site
                    || binding.source_site != site
                {
                    return Err("import availability changed its actual grouped source site");
                }
            }
        }
        Ok(())
    }
}

struct Validation<'a> {
    columns: &'a Columns,
    headers: &'a [Option<super::HeaderRef>],
    modules: &'a ModuleContractRows,
    ancestry: AvailabilityAncestry,
}
impl Validation<'_> {
    fn trait_implementation(&self, owner: DeclarationRef) -> bool {
        match self.headers.get(owner.0 as usize).copied().flatten() {
            Some(super::HeaderRef::Implementation(index)) => self
                .columns
                .implementations
                .get(index as usize)
                .is_some_and(|row| row.trait_application.is_some()),
            _ => false,
        }
    }
    fn constant(&self, owner: DeclarationRef) -> bool {
        match self.headers.get(owner.0 as usize).copied().flatten() {
            Some(super::HeaderRef::Function(index)) => self
                .columns
                .functions
                .get(index as usize)
                .is_some_and(|row| row.constness != crate::support::constness::Constness::Ordinary),
            Some(super::HeaderRef::Trait(index)) => self
                .columns
                .traits
                .get(index as usize)
                .is_some_and(|row| row.flags.bits() & super::TraitFlags::CONST != 0),
            Some(super::HeaderRef::Implementation(index)) => self
                .columns
                .implementations
                .get(index as usize)
                .is_some_and(|row| row.constant),
            None => false,
        }
    }
}

pub(crate) struct AvailabilityAncestry {
    intervals: Vec<[u32; 2]>,
}
impl AvailabilityAncestry {
    pub(crate) fn new(
        declarations: &[super::Declaration],
        root: DeclarationRef,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let count = declarations.len();
        let scratch = count
            .saturating_mul(size_of::<[u32; 8]>() + size_of::<u8>())
            .saturating_add(5 * size_of::<Vec<u32>>() + size_of::<Self>());
        work.charge(count.saturating_mul(3).saturating_add(1))?;
        if base.saturating_add(scratch) > MAX_BYTES {
            return Err("availability ancestry exceeds combined storage budget");
        }
        if root.0 as usize >= count {
            return Err("availability root module is foreign");
        }
        let mut heads = vec![u32::MAX; count];
        let mut next = vec![u32::MAX; count];
        for (index, row) in declarations.iter().enumerate() {
            if let Some(parent) = row.definition.parent {
                if parent.0 as usize >= count || parent.0 as usize == index {
                    return Err("availability parent is foreign or cyclic");
                }
                next[index] = heads[parent.0 as usize];
                heads[parent.0 as usize] =
                    u32::try_from(index).map_err(|_| "availability ancestry identity overflow")?;
            } else if index != root.0 as usize {
                return Err("availability declaration has a foreign root");
            }
        }
        let mut marks = vec![false; count];
        let mut intervals = vec![[0; 2]; count];
        let mut stack = Vec::with_capacity(count.saturating_mul(2));
        stack.push((root.0, false));
        let mut visited = 0_u32;
        while let Some((node, exit)) = stack.pop() {
            work.charge(1)?;
            let index = node as usize;
            if exit {
                intervals[index][1] = visited;
                continue;
            }
            if std::mem::replace(&mut marks[index], true) {
                return Err("availability declaration ancestry is cyclic");
            }
            intervals[index][0] = visited;
            visited = visited
                .checked_add(1)
                .ok_or("availability ancestry count overflow")?;
            stack.push((node, true));
            let mut child = heads[index];
            while child != u32::MAX {
                work.charge(1)?;
                stack.push((child, false));
                child = next[child as usize];
            }
        }
        if visited as usize != count {
            return Err("availability declaration ancestry is disconnected or cyclic");
        }
        Ok(Self { intervals })
    }
    pub(crate) fn interval(&self, owner: DeclarationRef) -> Option<[u32; 2]> {
        self.intervals.get(owner.0 as usize).copied()
    }
    pub(crate) fn contains(&self, parent: DeclarationRef, child: DeclarationRef) -> bool {
        self.intervals
            .get(parent.0 as usize)
            .zip(self.intervals.get(child.0 as usize))
            .is_some_and(|(p, c)| p[0] <= c[0] && c[1] <= p[1])
    }
}
